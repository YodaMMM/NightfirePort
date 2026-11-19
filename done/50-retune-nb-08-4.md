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

The drop was CHASED to its root in runs 8-11 (each on its own real run, [L]):

- Run 8: the raw press toward the anchor slid Bond AROUND the hole ((112.25, 13.52, −179.63),
  the zaps resumed, death t20879).
- Run 9: the press aimed AT the hole reached the exact run-1 spot (111.998, 13.52, −179.514)
  and the STANDING capsule BRIDGED it — run 1's drop happened under the CROUCH (the smaller
  capsule falls through).
- Run 10 (SHIPPED): the crouch-drop — walk_to the hole, one crouch press, the fall (bounded,
  break at y ≤ 13.2) — feet land 13.086-13.106 ON THE LOWER FLOOR, the zaps STOP (3 zaps
  during the upper-floor walk phases, ZERO in the fight), the best ledger of the window:
  death t21070, 55 hits (the fewest). But the fight ran CROUCHED on the lower floor (the
  second crouch press re-crouched him; eye 13.3 — the ledge blindfold again) and the only
  rays out are the steep rising ones — the t236 launch was the machine's HIGH close pass
  (y 20.4, dist 26.9) and the round self-blasted (−14.00, the west-band class).
- Run 11 (TRIED, REVERTED): forcing the stance standing after the drop (the default-tick
  release, a crouch toggle, an assert) FAILED — the port's crouch latch resists both
  semantics from the walk's side (crouched true after the release ticks AND the press) — and
  the extra ~6 upper-floor ticks caught a missile blast (death t20760, worse).

REMAINING (one bug, in the ignore message): the fight must run STANDING on the lower floor
(eye 13.77 — above the ledge, under the lintel — the proven flat-ray launch windows; the
standing launches landed 101.0 from the upper floor in runs 3-7) instead of crouched. The
stance latch needs the main agent's live check (below).

## What ran (release, --test-threads=1)

| binary | result |
|---|---|
| walkreal_07000008_live (whole) | **6 passed / 0 failed / 1 ignored** (the d3 ledger), 154.50 s on the final tree (149.02 s mid-window) — the d1 real-damage and d2 Agent walks green |
| the d3 test alone | 11 runs this window (the repro + 10 lever runs), each with NF_DMG_TRACE=1; the ledgers are in the ignore message |

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

1. The port's CROUCH LATCH semantics (the one remaining bug): at the bottom terminal,
   crouch Bond (the crouch drop into the lower cabin floor), then find what input state
   STANDS him up: a crouch:false tick, a second crouch:true press, the movement inputs —
   the run-11 measurements from the walk's side: 4 default (crouch:false) ticks no-op AND a
   crouch:true press no-op (bond.crouched stayed true) — is the latch edge-triggered with
   the release edge consumed by the drop's ticks, or animation-locked? The d2 fight's
   ledger ('the movement inputs stand Bond back up') suggests the uncrouch rides the dodge
   episodes' inputs — confirm which input shape flips the latch. With the stand-up answer,
   the fight runs standing on the lower floor (eye 13.77, the proven flat-ray windows) and
   the walk should go green.
2. The cabin floor topology: gdbwatch Bond's feet y at 60 Hz walking east from the ride-end
   spot (111.1, −179.7) — the drop edge at x ≈ 111.99 (feet 13.52 → 13.086) and the lower
   plate's extent (x 111.99-112.25, z −179.16..−179.52 per the runs).
3. The hurt volume's box: the volume's y band (the zaps hit feet 13.3-13.5, not 13.086) and
   its x/z extent — expected: it covers the upper floor plate and stops at the lower.

## ntfy

First round: curl `curl: (28) Connection timed out after 20002 milliseconds`; python urllib
`URLError: <urlopen error [Errno 101] Network is unreachable>`; wget killed at the tool's
90 s limit. Second round (after the final pushes): curl timed out again, python unreachable
again, and the wget fallback **exited 0 — sent**.

## Usage

not visible to me — this session's driver does not expose token, cost or context-window
figures to the worker.
