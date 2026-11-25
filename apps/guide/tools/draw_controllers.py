"""Traces the Quest 2 Touch controllers and the Wii Remote from product photos into SVGs: each Touch
controller beside a Wii Remote, the same numbered badge on a Touch button and the Wii button it
becomes.

Run: python -I apps/guide/tools/draw_controllers.py <touch-right.png> <wii-remote.png> apps/guide/assets/controllers
(needs Pillow). The photos, on plain or softly graded light backgrounds (coordinates below are in
their pixels):
  touch-right  1920 x 1080: the right controller from the side (trigger and grip) and the front;
               mirrored, it is the left controller too
  wii-remote   1024 x 1024: a Wii Remote from the side (B, underneath) and the front

Writes touch-left.svg, touch-right.svg and callouts.json: the badges' numbers and places, and
labels for parts no Wii Remote view shows (the Nunchuk's, recentering), which
components/ControllerDiagram.svelte lays over the SVG (WinUI's SVG renderer draws no text).
"""
import json
import os
import sys

from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageOps

sys.path.insert(0, os.path.dirname(__file__))
from trace import shade_layers, trace  # noqa: E402

RIGHT_PHOTO, WII_PHOTO, OUT = sys.argv[1:4]
VIEW_W = 520  # each SVG's width, in points
LABEL_W = 108  # the text labels' column on the left; their lines end at its edge
EDGE, HALO = "#9ea4ad", "#ffffff"
COLORS = ["#0067c0", "#c239b3", "#0f7b0f", "#ca5010", "#8764b8", "#038387"]
DARK = 105  # shades below this are the black face plate and buttons, drawn over light and shadow
TOUCH_SHADES = [*range(250, 150, -7), 130, DARK, 80, 62, 48, 36, 26]
# Per photo: how far its pixels may differ from its backdrop and still count as backdrop, and how
# far to close gaps in its outlines (its white edges fade into the backdrop).
THRESHOLD = {"right": 3}
CLOSE = {"right": 11}
WII_SHADES = [*range(252, 150, -6), 130, 100, 70, 40]

# The Wii Remote's two views (side, for B underneath; front), the front alone, its logo (blanked),
# points per photo pixel, where it sits, and where each button is and its badge goes (points
# from the button).
WII_BOTH, WII_FRONT, WII_LOGO = (212, 24, 806, 1000), (556, 24, 806, 1000), (600, 890, 780, 990)
WII_K, WII_X = 0.33, 322
WII_BUTTONS = {
    "b": ((236, 282), (-16, 0)), "dpad": ((685, 183), (-28, 0)), "a": ((685, 330), (-20, 0)),
    "minus": ((620, 492), (0, -16)), "home": ((685, 492), (0, -16)), "plus": ((752, 492), (0, -16)),
    "one": ((685, 742), (-18, 0)), "two": ((685, 826), (-18, 0)),
}

# The right controller photo's logo, and the letters on its B and A buttons.
RIGHT_LOGO = (1340, 160, 1430, 215)
RIGHT_LETTERS = [(1258, 394, 1293, 428), (1300, 433, 1336, 468)]

# Each controller: its views (photo, part of it, logo to blank, points per pixel, where it sits),
# and its callouts: the view, the part's place in that photo, then either where its badge goes
# (points from the part) and the Wii button, or a label's text and height.
CONTROLLERS = {
    "right": {
        "height": 480,
        "wii": (WII_BOTH, 70),
        "views": [
            # Front: the face buttons.
            ("right", (1030, 130, 1600, 936), [RIGHT_LOGO], 0.3, (120.5, 4)),
            # Side: the trigger (the hook under the ring's front) and the grip (the paddle).
            ("right", (232, 110, 975, 830), [], 0.3, (91, 254.5)),
        ],
        "callouts": [
            (0, (1275, 410), (-22, -14), "one"),
            (0, (1318, 450), (-20, 16), "a"),
            (0, (1390, 410), (6, -24), "dpad"),
            (1, (815, 640), (22, 8), "home"),
            (1, (565, 625), (-24, 4), "b"),
            (0, (1393, 425), None, ("Click: recenter", 40)),
        ],
    },
    "left": {
        # The right controller mirrored, its button letters blanked (they would read backwards).
        "mirror": True,
        "height": 480,
        "wii": (WII_FRONT, 70),
        "views": [
            ("right", (1030, 130, 1600, 936), [RIGHT_LOGO, *RIGHT_LETTERS], 0.3, (120.5, 4)),
            ("right", (232, 110, 975, 830), [], 0.3, (91, 254.5)),
        ],
        "callouts": [
            (0, (1275, 410), (22, -14), "two"),
            (0, (1318, 450), (20, 16), "minus"),
            (0, (1422, 462), (-6, 22), "plus"),
            (0, (1390, 410), None, ("Nunchuk stick", 40)),
            (1, (815, 640), None, ("Nunchuk Z", 398)),
            (1, (565, 625), None, ("Nunchuk C", 456)),
        ],
    },
}


