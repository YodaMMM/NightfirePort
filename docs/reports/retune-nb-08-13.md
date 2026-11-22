# retune-nb-08-13 — 08 whole-mission 00-Agent real-damage walk, round 13

Branch `s/retune-nb-08-13`, cut from origin/master (6a600cb02) with `s/retune-nb-08-12` merged in
first (clean). Still red — the walk dies t29801 (frame 29803) at (113.08, 13.09, −179.88), CROUCHED
on the trench — but round 13 moved the death **3525 ticks deeper** (t26276 → t29801) and shipped the
ledger's lever (2), the crouch-window stance plan: the sustained hurt-volume burst (the −24.0 that
killed round 12) is now taken as 3 rows (−6.0), and every stalemate-phase volume row after it is
crouched through.

## What was done

1. **The ledger's lever (1) — the zap-free standing spot OFF the hole's rim — was skipped as
   measured-dead by its own cited source.** Round 6 measured it on three lever runs: the stand
   press pops the standing capsule to feet 13.470367 at EVERY spot ((112.8, −179.15), (113.08,
   −179.02), round-5's hole footprint and plate edge) — "the trench is NARROWER than the standing
   capsule; no standing spot exists on the lower floor. The 13.77-eye fight is unreachable"
   [L docs/reports/retune-nb-08-6.md, point 4]. The lever's premise (a standing spot holding
   feet 13.086 off the hole centre) contradicts that measurement, so the window took lever (2).
