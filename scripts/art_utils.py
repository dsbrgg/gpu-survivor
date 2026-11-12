"""Shared source cleanup and deterministic pixel-art output."""
from collections import deque
from io import BytesIO

from PIL import Image


def clean(image, threshold):
    mask = image.getchannel("A").point(lambda a: 255 if a >= threshold else 0)
    width, height = image.size
    remaining = bytearray(mask.tobytes())
    largest = []
    for start in range(len(remaining)):
        if not remaining[start]:
            continue
        component, queue = [], deque([start])
        remaining[start] = 0
        while queue:
            index = queue.popleft()
            component.append(index)
            x, y = index % width, index // width
            for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                xx, yy = x + dx, y + dy
                if 0 <= xx < width and 0 <= yy < height:
                    other = yy * width + xx
                    if remaining[other]:
                        remaining[other] = 0
                        queue.append(other)
        if len(component) > len(largest):
            largest = component
    if len(largest) < width * height * 0.04:
        raise ValueError("Missing sprite silhouette")
    alpha = bytearray(width * height)
    for index in largest:
        alpha[index] = 255
    image.putalpha(Image.frombytes("L", image.size, bytes(alpha)))
    return image.crop(image.getbbox())


def pixelize(image, size):
    image = image.resize(size, Image.Resampling.LANCZOS)
    image.putalpha(image.getchannel("A").point(lambda a: 255 if a >= 128 else 0))
    return image


def palette_for(images):
    pixels = [pixel[:3] for image in images for pixel in image.get_flattened_data() if pixel[3]]
    sample = Image.new("RGB", (len(pixels), 1))
    sample.putdata(pixels)
    return sample.quantize(colors=64, method=Image.Quantize.MEDIANCUT)


def apply_palette(image, palette):
    result = image.convert("RGB").quantize(palette=palette, dither=Image.Dither.NONE).convert("RGBA")
    result.putalpha(image.getchannel("A"))
    transparent = Image.new("RGBA", result.size)
    transparent.paste(result, mask=result.getchannel("A"))
    return transparent


def write_assets(outputs, directory, check=False):
    for name, image in outputs.items():
        buffer = BytesIO()
        image.save(buffer, format="PNG", optimize=True)
        data = buffer.getvalue()
        path = directory / name
        if check:
            if not path.exists() or path.read_bytes() != data:
                raise SystemExit(f"Stale asset: {name}; run pnpm assets")
        else:
            path.write_bytes(data)