def placed(box, k, x0, y0, up, mirror=False):
    """Photo pixels, and tracing pixels, to viewBox points for a photo region placed at x0, y0
    (mirrored left to right when `mirror`: the left controller drawn from the right one)."""

    def from_photo(x, y):
        return (x0 + ((box[2] - x) if mirror else (x - box[0])) * k, y0 + (y - box[1]) * k)

    def from_trace(x, y):
        return (x0 + x / up * k, y0 + y / up * k)

    return from_photo, from_trace


def cut(photo, box, blanks, up, close=0, threshold=6, mirror=False):
    """The region at tracing resolution, its logo blended away, and its silhouette: whatever
    differs from the backdrop (graded backdrops included), holes filled."""
    crop = photo.crop(box)
    for area in blanks:
        x0, y0, x1, y1 = (area[0] - box[0], area[1] - box[1], area[2] - box[0], area[3] - box[1])
        crop.paste(crop.filter(ImageFilter.MedianFilter(15)).crop((x0, y0, x1, y1)), (x0, y0))
    size = (round(crop.width * up), round(crop.height * up))
    crop = crop.resize(size, Image.LANCZOS)
    # The backdrop on each row, blended from the crop's own left edge to its right edge: close to
    # the controller, so a vignetted or graded backdrop still matches.
    left = crop.crop((1, 0, 2, crop.height)).resize(size)
    right = crop.crop((crop.width - 2, 0, crop.width - 1, crop.height)).resize(size)
    ramp = Image.linear_gradient("L").rotate(90, expand=True).resize(size)
    backdrop = Image.composite(right, left, ramp)
    diff = ImageChops.difference(crop, backdrop).convert("L")
    mask = diff.point(lambda v: 255 if v > threshold else 0).filter(ImageFilter.MedianFilter(9))
    # Close small gaps in the outline (bright edges that match the backdrop) before filling.
    if close:
        mask = mask.filter(ImageFilter.MaxFilter(close)).filter(ImageFilter.MinFilter(close))
    ImageDraw.floodfill(mask, (0, 0), 128)
    body = mask.point(lambda v: 0 if v == 128 else 255)
    if close:
        # And drop thin slivers of floor shadow.
        body = body.filter(ImageFilter.MinFilter(close)).filter(ImageFilter.MaxFilter(close))
    if mirror:
        return ImageOps.mirror(crop), ImageOps.mirror(body)
    return crop, body


def view_parts(index, photo_name, photo, box, blanks, k, at, mirror):
    """A traced photo region: its shades, light from the upper left and shadow toward the lower
    right kept inside it, then its dark parts on top. Traced at about two pixels per point."""
    up = min(2.0, 2 * k)
    from_photo, from_trace = placed(box, k, at[0], at[1], up, mirror)
    crop, body = cut(photo, box, blanks, up, CLOSE.get(photo_name, 0), THRESHOLD.get(photo_name, 6), mirror)
    silhouette = trace(body, from_trace)
    x0, y0 = at
    x1, y1 = at[0] + (box[2] - box[0]) * k, at[1] + (box[3] - box[1]) * k
    parts = [
        f"""<defs>
    <radialGradient id="light{index}" gradientUnits="userSpaceOnUse" cx="{x0 + (x1 - x0) * 0.4:.1f}" cy="{y0 + (y1 - y0) * 0.2:.1f}" r="{(y1 - y0) * 0.6:.1f}">
      <stop offset="0" stop-color="#ffffff" stop-opacity="0.45"/><stop offset="1" stop-color="#ffffff" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="shadow{index}" gradientUnits="userSpaceOnUse" x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}">
      <stop offset="0.45" stop-color="#30343c" stop-opacity="0"/><stop offset="1" stop-color="#30343c" stop-opacity="0.14"/>
    </linearGradient>
    <clipPath id="body{index}"><path d="{silhouette}" clip-rule="evenodd"/></clipPath>
  </defs>""",
        # A faint grey base, so the brightest white plastic still stands out from a white page.
        f'<path d="{silhouette}" fill="#f3f4f6" fill-rule="evenodd"/>',
    ]
    for high, path in shade_layers(crop, body, from_trace, TOUCH_SHADES):
        if high == DARK:
            parts += [f'<rect x="{x0:.1f}" y="{y0:.1f}" width="{x1 - x0:.1f}" height="{y1 - y0:.1f}" '
                      f'fill="url(#{fill}{index})" clip-path="url(#body{index})"/>' for fill in ("light", "shadow")]
        parts.append(path)
    parts.append(f'<path d="{silhouette}" fill="none" stroke="{EDGE}" stroke-width="1.2"/>')
    return parts, from_photo


