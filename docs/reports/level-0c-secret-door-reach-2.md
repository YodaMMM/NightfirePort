# level-0c-secret-door-reach-2 — the report

Branch `s/level-0c-secret-door-reach-2` (based on `origin/s/level-0c-secret-door-reach`, merged
with `origin/master` b44ac857; WORKLOG start entry pushed as 894e5009). Task: write the walk legs
to the secret-door plates #558/#559 through the corridor's north end (the flood-proven component
[D]), open the door #544 through the plates, walk out, and delete the `[plate2 + 0.6]` placement
in `walkreal_0700000c_live`. **The window ended mid-rehearsal: the placement is NOT yet deleted;
the walkreal is untouched and stays green. This report hands the next session a proven route
rehearsal, one proven photograph relocation, and one located blocker.**

## What's done

**1. The route-rehearsal probe `walkreal_probe0c_northend_live.rs` (committed) ran and produced
three decisive facts.**

- **The photograph moves to the hall level — PROVEN.** From the STAIR APPROACH (13.0, 12.4,
  −128.2) the camera (0x54) fired ch 0x1c through the staircase opening: the probe printed
  `photograph from the stair approach = true` (the subject centre (5.49, 7.60, −110.37) [D
  #414/#454]; the same camera-subject channel the display-eye photograph used [M 0x1bc668:14-24]).
  The display basin (6.27) — whose every exit the prior session sealed [D probe rays] — no longer
  needs to be entered at all: objective 2 completes from the hall level, so the staircase descent
  + display-ground legs in `walkreal_0700000c_live` can be REPLACED by the stair-approach
  photograph.
- **The lane south extension is a DEAD END — proven empirically.** The lane floor (12.3) walks
  south from the stair approach to z ~−151.8 (the probe's legs, green, +115 ticks), then the floor
  steps down onto a small 11.27 shelf: Bond stood at (14.57, 12.40, −152.57) on floor 11.27 with
  a wall segment 0.117 ahead and the leg wedged at the 6000-tick cap. The scans agree: the 12.3
  floor is GONE at z −152, x 13–15 ((13/15, −152) show 11.3, no 12.3; (13/15, −153.5) show
  6.3/2.4 — the 2.37 deck pocket [D upperlane columns]). The prior report's "the lane south
  continues to z −158.5, joining the walkway/deck system at x 17" does NOT hold as a walking
  route: the 12.3 floor at (17, −155.5..−158.5) is an isolated pocket — the 13.5 slab caps it on
  the north ((17, −153.5): [13.5, 2.4], no floor under the slab) and the deck's west lip at x
  17.16 stands 1.0 above it (deck top 13.3 vs the lane 12.3 [D upperlane columns + the east
  rays: y 12.4/13.0 hit x 17.16, y 11.6 passes under]) while the jump apex is 0.78 [M player.rs
  3.92 m/s up] — the lip cannot be jumped.
- **The deck chain hangs off the WALKWAY, not the lane.** The scans put the ST-1 walkway (15.7)
  running south to z ~−153 at x 13 (the probe walked its south end, +71 ticks) and the step
  surfaces 14.6 ((13/17, −153.5..−155)) → 13.5 ((17, −152..−153.5)) → the deck 13.3 (x 18..21,
  z −157..−161.5 [D columns]) descending south-east toward the deck. The real route to the plates
  is therefore: [the walkway south end] → the steps → the deck 13.3 → the 9.24 patch (the deck's
  east drop, 13.3 → 9.24 = 4.06 m — inside the proven 4.2 m band walk-off [L 2026-10-30 fresh
  boot]; ~55 falling ticks < the 60-tick damage arm [M player.rs:1928-1931]) → the east ramp
  (the patch's north walk-off, 9.24 → ~4.65 = 4.6 m, ~58 ticks < 60 [M the same law]) → the ramp
  east (4.65 → 3.9 over x 23..29 [D columns]) → the pit jump over x 30..32 (floorless [D the
  same scan]) onto the north-east strip (0.1, x 33..39, z −149..−147.5 [D]; the drop-jump carry
  4.42 m flat [M player.rs; the walkreal header]) → the x-40 bridge (the knee rays clear, one
  post x 41.06 z −147 [D bridge probe]) → the warehouse ground (0.4..0.6) south-east → the
  corridor's NORTH end (step-continuous 0.69 → 0.90 at z −165, x 52.5..55.5 [D secretreach]) →
  south to the plates (the body-touch volume 52.24..53.29, 0.90..3.20, −174.45..−172.73 [D
  #559]).

**2. One blocker located for the next session: the walkway's south lip.** Walking off the
walkway's south edge slowly HANGS the body at (13.24, 15.77, −153.65): grounded=false, a scoped
floor 0.58 below, a segment 0.24 ahead (the walk's hold state; a jump cannot fire airborne — the
probe's pop loop ran 90 ticks without effect). The first fix to try: run THROUGH the lip (a
target 1.5+ past the edge so the walker arrives at full speed — the ballistic walk-off carries
~4.7 m), or fire a grounded jump 2–3 ticks BEFORE the edge. The probe's B–E legs (the patch drop,
the ramp, the pit jump, the strip, the bridge, the warehouse, the corridor, the plates, the E
chain) are already written in the probe behind that lip and validate the whole chain in one run
once the lip crosses.

**3. The order deadlock this resolves (the next session's walkreal edit, in full):** the plates'
deck chain branches off the WALKWAY (one-way down from the start area); the photograph no longer
needs the display basin (fact 1); the vent drop (objective 1) is one-way down to the hall. So the
`[plate2 + 0.6]` placement dies by: (a) replacing leg 3's staircase descent + display-eye
photograph with the stair-approach photograph (the probe's F code, proven); (b) replacing leg 4's
placement with a DOCUMENTED walkway-return stand-in (the hall → the ST-1 walkway south end — the
disc's static data carries no re-ascend link, the game's own exit is HIGH [D USATxt 0x4000063];
live check 1 below may kill this stand-in entirely) followed by the REAL walkway-south → steps →
deck → patch → ramp → pit → strip → bridge → warehouse → corridor legs above; (c) legs 5 (the
plates chain) and 5b (the walk out west) untouched.

## How it was verified

- `walkreal_0700000c_live`: **2 passed, 0 failed** (17.38 s, release, single thread, NF_ISO set)
  — the walkreal is untouched this session, the mission walk stays green.
- `interactables_0700000c_live`: **5 passed, 0 failed** (7.34 s).
- The probe's quoted lines are from its committed source on this tree.
- The full suite (`tools/test/suite.py`) was started per the task brief, but background jobs do
  not survive between tool calls in this sandbox and the suite needs most of a 2-hour window on
  this 2-core box — **no total line exists**; the next session should start it in its first
  minutes on the final tree.

## What's left

1. The walkway-lip crossing in the probe (run-through or the pre-edge jump), then the probe's
   B–E one-run validation of the deck→plates chain.
2. The walkreal edit per "The order deadlock" (a): the stair-approach photograph; (b) the
   documented walkway-return stand-in + the real deck-chain legs; (c) delete the
   `[plate2 + 0.6]` placement, keep legs 5+ untouched; then walkreal_0700000c_live +
   interactables_0700000c_live totals on the final tree.
3. The full-suite total line on the final tree.

## Live checks for the main agent

1. **The hall → walkway re-ascend.** Fresh boot: put Bond on the hall (11.27, ~(11, −120)) after
   the vent drop and walk the lane south (x 13.5, z −115..−153) onto the 11.27 shelf
   (z −152..−153.5) with gdbwatch on the body y: if the real game's body re-ascends anywhere (a
   step, a crate, a door), the walkreal's walkway-return stand-in dies and the whole walk goes
   placement-free.
2. **The walkway south lip in play.** Fresh boot: from the walkway (13.5, 15.7, −152) run south
   off the edge at full speed; record where the body lands (the 14.6 step at (13, −153.5) [D
   columns], the 13.5 surface at (17, −153.5), or the hang the port showed).
3. **The plates approach direction.** Fresh boot: put Bond on the warehouse ground (56, 1.0,
   −164), walk west into the corridor and south to the plates; record ch 2/ch 3 firing and the
   door — this confirms the north-end approach the port's chain already pins.

## Usage

not visible to me (no token, context-window or cost figures are shown in this environment).
