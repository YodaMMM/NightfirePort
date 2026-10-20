"""Chunked full-suite runner for sandboxes that kill background jobs (task 715c).

Same build + per-binary runs as tools/test/suite.py, but results land in a state file as each
binary finishes and a chunk stops before the tool-call window kills it. Re-invoke until it
prints ALL DONE, then the summed tally is the suite's total line.

Usage: python3 tools/test/chunk_suite.py [state.json] [--limit S] [--chunk-budget S]
  defaults: state ../suite715c_state.json, limit 570, chunk budget 30 s (start a new binary
  only while the chunk is younger than this; each binary itself runs up to --limit).
"""
import json, os, subprocess, sys, time

here = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
args = sys.argv[1:]
STATE = args[0] if args and not args[0].startswith("--") else os.path.join(here, "..", "suite715c_state.json")
LIMIT = int(args[args.index("--limit") + 1]) if "--limit" in args else 570
BUDGET = int(args[args.index("--budget") + 1]) if "--budget" in args else 30

env = dict(os.environ)
env["CARGO_TARGET_DIR"] = env.get("CARGO_TARGET_DIR", "/home/z/target-cache")
env["NF_SILENT"] = "1"
if not env.get("NF_ISO"):
    print("warning: NF_ISO is not set; the ISO-reading tests will skip", flush=True)

t0 = time.time()
build = subprocess.run(
    ["cargo", "test", "--offline", "--release", "-j4", "--no-run", "--message-format=json",
     "-p", "nf-game", "-p", "nf-formats", "-p", "nf-audio", "-p", "nf-view"],
    cwd=here, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace")
if build.returncode != 0:
    errs = [l for l in build.stderr.splitlines() if l.startswith("error")]
    print("BUILD FAILED:\n" + "\n".join(errs[:20]))
    sys.exit(2)
bins = []
for line in build.stdout.splitlines():
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("reason") == "compiler-artifact" and m.get("executable") and m.get("profile", {}).get("test"):
        bins.append((m["target"]["name"], m["executable"]))
print(f"build ok: {len(bins)} test binaries in {int(time.time() - t0)} s", flush=True)
t0 = time.time()  # the chunk budget measures the RUN time; the build's cost is the first chunk's alone

state = {}
if os.path.exists(STATE):
    with open(STATE) as f:
        state = json.load(f)

tot = {"passed": 0, "failed": 0, "ignored": 0}
bad = []
for name, exe in bins:
    if name in state:
        continue
    if time.time() - t0 > BUDGET:
        print(f"chunk budget out at {name}", flush=True)
        break
    ts = time.time()
    try:
        r = subprocess.run([exe, "--test-threads=1"], cwd=os.path.dirname(exe), env=env,
                           capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=LIMIT)
        out = r.stdout
        rc = r.returncode
    except subprocess.TimeoutExpired:
        out, rc = "", 124
    line = next((l for l in out.splitlines() if l.startswith("test result")), "test result: CRASH (no output)")
    parts = {}
    for p in line.split():
        if p.isdigit():
            parts.setdefault("n", []).append(int(p))
    nums = parts.get("n", [0, 0, 0]) + [0, 0, 0]
    passed, failed, ignored = nums[0], nums[1], nums[2]
    if rc == 124:
        failed = failed or 1
        bad.append(f"TIMEOUT {name} (>{LIMIT}s)")
    elif rc != 0 or failed > 0:
        fails = [l for l in out.splitlines() if l.startswith("test ") and l.endswith("FAILED")]
        bad.append(f"FAILED {name} ({failed}): " + "; ".join(fails[:6]))
    state[name] = {"passed": passed, "failed": failed, "ignored": ignored, "rc": rc, "s": int(time.time() - ts)}
    with open(STATE, "w") as f:
        json.dump(state, f)
    print(f"{name}: {line.strip()}  ({state[name]['s']} s)", flush=True)

# The tally over every binary recorded so far.
missing = [n for n, _ in bins if n not in state]
for name in state:
    tot["passed"] += state[name]["passed"]
    tot["failed"] += state[name]["failed"]
    tot["ignored"] += state[name]["ignored"]
print(f"recorded {len(state)}/{len(bins)} binaries" + (f", missing {len(missing)}: {', '.join(missing[:8])}" if missing else ""))
print(f"{tot['passed']} passed, {tot['failed']} failed, {tot['ignored']} ignored")
for b in bad:
    print(b, flush=True)
if not missing:
    print("ALL DONE")
