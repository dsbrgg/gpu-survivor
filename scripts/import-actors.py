# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow==13.0.0"]
# ///
"""Clean source sheets and pack foot-aligned, palette-limited actor atlases."""

import argparse
from pathlib import Path
from statistics import median

from PIL import Image, ImageDraw
from art_utils import clean, pixelize, palette_for, apply_palette, write_assets

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "src/web/public/assets"
ACTORS = (
    "jensen", "sam", "lawyer", "investor", "bot", "crawler", "bernie",
    "inspector", "dario", "elon", "engineer", "politician", "glm", "kimi", "deepseek",
    "aoc", "spy",
)
DIRECTIONS = ("S", "SW", "W", "NW", "N", "NE", "E", "SE")
CELL, COLUMNS, PER_PAGE = 128, 16, 4
ANCHOR = (64, 108)


def torso_center(image):
    width, height = image.size
    alpha = image.getchannel("A")
    return median(
        x for y in range(round(height * 0.3), round(height * 0.6))
        for x in range(width) if alpha.getpixel((x, y))
    )


def import_actor(name):
    sheet = Image.open(ROOT / "art/source" / f"{name}.png").convert("RGBA")
    frames = []
    for direction in range(8):
        for frame in range(4):
            box = (round(direction * sheet.width / 8), round(frame * sheet.height / 4),
                   round((direction + 1) * sheet.width / 8), round((frame + 1) * sheet.height / 4))
            frames.append(clean(sheet.crop(box), 224 if name == "elon" else 200))
    creature = name in ("crawler", "deepseek")
    target_height = 68 if name == "crawler" else 92 if name == "deepseek" else 100
    scale = min(target_height / median(frame.height for frame in frames[::4]),
                104 / max(frame.height for frame in frames),
                116 / max(frame.width for frame in frames))
    cells = []
    for image in frames:
        center = image.width / 2 if creature else torso_center(image)
        sprite = pixelize(image, (round(image.width * scale), round(image.height * scale)))
        left = round(ANCHOR[0] - center * scale)
        top = ANCHOR[1] - sprite.height
        if creature:
            top = round(72 - sprite.height / 2)
        if not (2 <= left and left + sprite.width <= CELL - 2 and 2 <= top and top + sprite.height <= CELL - 2):
            raise ValueError(f"{name}: frame exceeds cell padding")
        cell = Image.new("RGBA", (CELL, CELL))
        cell.paste(sprite, (left, top))
        cells.append(cell)
    # One palette across every direction prevents color shimmer during walking.
    palette = palette_for(cells)
    cells = [apply_palette(cell, palette) for cell in cells]
    for direction in range(8):
        if len({cell.tobytes() for cell in cells[direction * 4:direction * 4 + 4]}) != 4:
            raise ValueError(f"{name}: duplicated walk poses")
    if len({cells[direction * 4].tobytes() for direction in range(8)}) != 8:
        raise ValueError(f"{name}: duplicated facings")
    front = frames[0]
    side = round(front.height * 0.55)
    center = torso_center(front)
    portrait = front.crop((round(center - side / 2), -4, round(center - side / 2) + side, side - 4))
    portrait = pixelize(portrait, (128, 128))
    portrait = apply_palette(portrait, palette_for([portrait]))
    return cells, portrait


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Compare rebuilt assets without writing")
    parser.add_argument("--preview", action="store_true", help="Write a direction contact sheet to test-results")
    args = parser.parse_args()
    pages = [Image.new("RGBA", (COLUMNS * CELL, PER_PAGE * 2 * CELL)) for _ in range((len(ACTORS) + PER_PAGE - 1) // PER_PAGE)]
    outputs, actors = {}, []
    for row, name in enumerate(ACTORS):
        cells, portrait = import_actor(name)
        actors.append(cells)
        outputs[f"{name}.png"] = portrait
        for column, cell in enumerate(cells):
            pages[row // PER_PAGE].paste(cell, ((column % COLUMNS) * CELL,
                ((row % PER_PAGE) * 2 + column // COLUMNS) * CELL))
    outputs.update({f"actors-{page}.png": image for page, image in enumerate(pages)})
    bubble = clean(Image.open(ROOT / "art/source/bubble.png").convert("RGBA"), 128)
    bubble = pixelize(bubble, (round(bubble.width * 128 / bubble.height), 128))
    outputs["bubble.png"] = apply_palette(bubble, palette_for([bubble]))
    write_assets(outputs, OUTPUT, args.check)
    if args.preview:
        preview = Image.new("RGB", (9 * CELL, 32 + len(ACTORS) * CELL), "#263239")
        draw = ImageDraw.Draw(preview)
        for column, direction in enumerate(DIRECTIONS):
            draw.text(((column + 1) * CELL + 56, 12), direction, fill="#dbe7d7")
        for row, (name, cells) in enumerate(zip(ACTORS, actors)):
            draw.text((12, 88 + row * CELL), name.upper(), fill="#dbe7d7")
            for column in range(8):
                preview.paste(cells[column * 4], ((column + 1) * CELL, 32 + row * CELL), cells[column * 4])
        path = ROOT / "test-results/actor-directions.png"
        path.parent.mkdir(exist_ok=True)
        preview.save(path)
    print(f"{'Verified' if args.check else 'Imported'} {len(ACTORS) * 32} actor frames, {len(ACTORS)} portraits, {len(pages)} atlases and AI Bubble.")


if __name__ == "__main__":
    main()
