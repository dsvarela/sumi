"""Builds the built-in themes in crates/sumi-sudoku/themes/ (8 October 2026).

Each theme starts from a few colours of its source palette: background, text,
accent, digit, box mark, conflict and placement. The rest follows rules:

- Text colours are lifted in OKLCh lightness, hue kept, until each reads at
  least LIFT:1 on the background. That leaves room for the highlights, which
  must keep every text colour at 4.5:1 on them.
- Each highlight is the background moved toward its own source colour until
  it reaches its target contrast against the background, or as far as the
  text allows.
- Lines, the frame and box lines are the background moved toward the text
  until 3:1, 3.6:1 and 4.5:1, so each kind of line stands apart.

It prints, per theme, the worst text contrast, the closest two highlights in
OKLab, and the same under simulated protanopia, deuteranopia and tritanopia
(Machado, Oliveira and Fernandes 2009, severity 1). The Rust theme test
checks the same floors on the files this writes.

Run with `uv run theme-palettes.py ../crates/sumi-sudoku/themes`.
"""
import math
import sys
from pathlib import Path

LIFT = 8.0
# Contrast against the background each highlight aims for.
TARGET = {
    "cursor": 1.7,
    "hint_cell": 1.7,
    "same_digit": 1.45,
    "filter": 1.45,
    "hint_area": 1.25,
    "peer": 1.08,
}
# Surfaces that must look different, and the least OKLab distance between
# them, with and without simulated colour blindness. The cursor has its own
# shape (corner marks, or double lines in the dense layout), so it is only
# held apart from the background. Peer shading is meant to be faint.
APART = [
    ("background", "cursor", 0.04),
    ("background", "hint_cell", 0.04),
    ("background", "same_digit", 0.03),
    ("background", "filter", 0.03),
    ("background", "hint_area", 0.025),
    ("background", "peer", 0.012),
    ("hint_cell", "hint_area", 0.03),
    ("hint_area", "same_digit", 0.02),
    ("hint_area", "filter", 0.02),
    ("same_digit", "filter", 0.02),
    # Text that colour tells apart besides its other cue.
    ("digit", "clue", 0.06),
    ("conflict", "digit", 0.06),
    ("box_mark", "candidate", 0.06),
    ("hint_removal", "hint_placement", 0.06),
]
# The colour each highlight moves toward. Different hues keep them apart.
SOURCE = {
    "cursor": "accent",
    "hint_cell": "hint_placement",
    "same_digit": "digit",
    "filter": "box_mark",
    "hint_area": "hint_placement",
    "peer": "text",
}
TEXT = ["text", "dimmed", "accent", "clue", "digit", "candidate", "box_mark",
        "conflict", "wrong_digit", "hint_removal", "hint_placement"]
HIGHLIGHTS = list(TARGET)
ORDER = ["background", "text", "dimmed", "accent", "frame", "line", "box_line",
         "clue", "digit", "candidate", "box_mark", "cursor", "same_digit", "peer",
         "filter", "conflict", "wrong_digit", "hint_area", "hint_cell",
         "hint_removal", "hint_placement"]

# id: name, background, text, accent, digit, box mark, conflict, placement
THEMES = {
    "tokyo-night-amoled": ("Tokyo Night AMOLED", "#000000", "#c0caf5", "#7aa2f7",
                           "#7dcfff", "#ff9e64", "#f7768e", "#9ece6a"),
    "tokyo-night": ("Tokyo Night", "#1a1b26", "#c0caf5", "#7aa2f7", "#7dcfff",
                    "#ff9e64", "#f7768e", "#9ece6a"),
    "catppuccin-mocha": ("Catppuccin Mocha", "#1e1e2e", "#cdd6f4", "#89b4fa",
                         "#89dceb", "#fab387", "#f38ba8", "#a6e3a1"),
    "nord": ("Nord", "#2e3440", "#eceff4", "#88c0d0", "#8fbcbb", "#d08770",
             "#bf616a", "#a3be8c"),
    "gruvbox": ("Gruvbox", "#282828", "#ebdbb2", "#83a598", "#8ec07c", "#fe8019",
                "#fb4934", "#b8bb26"),
    "paper": ("Paper", "#f6f2e7", "#26283a", "#1d4f9f", "#0a5a85", "#8a4300",
              "#a3162f", "#2a6615"),
    # Okabe and Ito's palette: sky blue, orange, vermillion, bluish green.
    "okabe-ito": ("Colour-blind safe", "#000000", "#f2f2f2", "#56b4e9", "#56b4e9",
                  "#e69f00", "#d55e00", "#009e73"),
}


def rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))


def hexed(c):
    return "#" + "".join(f"{round(min(1, max(0, v)) * 255):02x}" for v in c)


def linear(v):
    return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4


def gamma(v):
    return v * 12.92 if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055


def luminance(h):
    r, g, b = map(linear, rgb(h))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a, b):
    la, lb = luminance(a), luminance(b)
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def oklab_lin(r, g, b):
    l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b) ** (1 / 3)
    m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b) ** (1 / 3)
    s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b) ** (1 / 3)
    return (0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s)


