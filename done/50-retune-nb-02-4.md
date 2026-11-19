## Your task: `retune-nb-02-4`, 02 castle 00 Agent real-health walk green again (round 4)

Branch from current origin/master; if origin/s/retune-nb-02-3 is not on master, merge it in first. Read docs/reports/retune-nb-02-3.md there and the ignore message on walk_the_castle_all_objectives_real_health_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000002_live binary and interactables_07000002_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-02-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-19T03:24Z

---
## Report (done, 2026-11-19T05:51Z)


---
## Report from the worker (done, 2026-11-19T05:51Z)

# retune-nb-02-4 — 02 castle 00 Agent real-health walk, round 4 — GREEN, ignore removed

Branch `s/retune-nb-02-4` = origin/master 73822d3a0 + the merge of origin/s/retune-nb-02-3
(which was not on master). The task: take the next lever from the round-3 ignore message and
win the bunker-ambush fight that kills Bond at the 12.2 pool floor. **The walk is GREEN —
the ignore is REMOVED from `walk_the_castle_all_objectives_real_health_00agent`** and the
whole binary passes 5/0/0.

## The lever trail (each lever judged on its own real run; every run is a full real-input walk)

- **Run I — the ledger's scoop widening (n 1 → 3, radius 8 → 9): REJECTED, death t34336.**
  The stair-bottom arrival is 24.8 health / 0 armour (t33683, the 0x12 at clip 5 spare 23 —
  the loud tree's measured prefix cost, identical across runs). Scoop 0 walked to the drop
  (−32.2, 4.6, −1.5) (d 4.1) fine; the scoop-1 walk to (−31.6, 4.5, −2.0) stood BACK in the
  sniper's sweep after the anchored wait, and drone #9's second 0x1e round killed Bond
  mid-scoop (the walk's ledger: 39.0 in 2 hits from #9) [L 2026-11-19]. The committed n 1 /
  8.0 shape stays — the extra scoop walks buy shells with ticks Bond does not have in the
  sniper's lane.
- **Run J — the door-band post-fight loot scoop (the stair scoop reverted): the loot is
  EMPTY, and the run reproduced the round-3 ledger exactly.** Nothing qualifies within
  12 u / 2.2 of the band stand (the sniper was already down — the door leg's own in-view
  fights killed him — and his drop is not in reach). The walk died at t37379, #14's single
  12.2 pistol round through the step-out — run A/H's tick to the frame. The run's pool map:
  24.8 stair bottom → 19.4 lane mouth (the stair-bottom block costs 5.4) → 12.2 bunker door
  (the crossing/door leg costs 7.2), and the leg's −7.2 is **#12's four unanswered −1.80
  pips** (d 4.0-10.5, the 6-tick cadence) landed inside his own fight's settle window — the
  door-leg in-view fights one-tapped #10, #9 and #12 with 3 shells before that [L].
- **Run K — the sensor-spot stand-off loop BEFORE the step-out: two free kills, one hp-1
  leak.** The doorway funnel works both ways: the loop (the retune-nb-02-3 peek-answering
  shape) killed #14 and #13 at ZERO pool cost through the door (12.2 held through both
  stands). #15 survived his stand at hp 1, held cover unseen through the loop's five 600-tick
  holds (3000 ticks wasted), and the step-out's own in-view fight ate his point-blank volley
  — dead t41124 at 8.4 [L].
- **Run L — the flush shell: GREEN at 8.4 health.** Before the step-out, the wheel's ONE
  0x1c shell is armed ("the ambush flush shell") and three SHORT holds (240) flush the
  cover-holder; the flush's stands found no peek (the pick broke — the arm cycle itself
  flushed #15's cover cycle), `arm_fullest` re-armed the 0x12, and the step-out's own
  in-view fight then killed the hp-1 survivor at ZERO cost ("returned fire on drone #15
  (killed true) at tick 42268, health 8.4 → 8.4"). The walk cleared the ambush, the
  computer shot (ch 0x48) and geisha-2 (0x4A/0x4B/0x4C) at 8.4 [L 2026-11-19].

The sim is deterministic (the shared bit-exact RNG, fixed 60 Hz), so the committed tree
walks green every run; the binary whole run below is the proof run.

## What landed (all inline in `crates/nf-view/tests/walkreal_07000002_live.rs`, d3-gated)

1. **The sensor-spot peek-answering stand-off loop** — after the ch-0x81 sensor status,
   BEFORE the step-out walk: the shooter-first pick within 16 u, `hold_for_sight(600)`,
   `settle_aim`, `stand_and_fire`, the 8.0 ring, 8 iterations, the rotate-on-failure shape
   from retune-nb-02-3. This is the ledger's own "the peek loop closes the funnel at ~0 cost
   per peek", moved one phase earlier so the step-out stops paying #14's free round.
2. **The flush shell** — when live guards remain within 16 u after the loop, the wheel's
   one 0x1c shell is armed (the one-tap mechanic), three 240-tick holds cycle the pick, then
   `arm_fullest` re-arms the 0x12 for the mop-up loop after the courtyard stand (unchanged).
3. **The door-band post-fight loot scoop** (n 2, radius 12, y_tol 2.2, before the corridor
   rearm) — the ledger's post-fight-loot hook; on the measured runs it finds nothing (the
   drops are not in reach) and its net effect on the ledger is nil (run J's death tick
   matched run H's exactly), but it is part of the verified green shape and stays: removing
   it would invalidate the run-L proof for a tree that never walked.
4. The ignore is REMOVED; the test's doc comment carries the round's landing note with the
   [L] citations.

No shared helper under `crates/nf-view/tests/common/` changed (the diff scope is the one
test file + docs), so per lesson 7 the other levels' walkreal binaries were not required.
`nf-game`/`nf-formats`/`nf-audio` source was not touched; the suite below covers them.

## How it was verified

- `walkreal_07000002_live` (whole binary, the FINAL tree, the walk unignored): **5 passed,
  0 failed, 0 ignored** (101.7 s) [L 2026-11-19].
- `interactables_07000002_live` (whole binary): **18 passed, 0 failed** (8.9 s) [L 2026-11-19].
- The ignored-target runs I-L above (each a full real-input walk, judged on its own run).
- The full suite (tools/test/suite.py on the final tree — origin/master has not moved since
  the branch point, so the branch IS master + this task): did not complete — see Suite.

## Suite

**The suite did not produce a total line this window — its build phase failed on sandbox
disk pressure three times (05:10, 05:16, restarted with 5.7 GB freed by deleting the
unreferenced build-hash dirs; the second restart's -j4 build still exhausted the disk at
658 MB free — the ~35 test executables plus the examples all live under
target/release/build, 5.5+ GB) and never reached the run phase, so there are no per-binary
totals to report.** The failing units were different each time (audio_probe /
cover_nodes_live / glass_pass_every_level_live, then probe_crane20 /
fix_context_hints_live / laser_watch_0c_live / rocket_origin_live, then the examples
probe14_mouthfin / probe_0a_beams + hud_blend_gpu_live) — disk-exhaustion flakes, not
code errors: hud_blend_gpu_live compiled clean when rebuilt alone. The task's own named
binaries ran green standalone on the final tree (the figures above). The next window
should run the suite first, before any lever work fills the disk.

## What's left

Nothing for the walk — it is green and unignored. The full suite's run phase is the only
unfinished item (the build could not fit the sandbox disk; see the Suite section).

## Live checks for the main agent

None required this round: every figure above is a sim measurement of the real-input walk.
The standing request from rounds 2-3 (the 07000002 captain step-aim siege watch) stands.

## Sandbox notes

Fresh sandbox: clone + branch + WORKLOG push first; rustup 1.100; the ALSA user prefix
rebuilt (`apt-get download libasound2-dev libasound2t64` + `dpkg-deb -x` into
/home/z/alsa, PKG_CONFIG_PATH + RUSTFLAGS -L); work-slim and the ISO re-fetched (the ISO
size-verified 369823203) and extracted with py7zr (no root, no 7z). The background suite
runner survived between tool calls this window, but its first build died on DISK (650 MB
free): the ~35 test executables live under target/release/build (5.5 GB). A registry-cache
cleanup freed too little and broke the offline extraction (cargo fetch restored it); two
restarts freed the unreferenced build-hash dirs (6.5 GB, which broke four in-flight
compiles once — the third start hit the same wall at 658 MB) and the suite never reached
its run phase within the window.

## Usage

Environment figures for this session: input/output token and context-window figures are
not visible to me; cost is not visible to me.

## ntfy failed

All three fallbacks failed (2026-11-19 ~05:49 UTC):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "retune-nb-02-4: GREEN — the 02 castle
   00-agent real-health walk passes 5/0/0, ignore removed (runs I-L; the sensor-spot peek
   loop + the flush shell)" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. python urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `timeout 45 wget -q -O- --post-data "retune-nb-02-4: done"
   https://ntfy.sh/nightfire-yoda-8k3x` → exit 124 (killed by the timeout), no output,
   no error text. The same round-2/3 signature: ntfy.sh is unreachable from this sandbox.
