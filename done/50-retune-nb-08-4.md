## Your task: `retune-nb-08-4`, 08 whole-mission 00 Agent real-damage walk green again (round 4)

Branch from current origin/master; if origin/s/retune-nb-08-3 is not on master, merge it in first. Read docs/reports/retune-nb-08-3.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. If no untried lever is left, scan the armour and health pickups near the death first, then sweep the idle wait before the fatal fight (each wait shifts the guards' roll). Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-08-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-19T07:18Z

---
## Report (done, 2026-11-19T08:10Z)

# retune-nb-08-4 — report

Branch `s/retune-nb-08-4` = `origin/s/retune-nb-08-3` tip 88887393d (contains `origin/master`
73822d3a0). Task: the ignored 00 Agent whole-mission real-damage walk
`walk_the_whole_mission_on_real_input_real_damage_00agent`; take the next untried lever from
the ignore message (lever (3) the Hind outlast) and judge it on its own real run. Still red
at the end of the window, but the fight is now winnable and the launches land — the ledger
and the next lever (one mechanism, high confidence) are in the ignore message.

## Baseline reproduced

The 08-3 ledger death reproduced exactly on the first run (deterministic): death t21777 at
(111.94, 13.09, −179.18), 61 hits for 100.0 — #16 28.0/5, the y-46 trio 36.0 (#14 16.0/4 +
#11 12.0/3 + #2 8.0/2), world 18.0/34 — Bond arrives at the cabin fight with exactly 18.0
health [L the NF_DMG_TRACE run, d3_repro.log].

## Lever (3) executed: the flight-log pin + the launch-geometry fix

The flight log pinned the machine's route bands and overturned the ledger's phase theory:
on the lever-1 phase the machine NEVER flies the HIGH band (y 25-29) in the fight window —
y 12.8-20.5 at every 30-tick sample — the LOW band is the whole window. The launches were
starved by the STANCE, not the phase [L the NF_DMG_TRACE runs]:

1. The d2 slit walk (`difficulty >= 2`) parked Bond on the cabin's step edge at
   (110.98, 13.09, −179.32) — wedged 700 fight ticks (x parked 110.982→110.985), standing IN
   the terminal's hurt volume (10 −2.00 zaps, frames 20731-21778), crouched, ZERO launches,
   clip 4 intact — the ledgered t21777 death.
2. The fight loop's walk-back to the d3 anchor OVERWRITES the aim's turn while it runs
   (its `input.look = 0.0`): with Bond parked off the anchor the aim froze — aim_yaw_err
   2.404→3.035 for 500 fight ticks (cross_still 101→501) while the machine's low passes
   crossed launchable bearings (the t0 sight stops t0.894-0.896 ≥ 0.8) [L the run
   d3_lever2.log].
3. The CROUCHED eye (13.3) is blindfolded at the anchor: the terminal's exterior ledge (its
   top y ≈ 13.3-13.5, 1.5-3 u out) stops EVERY bearing's ray at t0.04-0.15 [L the crouched
   16-ray sweep, the UP ray t0.15], while the same bearings from the STANDING eye (14.2)
   read CLEAR 30 u [L the t0 sweep]. The only rays the crouch lets out rise steeply — the
   machine's high close passes — and that launch climbed into the west band above the
   doorway slot and detonated 5.5 u from Bond (−20.00, run 2) [L d3_lever2.log].

The shipped d3 package (every piece d3-gated; the d1 real-damage and d2 Agent sisters
byte-exact by construction — the whole-binary run below proves it):

- The slit walk is `difficulty == 2` only (the d2 fight's own lever; the d2 anchor IS the
  slit).
