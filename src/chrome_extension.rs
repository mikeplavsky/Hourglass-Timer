use crate::resources::{AppearanceStateChanged, TimerState};
#[cfg(any(test, target_arch = "wasm32"))]
use crate::resources::{ColorMode, HourglassConfig, HourglassShape, ShapeMode};
use crate::timer::{TimerCommand, TimerSet, TimerStateChanged};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
#[cfg(target_arch = "wasm32")]
use web_sys::{CustomEvent, CustomEventInit};

#[cfg(any(test, target_arch = "wasm32"))]
const SNAPSHOT_VERSION: u8 = 1;
#[cfg(target_arch = "wasm32")]
const BOOTSTRAP_PROPERTY: &str = "__HOURGLASS_BOOTSTRAP_V1__";
#[cfg(target_arch = "wasm32")]
const STATE_CHANGED_EVENT: &str = "hourglass-state-v1";
#[cfg(target_arch = "wasm32")]
const RESTORE_EVENT: &str = "hourglass-restore-v1";
#[cfg(target_arch = "wasm32")]
const READY_EVENT: &str = "hourglass-ready-v1";
#[cfg(target_arch = "wasm32")]
const STARTUP_STAGE_EVENT: &str = "hourglass-startup-stage-v1";
#[cfg(any(test, target_arch = "wasm32"))]
const MAX_DURATION_MS: f64 = 24.0 * 60.0 * 60.0 * 1000.0;

pub struct ChromeExtensionPlugin;

impl Plugin for ChromeExtensionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtensionDeadline>()
            .init_resource::<ExtensionSyncRevision>()
            .add_systems(PreStartup, initialize_extension_bridge)
            .add_systems(PostStartup, signal_extension_ready)
            .add_systems(Update, apply_queued_snapshots.in_set(TimerSet::Restore))
            .add_systems(
                Update,
                update_deadline_from_changes.in_set(TimerSet::Deadline),
            )
            .add_systems(Update, update_wall_clock_timer.in_set(TimerSet::Tick))
            .add_systems(Update, emit_extension_state.in_set(TimerSet::Observe));
    }
}

#[derive(Resource, Debug, Default)]
struct ExtensionDeadline(Option<f64>);