def wii_remote(photo, box, y):
    """The Wii Remote's paths, and where each button lands, placed so its front view lines up
    whether or not the side view is shown."""
    x = WII_X + (box[0] - WII_BOTH[0]) * WII_K
    from_photo, from_trace = placed(box, WII_K, x, y, 0.8)
    crop, body = cut(photo, box, [WII_LOGO], 0.8)
    outline = trace(body, from_trace)
    parts = [f'<path d="{outline}" fill="#fbfbfc"/>']
    parts += [path for _, path in shade_layers(crop, body, from_trace, WII_SHADES, 4)]
    parts.append(f'<path d="{outline}" fill="none" stroke="{EDGE}" stroke-width="0.8"/>')
    return parts, {name: from_photo(*xy) for name, (xy, _) in WII_BUTTONS.items()}


def controller(hand, spec, photos):
    parts, mappers = [], []
    for index, (photo, box, blanks, k, at) in enumerate(spec["views"]):
        view, from_photo = view_parts(index, photo, photos[photo], box, blanks, k, at, spec.get("mirror", False))
        parts += view
        mappers.append(from_photo)
    wii_box, wii_y = spec["wii"]
    remote, buttons = wii_remote(photos["wii"], wii_box, wii_y)
    parts += remote

    # Parts the Wii Remote shows get the same numbered badge on both sides (the app draws the
    # numbers), each on a short leader; the rest get a line to their label.
    labels, badges = [], []
    for view, target, offset, wii in spec["callouts"]:
        color = COLORS[(len(labels) + len(badges) // 2) % len(COLORS)]
        x, y = mappers[view](*target)
        if isinstance(wii, str):
            number = len(badges) // 2 + 1
            for (px, py), (dx, dy) in (((x, y), offset), (buttons[wii], WII_BUTTONS[wii][1])):
                parts.append(f'<path d="M{px:.1f} {py:.1f}l{dx} {dy}" stroke="{color}" stroke-width="1.6" fill="none"/>')
                parts.append(f'<circle cx="{px:.1f}" cy="{py:.1f}" r="2.6" fill="{color}" stroke="{HALO}" stroke-width="1"/>')
                badges.append({"number": number, "color": color, "x": round(px + dx, 1), "y": round(py + dy, 1)})
            continue
        name, label_y = wii
        labels.append({"text": name, "y": label_y, "color": color})
        line = f"M{x:.1f} {y:.1f}L{LABEL_W} {label_y}"
        parts.append(f'<path d="{line}" stroke="{HALO}" stroke-width="4" stroke-linecap="round" fill="none"/>')
        parts.append(f'<path d="{line}" stroke="{color}" stroke-width="1.6" fill="none"/>')
        parts.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="2.6" fill="{color}" stroke="{HALO}" stroke-width="1"/>')

    height = spec["height"]
    body_svg = "\n  ".join(parts)
    with open(f"{OUT}/touch-{hand}.svg", "w", encoding="utf8") as file:
        file.write(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {VIEW_W} {height}" '
                   f'width="{VIEW_W}" height="{height}">\n  {body_svg}\n</svg>\n')
    return {"height": height, "labels": labels, "badges": badges}


photos = {name: Image.open(path).convert("RGB")
          for name, path in (("right", RIGHT_PHOTO), ("wii", WII_PHOTO))}
callouts = {hand: controller(hand, spec, photos) for hand, spec in CONTROLLERS.items()}
with open(f"{OUT}/callouts.json", "w", encoding="utf8") as file:
    json.dump({"width": VIEW_W, "labelWidth": LABEL_W, "hands": callouts}, file, ensure_ascii=False, indent=2)
    file.write("\n")
