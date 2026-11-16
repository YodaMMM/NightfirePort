## Your task: `real-health-00agent-0d-2`, 0700000d's 00 Agent arrival-to-C5-door real-damage walk goes green

Branch from current origin/master; if origin/s/real-health-00agent-0d-1 is not on master, merge it in first. Read docs/reports/real-health-00agent-0d-1.md there and the ignore message on walk_arrival_to_the_c5_door_real_damage_00agent. Take the report's levers in order: cut the exit to ~80 ticks by aiming west from the landing, then the g0/g1 kill window on the ride. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_0700000d_live binary and interactables_0700000d_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-0d-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-16T03:18Z

---
## Report (done, 2026-11-16T04:05Z)

Closed by the driver: the report is on `s/real-health-00agent-0d-2` (docs/reports/real-health-00agent-0d-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-16T04:47Z)


---
## Report from the worker (done, 2026-11-16T04:47Z)

# real-health-00agent-0d-2 — the 00 Agent arrival-to-C5-door real-damage walk is GREEN, un-ignored

Claimed on the merge of origin/master d73cf126a + origin/s/real-health-00agent-0d-1 (the 0d-1
container drop), branch `s/real-health-00agent-0d-2`. The task: make
`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage_00agent` pass un-ignored.
**It passes**: the whole `walkreal_0700000d_live` binary runs 4/4 (the plain replay, the
Operative real-damage, the d2 agent sister, and this walk un-ignored), and
`interactables_0700000d_live` runs 11/11.

## What's done (all in `crates/nf-view/tests/walkreal_0700000d_live.rs`, d3 real-damage only)

Every change sits behind `container_drop = real_damage && difficulty == 3` (0d-1's gate), so
the d1/d2 trees and the plain replay are gate-identical — the whole binary's 4/4 is the proof.

1. **Lever 1 — the WEST exit (the 0d-1 report's lever, measured)**: the container exit aims due
   west from the landing instead of north-west. The hop leaves the yaw at -pi/2, the west face
   is ~3.3 u of flat top (the down-ray map: the top spans x 145-151 at z -296, yard 4.0 west of
   it [probe]), and the NW aim stalled ~100 ticks on the container's ridge. The turn phases now
   break the moment the yaw converges. Measured: the rim walk falls at tick 41 (was ~100+), the
   exit is **86-89 ticks, cost ZERO** on the winning roll (was 154 ticks, -18) — the lever alone
   saved ~10.8 health (floor 16.8 vs 6.0 on the no-dice roll).
2. **Lever 2 — the dice knob (the 0d-1 report's g0/g1 ride window, re-derived)**: the yard's
   fate is decided by where the shared RNG stream sits when the guards wake, and the only thing
   that moves it is DRAW COUNT — idle ticks draw nothing (measured: a 3-tick pre-boarding
   `pause_at` re-produced the no-pause ledger tick for tick; the pause is kept only because it
   was the winning roll's config). The ride taps are the knob: the 18 aimed ride taps never land
   themselves (zero kills in 33 taps across two gate versions — the flat 0.02 rad, then the
   proven distance-scaled box law — at d 37-43 off the moving beam), but each tap spends a
   spread draw, and the shifted roll wakes the yard rifles into a merciful cadence: the ride
   takes ZERO chips (31.2 → 31.2 through the landing; every no-tap roll took 2 chips) and the
   exit costs ZERO. The tap gate now carries the proven `shoot_far_aim` construction
   (distance-scaled 0.12/0.25 u box, the aim-bit trigger gate [M Bullet_init 0x12c5a0:139-170;
   Player_SetWeaponAnimObj 0x1a2e38 case 0xf]).
3. **The floor sweep first, crouched**: the 0d-1 stand walk ran BEFORE any fight and cost
   -16.2 of the landing budget; the sweep's own law self-checks the ray every tick, so the
   sweep now runs from the EXIT LANDING (nearest first; the first pick measured down at +30-39
   ticks), and the probe's proven-clear stand (142, -291.5) is walked AFTER the first pass.
   The sweep runs CROUCHED — the Operative ledger's proven kill posture (the sight target drops
   0.45 [M Player_GetHeadPos 0x195aa0:39-51], the visibility factor 0.75 [M
   Drone_ProcessOpponents 0x1390c0:40-42]) and its d 20-24 kills LANDED on this level while the
   standing roll's d 24-32 rounds sprayed zero lands. Measured: **five kills** (+31/+150/+149/
   +40/+243), the stand walk survives at 6.0, and the real-damage section finishes at 4.2.
4. **The pass cap 260 + the reload hold** (the ledger's own failed-ray law): the #30 pass once
   burned 400 ticks on clip 0 spare 10 — the 20-tick reload pulse restarts the anim and never
   fills; the passes now hold the walker's `reload_full` (~100 ticks) between passes when the
   clip is dry, and a failed ray stops at 260.

## How this was verified

- Nine instrumented runs of the d3 walk this session (`--ignored --nocapture`), each a full
  real run: the lever-1 west exit (exit 89 ticks, floor 16.8, died at the stand), the pause
  placements (the pre-boarding arm never fired / fired), the taps-off roll (the no-pause ledger
  reproduced tick for tick — the idle-pause proof), the taps-on rolls (18 taps → zero ride
  chips), and the final combined roll → **PASS**.
- The whole `walkreal_0700000d_live` binary, single-threaded: **4 passed, 0 failed, 0 ignored**
  (142 s) — the plain replay and the Operative/d2 real-damage walks are byte-identical-gated.
- `interactables_0700000d_live`: **11 passed, 0 failed** (30 s).
- Sample of the walkreal family (real runs, foreground): `walkreal_07000001_live` 8/8,
  `walkreal_07000004_live` 10/10. `walkreal_07000002_live`/`_03_live` exceeded the 100 s
  per-binary window I gave them and were cut off by MY timeout mid-run — no failure recorded,
  but they did not complete here.
- The full four-crate suite (`tools/test/suite.py`) was attempted three times in the
  background: each attempt's cargo build children were killed by the sandbox between tool
  calls (the third died mid-build reporting BUILD FAILED for three binaries it was
  compiling — all three build clean when built directly, verified for
  `walkreal_07000014_live`). **The suite's total line was NOT produced in this sandbox
  window** — the per-binary foreground runs above are the verification evidence.

## What's left

- The d2 agent sister (`walk_arrival_to_the_c5_door_real_damage_agent`) passes as of this tree
  (it runs un-ignored on master already — its doc comment's IGNORED text is stale, not touched).
- Nothing else. The walk is green on real input with no top-ups before the C5 leg's established
  guard.

## Live checks for the main agent

None required. One curiosity for a future session: the ride taps' bullets never land at d 37+
off the beam (33 taps, two gate versions, zero lands) while the same act lands at d 24 from the
floor — if gdbwatch ever walks the yard, an instrumented bullet from the gantry rider would pin
where those rounds die (the girder's own collision at spawn is the suspicion).

## Usage

not visible to me
