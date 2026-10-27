#!/usr/bin/env python3
"""Resumable wrapper for NightfirePort's tools/test/suite.py logic.

This sandbox reaps every background process between tool calls, so the suite
cannot run detached. This driver records per-binary results into a state file
and skips binaries already recorded, so repeated foreground invocations make
forward progress until every binary has run. Same binaries, same flags, same
600 s per-binary limit, same totals shape as suite.py.

Usage: python3 run_suite_resume.py [--only bin1,bin2] [--state FILE] [--budget SEC]
"""
import json, os, re, subprocess, sys, time

REPO = "/home/z/my-project/NightfirePort"
ENV = dict(os.environ)
ENV["PATH"] = os.path.expanduser("~/.cargo/bin") + ":" + ENV.get("PATH", "")
ENV["PKG_CONFIG_PATH"] = "/home/z/my-project/alsausr/prefix/usr/lib/x86_64-linux-gnu/pkgconfig"
ENV["LD_LIBRARY_PATH"] = "/home/z/my-project/alsausr/prefix/usr/lib/x86_64-linux-gnu"
ENV["NF_ISO"] = REPO + "/Nightfire-onfoot.iso"
ENV["NF_SILENT"] = "1"
LIMIT = 600
BUDGET = 540  # stop starting new binaries after this many seconds in the call

args = sys.argv[1:]
ONLY = set(args[args.index("--only") + 1].split(",")) if "--only" in args else None
STATE = args[args.index("--state") + 1] if "--state" in args else "/home/z/my-project/suite_state.json"
LOG = os.path.join(os.path.dirname(STATE), "suite_resume.log")

state = {"bins": {}}
if os.path.exists(STATE):
    try:
        state = json.load(open(STATE))
    except Exception:
        pass

t0 = time.time()
build = subprocess.run(
    ["cargo", "test", "--offline", "--release", "-j2", "--no-run", "--message-format=json",
     "-p", "nf-game", "-p", "nf-formats", "-p", "nf-audio", "-p", "nf-view"],
    cwd=REPO, env=ENV, capture_output=True, text=True, encoding="utf-8", errors="replace")
bins = []
for line in build.stdout.splitlines():
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("reason") == "compiler-artifact" and m.get("executable") and m.get("profile", {}).get("test"):
        bins.append((m["target"]["name"], m["executable"], os.path.dirname(m["manifest_path"])))
if build.returncode != 0:
    errs = [l for l in build.stderr.splitlines() if l.startswith("error")]
    print("BUILD FAILED:\n" + "\n".join(errs[:20]))
    sys.exit(2)
print(f"build ok: {len(bins)} test binaries in {int(time.time() - t0)} s", flush=True)

tot = {"passed": 0, "failed": 0, "ignored": 0}
bad = []
ran = skipped = 0
with open(LOG, "a", encoding="utf-8") as log:
    for name, exe, pkg_dir in bins:
        if ONLY and name not in ONLY:
            continue
        if name in state["bins"]:
            r = state["bins"][name]
            tot["passed"] += r["passed"]; tot["failed"] += r["failed"]; tot["ignored"] += r["ignored"]
            if r.get("bad"):
                bad.extend(r["bad"])
            skipped += 1
            continue
        if time.time() - t0 > BUDGET:
            print(f"budget reached; stopping before {name}", flush=True)
            break
        e = dict(ENV)
        e["CARGO_MANIFEST_DIR"] = pkg_dir
        s = time.time()
        try:
            r = subprocess.run([exe, "--test-threads=1"], cwd=pkg_dir, env=e, capture_output=True,
                               text=True, encoding="utf-8", errors="replace", timeout=LIMIT)
            out = r.stdout + r.stderr
            status = r.returncode
        except subprocess.TimeoutExpired as te:
            out = (te.stdout or "") if isinstance(te.stdout, str) else (te.stdout or b"").decode("utf-8", "replace")
            running = re.findall(r"^test (\S+) \.\.\. ?$", out, re.M)
            rec = {"passed": 0, "failed": 1, "ignored": 0,
                   "bad": [f"TIMEOUT {name} after {LIMIT} s (last started: {running[-1] if running else '?'})"],
                   "status": "timeout"}
            state["bins"][name] = rec
            json.dump(state, open(STATE, "w"))
            tot["failed"] += 1; bad.extend(rec["bad"]); ran += 1
            log.write(f"===== {name} TIMEOUT\n{out[-4000:]}\n")
            print(bad[-1], flush=True)
            continue
        rec = {"passed": 0, "failed": 0, "ignored": 0, "bad": [], "status": status}
        res = re.findall(r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored", out)
        for p, f, i in res:
            rec["passed"] += int(p); rec["failed"] += int(f); rec["ignored"] += int(i)
        for t in re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M):
            rec["bad"].append(f"FAILED {name}::{t}")
        if status != 0 and not res:
            rec["bad"].append(f"CRASH {name} (exit {status})")
        state["bins"][name] = rec
        json.dump(state, open(STATE, "w"))
        tot["passed"] += rec["passed"]; tot["failed"] += rec["failed"]; tot["ignored"] += rec["ignored"]
        bad.extend(rec["bad"]); ran += 1
        log.write(f"===== {name} ({int(time.time() - s)} s, exit {status})\n{out[-4000:]}\n")
        for b in rec["bad"]:
            print(b, flush=True)

for b in bad:
    print(b)
print(f"ran {ran}, skipped {skipped}")
print(f"{tot['passed']} passed, {tot['failed']} failed, {tot['ignored']} ignored  ({int((time.time() - t0) / 60)} min)")
