"""Procedural scenery, modular props, and projectile logos. Actors: import-actors.py."""
from pathlib import Path
import struct
import zlib
from logo_masks import LOGOS

ROOT = Path(__file__).resolve().parents[1] / "src/web/public/assets"
ROOT.mkdir(parents=True, exist_ok=True)


class Canvas:
    def __init__(self, width, height):
        self.width, self.height = width, height
        self.data = bytearray(width * height * 4)
        self.offset = (0, 0)

    def pixel(self, x, y, color):
        x, y = int(x) + self.offset[0], int(y) + self.offset[1]
        if 0 <= x < self.width and 0 <= y < self.height:
            color = color.lstrip("#")
            self.data[(y * self.width + x) * 4:(y * self.width + x) * 4 + 4] = bytes.fromhex(color) + b"\xff"

    def rect(self, x, y, w, h, color):
        for j in range(int(y), int(y + h)):
            for i in range(int(x), int(x + w)):
                self.pixel(i, j, color)

    def ellipse(self, x, y, w, h, color):
        for j in range(y, y + h):
            for i in range(x, x + w):
                if ((i - x - w / 2 + .5) / (w / 2)) ** 2 + ((j - y - h / 2 + .5) / (h / 2)) ** 2 <= 1:
                    self.pixel(i, j, color)

    def save(self, name):
        def chunk(kind, data):
            return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
        raw = b"".join(b"\x00" + self.data[y * self.width * 4:(y + 1) * self.width * 4] for y in range(self.height))
        png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", self.width, self.height, 8, 6, 0, 0, 0))
        (ROOT / name).write_bytes(png + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))

    def copy(self, source, sx, sy, width, height, dx=0, dy=0):
        for y in range(height):
            for x in range(width):
                start = ((sy + y) * source.width + sx + x) * 4
                end = ((dy + y) * self.width + dx + x) * 4
                self.data[end:end + 4] = source.data[start:start + 4]

    def stamp(self, source, x, y, width, height):
        for row in range(height):
            for column in range(width):
                index = ((row * source.height // height) * source.width + column * source.width // width) * 4
                if source.data[index + 3]:
                    self.pixel(x + column, y + row, source.data[index:index + 3].hex())


c = Canvas(512, 64)
INK = "151b23"
for i in range(8):
    c.offset = (i * 64, 0)
    if i in [0, 1]:
        c.rect(14, 5, 36, 48, INK)
        c.rect(16, 5, 32, 45, "354548")
        c.rect(17, 5, 30, 4, "627172")
        c.rect(18, 11, 27, 36, "1d2a2e")
        for y in range(13, 44, 7):
            c.rect(20, y, 23, 5, "435255")
            c.rect(22, y + 1, 13, 1, "657471")
            c.rect(22, y + 3, 13, 1, "1a282e")
            c.rect(39, y + 1, 2, 2, "c0e582" if i == 0 else "dfaa6a")
        c.rect(17, 49, 5, 4, INK)
        c.rect(42, 49, 5, 4, INK)
    elif i == 2:
        c.rect(14, 5, 36, 48, INK)
        c.rect(16, 6, 32, 44, "677e89")
        c.rect(18, 8, 28, 5, "9db7bf")
        for y in [23, 39]:
            c.ellipse(21, y - 7, 22, 14, INK)
            c.ellipse(23, y - 6, 18, 12, "374d5b")
            c.rect(30, y - 5, 4, 10, "9cbfc6")
            c.rect(25, y - 1, 14, 3, "9cbfc6")
            c.rect(31, y - 1, 2, 2, INK)
        c.rect(19, 10, 4, 2, "7dd2ef")
    elif i == 3:
        c.rect(14, 5, 36, 48, INK)
        c.rect(16, 6, 32, 44, "5e6868")
        c.rect(18, 9, 28, 38, "343f46")
        c.rect(21, 12, 21, 10, "152a35")
        c.rect(23, 14, 10, 2, "c0e582")
        c.rect(23, 18, 16, 1, "738c81")
        for y in [28, 35, 42]:
            c.rect(22, y, 18, 4, INK)
            c.rect(24, y, 4, 3, "d5ac65")
        c.rect(44, 25, 2, 9, "95a29b")
    elif i == 4:
        c.rect(20, 30, 25, 15, INK)
        c.rect(21, 31, 23, 12, "588c59")
        c.rect(23, 32, 7, 9, "96ba66")
        c.ellipse(32, 32, 10, 10, "1e3c3b")
        c.rect(36, 34, 2, 6, "6d9e72")
        c.rect(34, 36, 6, 2, "6d9e72")
        for x in range(25, 42, 3):
            c.rect(x, 44, 2, 3, "e9c675")
    elif i == 5:
        c.rect(22, 32, 23, 13, "6c815b")
        c.rect(20, 29, 23, 13, "bccd8a")
        c.rect(22, 31, 19, 9, "6c9566")
        c.ellipse(28, 31, 7, 9, "c4d591")
        c.rect(29, 28, 4, 16, "e9d8b2")
    elif i == 6:
        c.rect(17, 21, 31, 29, INK)
        c.rect(18, 22, 29, 26, "9e7762")
        c.rect(20, 24, 25, 21, "caa580")
        c.rect(28, 23, 7, 23, "e2c596")
        c.rect(23, 30, 4, 8, "7d6658")
    elif i == 7:
        c.rect(22, 22, 21, 30, INK)
        c.rect(23, 23, 19, 27, "8e6662")
        c.rect(25, 25, 15, 10, "c7ac8b")
        c.rect(27, 27, 11, 5, "453f3d")
        c.rect(29, 38, 7, 6, "c7ac8b")
c.save("sprites.png")

democrats = Canvas(32, 32)
democrats.ellipse(2, 2, 28, 28, "ffffff")
democrats.ellipse(3, 3, 26, 26, "00aef3")
democrats.ellipse(5, 5, 22, 22, "ffffff")
democrats.ellipse(11, 9, 12, 14, "00aef3")
democrats.rect(10, 9, 7, 14, "00aef3")
democrats.ellipse(13, 12, 7, 8, "ffffff")
democrats.rect(13, 12, 3, 8, "ffffff")
democrats.save("democrats.png")

audit = Canvas(32, 32)
audit.rect(6, 3, 20, 26, INK)
audit.rect(8, 4, 16, 23, "e5e7d7")
audit.rect(12, 3, 8, 4, "748a9a")
for y in [10, 14, 18]:
    audit.rect(11, y, 10, 2, "485e73")
audit.ellipse(18, 21, 6, 6, "dbb880")
audit.save("regulatory_crackdown.png")

stock_icons = []
for rising in [False, True]:
    icon = Canvas(32, 32)
    icon.rect(1, 3, 30, 25, INK)
    icon.rect(3, 5, 26, 20, "253536")
    for x in [8, 16, 24]:
        icon.rect(x, 6, 1, 18, "344647")
    for y in [11, 18]:
        icon.rect(4, y, 24, 1, "344647")
    color = "b5ec83" if rising else "ed8b79"
    for x in range(5, 27):
        y = 22 - (x - 5) * 14 // 21 if rising else 8 + (x - 5) * 14 // 21
        icon.rect(x, y, 2, 3, color)
    icon.rect(22, 7 if rising else 20, 6, 3, color)
    icon.rect(25, 7 if rising else 17, 3, 6, color)
    icon.save("ai_stock_rally.png" if rising else "ai_stock_crash.png")
    stock_icons.append(icon)

pack = Canvas(320, 64)
pack.rect(23, 7, 3, 44, INK)
pack.rect(24, 8, 1, 42, "b3c4c9")
pack.rect(25, 8, 29, 25, INK)
pack.rect(26, 9, 27, 22, "24588f")
pack.rect(28, 11, 23, 2, "74baf0")
pack.stamp(democrats, 31, 11, 19, 19)
pack.rect(19, 49, 12, 3, INK)
pack.offset = (64, 0)
pack.rect(25, 36, 14, 14, INK)
pack.rect(26, 37, 12, 11, "74baf0")
pack.rect(29, 39, 6, 6, "24588f")
pack.rect(30, 40, 4, 4, "bee7ff")
for x in [27, 31, 35]:
    pack.rect(x, 48, 2, 3, "efcf85")
for column, icon in enumerate([audit, *stock_icons], start=2):
    pack.offset = (column * 64, 0)
    pack.rect(29, 29, 4, 21, INK)
    pack.rect(25, 49, 12, 3, INK)
    pack.stamp(icon, 16, 6, 32, 32)
pack.save("packs.png")

logos = Canvas(128, 32)
for column, (name, color, mask) in enumerate(LOGOS):
    icon = Canvas(32, 32)
    pixels = [(x + 2, y + 2) for y, row in enumerate(mask.strip().splitlines())
              for x, value in enumerate(row) if value == "#"]
    for x, y in pixels:
        for dx, dy in [(-1, 0), (1, 0), (0, -1), (0, 1)]:
            icon.pixel(x + dx, y + dy, INK)
    for x, y in pixels:
        icon.pixel(x, y, color)
    logos.copy(icon, 0, 0, 32, 32, column * 32, 0)
    icon.save(name + ".png")
logos.save("logos.png")
print("Generated scenery, modular props, and projectile logos.")
