"""Writes THIRD_PARTY_NOTICES.txt for a release zip: every crate linked into the shipped binaries (nf-view's normal
dependency graph for the release features, from `cargo metadata`), with its declared licence and the licence/notice
files from its source folder. Usage: python -I notices.py <workspace dir> <out file> [features]"""

import json
import os
import subprocess
import sys


def main():
    ws, out = sys.argv[1], sys.argv[2]
    features = sys.argv[3] if len(sys.argv) > 3 else "movies"
    meta = json.loads(subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--features", features,
         "--filter-platform", "x86_64-pc-windows-msvc", "--manifest-path", os.path.join(ws, "Cargo.toml")],
        check=True, capture_output=True, text=True).stdout)
    pkgs = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    root = next(i for i, p in pkgs.items() if p["name"] == "nf-view" and p["source"] is None)
    # Walk normal (non-dev, non-build) dependency edges from nf-view.
    seen, stack = set(), [root]
    while stack:
        i = stack.pop()
        if i in seen:
            continue
        seen.add(i)
        for d in nodes[i]["deps"]:
            if any(k["kind"] is None for k in d["dep_kinds"]):
                stack.append(d["pkg"])
    third = sorted((pkgs[i] for i in seen if pkgs[i]["source"] is not None), key=lambda p: (p["name"], p["version"]))
    lines = ["NightfirePort — third-party software notices",
             "",
             "The NightfirePort binaries include the following open-source crates. Each is listed with its declared",
             "licence, followed by the licence and notice files shipped in its source package.",
             "",
             "In-tree code from rerac (ISC) is noticed in THIRD_PARTY_RERAC_LICENSE.txt.",
             ""]
    for p in third:
        lines.append("=" * 100)
        lines.append(f"{p['name']} {p['version']} — {p.get('license') or p.get('license_file') or 'see files below'}")
        if p.get("repository"):
            lines.append(p["repository"])
        lines.append("=" * 100)
        d = os.path.dirname(p["manifest_path"])
        files = sorted(f for f in os.listdir(d)
                       if f.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT")) and os.path.isfile(os.path.join(d, f)))
        for f in files:
            lines.append(f"--- {f} ---")
            with open(os.path.join(d, f), encoding="utf-8", errors="replace") as fh:
                lines.append(fh.read().rstrip())
        if not files:
            lines.append("(no licence file in the package; licence as declared above)")
        lines.append("")
    with open(out, "w", encoding="utf-8", newline="\r\n") as fh:
        fh.write("\n".join(lines) + "\n")
    print(f"{len(third)} crates noticed -> {out}")


if __name__ == "__main__":
    main()
