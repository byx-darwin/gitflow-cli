#!/usr/bin/env python3
"""Geometric review for Graphviz-rendered SVG diagrams.

Checks two failure classes in an SVG produced by `dot -Tsvg`:
  1. Label overflow: a <text> element's estimated rendered width exceeds
     its enclosing node's shape width.
  2. Element overlap: two distinct node shapes' bounding boxes intersect.

Usage: review_svg.py <path-to.svg>
Exit code: 0 = clean, 1 = findings present, 2 = usage error.
"""
import sys
import xml.etree.ElementTree as ET

SVG_NS = "{http://www.w3.org/2000/svg}"
OVERFLOW_TOLERANCE = 1.05
AVG_CHAR_WIDTH_RATIO = 0.6
DEFAULT_FONT_SIZE = 14.0


def _bbox_from_points(points_attr):
    pts = []
    for pair in points_attr.strip().split(" "):
        pair = pair.strip()
        if not pair:
            continue
        x_str, y_str = pair.split(",")
        pts.append((float(x_str), float(y_str)))
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    return min(xs), min(ys), max(xs), max(ys)


def _bbox_from_ellipse(el):
    cx = float(el.get("cx"))
    cy = float(el.get("cy"))
    rx = float(el.get("rx"))
    ry = float(el.get("ry"))
    return cx - rx, cy - ry, cx + rx, cy + ry


def node_bbox(node_g):
    poly = node_g.find(f"{SVG_NS}polygon")
    if poly is not None:
        return _bbox_from_points(poly.get("points"))
    ellipse = node_g.find(f"{SVG_NS}ellipse")
    if ellipse is not None:
        return _bbox_from_ellipse(ellipse)
    return None


def text_width(text_el):
    content = text_el.text or ""
    font_size = float(text_el.get("font-size", DEFAULT_FONT_SIZE))
    return len(content) * font_size * AVG_CHAR_WIDTH_RATIO


def boxes_overlap(a, b):
    ax0, ay0, ax1, ay1 = a
    bx0, by0, bx1, by1 = b
    return ax0 < bx1 and bx0 < ax1 and ay0 < by1 and by0 < ay1


def collect_nodes(root):
    nodes = []
    for g in root.iter(f"{SVG_NS}g"):
        if g.get("class") != "node":
            continue
        title_el = g.find(f"{SVG_NS}title")
        name = title_el.text if title_el is not None else "<unnamed>"
        bbox = node_bbox(g)
        if bbox is None:
            continue
        nodes.append((name, bbox, g))
    return nodes


def review(svg_path):
    tree = ET.parse(svg_path)
    root = tree.getroot()
    findings = []

    nodes = collect_nodes(root)

    for name, bbox, g in nodes:
        bbox_w = bbox[2] - bbox[0]
        for text_el in g.findall(f"{SVG_NS}text"):
            w = text_width(text_el)
            if w > bbox_w * OVERFLOW_TOLERANCE:
                findings.append(
                    f"overflow: node '{name}' label '{text_el.text}' "
                    f"estimated width {w:.1f} exceeds shape width {bbox_w:.1f}"
                )

    for i in range(len(nodes)):
        for j in range(i + 1, len(nodes)):
            name_a, bbox_a, _ = nodes[i]
            name_b, bbox_b, _ = nodes[j]
            if boxes_overlap(bbox_a, bbox_b):
                findings.append(
                    f"overlap: node '{name_a}' and node '{name_b}' bounding boxes intersect"
                )

    return findings


def main(argv):
    if len(argv) != 2:
        print("usage: review_svg.py <path-to.svg>", file=sys.stderr)
        return 2
    findings = review(argv[1])
    if not findings:
        print("CLEAN: no label overflow or element overlap detected")
        return 0
    for finding in findings:
        print(f"✗ {finding}")
    print(f"FAILED: {len(findings)} finding(s)")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
