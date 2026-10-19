"""Fetches players' bug reports from GitHub Issues and re-runs them (docs/tools/bug-reports.md).

For every open issue labelled `bug-report` not seen before: downloads each NightfirePort-report-*.zip attached to
the issue or its comments into work/bugreports/<issue>/, runs `nf-view --bug-replay <zip> --shot replayed.png`
(NF_ISO must point at the disc image) and writes summary.txt with the issue text, report.txt and the replay result.
Prints one line per new report. Seen issues are kept in work/bugreports/seen.json.

    NF_ISO=<iso> python -I tools/bugreport/fetch.py [--exe <nf-view.exe>] [--all]
"""

import json
import os
import re
import subprocess
import sys
import urllib.request
import zipfile

REPO = "YodaMMM/NightfirePort"
ZIP_RE = re.compile(r"https://github\.com/[^\s)\]\"']+?NightfirePort-report-[0-9-]+\.zip|https://github\.com/user-attachments/files/\d+/[^\s)\]\"']+?\.zip")


def gh_json(args):
    return json.loads(subprocess.run(["gh"] + args, check=True, capture_output=True, text=True, encoding="utf-8").stdout)


def main():
    ws = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    exe = sys.argv[sys.argv.index("--exe") + 1] if "--exe" in sys.argv else os.path.join(ws, "target", "release", "nf-view.exe")
    out_root = os.path.join(ws, "work", "bugreports")
    os.makedirs(out_root, exist_ok=True)
    seen_path = os.path.join(out_root, "seen.json")
    seen = set(json.load(open(seen_path))) if os.path.exists(seen_path) and "--all" not in sys.argv else set()
    issues = gh_json(["issue", "list", "--repo", REPO, "--label", "bug-report", "--state", "open", "--limit", "200",
                      "--json", "number,title,body,author,createdAt,url"])
    new = [i for i in issues if i["number"] not in seen]
    if not new:
        print("no new bug reports")
    for iss in sorted(new, key=lambda i: i["number"]):
        n = iss["number"]
        d = os.path.join(out_root, str(n))
        os.makedirs(d, exist_ok=True)
        comments = gh_json(["issue", "view", str(n), "--repo", REPO, "--json", "comments"])["comments"]
        text = iss["body"] + "\n".join(c["body"] for c in comments)
        urls = list(dict.fromkeys(ZIP_RE.findall(text)))
        lines = [f"#{n} {iss['title']}", iss["url"], f"by {iss['author']['login']} at {iss['createdAt']}", "", iss["body"], ""]
        if not urls:
            lines.append("NO REPORT ZIP ATTACHED (ask the reporter to press F10 / Report crash and attach the zip)")
        for k, u in enumerate(urls):
            zp = os.path.join(d, f"report{k}.zip")
            try:
                urllib.request.urlretrieve(u, zp)
                with zipfile.ZipFile(zp) as z:
                    z.extractall(os.path.join(d, f"report{k}"))
                    lines.append(f"--- report{k}.zip: {', '.join(z.namelist())}")
                rt = os.path.join(d, f"report{k}", "report.txt")
                if os.path.exists(rt):
                    lines.append(open(rt, encoding="utf-8", errors="replace").read())
                if os.path.exists(exe) and os.environ.get("NF_ISO"):
                    r = subprocess.run([exe, "--bug-replay", zp, "--shot", os.path.join(d, f"replayed{k}.png")],
                                       capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=900)
                    lines.append("--- replay")
                    lines.append(r.stdout.strip() or r.stderr.strip()[-2000:])
            except Exception as e:  # keep going: one bad attachment must not stop the others
                lines.append(f"--- report{k}: {e}")
        with open(os.path.join(d, "summary.txt"), "w", encoding="utf-8") as fh:
            fh.write("\n".join(lines) + "\n")
        print(f"#{n}: {iss['title']} — {len(urls)} report zip(s) -> {d}")
        seen.add(n)
    json.dump(sorted(seen), open(seen_path, "w"))


if __name__ == "__main__":
    main()