#[derive(Resource, Debug, Default)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
struct ExtensionSyncRevision(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ExtensionTimerStatus {
    Idle,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ExtensionColorMode {
    Static,
    Random,
    Rainbow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ExtensionShape {
    Classic,
    Modern,
    Slim,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ExtensionShapeMode {
    Static,
    Morphing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionAppearanceV1 {
    color_mode: ExtensionColorMode,
    color_rgba: [f32; 4],
    shape: ExtensionShape,
    shape_mode: ExtensionShapeMode,
}

/// Versioned wire format shared with the side-panel loader and service worker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionSnapshotV1 {
    version: u8,
    #[serde(default)]
    revision: u64,
    #[serde(default)]
    source_id: String,
    duration_ms: f64,
    remaining_ms: f64,
    status: ExtensionTimerStatus,
    deadline_ms: Option<f64>,
    #[serde(default)]
    run_id: Option<String>,
    #[serde(default)]
    notified_run_id: Option<String>,
    appearance: ExtensionAppearanceV1,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static RESTORE_QUEUE: RefCell<Vec<ExtensionSnapshotV1>> = const { RefCell::new(Vec::new()) };
}

fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0
    }
}

#[cfg(target_arch = "wasm32")]
fn initialize_extension_bridge(
    mut timer_state: ResMut<TimerState>,
    mut deadline: ResMut<ExtensionDeadline>,
    mut sync_revision: ResMut<ExtensionSyncRevision>,
    mut config: ResMut<HourglassConfig>,
) {
    install_restore_listener();

    let Some(window) = web_sys::window() else {
        warn!("Chrome extension bridge could not access window");
        return;
    };
    let Ok(value) = js_sys::Reflect::get(&window, &JsValue::from_str(BOOTSTRAP_PROPERTY)) else {
        return;
    };
    let Some(json) = value.as_string() else {
        return;
    };
    match serde_json::from_str::<ExtensionSnapshotV1>(&json) {
        Ok(snapshot) => {
            let revision = snapshot.revision;
            if apply_snapshot(
                snapshot,
                now_ms(),
                &mut timer_state,
                &mut deadline,
                &mut config,
            ) {
                sync_revision.0 = revision;
            }
        }
        Err(error) => warn!("Ignoring invalid extension bootstrap state: {error}"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn initialize_extension_bridge() {}

#[cfg(target_arch = "wasm32")]
fn signal_extension_ready() {
    let Ok(event) = CustomEvent::new(READY_EVENT) else {
        warn!("Could not create extension ready event");
        return;
    };
    if let Some(window) = web_sys::window()
        && let Err(error) = window.dispatch_event(&event)
    {
        warn!("Could not dispatch extension ready event: {error:?}");
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn report_startup_stage(stage: &str) {
    let init = CustomEventInit::new();
    init.set_detail(&JsValue::from_str(stage));
    if let Ok(event) = CustomEvent::new_with_event_init_dict(STARTUP_STAGE_EVENT, &init)
        && let Some(window) = web_sys::window()
    {
        let _ = window.dispatch_event(&event);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub(crate) fn report_startup_stage(_stage: &str) {}

#[cfg(not(target_arch = "wasm32"))]
fn signal_extension_ready() {}

#[cfg(target_arch = "wasm32")]
fn install_restore_listener() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let listener = Closure::<dyn FnMut(CustomEvent)>::new(|event: CustomEvent| {
        let Some(json) = event.detail().as_string() else {
            return;
        };
        match serde_json::from_str::<ExtensionSnapshotV1>(&json) {
            Ok(snapshot) => RESTORE_QUEUE.with(|queue| queue.borrow_mut().push(snapshot)),
            Err(error) => warn!("Ignoring invalid extension restore state: {error}"),
        }
    });
    if let Err(error) =
        window.add_event_listener_with_callback(RESTORE_EVENT, listener.as_ref().unchecked_ref())
    {
        warn!("Could not install extension restore listener: {error:?}");
    }
    listener.forget();
}

#[cfg(target_arch = "wasm32")]
fn apply_queued_snapshots(
    mut timer_state: ResMut<TimerState>,
    mut deadline: ResMut<ExtensionDeadline>,
    mut sync_revision: ResMut<ExtensionSyncRevision>,
    mut config: ResMut<HourglassConfig>,
) {
    let snapshot = RESTORE_QUEUE.with(|queue| {
        queue
            .borrow_mut()
            .drain(..)
            .filter(|snapshot| snapshot.revision > sync_revision.0)
            .max_by_key(|snapshot| snapshot.revision)
    });
    if let Some(snapshot) = snapshot {
        let revision = snapshot.revision;
        if apply_snapshot(
            snapshot,
            now_ms(),
            &mut timer_state,
            &mut deadline,
            &mut config,
        ) {
            sync_revision.0 = revision;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_queued_snapshots() {}

fn update_wall_clock_timer(
    mut timer_state: ResMut<TimerState>,
    mut deadline: ResMut<ExtensionDeadline>,
    mut changed: EventWriter<TimerStateChanged>,
) {
    if !timer_state.is_running {
        return;
    }
    let Some(deadline_ms) = deadline.0 else {
        return;
    };

    let remaining_ms = (deadline_ms - now_ms()).max(0.0);
    timer_state.remaining = (remaining_ms / 1000.0) as f32;
    if remaining_ms <= 0.0 {
        timer_state.is_running = false;
        deadline.0 = None;
        changed.write(TimerStateChanged(TimerCommand::Finish));
    }
}

fn update_deadline_from_changes(
    mut changes: EventReader<TimerStateChanged>,
    timer_state: Res<TimerState>,
    mut deadline: ResMut<ExtensionDeadline>,
) {
    for _ in changes.read() {
        deadline.0 = if timer_state.is_running && timer_state.remaining > 0.0 {
            Some(now_ms() + f64::from(timer_state.remaining) * 1000.0)
        } else {
            None
        };
    }
}

#[cfg(target_arch = "wasm32")]
fn emit_extension_state(
    mut timer_changes: EventReader<TimerStateChanged>,
    mut appearance_changes: EventReader<AppearanceStateChanged>,
    timer_state: Res<TimerState>,
    deadline: Res<ExtensionDeadline>,
    config: Res<HourglassConfig>,
) {
    let timer_dirty = timer_changes.read().count() > 0;
    let appearance_dirty = appearance_changes.read().count() > 0;
    if !timer_dirty && !appearance_dirty {
        return;
    }

    let snapshot = snapshot_from_resources(&timer_state, deadline.0, &config);
    let Ok(json) = serde_json::to_string(&snapshot) else {
        warn!("Could not serialize extension state");
        return;
    };
    let init = CustomEventInit::new();
    init.set_detail(&JsValue::from_str(&json));
    let Ok(event) = CustomEvent::new_with_event_init_dict(STATE_CHANGED_EVENT, &init) else {
        warn!("Could not create extension state event");
        return;
    };
    if let Some(window) = web_sys::window()
        && let Err(error) = window.dispatch_event(&event)
    {
        warn!("Could not dispatch extension state: {error:?}");
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn emit_extension_state(
    mut timer_changes: EventReader<TimerStateChanged>,
    mut appearance_changes: EventReader<AppearanceStateChanged>,
) {
    timer_changes.clear();
    appearance_changes.clear();
}

#[cfg(target_arch = "wasm32")]
fn snapshot_from_resources(
    timer_state: &TimerState,
    deadline_ms: Option<f64>,
    config: &HourglassConfig,
) -> ExtensionSnapshotV1 {
    let color = config.color.to_srgba();
    ExtensionSnapshotV1 {
        version: SNAPSHOT_VERSION,
        revision: 0,
        source_id: String::new(),
        duration_ms: f64::from(timer_state.duration) * 1000.0,
        remaining_ms: f64::from(timer_state.remaining) * 1000.0,
        status: timer_status(timer_state),
        deadline_ms,
        run_id: None,
        notified_run_id: None,
        appearance: ExtensionAppearanceV1 {
            color_mode: match config.color_mode {
                ColorMode::Static => ExtensionColorMode::Static,
                ColorMode::Random => ExtensionColorMode::Random,
                ColorMode::Rainbow => ExtensionColorMode::Rainbow,
            },
            color_rgba: [color.red, color.green, color.blue, color.alpha],
            shape: match config.shape_type {
                HourglassShape::Classic => ExtensionShape::Classic,
                HourglassShape::Modern => ExtensionShape::Modern,
                HourglassShape::Slim => ExtensionShape::Slim,
                HourglassShape::Wide => ExtensionShape::Wide,
            },
            shape_mode: match config.shape_mode {
                ShapeMode::Static => ExtensionShapeMode::Static,
                ShapeMode::Morphing => ExtensionShapeMode::Morphing,
            },
        },
    }
}

#[cfg(target_arch = "wasm32")]
fn timer_status(timer_state: &TimerState) -> ExtensionTimerStatus {
    if timer_state.is_running {
        ExtensionTimerStatus::Running
    } else if timer_state.remaining <= 0.0 {
        ExtensionTimerStatus::Finished
    } else if timer_state.remaining >= timer_state.duration {
        ExtensionTimerStatus::Idle
    } else {
        ExtensionTimerStatus::Paused
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn apply_snapshot(
    snapshot: ExtensionSnapshotV1,
    now_ms: f64,
    timer_state: &mut TimerState,
    deadline: &mut ExtensionDeadline,
    config: &mut HourglassConfig,
) -> bool {
    if snapshot.version != SNAPSHOT_VERSION {
        return false;
    }

    let has_usable_duration = snapshot.duration_ms.is_finite() && snapshot.duration_ms > 0.0;
    let duration_ms = if has_usable_duration {
        finite_clamp(snapshot.duration_ms, 0.0, MAX_DURATION_MS)
    } else {
        f64::from(TimerState::default().duration) * 1000.0
    };
    let stored_remaining_ms = if has_usable_duration {
        finite_clamp(snapshot.remaining_ms, 0.0, duration_ms)
    } else {
        duration_ms
    };
    let mut resolved_deadline = snapshot.deadline_ms.filter(|value| value.is_finite());

    let (remaining_ms, is_running) = if !has_usable_duration {
        resolved_deadline = None;
        (duration_ms, false)
    } else {
        match snapshot.status {
            ExtensionTimerStatus::Running => {
                let value = resolved_deadline
                    .map(|value| (value - now_ms).max(0.0))
                    .unwrap_or(stored_remaining_ms);
                if resolved_deadline.is_none() && value > 0.0 {
                    resolved_deadline = Some(now_ms + value);
                }
                (value, value > 0.0)
            }
            ExtensionTimerStatus::Paused => (stored_remaining_ms, false),
            ExtensionTimerStatus::Idle => (duration_ms, false),
            ExtensionTimerStatus::Finished => (0.0, false),
        }
    };

    timer_state.duration = (duration_ms / 1000.0) as f32;
    timer_state.remaining = (remaining_ms / 1000.0) as f32;
    timer_state.is_running = is_running;
    deadline.0 = if is_running { resolved_deadline } else { None };

    let [red, green, blue, alpha] = snapshot.appearance.color_rgba;
    config.color = Color::srgba(
        red.clamp(0.0, 1.0),
        green.clamp(0.0, 1.0),
        blue.clamp(0.0, 1.0),
        alpha.clamp(0.0, 1.0),
    );
    config.color_mode = match snapshot.appearance.color_mode {
        ExtensionColorMode::Static => ColorMode::Static,
        ExtensionColorMode::Random => ColorMode::Random,
        ExtensionColorMode::Rainbow => ColorMode::Rainbow,
    };
    config.shape_type = match snapshot.appearance.shape {
        ExtensionShape::Classic => HourglassShape::Classic,
        ExtensionShape::Modern => HourglassShape::Modern,
        ExtensionShape::Slim => HourglassShape::Slim,
        ExtensionShape::Wide => HourglassShape::Wide,
    };
    config.shape_mode = match snapshot.appearance.shape_mode {
        ExtensionShapeMode::Static => ShapeMode::Static,
        ExtensionShapeMode::Morphing => ShapeMode::Morphing,
    };
    true
}

#[cfg(any(test, target_arch = "wasm32"))]
fn finite_clamp(value: f64, minimum: f64, maximum: f64) -> f64 {
    if value.is_finite() {
        value.clamp(minimum, maximum)
    } else {
        minimum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(status: ExtensionTimerStatus, deadline_ms: Option<f64>) -> ExtensionSnapshotV1 {
        ExtensionSnapshotV1 {
            version: SNAPSHOT_VERSION,
            revision: 2,
            source_id: "test".to_string(),
            duration_ms: 180_000.0,
            remaining_ms: 90_000.0,
            status,
            deadline_ms,
            run_id: Some("run".to_string()),
            notified_run_id: None,
            appearance: ExtensionAppearanceV1 {
                color_mode: ExtensionColorMode::Static,
                color_rgba: [0.8, 0.6, 0.2, 1.0],
                shape: ExtensionShape::Classic,
                shape_mode: ExtensionShapeMode::Static,
            },
        }
    }

    #[test]
    fn running_restore_uses_absolute_deadline() {
        let mut timer = TimerState::default();
        let mut deadline = ExtensionDeadline::default();
        let mut config = HourglassConfig::default();
        assert!(apply_snapshot(
            snapshot(ExtensionTimerStatus::Running, Some(160_000.0)),
            100_000.0,
            &mut timer,
            &mut deadline,
            &mut config,
        ));
        assert_eq!(timer.remaining, 60.0);
        assert!(timer.is_running);
        assert_eq!(deadline.0, Some(160_000.0));
    }

    #[test]
    fn expired_restore_finishes_immediately() {
        let mut timer = TimerState::default();
        let mut deadline = ExtensionDeadline::default();
        let mut config = HourglassConfig::default();
        apply_snapshot(
            snapshot(ExtensionTimerStatus::Running, Some(99_000.0)),
            100_000.0,
            &mut timer,
            &mut deadline,
            &mut config,
        );
        assert_eq!(timer.remaining, 0.0);
        assert!(!timer.is_running);
        assert_eq!(deadline.0, None);
    }

    #[test]
    fn unsupported_snapshot_version_is_ignored() {
        let mut value = snapshot(ExtensionTimerStatus::Paused, None);
        value.version = 9;
        let mut timer = TimerState::default();
        let before = timer.clone();
        assert!(!apply_snapshot(
            value,
            100_000.0,
            &mut timer,
            &mut ExtensionDeadline::default(),
            &mut HourglassConfig::default(),
        ));
        assert_eq!(timer, before);
    }

    #[test]
    fn zero_duration_restore_recovers_three_minute_default() {
        let mut value = snapshot(ExtensionTimerStatus::Finished, None);
        value.duration_ms = 0.0;
        value.remaining_ms = 0.0;
        let mut timer = TimerState::default();
        let mut deadline = ExtensionDeadline::default();
        let mut config = HourglassConfig::default();
        assert!(apply_snapshot(
            value,
            100_000.0,
            &mut timer,
            &mut deadline,
            &mut config,
        ));
        assert_eq!(timer.duration, 180.0);
        assert_eq!(timer.remaining, 180.0);
        assert!(!timer.is_running);
        assert_eq!(deadline.0, None);
    }

    #[test]
    fn restart_replaces_stale_deadline_before_wall_clock_tick() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, crate::timer::TimerPlugin));
        app.add_event::<AppearanceStateChanged>();
        app.insert_resource(TimerState {
            duration: 180.0,
            remaining: 10.0,
            is_running: true,
        });
        app.add_plugins(ChromeExtensionPlugin);
        app.world_mut().resource_mut::<ExtensionDeadline>().0 = Some(now_ms() + 10_000.0);
        app.add_systems(
            Update,
            (|mut commands: EventWriter<TimerCommand>| {
                commands.write(TimerCommand::Restart);
            })
            .in_set(TimerSet::Input),
        );

        app.update();

        let timer = app.world().resource::<TimerState>();
        assert!(timer.is_running);
        assert!(timer.remaining > 179.0);
        let deadline = app.world().resource::<ExtensionDeadline>().0.unwrap();
        assert!(deadline > now_ms() + 179_000.0);
    }

    #[test]
    fn snapshot_round_trip_preserves_wire_shape() {
        let value = snapshot(ExtensionTimerStatus::Paused, None);
        let json = serde_json::to_string(&value).unwrap();
        assert!(json.contains("\"durationMs\":180000.0"));
        assert!(json.contains("\"shapeMode\":\"static\""));
        assert_eq!(
            serde_json::from_str::<ExtensionSnapshotV1>(&json).unwrap(),
            value
        );
    }
}
