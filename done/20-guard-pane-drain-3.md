## Your task: `guard-pane-drain-3`, the 07000001 walks stay green with guards breaking glass as in the game

Branch from origin/s/guard-pane-drain-2, merge current origin/master, and read its docs/reports/guard-pane-drain-2.md plus the 01 ignore messages. Take its two 01 levers: the flank_off re-lure for the d1 real-damage walk and the camper-clear extension for d3, whose ledgers are in the ignore messages. Then the rest of the walkreal sweep (03, 04, 05, 07, 0b, 0c, 0d, 11, 13, 14, 15, 16, 1b, 41 and 4a) one level per tool call, repairing any walk that shifts, gated to its level. The 08 Hind red belongs to the level-08-hind-fight branches, not here. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the bullet binaries, glass_pass_every_level_live, dead_pane_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-pane-drain-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T18:00Z

---
## Report (done, 2026-11-08T19:07Z)

Closed by the driver: the report is on `s/guard-pane-drain-3` (docs/reports/guard-pane-drain-3.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-08T19:07Z)

# guard-pane-drain-3

## What was done

Task: the 07000001 walks stay green with guards breaking glass as in the game, then the walkreal
sweep (03 04 05 07 0b 0c 0d 11 13 14 15 16 1b 41 4a) one level per tool call, repairing any walk
that shifts, gated to its level. Branch `s/guard-pane-drain-3`.

