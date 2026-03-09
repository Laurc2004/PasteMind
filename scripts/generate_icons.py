#!/usr/bin/env python3
"""Generate PasteMind icon assets locally (SVG + compressed RGBA PNG)."""

from __future__ import annotations

import pathlib
import struct
import zlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
TAURI_ICONS = ROOT / "src-tauri" / "icons"
STATIC_DIR = ROOT / "static"


def crc32(data: bytes) -> int:
    return zlib.crc32(data) & 0xFFFFFFFF


def png_chunk(kind: bytes, payload: bytes) -> bytes:
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", crc32(kind + payload))
    )


def write_png(path: pathlib.Path, width: int, height: int, rgba: bytearray) -> None:
    rows = []
    stride = width * 4
    for y in range(height):
        start = y * stride
        rows.append(b"\x00" + bytes(rgba[start : start + stride]))
    raw = b"".join(rows)

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    idat = zlib.compress(raw, level=9)
    png = b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", ihdr) + png_chunk(b"IDAT", idat) + png_chunk(b"IEND", b"")
    path.write_bytes(png)


def clear_canvas(width: int, height: int) -> bytearray:
    return bytearray(width * height * 4)


def set_px(buf: bytearray, width: int, x: int, y: int, color: tuple[int, int, int, int]) -> None:
    if x < 0 or y < 0:
        return
    index = (y * width + x) * 4
    if index < 0 or index + 3 >= len(buf):
        return
    r, g, b, a = color
    buf[index] = r
    buf[index + 1] = g
    buf[index + 2] = b
    buf[index + 3] = a


def inside_rounded_rect(
    x: int,
    y: int,
    left: int,
    top: int,
    right: int,
    bottom: int,
    radius: int,
) -> bool:
    if x < left or x > right or y < top or y > bottom:
        return False

    if left + radius <= x <= right - radius or top + radius <= y <= bottom - radius:
        return True

    cx = left + radius if x < left + radius else right - radius
    cy = top + radius if y < top + radius else bottom - radius
    dx = x - cx
    dy = y - cy
    return dx * dx + dy * dy <= radius * radius


def draw_rounded_gradient(
    buf: bytearray,
    width: int,
    height: int,
    left: int,
    top: int,
    right: int,
    bottom: int,
    radius: int,
) -> None:
    top_color = (27, 113, 255)
    bottom_color = (90, 215, 255)

    for y in range(top, bottom + 1):
        t = (y - top) / max(1, (bottom - top))
        r = int(top_color[0] * (1 - t) + bottom_color[0] * t)
        g = int(top_color[1] * (1 - t) + bottom_color[1] * t)
        b = int(top_color[2] * (1 - t) + bottom_color[2] * t)

        for x in range(left, right + 1):
            if inside_rounded_rect(x, y, left, top, right, bottom, radius):
                set_px(buf, width, x, y, (r, g, b, 255))


def draw_clipboard_symbol(
    buf: bytearray,
    width: int,
    scale: float,
    offset_x: int,
    offset_y: int,
) -> None:
    body_l = offset_x + int(6 * scale)
    body_t = offset_y + int(7 * scale)
    body_r = offset_x + int(17 * scale)
    body_b = offset_y + int(19 * scale)
    body_radius = max(2, int(2 * scale))

    for y in range(body_t, body_b + 1):
        for x in range(body_l, body_r + 1):
            if inside_rounded_rect(x, y, body_l, body_t, body_r, body_b, body_radius):
                set_px(buf, width, x, y, (244, 250, 255, 245))

    clip_l = offset_x + int(9 * scale)
    clip_t = offset_y + int(4 * scale)
    clip_r = offset_x + int(14 * scale)
    clip_b = offset_y + int(8 * scale)

    for y in range(clip_t, clip_b + 1):
        for x in range(clip_l, clip_r + 1):
            if inside_rounded_rect(x, y, clip_l, clip_t, clip_r, clip_b, max(1, int(scale))):
                set_px(buf, width, x, y, (225, 240, 255, 255))

    line_color = (66, 126, 228, 215)
    for row in range(10, 17, 3):
        y = offset_y + int(row * scale)
        for x in range(offset_x + int(8 * scale), offset_x + int(15 * scale)):
            set_px(buf, width, x, y, line_color)


