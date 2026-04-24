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
    top_color: tuple[int, int, int] = (108, 92, 231),
    bottom_color: tuple[int, int, int] = (162, 155, 254),
) -> None:
    for y in range(top, bottom + 1):
        t = (y - top) / max(1, (bottom - top))
        r = int(top_color[0] * (1 - t) + bottom_color[0] * t)
        g = int(top_color[1] * (1 - t) + bottom_color[1] * t)
        b = int(top_color[2] * (1 - t) + bottom_color[2] * t)

        for x in range(left, right + 1):
            if inside_rounded_rect(x, y, left, top, right, bottom, radius):
                set_px(buf, width, x, y, (r, g, b, 255))


def draw_filled_rounded_rect(
    buf: bytearray,
    width: int,
    left: int,
    top: int,
    right: int,
    bottom: int,
    radius: int,
    color: tuple[int, int, int, int],
) -> None:
    for y in range(top, bottom + 1):
        for x in range(left, right + 1):
            if inside_rounded_rect(x, y, left, top, right, bottom, radius):
                set_px(buf, width, x, y, color)


def draw_circle(
    buf: bytearray,
    width: int,
    cx: int,
    cy: int,
    r: int,
    color: tuple[int, int, int, int],
) -> None:
    for y in range(cy - r, cy + r + 1):
        for x in range(cx - r, cx + r + 1):
            dx = x - cx
            dy = y - cy
            if dx * dx + dy * dy <= r * r:
                set_px(buf, width, x, y, color)


def lerp_color(
    c1: tuple[int, int, int],
    c2: tuple[int, int, int],
    t: float,
) -> tuple[int, int, int, int]:
    return (
        int(c1[0] * (1 - t) + c2[0] * t),
        int(c1[1] * (1 - t) + c2[1] * t),
        int(c1[2] * (1 - t) + c2[2] * t),
        255,
    )


def draw_p_icon(
    buf: bytearray,
    width: int,
    scale: float,
    offset_x: int,
    offset_y: int,
) -> None:
    purple = (108, 92, 231, 255)

    # P vertical stem
    stem_x = offset_x + int(10.5 * scale)
    stem_y1 = offset_y + int(12 * scale)
    stem_y2 = offset_y + int(21 * scale)
    stem_w = max(2, int(2 * scale))
    for y in range(stem_y1, stem_y2 + 1):
        for x in range(stem_x, stem_x + stem_w):
            set_px(buf, width, x, y, purple)

    # P top bar
    bar_x1 = stem_x
    bar_x2 = offset_x + int(16.5 * scale)
    bar_y = stem_y1
    bar_h = max(2, int(2 * scale))
    for y in range(bar_y, bar_y + bar_h):
        for x in range(bar_x1, bar_x2 + 1):
            set_px(buf, width, x, y, purple)

    # P right vertical (bowl)
    rv_x = bar_x2 - stem_w + 1
    rv_y1 = bar_y
    rv_y2 = offset_y + int(16 * scale)
    for y in range(rv_y1, rv_y2 + 1):
        for x in range(rv_x, rv_x + stem_w):
            set_px(buf, width, x, y, purple)

    # P bottom bar (bowl closure)
    for y in range(rv_y2 - bar_h + 1, rv_y2 + 1):
        for x in range(stem_x, rv_x + stem_w):
            set_px(buf, width, x, y, purple)