2. **SHIPPED lever (2) — the crouch-window stance plan** (d3-gated, in
   `crates/nf-view/tests/walkreal_07000008_live.rs`; the only file changed). The fight's `zap_now`
   was hoisted out of the zap detector block (behavior-identical; the d1/d2 sisters read the same
   value). A stance scheduler runs before the tick: on a fresh volume row while standing AND no
   launch possible (`cp[1] > 13.7 || cp[0] > 111.0`), `burst_hold = 160` holds the crouch (press
   until flipped — the shipped stand-off block's own press shape); the stand press fires when the
   launch band re-arms (`cp[1] <= 13.7` AND `cp[0] <= 111.0` — the WEST gate). The 60-tick
   substate lockout [M Player_ChangeSubState 0x190630:48 arms BL+0x960 = 60 on every substate
   change] is waited out inside the hold; no press while the rocket rides or the missile cam is up
   [M scene.rs ride block; Player_Update 0x193600 case 10].
3. **MEASURED on the lever runs [L this window's five runs, 22-26 s each]:** rocket 1 is
   byte-exact (launched t49, landed −101.0, the ride t50-t188, the reply 0 rows — the shipped SE
   timeline is untouched through fight t433; the approach burst's four in-loop rows t11-t35 are
   the shipped standing cost, gated off the crouch by the band test because the dive's launch
   window overlaps them and the 60-tick lockout forbids a t12-crouch → t72-stand). The sustained
   burst (12 rows, t434-t512) took THREE rows (−6.0) then crouched — nine rows avoided. The fight
   then runs as a CROUCHED STALEMATE: the machine's post-dive circuit flies WEST-HIGH (y 16.6-20.3;
   the eye rays stop t0.748 at (101.6, 18.5, −201.6) — 0.052 short of the 0.8 sight gate) and
   EAST-LOW (y 12.65-14.1 at x 150+; the east band stops every ray 0.3-0.5 u out, stops
   t0.015-0.019), so the `cp[1] <= 13.7` west band NEVER re-arms and no rocket 2 ever fires (two
   rocket lines total: the t49 launch and the t188 landing). The remaining bleeds: the w61 cannon
   chips (−2.00 at miss 0.09-0.68, both stances — the dodge gate excludes w61 while rockets < 3
   [M the dodge gate's own line]) and one killing w62 missile blast — the missile's dir is fixed
   at spawn through Bond's PARKED chest, the crouched strafe covers ~0.02 u/tick against the
   missile's 0.7 u/tick closure, a full 24-tick episode achieved 0.56 u of separation, and the
   −6.00 falloff closed the walk at t29801.
4. **TRIED and DEAD on their own runs** (each judged on its own run, then reverted):
   - **The east-dip stand** (the first stand shape, no west gate): the machine's east dip crossed
     the band (t2420, y 13.24) and the stand bought nothing — the east band blocks every eye ray —
     while the machine's state-2 cannon pass chewed the standing capsule (w61 rounds at miss
     0.38-0.68). Death t28257 at (113.05, 13.47, −179.12) [L the lever-1 run].
   - **The salvo override** (release the hold and stand for a closing salvo): the dodge episodes'
     side pick walked Bond 2.3 u WEST onto the band corner — (110.94, 13.50, −180.95), the
     round-11 mullion blast ground — death t29352, 449 ticks EARLIER than the shipped state
     [L the lever-3 run].
   - The WEST gate (the shipped shape) re-measured byte-identical after the revert: death t29801
     (frame 29803) at (113.08, 13.09, −179.88), the same position and frames as the pre-revert run
     [L the lever-2 and lever-4 runs].

## How it was verified

- The baseline on the merged tree reproduced round 12 byte-identical: death t26276 (frame 26278)
  at (113.00, 13.47, −179.31), 64 hits for 100.0, 44.0 world [L the baseline run, 18.16 s].
- Each lever judged on its own real run of the d3 00-Agent walk with NF_DMG_TRACE: the east-dip
  stand t28257, the west gate t29801, the salvo override t29352, the revert re-verify t29801
  (byte-identical frames and position) [L four runs, 22-26 s each].
- The whole `walkreal_07000008_live` binary on the final tree: **6 passed / 1 failed (the target
  walk) in 176.86 s** — the d1/d2 real-damage sisters, the plain walks and both ally tests stay
  green (the plan is d3-gated; the zap_now hoist is behavior-identical; no `crates/*/src/` and no
  `crates/nf-view/tests/common/` change, so the affected-binary set is `walkreal_07000008_live`
  alone) [L the final-tree run].
- The full suite (tools/test/suite.py) did NOT fit the window, for the same environment reason as
  rounds 8-12: background jobs are killed between tool calls in this sandbox (verified this window
  with a nohup'd sleep that did not survive one tool call), and the sandbox was freshly reset —
  clone, the 370 MB ISO download, work-slim, rustup and the py7zr fallback all ran from scratch.
  What ran instead: the whole `walkreal_07000008_live` binary above (6/1, 176.86 s), plus the
  baseline/lever runs. No `crates/*/src/` change this window, so every other binary is
  byte-identical in behavior to the merged 08-12 state, whose numbers are in
  docs/reports/retune-nb-08-12.md.

## What's left (the next window's levers, in priority order)

1. **The launch starvation** — the fight cannot be won on the dive's single launch (the machine
   497 hp, clip 3, and no second launchable pass measured in ~2400 fight ticks): the west-high
   passes' eye rays stop t0.748 (0.052 short of the 0.8 gate) — a widened stop threshold for the
   WEST passes only (its own run), or stand at the west-high passes anyway (the round-6 launches
   FIRED from the rim at the y 18.4/20.8 passes — both damage 0.0 MISS [L the round-6 ledger] —
   whether the guided chase can land on this phase's pass geometry needs the main agent's live
   check).
2. **The missile dodge from the crouched state** — the −6.00-class falloff needs ~2.5-3 u of
   separation; the crouched strafe covers ~1.4-1.7 u over a missile's full flight (the detection
   band widened to the spawn, d ≤ 50, is still short). The standing dodge is required, but its
   side pick must pin ALONG THE TRENCH (the plate runs north-east to x 113.6 [L the round-6 walk
   logs]) — the lever-3 side pick walked west onto the band corner. Judge on its own run.
3. **The reply-cadence live check** (carried from rounds 11-12): after Bond's rocket launch, when
   does the Hind spawn its w62 reply — provoked or scheduled? A gdbwatch on the missile spawn
   during a dive-window launch answers it; it decides whether a delayed launch dodges the reply.

## Live checks for the main agent

- The reply cadence (unchanged, rounds 11-13): a gdbwatch on the w62 spawn after the t49 launch —
  provoked or scheduled?
- The west-high pass geometry: from the standing rim eye (14.18) at the anchor (113.3, −179.3),
  during a machine pass at y ≈ 16.6-20.3 south-west (x 97-116, z −209..−214), which triangle
  stops the eye ray at t0.748 (the stop measured at (101.6, 18.5, −201.6))? If it is the cable or
  a non-wall band, the sight gate's stop threshold for WEST passes may be widened to let the
  launch gates open on those passes.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed from this sandbox (ntfy.sh unreachable — the same failure as rounds
5-12; each attempt at 2026-11-21 ~04:05-04:06 UTC):

1. curl: `curl: (28) Connection timed out after 20002 milliseconds` (rc 28)
2. python3 urllib: killed by the outer 25 s timeout (rc 124) — no response within the 20 s
   urlopen timeout, no traceback, nothing written
3. wget: killed by the outer 25 s timeout mid-connect (rc 124, quiet mode, no output)

The notification text that could not be delivered:
`retune-nb-08-13: the crouch-window lever shipped (the sustained burst crouched), death t26276 ->
t29801 (+3525 ticks) still red to a w62 falloff in the crouched stalemate; whole binary 6/1`
