"""Tracing a photo into SVG: masks to smooth outlines, and stacked shade layers that rebuild its
shading. Shared by draw_controllers.py's Touch controllers and Wii Remote buttons."""
from collections import deque

from PIL import ImageChops, ImageFilter

def components(mask, w, h, value):
    """Each 4-connected region of pixels equal to `value`, as a set of indices, with whether it
    touches the border."""
    seen = bytearray(w * h)
    for start in range(w * h):
        if seen[start] or mask[start] != value:
            continue
        region, touches, queue = [], False, deque([start])
        seen[start] = 1
        while queue:
            i = queue.popleft()
            region.append(i)
            x, y = i % w, i // w
            touches |= x in (0, w - 1) or y in (0, h - 1)
            for n in (i - 1 if x else -1, i + 1 if x < w - 1 else -1, i - w if y else -1, i + w if y < h - 1 else -1):
                if n >= 0 and not seen[n] and mask[n] == value:
                    seen[n] = 1
                    queue.append(n)
        yield set(region), touches


def outline(region, w):
    """The region's boundary as pixel corners, by walking its edge (Moore neighbour tracing)."""
    start = min(region, key=lambda i: (i // w, i % w))
    dirs = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)]
    inside = lambda x, y: y * w + x in region and 0 <= x < w
    x, y = start % w, start // w
    path, d = [(x, y)], 6
    for _ in range(len(region) * 4):
        for k in range(8):
            nd = (d + 6 + k) % 8
            nx, ny = x + dirs[nd][0], y + dirs[nd][1]
            if inside(nx, ny):
                x, y, d = nx, ny, nd
                break
        else:
            break
        if (x, y) == path[0]:
            break
        path.append((x, y))
    return path


def simplify(points, eps):
    """Douglas-Peucker."""
    if len(points) < 3:
        return points
    (ax, ay), (bx, by) = points[0], points[-1]
    length = ((bx - ax) ** 2 + (by - ay) ** 2) ** 0.5 or 1
    far, index = 0.0, 0
    for i, (px, py) in enumerate(points[1:-1], 1):
        dist = abs((bx - ax) * (ay - py) - (ax - px) * (by - ay)) / length
        if dist > far:
            far, index = dist, i
    if far <= eps:
        return [points[0], points[-1]]
    return simplify(points[:index + 1], eps)[:-1] + simplify(points[index:], eps)


def smooth_path(points):
    """A closed path of quadratic curves through the midpoints, so corners round off."""
    n = len(points)
    mids = [((points[i][0] + points[(i + 1) % n][0]) / 2, (points[i][1] + points[(i + 1) % n][1]) / 2) for i in range(n)]
    parts = [f"M{mids[-1][0]:.1f} {mids[-1][1]:.1f}"]
    for i in range(n):
        parts.append(f"Q{points[i][0]:.1f} {points[i][1]:.1f} {mids[i][0]:.1f} {mids[i][1]:.1f}")
    return "".join(parts) + "Z"


def trace(mask_img, to_view, min_area=80):
    """SVG path data for a mask's regions and holes (fill-rule evenodd)."""
    w, h = mask_img.size
    mask = bytes(1 if v else 0 for v in mask_img.tobytes())
    paths = []
    for value in (1, 0):
        for region, touches in components(mask, w, h, value):
            if len(region) < min_area or (value == 0 and touches):
                continue
            points = simplify(outline(region, w), 0.9)
            if len(points) >= 3:
                paths.append(smooth_path([to_view(x, y) for x, y in points]))
    return "".join(paths)



def gray_hex(level):
    """A neutral grey with a slight cool tint."""
    v = max(0, min(255, round(level)))
    return f"#{v:02x}{min(255, v + 1):02x}{min(255, v + 4):02x}"


def mean_hex(rgb, mask):
    """The average colour of `rgb` where `mask` is set, or None when nothing is."""
    histogram = [rgb.getchannel(i).histogram(mask) for i in range(3)]
    count = sum(histogram[0])
    if not count:
        return None
    return "#" + "".join(f"{round(sum(v * n for v, n in enumerate(h)) / count):02x}" for h in histogram)


def shade_layers(rgb, body, to_view, shades, min_area=12):
    """One traced path per shade, light to dark, each covering everything at least that dark and
    filled with the photo's average colour of its own band: stacked, they rebuild the shading.
    Yields (upper shade, path element)."""
    gray = rgb.convert("L").filter(ImageFilter.GaussianBlur(1.5))
    for high, low in zip(shades, [*shades[1:], 0]):
        covered = gray.point(lambda v, high=high: 255 if v < high else 0).filter(ImageFilter.MedianFilter(5))
        covered = ImageChops.multiply(covered, body)
        own = ImageChops.multiply(covered, gray.point(lambda v, low=low: 255 if v >= low else 0))
        color = mean_hex(rgb, own) or gray_hex(high)
        d = trace(covered, to_view, min_area)
        if d:
            yield high, f'<path d="{d}" fill="{color}" fill-rule="evenodd"/>'