def draw_icon_content(
    buf: bytearray,
    width: int,
    size: int,
) -> None:
    padding = max(2, size // 16)
    s = size  # canvas size

    # Card body — white rounded rect
    card_l = int(s * 0.266)
    card_t = int(s * 0.25)
    card_r = int(s * 0.734)
    card_b = int(s * 0.781)
    card_radius = max(2, int(s * 0.078))

    # Draw card with subtle vertical gradient (white → light lavender)
    for y in range(card_t, card_b + 1):
        t = (y - card_t) / max(1, (card_b - card_t))
        color = lerp_color((255, 255, 255), (240, 237, 255), t)
        for x in range(card_l, card_r + 1):
            if inside_rounded_rect(x, y, card_l, card_t, card_r, card_b, card_radius):
                set_px(buf, width, x, y, color)

    # Clip tab background
    clip_l = int(s * 0.39)
    clip_t = int(s * 0.1875)
    clip_r = int(s * 0.609)
    clip_b = int(s * 0.3125)
    clip_radius = max(1, int(s * 0.0625))
    draw_filled_rounded_rect(buf, width, clip_l, clip_t, clip_r, clip_b, clip_radius, (232, 228, 255, 255))

    # Clip tab inner
    inner_l = int(s * 0.422)
    inner_t = int(s * 0.21875)
    inner_r = int(s * 0.578)
    inner_b = int(s * 0.296875)
    inner_radius = max(1, int(s * 0.04))
    draw_filled_rounded_rect(buf, width, inner_l, inner_t, inner_r, inner_b, inner_radius, (213, 207, 255, 255))

    # P glyph
    scale = size / 24
    draw_p_icon(buf, width, scale, 0, 0)

    # Sparkle dots
    sparkle_color = (255, 234, 167, 230)
    cx1 = int(s * 0.688)
    cy1 = int(s * 0.234)
    r1 = max(1, int(s * 0.031))
    draw_circle(buf, width, cx1, cy1, r1, sparkle_color)

    sparkle2 = (255, 234, 167, 153)
    cx2 = int(s * 0.734)
    cy2 = int(s * 0.172)
    r2 = max(1, int(s * 0.02))
    draw_circle(buf, width, cx2, cy2, r2, sparkle2)

    sparkle3 = (255, 234, 167, 128)
    cx3 = int(s * 0.641)
    cy3 = int(s * 0.18)
    r3 = max(1, int(s * 0.016))
    draw_circle(buf, width, cx3, cy3, r3, sparkle3)


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
        max(3, int(size * 0.203)),
    )

    # Subtle highlight strip at top
    for y in range(padding + 1, padding + max(2, size // 8)):
        alpha = int(60 * (1 - (y - padding) / max(1, size // 8)))
        for x in range(padding + 2, size - padding - 2):
            if inside_rounded_rect(
                x,
                y,
                padding,
                padding,
                size - 1 - padding,
                size - 1 - padding,
                max(3, int(size * 0.203)),
            ):
                set_px(buf, size, x, y, (255, 255, 255, alpha))

    draw_icon_content(buf, size, size)
    write_png(path, size, size, buf)


def generate_svg(path: pathlib.Path) -> None:
    svg = """<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 128 128" fill="none">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="128" y2="128" gradientUnits="userSpaceOnUse">
      <stop stop-color="#6C5CE7"/>
      <stop offset="1" stop-color="#A29BFE"/>
    </linearGradient>
    <linearGradient id="card" x1="64" y1="30" x2="64" y2="100" gradientUnits="userSpaceOnUse">
      <stop stop-color="#FFFFFF"/>
      <stop offset="1" stop-color="#F0EDFF"/>
    </linearGradient>
  </defs>
  <rect x="8" y="8" width="112" height="112" rx="26" fill="url(#bg)"/>
  <rect x="34" y="32" width="60" height="68" rx="10" fill="url(#card)"/>
  <rect x="50" y="24" width="28" height="16" rx="8" fill="#E8E4FF"/>
  <rect x="54" y="28" width="20" height="10" rx="5" fill="#D5CFFF"/>
  <rect x="46" y="50" width="7" height="38" rx="3.5" fill="#6C5CE7"/>
  <rect x="50" y="50" width="20" height="7" rx="3.5" fill="#6C5CE7"/>
  <rect x="63" y="50" width="7" height="20" rx="3.5" fill="#6C5CE7"/>
  <rect x="50" y="63" width="20" height="7" rx="3.5" fill="#6C5CE7"/>
  <circle cx="88" cy="30" r="4" fill="#FFEAA7" opacity="0.9"/>
  <circle cx="94" cy="22" r="2.5" fill="#FFEAA7" opacity="0.6"/>
  <circle cx="82" cy="23" r="2" fill="#FFEAA7" opacity="0.5"/>
</svg>
"""
    path.write_text(svg, encoding="utf-8")


def generate_tray_png(path: pathlib.Path, size: int) -> None:
    buf = clear_canvas(size, size)
    scale = size / 24
    white = (255, 255, 255, 255)

    stem_w = max(2, int(3 * scale))
    bar_h = max(2, int(3 * scale))

    stem_x1 = int(8 * scale)
    stem_y1 = int(6 * scale)
    stem_y2 = int(19 * scale)

    bar_x2 = int(17 * scale)
    bowl_y2 = int(14 * scale)

    # P vertical stem
    for y in range(stem_y1, stem_y2):
        for x in range(stem_x1, stem_x1 + stem_w):
            set_px(buf, size, x, y, white)

    # P top bar
    for y in range(stem_y1, stem_y1 + bar_h):
        for x in range(stem_x1, bar_x2):
            set_px(buf, size, x, y, white)

    # P right vertical (bowl)
    rv_x1 = bar_x2 - stem_w
    for y in range(stem_y1, bowl_y2):
        for x in range(rv_x1, bar_x2):
            set_px(buf, size, x, y, white)

    # P bottom bar (bowl closure)
    for y in range(bowl_y2 - bar_h, bowl_y2):
        for x in range(stem_x1, bar_x2):
            set_px(buf, size, x, y, white)

    write_png(path, size, size, buf)


def main() -> None:
    TAURI_ICONS.mkdir(parents=True, exist_ok=True)
    STATIC_DIR.mkdir(parents=True, exist_ok=True)

    generate_png(TAURI_ICONS / "icon.png", 256)
    generate_tray_png(TAURI_ICONS / "tray.png", 32)
    generate_svg(STATIC_DIR / "app-icon.svg")

    print("Generated:")
    print(f"- {TAURI_ICONS / 'icon.png'}")
    print(f"- {TAURI_ICONS / 'tray.png'}")
    print(f"- {STATIC_DIR / 'app-icon.svg'}")


if __name__ == "__main__":
    main()
