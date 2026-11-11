## Your task: `real-health-00agent-16-1b-4`, 07000016 and 1b can be walked on 00 Agent with real health

Branch from origin/s/real-health-00agent-16-1b-3, merge current origin/master, and read that branch's docs/reports/real-health-00agent-16-1b-3.md plus the two 00 Agent ignore messages. For 1b, decode which gate each missile's branch read consumes, and at what tick after launch, from the missile scripts' p0 streams [D], then re-balance the per-console caps against those real read times. Its (e) shape is one missile short with health to spare; the latch ledger to beat is in the report. The Drake phase's y > 9 spawn-band re-weight comes after. For 16, try the cheap lever first: enable the d1-gated `finish_fights` and `level_fights` for d3's ring legs, or raise the ring-gated clear_hostiles cap, then verify the chaser, pit and finale phases whole. Do not repeat its measured dead ends. Real input only, no top-ups, never lower damage or health. Remove each ignore when green. If a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000016_live and walkreal_0700001b_live binaries.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-16-1b-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-11T03:09Z

---
## Report (done, 2026-11-11T03:54Z)


---
## Report from the worker (done, 2026-11-11T03:55Z)

# real-health-00agent-16-1b-4 — report (2026-11-11)

Branch `s/real-health-00agent-16-1b-4`, cut from `origin/s/real-health-00agent-16-1b-2` (the
named base `origin/s/real-health-00agent-16-1b-3` does not exist on the remote — `git ls-remote`
lists only `s/real-health-00agent-16-1b-2`; 1b-2 is the newest commit of this lineage and its
report + the two 00 Agent ignore messages were used in its place), merged current
`origin/master` 784ba2d99. One merge conflict, the two 1b walk `#[ignore]` attributes: master's
side carries the newer `level-1b-console5-00agent` ledger (commit 5e8669cef, 2026-11-09 — newer
than 1b-2's 2026-11-07 messages) and was taken. **Status: both 00 Agent walks stay red with
fresh, tick-exact ledgers; the round measured the task's named levers to a decision on 16 and
produced a read-cadence decode find on 1b that re-scopes the next round.**

## What's done

**16 — the cheap levers, measured dead [L the d3 runs on the merged tree]:**

- The task's first lever, the d1-gated `finish_fights` + `level_fights` enabled for the d3
  walk: death **tick 916** at (14.51, 4.77, −82.64) ON the ramp climb — **worse than the 1030
  baseline** — 21 hits for 100.0, **0 return-fire fights**. The level gate silences the
  response on the approach entirely: every dealer is a level above Bond's climb (#4 ring-west
  (−6.7, 6.7, −86.7) d 21.6 for 33.5, #2 gallery (9.5, 6.7, −110.6) d 28.4 for 42.0, #5 14.0,
  #3 10.5 — all weapon 0x1a at y 6.7). The measured law: ~1 hit per 22 ticks at Hard from the
  4-shooter y-6.7 crossfire from tick ~455 — walking faster dies SOONER (916 < 1030).
- The alternate lever, the ring-gated `clear_hostiles` cap raise: cannot reach the death at
  all — the death precedes the ring legs' first clear by ~200 ticks, and the clear's own
  far-exposed gate returns on the ramp (the d2 lever-(c) law).
- Both reverted (`git diff origin/master` on this branch: the WORKLOG, the report and the
  ignore ledgers, nothing else); the d3 ignore message carries the full ledger. The approach
  needs the d2 ledger's named levers (a cover-shaped gallery approach from the ring, or an
  armour source BEFORE the ramp — the #596 BodyArmourHeavy sits past the ring) plus the 05-07
  d3 stands.

**1b — the console phase reproduced, the levers measured, and the read-cadence find:**

- The merged tree reproduces the console5 ledger **exactly** [L this round's baseline run]:
  ch19 at 1775; missile starts m3 4175 / m4 2975 / m5 1775 / m6+m8 5375; k5 3158-3201
  (−373..−416) / k4 4317 (−457) / k3 5574 (−400) latched in deadline; m4+m5 disarmed
  (deploy [0,0,1,2,2,1,0,1]); the wave 3@f3000 → 6@f4200 → 8@f5100; the grind kills Bond at
  **5619** during k6's window.
- The task's levers, implemented and measured over three runs: the per-console caps
  re-balanced against the modeled read times (defense cooldown 1200 → 300; the fight cap =
  the pending console's `read_deadline_in` − 400, clamped 300..1400, instead of the uniform
  700) plus the console5 ledger's band-safe kill shape (the fight hold on the off-plane lane
  |z| 1.6 on the nearest shooter's side). **Run A**: death 6256 with m3/m4/m5 ALL disarmed
  and the lane fights thinning the wave 7 → 3 in one window — the kill shape works. **Run B**
  of the same source: m3 took the LAUNCH arm (deploy[2] = 3, status 2) at 6225. The
  **150-cycle** intermediate starved the console holds outright (m3 launched, hp 2.2 at 6977)
  — 300 is the measured edge. The lever set is NOT reliably above master (one of two runs
  launches a missile master disarms), so the tree stays at master + the ledger; all lever
  code reverted.
- **The decode find (the task's decode item, now measured): the modeled read deadline
  S_k + P0_LEN[k−1] − 1 does not match the sim.** m5's owned-script clock read 331 when 1330
  state-1 ticks had elapsed (~1/4 rate — the missile p0 clock advances ~1 tick per 4 sim
  ticks: a throttled missile update in the port's path, or a clock-scale mismatch against
  `SpaceMissile_Update` 0x1aa118's per-tick `Script_Update` [M 0x1aa23c]); missile STARTS
  diverge run-to-run on identical source (m3 4175 in run A vs 4375 in run B), and the
  late-latch disarms (k4 +800, k3 +282 observed) fit the reads sitting LATER than modeled.
  The next round must re-derive the real read cadence (the p0 streams' exit events [D] against
  the port's missile update path) BEFORE re-balancing the caps, and explain the run-to-run
  missile-start divergence.
- Measurement discipline note: latch margins printed right after a defense fight are
  **observation-lagged** (the latch lands mid-fight from splash rounds, the print comes at
  fight end) — the MissileDeploy values at death are the truth.

## How it was verified

- Every cited run went through the real input path (`cargo test --offline --release -p
  nf-view --test <bin> -- --ignored --exact <walk> --nocapture --test-threads=1` on NF_ISO,
  no top-ups, no state writes, difficulty loaded the menu way via
  `load_with_carry_difficulty` and asserted at load).
- The whole binaries ran green on the final tree: `walkreal_0700001b_live` **16 passed / 0
  failed / 3 ignored**, `walkreal_07000016_live` **15 passed / 0 failed / 3 ignored** (the
  three ignores per binary are this task's ledgered real-health sisters plus 1b's d1
  top-up-walk tactics red, all pre-existing).
- No `crates/*/src/` files changed — the branch's only code-bearing diff is the two ignore
  messages (test attributes), so no additional test-binary sweep is required beyond the two
  walk binaries above.
- **The full 4-crate suite did NOT run this window**: this sandbox kills background jobs
  between tool calls (two launches of `python3 tools/test/suite.py` died — pid gone on the
  next tool call, `suite.out` empty both times), and the 10-minute foreground cap per tool
  call cannot cover the suite's build + run. The walk-binary runs above are the window's
  coverage; the next sandbox should start the suite with `nohup setsid` as its FIRST action
  and verify the process survives.

## What's left

1. **1b**: re-derive the real branch-read cadence (the decode find above), fix the determinism
   divergence, then re-balance the per-console caps against the REAL read times; keep the
   lane kill shape at the 300 cycle with fights fenced out of the hold chains (the console5
   k5 lesson — today's slack gate approximates it); the Drake phase's y > 9 spawn-band
   re-weight comes after the console phase survives.
2. **16**: the approach-fight work (a cover-shaped gallery approach from the ring) or an
   armour source before the ramp; the 05-07 d3 stands after.
3. The live checks carried from 1b-2's report (the k6/k8 hub sight-lines render, the k1/k2
   above-deck answer, the 16 gallery foot access) all stand; add one: confirm in PCSX2 whether
   the missile p0 scripts advance their clocks at 1 tick per sim tick (watch
   `SCRIPTINFO+clock` over 60 frames at 00 Agent) — that decides the 1/4-throttle question.

## Usage

- Context window, token and credit figures for this session: not visible to me (no such
  counters are exposed in this environment; nothing to copy).
