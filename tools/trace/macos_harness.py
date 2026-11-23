#!/usr/bin/env python3
"""
1:1 macOS Metal verification harness for NightfirePort (`nf-view --shot`).

Runs headless `--shot` captures across Front-End menus, On-Foot campaign missions,
Driving missions (with throttle), and Multiplayer arenas, verifying exit codes and
pixel statistics (non-blank, non-solid) for each frame.
"""
import argparse
import os
import pathlib
import struct
import subprocess
import sys
import time
import zlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
DEFAULT_ISO = ROOT.parent / "007 - Nightfire (USA).iso"
DEFAULT_OUT = ROOT / "work" / "macos_harness"
DEFAULT_BIN = ROOT / "target" / "release" / "nf-view"

CASES = [
    ("fe_main_menu", ["07000048.bin", "--page", "main", "--frame", "30"]),
    ("onfoot_01_exchange", ["07000001.bin", "--frame", "30"]),
    ("onfoot_05_alpine", ["07000005.bin", "--frame", "30"]),
    ("onfoot_08_nightshift", ["07000008.bin", "--frame", "30"]),
    ("driving_mis01_paris", ["09000001.bin", "--frame", "60", "--drive-throttle", "1.0"]),
    ("driving_mis3_alpine", ["09000005.bin", "--frame", "60", "--drive-throttle", "1.0"]),
    ("driving_mis4_vanquished", ["09000006.bin", "--frame", "60", "--drive-throttle", "1.0"]),
    ("driving_mis11_sub", ["09000002.bin", "--frame", "60", "--drive-throttle", "1.0"]),
    ("driving_mis13a_ultralight", ["09000003.bin", "--frame", "60", "--drive-throttle", "1.0"]),
    ("driving_mis13b_island", ["09000004.bin", "--frame", "60", "--drive-throttle", "1.0"]),
    ("driving_mis13c_boat", ["09000007.bin", "--frame", "60", "--drive-throttle", "1.0"]),
    ("driving_race_bonus", ["09000008.bin", "--frame", "60", "--drive-throttle", "1.0"]),
    ("mp_23_skyrail", ["07000023.bin", "--mp-match", "1,3", "--frame", "60"]),
]


def png_stats(path: pathlib.Path):
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG")
    pos = 8
    w = h = 0
    idat = bytearray()
    while pos + 8 <= len(data):
        ln, ctype = struct.unpack_from(">I4s", data, pos)
        chunk = data[pos + 8 : pos + 8 + ln]
        pos += 12 + ln
        if ctype == b"IHDR":
            w, h = struct.unpack_from(">II", chunk, 0)
        elif ctype == b"IDAT":
            idat.extend(chunk)
        elif ctype == b"IEND":
            break
    raw = zlib.decompress(bytes(idat))
    stride = 1 + w * 4
    lumas = []
    for y in range(0, h, 8):
        row_off = y * stride + 1
        for x in range(0, w, 8):
            o = row_off + x * 4
            r, g, b = raw[o], raw[o + 1], raw[o + 2]
            lumas.append(0.299 * r + 0.587 * g + 0.114 * b)
    mean = sum(lumas) / max(len(lumas), 1)
    var = sum((v - mean) ** 2 for v in lumas) / max(len(lumas), 1)
    return w, h, mean, var ** 0.5


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--iso", type=pathlib.Path, default=DEFAULT_ISO)
    ap.add_argument("--bin", type=pathlib.Path, default=DEFAULT_BIN)
    ap.add_argument("--out", type=pathlib.Path, default=DEFAULT_OUT)
    ap.add_argument("--only", help="Run only cases containing this substring")
    args = ap.parse_args()

    if not args.bin.is_file():
        print(f"ERROR: missing binary {args.bin}; run macos/build_app.sh first", file=sys.stderr)
        sys.exit(2)
    if not args.iso.exists():
        print(f"ERROR: missing ISO or carved dir {args.iso}", file=sys.stderr)
        sys.exit(2)

    args.out.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ)
    env.setdefault("NF_DRIVING", "1")

    failures = 0
    for name, extra in CASES:
        if args.only and args.only not in name:
            continue
        out_png = args.out / f"{name}.png"
        if out_png.exists():
            out_png.unlink()
        cmd = [str(args.bin), str(args.iso), *extra, "--shot", str(out_png)]
        t0 = time.monotonic()
        proc = subprocess.run(cmd, capture_output=True, text=True, env=env)
        dt = time.monotonic() - t0
        if proc.returncode != 0 or not out_png.is_file():
            print(f"[FAIL] {name:26s} rc={proc.returncode} ({dt:.2f}s)")
            print("  stderr:", proc.stderr.strip().splitlines()[-3:])
            failures += 1
            continue
        w, h, mean, std = png_stats(out_png)
        ok = (w >= 640 and h >= 480 and mean > 3.0 and std > 4.0)
        tag = "PASS" if ok else "WARN"
        if not ok:
            failures += 1
        print(f"[{tag}] {name:26s} {w}x{h} mean={mean:5.1f} std={std:5.1f} ({dt:.2f}s) -> {out_png.name}")

    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
