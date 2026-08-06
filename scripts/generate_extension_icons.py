#!/usr/bin/env python3
"""Generate the committed Chrome extension icons from simple vector geometry."""

from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "extension" / "icons"
SIZES = (16, 32, 48, 128)


def render(size: int) -> Image.Image:
    scale = 8
    canvas = size * scale
    image = Image.new("RGBA", (canvas, canvas), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)

    margin = int(canvas * 0.06)
    draw.rounded_rectangle(
        (margin, margin, canvas - margin, canvas - margin),
        radius=int(canvas * 0.2),
        fill=(18, 18, 20, 255),
    )

    left = int(canvas * 0.27)
    right = int(canvas * 0.73)
    top = int(canvas * 0.2)
    bottom = int(canvas * 0.8)
    neck_y = canvas // 2
    gold = (224, 174, 69, 255)
    glass = (210, 238, 248, 235)

    bar_height = max(scale, int(canvas * 0.075))
    draw.rounded_rectangle(
        (left - int(canvas * 0.05), top - bar_height // 2,
         right + int(canvas * 0.05), top + bar_height // 2),
        radius=bar_height // 2,
        fill=gold,
    )
    draw.rounded_rectangle(
        (left - int(canvas * 0.05), bottom - bar_height // 2,
         right + int(canvas * 0.05), bottom + bar_height // 2),
        radius=bar_height // 2,
        fill=gold,
    )

    stroke = max(scale, int(canvas * 0.035))
    draw.line((left, top, canvas // 2, neck_y, left, bottom), fill=glass, width=stroke, joint="curve")
    draw.line((right, top, canvas // 2, neck_y, right, bottom), fill=glass, width=stroke, joint="curve")

    inset = int(canvas * 0.045)
    draw.polygon(
        ((left + inset, top + bar_height // 2),
         (right - inset, top + bar_height // 2),
         (canvas // 2, neck_y - int(canvas * 0.04))),
        fill=gold,
    )
    draw.polygon(
        ((canvas // 2, neck_y + int(canvas * 0.08)),
         (left + inset, bottom - bar_height // 2),
         (right - inset, bottom - bar_height // 2)),
        fill=gold,
    )

    return image.resize((size, size), Image.Resampling.LANCZOS)


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for size in SIZES:
        render(size).save(OUTPUT / f"icon-{size}.png", optimize=True)


if __name__ == "__main__":
    main()
