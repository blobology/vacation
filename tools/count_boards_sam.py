#!/usr/bin/env python3
"""Count end-on boards with a 'segment everything' model (FastSAM).

Runs FastSAM over the stack ROI, keeps masks whose size/shape look like a
board end (rectangular, within a plausible area band), and counts them.
Writes an annotated overlay. Offline; weights auto-download on first run.

Usage: python tools/count_boards_sam.py [input.jpg]
"""
import argparse
import cv2
import numpy as np
from ultralytics import FastSAM


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("input", nargs="?", default="wood/wood.JPG")
    ap.add_argument("--out-dir", default="wood/out")
    ap.add_argument("--width", type=int, default=2000)
    ap.add_argument("--roi", default="214,1900,357,1086", help="x0,x1,y0,y1 @width")
    ap.add_argument("--model", default="FastSAM-s.pt")
    args = ap.parse_args()

    img = cv2.imread(args.input)
    h, w = img.shape[:2]
    s = args.width / w
    work = cv2.resize(img, (args.width, int(h * s)), interpolation=cv2.INTER_AREA)
    x0, x1, y0, y1 = (int(v) for v in args.roi.split(","))
    roi = work[y0:y1, x0:x1]
    cv2.imwrite(f"{args.out_dir}/sam_roi.png", roi)

    model = FastSAM(args.model)
    res = model(roi, retina_masks=True, imgsz=1024, conf=0.15, iou=0.9, verbose=False)
    r = res[0]
    if r.masks is None:
        raise SystemExit("no masks returned")
    masks = r.masks.data.cpu().numpy()  # [N, H, W] in {0,1}
    H, W = roi.shape[:2]
    roi_area = H * W

    # First pass: size/shape filter, remember each board-like mask.
    cand = []
    for m in masks:
        m = (m > 0.5).astype(np.uint8)
        area = int(m.sum())
        frac = area / roi_area
        if frac < 0.0008 or frac > 0.04:  # too tiny / whole-stack blobs
            continue
        cnts, _ = cv2.findContours(m, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        if not cnts:
            continue
        c = max(cnts, key=cv2.contourArea)
        x, y, bw, bh = cv2.boundingRect(c)
        rect_fill = area / float(bw * bh + 1e-6)   # how rectangular
        if rect_fill < 0.55:
            continue
        cand.append((area, m, (x, y, bw, bh, c)))

    # Greedy overlap dedup: largest first, drop a mask mostly covered by a kept one.
    cand.sort(key=lambda t: -t[0])
    union = np.zeros((H, W), np.uint8)
    kept = []
    for area, m, meta in cand:
        inter = int((m & union).sum())
        if inter / area > 0.40:   # this region is already mostly claimed
            continue
        kept.append(meta)
        union |= m

    # Draw
    ann = roi.copy()
    overlay = roi.copy()
    rng = np.random.default_rng(0)
    for (x, y, bw, bh, c) in kept:
        color = tuple(int(v) for v in rng.integers(60, 255, size=3))
        cv2.drawContours(overlay, [c], -1, color, -1)
    ann = cv2.addWeighted(overlay, 0.45, ann, 0.55, 0)
    for (x, y, bw, bh, c) in kept:
        cv2.drawContours(ann, [c], -1, (255, 255, 255), 1)

    count = len(kept)

    # Clean web overlay (no banner) for embedding in the site.
    web = ann.copy()
    tw = 1100
    web = cv2.resize(web, (tw, int(web.shape[0] * tw / web.shape[1])),
                     interpolation=cv2.INTER_AREA)
    cv2.imwrite(f"{args.out_dir}/sam_overlay_web.jpg", web,
                [cv2.IMWRITE_JPEG_QUALITY, 80])

    # Debug copy with a baked banner.
    cv2.rectangle(ann, (0, 0), (ann.shape[1], 40), (0, 0, 0), -1)
    cv2.putText(ann, f"boards detected: {count}", (10, 29),
                cv2.FONT_HERSHEY_SIMPLEX, 0.9, (0, 255, 255), 2, cv2.LINE_AA)
    cv2.imwrite(f"{args.out_dir}/sam_annotated.png", ann)
    print(f"raw masks: {len(masks)}   kept (board-like): {count}")
    print(f"COUNT={count}")


if __name__ == "__main__":
    main()
