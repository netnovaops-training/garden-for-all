#!/usr/bin/env python3
"""Draw the three pictures of the "growing vines" header pack (top.svg, bottom.svg, hero.svg).

    python tools/release/make_vines_art.py

Writes rust/junsgarden/packs/header-vines/1.0.0/{top,bottom,hero}.svg. The drawing is made by a
small program with a fixed seed, so it is the same every time; change the numbers here and run it
again, then run tools/release/make_packs.py (the index lists every file's fingerprint).
The pictures use only what the pack reader allows (see docs/page-builder/PACKS.md): paths, ellipses,
groups, classes that start with pk-header-vines-, and the site's colour variables for the bands.
"""
import math, os, random

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "rust", "junsgarden", "packs", "header-vines", "1.0.0")
P = "pk-header-vines-"
VINE = "#3e7c3a"
LEAVES = ["#7cb342", "#9ccc65", "#689f38", "#8bc34a"]


def bez(p0, p1, p2, p3, t):
    u = 1 - t
    x = u**3 * p0[0] + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t**3 * p3[0]
    y = u**3 * p0[1] + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t**3 * p3[1]
    dx = 3 * u * u * (p1[0] - p0[0]) + 6 * u * t * (p2[0] - p1[0]) + 3 * t * t * (p3[0] - p2[0])
    dy = 3 * u * u * (p1[1] - p0[1]) + 6 * u * t * (p2[1] - p1[1]) + 3 * t * t * (p3[1] - p2[1])
    return x, y, math.degrees(math.atan2(dy, dx))


def vines(rng, count, y0, direction, reach, margin=40):
    """`count` vines along the edge at y0; direction +1 grows down, -1 grows up; reach = longest."""
    out = []
    step = (1440 - 2 * margin) / (count - 1)
    for i in range(count):
        x = margin + i * step + rng.uniform(-18, 18)
        length = rng.uniform(0.55, 1.0) * reach
        sway = rng.uniform(10, 20) * rng.choice([-1, 1])
        p0 = (x, y0)
        p1 = (x + sway, y0 + direction * length * 0.33)
        p2 = (x - sway * 0.9, y0 + direction * length * 0.66)
        p3 = (x + sway * 0.15, y0 + direction * length)
        d = "M%.0f,%.0f C%.0f,%.0f %.0f,%.0f %.0f,%.0f" % (*p0, *p1, *p2, *p3)
        k = i % 8 + 1
        w = rng.choice([2.8, 3.2, 3.6])
        out.append('    <path class="%svine %sd%d" pathLength="100" d="%s" fill="none" stroke="%s" stroke-width="%s" stroke-linecap="round"/>' % (P, P, k, d, VINE, w))
        side = rng.choice([-1, 1])
        for t in (0.3, 0.55, 0.8):
            px, py, ang = bez(p0, p1, p2, p3, t)
            a = ang + side * rng.uniform(45, 70)
            rx = rng.uniform(8, 12)
            out.append('    <g transform="translate(%.0f %.0f) rotate(%.0f)"><ellipse class="%sleaf %sd%d" cx="%.1f" rx="%.1f" ry="%.1f" fill="%s"/></g>' % (px, py, a, P, P, k, rx, rx, rx * 0.46, rng.choice(LEAVES)))
            side = -side
        # a small curl at the tip
        px, py, ang = bez(p0, p1, p2, p3, 1.0)
        out.append('    <g transform="translate(%.0f %.0f) rotate(%.0f)"><ellipse class="%sleaf %sd%d" cx="5" rx="5" ry="2.4" fill="%s"/></g>' % (px, py, ang, P, P, k, rng.choice(LEAVES)))
    return "\n".join(out)


def svg(comment, bands, body):
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1440 60">\n  <!-- %s -->\n%s\n  <g class="%sset">\n%s\n  </g>\n</svg>\n' % (comment, bands, P, body)


def main():
    rng = random.Random(2026)
    top = svg(
        "Under the top bar: two green bands with a soft edge, and vines that grow down from it.",
        '  <path d="M0,0 H1440 V22 C1200,34 960,16 720,26 C480,36 240,18 0,24 Z" fill="var(--site-wave-back, #4a7c59)" opacity="0.5"/>\n'
        '  <path d="M0,0 H1440 V15 C1100,28 900,10 720,19 C500,29 260,12 0,17 Z" fill="var(--site-wave-front, #2e7d32)"/>',
        vines(rng, 17, 15, 1, 44),
    )
    bottom = svg(
        "Above the footer. The site flips this upside down, so the vines grow UP from the footer.",
        '  <path d="M0,0 H1440 V24 C1180,12 940,32 700,22 C460,12 220,30 0,20 Z" fill="var(--site-footer-bg, #467554)" opacity="0.5"/>\n'
        '  <path d="M0,0 H1440 V16 C1120,6 880,26 660,15 C440,5 240,22 0,13 Z" fill="var(--site-footer-bg, #467554)"/>',
        vines(rng, 13, 14, 1, 42),
    )
    hero = svg(
        "The bottom edge of the photo banner: the page colour, with vines that grow up into the photo.",
        '  <path d="M0,60 V40 C240,30 480,48 720,38 C960,28 1200,46 1440,36 V60 Z" fill="var(--site-wave-back, #4a7c59)" opacity="0.55"/>\n'
        '  <path d="M0,60 V46 C260,36 500,54 740,44 C980,34 1200,52 1440,42 V60 Z" fill="var(--site-body-bg, #fffff0)"/>',
        vines(rng, 11, 46, -1, 40),
    )
    os.makedirs(OUT, exist_ok=True)
    for name, text in (("top.svg", top), ("bottom.svg", bottom), ("hero.svg", hero)):
        with open(os.path.join(OUT, name), "w", encoding="utf-8", newline="") as f:
            f.write(text)
        print("wrote", name, len(text), "bytes")


if __name__ == "__main__":
    main()