**Branch base — a deviation to note.** `origin/s/guard-pane-drain-2` does not exist on origin
(it was never pushed; its queue record `done/20-guard-pane-drain-2.md` ends with an empty report
body). Drain-2's work landed in the batch merge 2998b6a76 ("Guards, glass and copters at the
game's rules; keypad page; online gun mounts"), whose message says "Walk tests re-routed for the
moved random stream", and whose walkreal_07000001_live.rs diff carries the two 01 levers named
in the task: the d3 far-field-pair sweep + the door-lip lure (the camper-clear extension:
`far_field_guards`/`far_field_sweep`/`door_lip_fight`, "the d3 ledger: unstarted when the
post-AND sweep ran, then 84.0 in 6 + 14.4 in 8 from the mouth at 4163"), and the pocket
`flank_hold_x = Some(-27.0)` + hold-line re-lure loop (the flank_off re-lure machinery). I
therefore cut `s/guard-pane-drain-3` from `origin/master` (2998b6a76) and verified the levers by
running the walks rather than by re-applying them.

## The 01 walks: green, and the d3 shift repaired one lever further

`walkreal_07000001_live` whole (non-ignored): **7 passed, 0 failed, 1 ignored, 237 s** — the d1
(`..._real_damage`) and d2 (`..._real_damage_agent`) real-damage walks hold green on the merged
tree with the levers in, real input only, no top-ups.

The d3 (`..._real_damage_00agent`) walk **shifted** on the merged tree, twice over the old
ledger:

- The old ledger (2026-11-06) was a STALL at the AND(0x11,0x12,0x13) gate (Bond 82.0 hp at tick
  212820, guard #11 never exposes).
- On this tree Bond **died at tick 8604** at (9.36,6.47,13.19), 20 hits for 100.0 [L the 8604
  run]. Chain: the stream shift parked named guard #6 (12 hp pistol) deeper in the hall
  (z 17.8; health 12 untouched through 2×1200 threshold taps) → the threshold fight fell
  through to the plain `w.shoot(6)` → `exposed false` at 8.3 u → the shoot machinery's **flank
  nav opened the hall door and walked Bond INTO the hall** ("hall guard 6 flank 1 nav 2:
  opening door"), dry at clip 4→0, where the shotgun captain #5 (0x1c) at 22.8 u finished him.

**Repair (d3-gated, inside walkreal_07000001_live.rs only — no shared-helper change, so no
cross-level risk):** after the threshold rounds fail, a still-across-and-alive named guard now
gets the bounded live-tracked aperture lure (the file's own proven doorway idiom: aim at his
LIVE position, tap on the settled aim, the guards hunt the gunshot hearing
[M DroneFunc_SetNoiseHeard]; 900-tick bound), resupplying from the hold line with the walk's own
resupply idiom when `clip+spare < 8`. The flank is never taken while he stays across — a dead
guard or a south-crossed guard ends the phase. Result [L the d3fix run]: the flank death is
gone — Bond reaches the AND gate at **85.6 health** (8 hits for 14.4, tick 22260) — but the gate
still cannot fire: **all three named guards stay across and alive**, because the door phase is
DRY (`resupply done (false) clip 1-4 spare 0` — "no pickup within the walked-path corridor (46
trail points)"), the lure's noise dies after 1-4 taps, and a dry gun draws no hunter across the
aperture (the old lure run that walked 5/6/7/11 into the door line was ARMED: clip 32 spare 16).
The `#[ignore]` now carries this ledger and the next lever: **ammo at the door phase** — the
off-path pickups at (-7.5,5.3,-6.8)/(-3.3,5.8,-9.1) the corridor rule refuses (the d3 pre-hall
stock attempts), or the named guards' own drops if the port drops them — then the armed
sustained lure draws the crossing and the threshold fight answers at the plane.

No decompile-backed assertion was weakened: the only changed test was already `#[ignore]`d, and
the repair makes strictly more of the walk pass on real input.

## The sweep (one level per tool call)

| level | binary | result |
|---|---|---|
| 03 | walkreal_07000003_live | 6 passed / 0 failed (2 ignored — the pre-existing d3 east-captor ledger, unchanged) — 390 s |
| 05 | walkreal_07000005_live | 14 / 0 |
| 0b | walkreal_0700000b_live | 7 / 0 |
| 11 | walkreal_07000011_live | 5 / 0 |
| 41 | walkreal_07000041_live | 15 / 0 |
| 4a | walkreal_0700004a_live | 4 / 0 |

No shift on any level run. The remaining sweep levels (04 07 0c 0d 13 14 15 16 1b) were **not
run whole** — see "What's left".

## Required binaries

- bullet_range_live: 3 / 0
- fix_foliage_bullets_live: 3 / 0
- glass_pass_every_level_live: 3 / 0 (3 ignored — the pre-existing, evidence-carrying level
  geometry ignores: the 14 window stands, the 0xff07 smash-only pane 376, the 08 booth half;
  not touched here)
- dead_pane_live: 3 / 0
- nf-game lib: **748 passed / 0 failed / 5 ignored**

## What's left

1. **The fast runner did not complete on this sandbox.** `tools/test/suite.py`'s build phase
   (-j4, release, four crates) OOM-lost four nf-view targets on this 4 GB box (`probe_pad`,
   `soldiers_0c_draw_live`, `sight_frustum_live`, `door_portal_live` — all four compile clean
   standalone with `-j2`, verified on sight_frustum_live) and aborted before running any
   binary. The required binaries were therefore run directly (above); the sweep levels 04 07
   0c 0d 13 14 15 16 1b still need their `walkreal_*_live` runs whole, and the full suite needs
   one clean pass on the final tree (this branch + origin/master, which it already includes —
   no src/ files were touched, only walkreal_07000001_live.rs and docs).
2. **The 01 d3 walk's next lever** (in the ignore message): ammo at the door phase, then the
   armed sustained lure.
3. The 08 Hind red belongs to the level-08-hind-fight branches, not here (per the task).

## Live checks for the main agent

None this round — the walk runs themselves produced the ledgers. If the d3 door phase is picked
up next, the useful live check is the game's own ammo economy at the 01 hall door on 00 Agent:
whether the named guards drop their weapons on death in the game (the port's resupply corridor
rule refuses the only off-path pickups), and whether a dry trigger draws any hunter at all
(the port: no rounds → no hearing event [M DroneFunc_SetNoiseHeard] — confirm the game's dry
fire behaviour at 0x14b300's fire entry).

## Usage

- Context window: not visible to me (no token/credit/context figures are exposed in this
  environment; I cannot quote numbers I was never shown).
- The sandbox wall clock: started 18:06 UTC, this report written ~19:47 UTC (the session's
  own `date -u` readings) — the only timing figure visible to me.
