#!/usr/bin/env python3
"""Draw the README "Openergy vs Synergy" bar chart as two standalone SVGs.

Usage:
    python3 tools/comparison-chart/chart.py          # write the light and dark SVGs
    python3 tools/comparison-chart/chart.py --check  # exit 1 if either SVG is out of date

The numbers live in data.json next to this script, in KiB as recorded
(`du -sk`, `ps -o rss=`). Every value must be a real measurement with a date
and a source; the README table cites the same sources.

The output is assets/openergy-vs-synergy-light.svg and -dark.svg. Each has
fixed colors and a transparent background. The README selects one with a
<picture> element, which follows the reader's GitHub theme. No external
dependencies.
"""
import argparse
import json
import sys
from pathlib import Path
from xml.sax.saxutils import escape

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
DATA = HERE / "data.json"
OUTPUTS = {
    "light": ROOT / "assets" / "openergy-vs-synergy-light.svg",
    "dark": ROOT / "assets" / "openergy-vs-synergy-dark.svg",
}

# Text colors match GitHub's light and dark themes. Series colors are
# categorical slots 1 (blue) and 2 (orange), stepped per theme; both pairs pass
# lightness, chroma, color-vision-deficiency separation and 3:1 contrast
# against the page background.
THEMES = {
    "light": {"t1": "#1f2328", "t2": "#59636e", "rule": "#d1d9e0",
              "openergy": "#2a78d6", "synergy": "#eb6834"},
    "dark": {"t1": "#f0f6fc", "t2": "#9198a1", "rule": "#3d444d",
             "openergy": "#3987e5", "synergy": "#d95926"},
}

WIDTH = 720
MARGIN = 16
LABEL_COL = 92          # category label column
VALUE_COL = 96          # room for the value label after the longest bar
BAR = 20                # bar thickness (<= 24px)
ROW = 30                # row pitch
RADIUS = 4              # rounded data end
UNIT = "MiB"

FONTS = '-apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif'


def style(theme):
    c = THEMES[theme]
    return f"""
  .t1 {{ fill: {c["t1"]}; }}
  .t2 {{ fill: {c["t2"]}; }}
  .rule {{ stroke: {c["rule"]}; }}
  .s-openergy {{ fill: {c["openergy"]}; }}
  .s-synergy {{ fill: {c["synergy"]}; }}
  text {{ font-family: {FONTS}; }}
  .title {{ font-size: 16px; font-weight: 600; }}
  .sub {{ font-size: 12px; }}
  .head {{ font-size: 13px; font-weight: 600; }}
  .cat {{ font-size: 13px; }}
  .val {{ font-size: 13px; font-weight: 600; }}
  .legend {{ font-size: 12px; }}
  .note {{ font-size: 11px; }}
"""


def fmt(value):
    return f"{value:,.1f}"


def mib(entry):
    return entry["kib"] / 1024


def bar_path(x, y, width, height, radius):
    """Bar grown from a square baseline at x, with a rounded data end."""
    r = min(radius, width / 2, height / 2)
    return (f"M{x:.2f},{y:.2f}h{width - r:.2f}"
            f"a{r:.2f},{r:.2f} 0 0 1 {r:.2f},{r:.2f}"
            f"v{height - 2 * r:.2f}"
            f"a{r:.2f},{r:.2f} 0 0 1 {-r:.2f},{r:.2f}"
            f"h{-(width - r):.2f}z")


def render(data, theme):
    series = data["series"]
    labels = {s["id"]: s["label"] for s in series}
    body = []
    y = 28
    body.append(f'<text class="title t1" x="{MARGIN}" y="{y}">{escape(data["title"])}</text>')
    y += 20
    body.append(f'<text class="sub t2" x="{MARGIN}" y="{y}">{escape(data["subtitle"])}</text>')

    # Legend: a colored swatch beside ink-colored text.
    y += 22
    x = MARGIN
    for s in series:
        body.append(f'<rect class="s-{s["id"]}" x="{x}" y="{y - 10}" width="12" height="12" rx="3"/>')
        body.append(f'<text class="legend t1" x="{x + 18}" y="{y}">{escape(s["label"])}</text>')
        x += 18 + 7 * len(s["label"]) + 24

    bar_left = MARGIN + LABEL_COL
    bar_max = WIDTH - bar_left - VALUE_COL
    desc = []
    y += 20
    for panel in data["panels"]:
        y += 18
        body.append(f'<text class="head t1" x="{MARGIN}" y="{y}">{escape(panel["title"])}</text>')
        y += 10
        top = y
        scale = max(mib(v) for v in panel["values"].values())
        for s in series:
            value = mib(panel["values"][s["id"]])
            width = max(bar_max * value / scale, 2.0)
            name = labels[s["id"]].split(" ")[0]
            baseline = y + BAR / 2 + 4.5
            body.append(f'<text class="cat t1" x="{MARGIN}" y="{baseline:.1f}">{escape(name)}</text>')
            body.append(f'<path class="s-{s["id"]}" d="{bar_path(bar_left, y, width, BAR, RADIUS)}">'
                        f'<title>{escape(labels[s["id"]])}: {fmt(value)} {UNIT}</title></path>')
            body.append(f'<text class="val t1" x="{bar_left + width + 8:.2f}" y="{baseline:.1f}">'
                        f"{fmt(value)} {UNIT}</text>")
            desc.append(f'{panel["title"]}: {labels[s["id"]]} {fmt(value)} {UNIT}.')
            y += ROW
        body.append(f'<line class="rule" x1="{bar_left - 0.5}" y1="{top - 4}" x2="{bar_left - 0.5}" '
                    f'y2="{y - ROW + BAR + 4}" stroke-width="1"/>')
        y += 6

    y += 10
    for line in data["footnote"]:
        y += 16
        body.append(f'<text class="note t2" x="{MARGIN}" y="{y}">{escape(line)}</text>')
    height = y + 14

    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{height}" '
           f'viewBox="0 0 {WIDTH} {height}" role="img" aria-labelledby="chart-title chart-desc">',
           f'<title id="chart-title">{escape(data["title"])}</title>',
           f'<desc id="chart-desc">{escape(" ".join(desc))}</desc>',
           f"<style>{style(theme)}</style>"]
    out.extend(body)
    out.append("</svg>")
    return "\n".join(out) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="fail if an SVG is out of date")
    args = parser.parse_args()
    data = json.loads(DATA.read_text(encoding="utf-8"))
    stale = 0
    for theme, path in OUTPUTS.items():
        svg = render(data, theme)
        name = path.relative_to(ROOT).as_posix()
        if args.check:
            current = path.read_text(encoding="utf-8") if path.exists() else ""
            if current != svg:
                print(f"{name} is out of date; run tools/comparison-chart/chart.py", file=sys.stderr)
                stale += 1
            continue
        path.write_bytes(svg.encode("utf-8"))
        print(f"wrote {name}")
    if args.check and not stale:
        print("charts are up to date")
    return 1 if stale else 0


if __name__ == "__main__":
    sys.exit(main())