def generate_png(path: pathlib.Path, size: int) -> None:
    buf = clear_canvas(size, size)

    padding = max(2, size // 16)
    draw_rounded_gradient(
        buf,
        size,
        size,
        padding,
        padding,
        size - 1 - padding,
        size - 1 - padding,
        max(3, size // 5),
    )

    # soft highlight strip
    for y in range(padding + 1, padding + max(2, size // 8)):
        alpha = int(95 * (1 - (y - padding) / max(1, size // 8)))
        for x in range(padding + 2, size - padding - 2):
            if inside_rounded_rect(
                x,
                y,
                padding,
                padding,
                size - 1 - padding,
                size - 1 - padding,
                max(3, size // 5),
            ):
                set_px(buf, size, x, y, (255, 255, 255, alpha))

    scale = size / 24
    draw_clipboard_symbol(buf, size, scale, 0, 0)

    write_png(path, size, size, buf)


def generate_svg(path: pathlib.Path) -> None:
    svg = """<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"128\" height=\"128\" viewBox=\"0 0 128 128\" fill=\"none\">
  <defs>
    <linearGradient id=\"pm-bg\" x1=\"20\" y1=\"12\" x2=\"102\" y2=\"116\" gradientUnits=\"userSpaceOnUse\">
      <stop stop-color=\"#1B71FF\"/>
      <stop offset=\"1\" stop-color=\"#5AD7FF\"/>
    </linearGradient>
    <linearGradient id=\"pm-clip\" x1=\"48\" y1=\"35\" x2=\"48\" y2=\"93\" gradientUnits=\"userSpaceOnUse\">
      <stop stop-color=\"#F8FCFF\"/>
      <stop offset=\"1\" stop-color=\"#DDEEFF\"/>
    </linearGradient>
  </defs>
  <rect x=\"12\" y=\"12\" width=\"104\" height=\"104\" rx=\"24\" fill=\"url(#pm-bg)\"/>
  <path d=\"M40 42a6 6 0 0 1 6-6h8.3a10 10 0 0 1 19.4 0H82a6 6 0 0 1 6 6v45a8 8 0 0 1-8 8H48a8 8 0 0 1-8-8V42Z\" fill=\"url(#pm-clip)\"/>
  <rect x=\"52\" y=\"31\" width=\"24\" height=\"12\" rx=\"6\" fill=\"#E8F3FF\"/>
  <rect x=\"50\" y=\"56\" width=\"28\" height=\"4\" rx=\"2\" fill=\"#4A8AF0\" opacity=\"0.6\"/>
  <rect x=\"50\" y=\"66\" width=\"28\" height=\"4\" rx=\"2\" fill=\"#4A8AF0\" opacity=\"0.6\"/>
  <rect x=\"50\" y=\"76\" width=\"22\" height=\"4\" rx=\"2\" fill=\"#4A8AF0\" opacity=\"0.6\"/>
</svg>
"""
    path.write_text(svg, encoding="utf-8")


def main() -> None:
    TAURI_ICONS.mkdir(parents=True, exist_ok=True)
    STATIC_DIR.mkdir(parents=True, exist_ok=True)

    generate_png(TAURI_ICONS / "icon.png", 256)
    generate_png(TAURI_ICONS / "tray.png", 32)
    generate_svg(STATIC_DIR / "app-icon.svg")

    print("Generated:")
    print(f"- {TAURI_ICONS / 'icon.png'}")
    print(f"- {TAURI_ICONS / 'tray.png'}")
    print(f"- {STATIC_DIR / 'app-icon.svg'}")


if __name__ == "__main__":
    main()