- d3 walks to the fight anchor BEFORE the loop (`walk_to`, the walker's wedge mitigation) —
  the walk-back never runs, the aim tracks from tick 0 (measured: yaw_err 2.2 → 0.004 by
  t100).
- The fight STANDS on d3 (the crouch-open is `difficulty == 2` only) — the standing eye
  clears the ledge, the flat rays thread the doorway slot.
- The LOW-PASS OUTLAST gate joins `ready` on d3: `cp[1] <= 25.0` — the flight-log bands
  (LOW 13-24 = the launch windows, HIGH 25-29 = the band-stopped no-windows) make the
  outlast explicit; the eye/lane sight gates dominate, so the gate is the documented hold
  rule, not a new behaviour.

**Measured effect [L the runs d3_lever3..7.log]: the launches fire and land.** Rocket 1
launches at t36 from the standing eye (14.22) at the machine's LOW pass (y 12.8-13.7, dist
45-48 u) and LANDS 101.0 damage at t192-198 with ZERO self-damage — five runs in a row
(clip 4 → 3). The fight is winnable from here.

## The remaining death (measured, runs 3-7)

The walk_to cannot STEP Bond down: its arrival is the x/z radius, and every target tried —
r 0.6 at (112.5, −179.4), r 0.45 at (112.4, −179.3), r 0.5 at (113.0, −179.2), r 0.3 and
r 0.1 at (112.0, −179.5) — "arrives" on the UPPER floor (feet 13.52) short of the hole edge
at x ≈ 111.99. The hurt volume's band zaps the upper floor (−2.00 every 6-24 ticks, feet
y 13.3-13.5; the lower floor feet 13.086 is zap-free [L run 1's in-place drop y 13.52 →
13.0856 at (111.99, −179.52)]): 8-10 zaps drain the arriving 18.0 health mid-fight —
rockets 1 landed (101.0), 2 launched, death t20993-t21077 [L the runs].

NEXT LEVER (in the ignore message): finish the drop with the fight loop's own raw press —
after the walk_to arrival, press forward toward the anchor with the walk-back's input shape
for ≤ 80 ticks or until feet y ≤ 13.2 (the run-1 precedent: the raw walk-back's continuous
press walked Bond ONTO the hole and gravity dropped him; the walk_to stops AT its target).
Bond then fights from feet 13.086 — zap-free, standing eye 13.77 (above the ledge, under
the lintel) — and the proven low-pass windows land the remaining 5 rockets (clip 3 +
carried 4 = 7 rounds for 5 hits needed).

## What ran (release, --test-threads=1)

| binary | result |
|---|---|
| walkreal_07000008_live (whole) | **6 passed / 0 failed / 1 ignored** (the d3 ledger), 146.23 s — the d1 real-damage and d2 Agent walks green on the final tree |
| the d3 test alone | 8 runs this window (the repro + 7 lever runs), each with NF_DMG_TRACE=1; the ledgers are in the ignore message |

The changed file is the 08 test file only (no crates/*/src/, no tests/common/), so no other
binary's timeline can move; the whole-binary run above is the verification the gating needs.

## What did not run

- The full four-crate suite (`tools/test/suite.py`): this sandbox kills background jobs
  between tool calls (confirmed twice this window — the nohup'd build died with 0 cargo
  processes), so the suite cannot run alongside the lever iterations; the disk (9.9 GB, the
  same constraint the previous two windows recorded) rules out the four-crate build. The
  whole walkreal binary + the d3 lever runs are what fit. No src changes exist to move
  library tests.
- The sister walkreal binaries (03/11/12/…): their walks share no file with this change
  (the change is inside walkreal_07000008_live.rs only).

## Live checks for the main agent

1. The cabin floor topology (the next lever's target): during a fresh-boot 08 playthrough
   at the bottom terminal (Bond in the cabin after the ride), `gdbwatch` Bond's feet y at
   60 Hz while walking east from the ride-end spot (111.1, −179.7) toward the anchor
   (112.5, −179.4) — expected: the drop edge at x ≈ 111.99 (feet 13.52 → 13.086, the
   measured hole) and the lower plate's extent (the traces cover x 111.99-112.25,
   z −179.16..−179.52). The exact hole/plate geometry decides the raw-press target.
2. The hurt volume's box: `fieldprobe`/gdbwatch the world-damage call at the cabin — the
   volume's y band (the zaps hit feet 13.3-13.5, not 13.086) and its x/z extent — expected:
   the volume covers the upper floor plate and stops at the lower.

## ntfy

(written before the send — see the WORKLOG entry for the outcome)

## Usage

not visible to me — this session's driver does not expose token, cost or context-window
figures to the worker.
