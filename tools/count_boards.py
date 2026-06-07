#!/usr/bin/env python3
"""Prototype: count end-on boards in a lumber-stack photo with classical CV.

Strategy: each board end is a bright rectangle bounded by darker seams.
  1. Crop to the stack ROI (the photo has shelf/floor clutter around it).
  2. Grayscale + CLAHE, then blackhat morphology to pull out the dark seams.
  3. Subtract seams -> a foreground mask whose connected board faces are split
     where seams run.
  4. Distance transform + local-maxima markers (one per face) -> watershed.
  5. Count watershed regions with a plausible board-face area; annotate.

Usage: python tools/count_boards.py [input.jpg] [--out-dir wood/out]
ROI is expressed in the working (downscaled) frame; tune --roi if needed.
"""
import argparse
import os
import cv2
import numpy as np


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("input", nargs="?", default="wood/wood.JPG")
    ap.add_argument("--out-dir", default="wood/out")
    ap.add_argument("--width", type=int, default=1400)
    ap.add_argument("--roi", default="150,1330,250,760", help="x0,x1,y0,y1 in working px")
    args = ap.parse_args()
    os.makedirs(args.out_dir, exist_ok=True)

    img = cv2.imread(args.input)
    if img is None:
        raise SystemExit(f"could not read {args.input}")
    h, w = img.shape[:2]
    scale = args.width / w
    work = cv2.resize(img, (args.width, int(h * scale)), interpolation=cv2.INTER_AREA)

    x0, x1, y0, y1 = (int(v) for v in args.roi.split(","))
    roi = work[y0:y1, x0:x1]

    gray = cv2.cvtColor(roi, cv2.COLOR_BGR2GRAY)
    gray = cv2.createCLAHE(clipLimit=2.5, tileGridSize=(8, 8)).apply(gray)
    cv2.imwrite(f"{args.out_dir}/1_gray.png", gray)

    # Blackhat pulls out dark structures (the seams) smaller than the kernel.
    k = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (21, 21))
    blackhat = cv2.morphologyEx(gray, cv2.MORPH_BLACKHAT, k)
    cv2.imwrite(f"{args.out_dir}/2_blackhat.png", blackhat)

    # Seam mask: where blackhat is strong. Otsu picks the cut automatically.
    _, seams = cv2.threshold(blackhat, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    seams = cv2.dilate(seams, np.ones((3, 3), np.uint8), iterations=1)
    cv2.imwrite(f"{args.out_dir}/3_seams.png", seams)

    # Board faces = everything that is NOT a seam.
    faces = cv2.bitwise_not(seams)
    faces = cv2.morphologyEx(faces, cv2.MORPH_OPEN, np.ones((3, 3), np.uint8))
    cv2.imwrite(f"{args.out_dir}/4_faces.png", faces)

    # One marker per face via local maxima of the distance transform.
    dist = cv2.distanceTransform(faces, cv2.DIST_L2, 5)
    dist_n = cv2.normalize(dist, None, 0, 1.0, cv2.NORM_MINMAX)
    cv2.imwrite(f"{args.out_dir}/5_dist.png", (dist_n * 255).astype(np.uint8))

    # Local maximum where the value equals its dilation and clears a floor.
    dil = cv2.dilate(dist, cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (13, 13)))
    peak_mask = (dist >= dil - 1e-3) & (dist > 0.30 * dist.max())
    peak_mask = peak_mask.astype(np.uint8)
    n_markers, markers = cv2.connectedComponents(peak_mask)

    # Watershed needs an unknown region; flood markers across the faces.
    markers = markers + 1
    markers[faces == 0] = 0
    cv2.watershed(roi, markers)

    counts = 0
    annotated = roi.copy()
    for lbl in range(2, n_markers + 1):
        mask = (markers == lbl).astype(np.uint8)
        area = int(mask.sum())
        if area < 60 or area > 4000:
            continue
        counts += 1
        M = cv2.moments(mask)
        if M["m00"] == 0:
            continue
        cx, cy = int(M["m10"] / M["m00"]), int(M["m01"] / M["m00"])
        cnts, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        cv2.drawContours(annotated, cnts, -1, (0, 200, 0), 1)
        cv2.circle(annotated, (cx, cy), 3, (0, 0, 255), -1)

    banner = annotated.copy()
    cv2.rectangle(banner, (0, 0), (annotated.shape[1], 40), (0, 0, 0), -1)
    cv2.putText(banner, f"boards detected: {counts}", (10, 28),
                cv2.FONT_HERSHEY_SIMPLEX, 0.9, (0, 255, 255), 2, cv2.LINE_AA)
    cv2.imwrite(f"{args.out_dir}/6_annotated.png", banner)

    print(f"roi: {roi.shape[1]}x{roi.shape[0]}  markers(raw): {n_markers-1}  "
          f"boards(area-filtered): {counts}")


if __name__ == "__main__":
    main()