def oklab(h):
    return oklab_lin(*map(linear, rgb(h)))


def from_oklab(L, a, b):
    l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3
    m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3
    s = (L - 0.0894841775 * a - 1.2914855480 * b) ** 3
    lin = (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
           -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
           -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s)
    return lin


def in_gamut(lin):
    return all(-1e-4 <= v <= 1 + 1e-4 for v in lin)


def from_lch(L, C, H):
    """The colour at OKLCh (L, C, H), with chroma cut until it is in sRGB."""
    while True:
        lin = from_oklab(L, C * math.cos(H), C * math.sin(H))
        if in_gamut(lin) or C < 1e-4:
            return hexed(gamma(min(1, max(0, v))) for v in lin)
        C *= 0.97


def lch(h):
    L, a, b = oklab(h)
    return L, math.hypot(a, b), math.atan2(b, a)


def lift(h, bg, floor):
    """`h` with its OKLCh lightness moved away from `bg` until `floor`:1."""
    L, C, H = lch(h)
    step = 0.005 if luminance(bg) < 0.18 else -0.005
    while contrast(h, bg) < floor and 0 < L < 1:
        L += step
        h = from_lch(L, C, H)
    return h


def mix(a, b, t):
    a, b = rgb(a), rgb(b)
    return hexed(x + (y - x) * t for x, y in zip(a, b))


# Machado et al. 2009, severity 1.0, applied to linear RGB.
CVD = {
    "protanopia": ((0.152286, 1.052583, -0.204868), (0.114503, 0.786281, 0.099216),
                   (-0.003882, -0.048116, 1.051998)),
    "deuteranopia": ((0.367322, 0.860646, -0.227968), (0.280085, 0.672501, 0.047413),
                     (-0.011820, 0.042940, 0.968881)),
    "tritanopia": ((1.255528, -0.076749, -0.178779), (-0.078411, 0.930809, 0.147602),
                   (0.004733, 0.691367, 0.303900)),
}


def seen(h, kind):
    if kind is None:
        return oklab(h)
    lin = [linear(v) for v in rgb(h)]
    m = CVD[kind]
    out = [max(0.0, min(1.0, sum(m[i][j] * lin[j] for j in range(3)))) for i in range(3)]
    return oklab_lin(*out)


def distance(a, b, kind=None):
    return math.dist(seen(a, kind), seen(b, kind))


def build(_name, bg, text, accent, digit, box, conflict, placement):
    light = luminance(bg) > 0.18
    toward = "#000000" if light else "#ffffff"
    c = {"background": bg, "text": text, "clue": text, "accent": accent,
         "digit": digit, "box_mark": box, "conflict": conflict,
         "hint_placement": placement}
    c["wrong_digit"] = mix(conflict, toward, 0.2)
    c["hint_removal"] = conflict
    c["dimmed"] = mix(text, bg, 0.22)
    c["candidate"] = mix(text, bg, 0.12)
    for role in TEXT:
        c[role] = lift(c[role], bg, LIFT)
    # Box lines stand out from the lines between cells, the frame between.
    for role, floor in (("line", 3.05), ("frame", 3.6), ("box_line", 4.5)):
        t = 0.0
        while contrast(mix(bg, text, t), bg) < floor:
            t += 0.005
        c[role] = mix(bg, text, t)
    for role in HIGHLIGHTS:
        source = c[SOURCE[role]]
        t, best = 0.0, bg
        while t < 1:
            t += 0.005
            colour = mix(bg, source, t)
            if any(contrast(c[x], colour) < 4.55 for x in TEXT):
                break
            if role == "cursor" and contrast(c["accent"], colour) < 3.05:
                break
            best = colour
            if contrast(colour, bg) >= TARGET[role]:
                break
        c[role] = best
    return c


def report(id_, c, kinds):
    """Prints the worst text contrast and every pair in APART that is too
    close; returns whether all are apart."""
    surfaces = ["background"] + HIGHLIGHTS
    worst = min(contrast(c[t], c[s]) for t in TEXT for s in surfaces)
    close = []
    for kind in kinds:
        for a, b, floor in APART:
            d = distance(c[a], c[b], kind)
            if d < floor:
                close.append(f"{kind or 'seen'} {a}/{b} {d:.3f}<{floor}")
    print(f"{id_:20} text {worst:.2f}  " + ("ok" if not close else "; ".join(close)))
    return not close


def main():
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else None
    for id_, spec in THEMES.items():
        c = build(*spec)
        # Only the colour-blind theme promises the simulated floors.
        kinds = [None] + (list(CVD) if id_ == "okabe-ito" else [])
        report(id_, c, kinds)
        if out:
            body = f'name = "{spec[0]}"\n\n[colors]\n'
            body += "".join(f'{role} = "{c[role]}"\n' for role in ORDER)
            (out / f"{id_}.toml").write_text(body)


if __name__ == "__main__":
    main()
