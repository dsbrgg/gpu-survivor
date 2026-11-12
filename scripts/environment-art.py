# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow==13.0.0"]
# ///
"""Data-center props, industrial flooring, and rival/founder weapon sprites."""
import argparse
import math
import random
from pathlib import Path

from PIL import Image, ImageDraw
from art_utils import clean, pixelize, palette_for, apply_palette, write_assets

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "src/web/public/assets"
INK = "#18212a"


def environment():
    sheet = Image.open(ROOT / "art/source/environment.png").convert("RGBA")
    atlas = Image.new("RGBA", (512, 320))
    for index in range(8):
        x, y = index % 4, index // 4
        image = clean(sheet.crop((round(x * sheet.width / 4), round(y * sheet.height / 2),
            round((x + 1) * sheet.width / 4), round((y + 1) * sheet.height / 2))), 200)
        scale = min(72 / image.width, (82 if index == 7 else 140) / image.height)
        image = pixelize(image, (round(image.width * scale), round(image.height * scale)))
        image = apply_palette(image, palette_for([image]))
        atlas.paste(image, (x * 128 + (128 - image.width) // 2, y * 160 + 148 - image.height))
    return atlas


def flooring():
    atlas = Image.new("RGBA", (256, 64))
    for variant in range(4):
        image = Image.new("RGBA", (64, 64))
        draw = ImageDraw.Draw(image)
        rng = random.Random(900 + variant)
        for y in range(64):
            for x in range(64):
                grain = rng.choice((-1, 0, 0, 0, 1)) + variant
                draw.point((x, y), fill=(47 + grain, 57 + grain, 61 + grain, 255))
        draw.rectangle((0, 0, 63, 63), outline="#202a30")
        draw.line((1, 1, 62, 1), fill="#424d50")
        draw.line((1, 1, 1, 62), fill="#3c484b")
        draw.line((62, 2, 62, 62), fill="#273236")
        draw.line((2, 62, 62, 62), fill="#273236")
        for x, y in ((4, 4), (58, 4), (4, 58), (58, 58)):
            draw.rectangle((x, y, x + 1, y + 1), fill="#222d32")
            draw.point((x, y), fill="#596366")
        if variant == 3:
            draw.rectangle((9, 7, 54, 56), fill="#253239", outline="#546065")
            for y in range(11, 54, 4):
                draw.line((13, y, 50, y), fill="#141e25", width=2)
                draw.line((13, y + 2, 50, y + 2), fill="#3d4c54")
        elif variant == 1:
            draw.line((18, 38, 31, 36), fill="#3a4649")
            draw.line((24, 41, 38, 39), fill="#293438")
        atlas.paste(image, (variant * 64, 0))
    return atlas


def star(draw, center, radius, color, angle=-math.pi / 2):
    x, y = center
    points = [(x + math.cos(angle + i * math.pi / 5) * radius * (1 if i % 2 == 0 else .38),
               y + math.sin(angle + i * math.pi / 5) * radius * (1 if i % 2 == 0 else .38)) for i in range(10)]
    draw.polygon(points, fill=color)


def projectile(kind, frame):
    image = Image.new("RGBA", (64, 64))
    d = ImageDraw.Draw(image)
    if kind in (0, 1, 2):
        d.rectangle((17, 12, 49, 56), fill=INK)
        d.rectangle((16, 10, 46, 53), fill="#9ba2a5")
        d.polygon([(13, 7), (40, 7), (49, 16), (49, 51), (13, 51)], fill="#f5ebcf", outline=INK, width=2)
        d.polygon([(40, 8), (40, 17), (48, 17)], fill="#bbb69f")
        accent = ("#a54e49", "#6f5c8b", "#3c7eae")[kind]
        d.rectangle((18, 17, 35, 20), fill=accent)
        for y, end in ((25, 43), (29, 40), (33, 43), (37, 34)):
            d.line((18, y, end, y), fill="#93938b", width=2)
        if kind == 2:
            d.rectangle((32, 38, 43, 46), outline=accent, width=2)
            d.line((34, 41, 37, 44, 41, 39), fill=accent, width=2)
        else:
            d.ellipse((35, 39, 43, 47), fill=accent)
    elif kind == 3:
        flag = Image.new("RGBA", (48, 32), "#d6232b")
        f = ImageDraw.Draw(flag)
        star(f, (10, 10), 6, "#ffe174")
        for x, y in ((20, 4), (25, 9), (25, 17), (20, 23)):
            star(f, (x, y), 2.5, "#ffe174", math.atan2(10 - y, 10 - x))
        for x in range(48):
            wave = round(math.sin(x / 9 + frame * math.pi / 2) * 2)
            image.paste(flag.crop((x, 0, x + 1, 32)), (8 + x, 15 + wave))
        d.line((7, 12, 7, 52), fill=INK, width=3)
        d.line((7, 12, 7, 52), fill="#d9bd78")
    elif kind == 4:
        d.polygon([(23, 21), (38, 21), (36, 46), (37, 53), (32, 57), (26, 54)], fill=INK)
        d.polygon([(27, 21), (34, 21), (32, 50), (30, 52), (28, 50)], fill="#a6bac4")
        d.line((28, 26, 29, 49), fill="#edf0e3", width=2)
        d.polygon([(20, 7), (25, 6), (25, 16), (34, 16), (37, 7), (42, 9), (44, 19), (37, 28), (25, 28), (18, 20)], fill=INK)
        d.polygon([(22, 9), (24, 9), (24, 19), (36, 19), (39, 10), (40, 11), (41, 19), (35, 25), (26, 25), (21, 19)], fill="#b5c6cd")
        d.line((23, 21, 27, 24, 35, 24), fill="#edf0e3", width=2)
        d.ellipse((28, 49, 32, 53), fill=INK)
    elif kind == 5:
        half = (18, 12, 5, 12)[frame]
        d.ellipse((32 - half - 2, 11, 32 + half + 2, 53), fill=INK)
        d.ellipse((32 - half, 13, 32 + half, 51), fill="#b77c30")
        d.ellipse((34 - half, 14, 30 + half, 48), fill="#edc867", outline="#ffed9e", width=2)
        if half >= 12:
            d.line((32, 21, 32, 41), fill="#8e5b2b", width=2)
            d.line((38, 23, 27, 23, 27, 31, 37, 32, 37, 39, 26, 39), fill="#8e5b2b", width=2)
    elif kind == 6:
        points = [(8, 33), (28, 15), (26, 29), (49, 20), (56, 28), (32, 48), (36, 34), (15, 41)]
        d.polygon(points, fill=INK)
        d.polygon([(12, 33), (25, 21), (23, 32), (48, 24), (50, 28), (37, 40), (39, 31), (17, 37)], fill="#62d9ed")
        d.line((16, 33, 28, 28, 33, 32, 47, 27), fill="#e3ffff", width=2)
    elif kind == 7:
        d.regular_polygon((32, 32, 20), 6, rotation=15, fill=INK)
        d.regular_polygon((32, 31, 17), 6, rotation=15, fill="#c79573")
        d.regular_polygon((32, 29, 13), 6, rotation=15, fill="#edc4a0")
        d.ellipse((24, 22, 40, 38), fill=INK, outline="#9c6c52", width=3)
    elif kind == 8:
        # Angular stainless body, black wheel arches, and a continuous light bar.
        d.polygon([(3, 29), (17, 25), (32, 14), (44, 16), (59, 29), (61, 43), (4, 43)], fill=INK)
        d.polygon([(6, 29), (18, 27), (33, 17), (44, 19), (57, 30)], fill="#e0e5e2")
        d.polygon([(20, 27), (33, 19), (42, 20), (51, 28)], fill="#273c45")
        d.line((34, 20, 34, 27), fill="#9aaeb2", width=2)
        d.polygon([(6, 31), (58, 32), (58, 40), (7, 40)], fill="#a6b3b4")
        d.line((7, 33, 57, 34), fill="#dce5df")
        d.line((33, 30, 33, 40), fill="#687d86")
        d.line((36, 32, 40, 32), fill=INK)
        d.rectangle((5, 31, 7, 35), fill="#db7461")
        d.rectangle((57, 29, 60, 32), fill="#edfff5")
        d.line((7, 42, 57, 42), fill="#4e6168", width=2)
        for x in (16, 47):
            d.ellipse((x - 7, 36, x + 7, 50), fill=INK)
            d.ellipse((x - 4, 39, x + 4, 47), fill="#768b91")
            angle = frame * math.pi / 4
            d.line((x - math.cos(angle) * 3, 43 - math.sin(angle) * 3,
                    x + math.cos(angle) * 3, 43 + math.sin(angle) * 3), fill="#d6e2db", width=2)
    elif kind == 9:
        d.polygon([(23, 10), (30, 13), (36, 10), (47, 15), (57, 39), (46, 43),
                   (42, 32), (44, 55), (20, 55), (21, 32), (17, 43), (6, 39), (15, 15)], fill=INK)
        d.polygon([(16, 18), (24, 14), (31, 23), (30, 51), (22, 51), (24, 26),
                   (15, 39), (10, 37)], fill="#574639")
        d.polygon([(37, 14), (45, 18), (53, 37), (47, 39), (39, 26), (42, 51), (33, 51), (33, 23)], fill="#3e3730")
        d.polygon([(23, 13), (31, 17), (27, 27), (20, 20)], fill="#8c7761")
        d.polygon([(36, 13), (32, 17), (36, 27), (42, 20)], fill="#70604f")
        d.line((32, 23, 32, 52), fill="#c4b69c", width=2)
        for y in range(27, 50, 4):
            d.point((31, y), fill="#ebe2ca")
        d.line((25, 34, 23, 43), fill="#a18b6e", width=2)
        d.line((37, 34, 39, 43), fill="#8b775d", width=2)
        d.line((18, 21, 12, 34), fill="#8c7761")
        d.rectangle((21, 51, 43, 54), fill="#29292a")
        d.rectangle((37, 26, 41, 29), fill="#a1d861")
        d.point((38 + frame % 2, 27), fill="#e6ffc1")
    elif kind == 10:
        d.polygon([(22, 7), (42, 7), (37, 20), (49, 32), (53, 49), (46, 57),
                   (18, 57), (11, 49), (15, 32), (27, 20)], fill=INK)
        d.polygon([(25, 10), (38, 10), (34, 21), (45, 33), (49, 48), (44, 53),
                   (20, 53), (15, 48), (19, 33), (29, 21)], fill="#679684")
        d.polygon([(20, 33), (29, 24), (29, 49), (20, 49), (17, 44)], fill="#94bea4")
        d.line((26, 20, 38, 20), fill="#e5c576", width=3)
        d.line((36, 21, 43, 24), fill="#e5c576", width=2)
        d.line((32, 29, 32, 47), fill="#f4df9f", width=2)
        d.line((38, 31, 27, 31, 26, 37, 37, 39, 37, 44, 26, 44), fill="#f4df9f", width=2)
        d.ellipse((43, 45, 55, 57), fill="#ae7b3b", outline=INK, width=2)
        d.ellipse((46, 47, 52, 54), fill=("#f0d487", "#fff0bb")[frame % 2])
    elif kind == 11:
        d.rectangle((21, 4, 43, 59), fill=INK)
        d.rectangle((24, 7, 40, 56), fill="#885f4d")
        d.rectangle((26, 9, 38, 54), fill="#e0a98a")
        d.rectangle((28, 11, 36, 52), fill="#f4d7b4")
        for y in range(12 + frame * 2, 52, 10):
            d.polygon([(28, y), (36, y + 4), (36, min(y + 7, 52)), (28, y + 3)], fill="#9d715a")
        for y in (9, 51):
            d.rectangle((17, y, 46, y + 4), fill=INK)
            d.line((19, y + 1, 44, y + 1), fill="#f7e2c6", width=2)
        d.line((23, 16, 23, 47), fill="#f9b88e")
        d.line((41, 16, 41, 47), fill="#f9b88e")
    return image


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    shots = Image.new("RGBA", (768, 256))
    for kind in range(12):
        for frame in range(4):
            shots.paste(projectile(kind, frame), (kind * 64, frame * 64))
    write_assets({"environment.png": environment(), "floor.png": flooring(), "projectiles.png": shots}, OUTPUT, args.check)
    print("Verified environment/projectile assets." if args.check else "Built eight props, four floor panels, and twelve animated rival/founder weapons.")


if __name__ == "__main__":
    main()
