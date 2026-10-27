# level-14-full-walk-3 — Countdown Part 1's walk past the hood to the exit (in progress)

## WORKLOG entry

level-14-full-walk-3 (2026-10-27): branch s/level-14-full-walk-3 from origin/master (carries
level-14-full-walk-2: the release onto the east vent lane is green, the walkable gate is
up-facing only [M Intersect_CylGeom 0x109da0 0x10a650-0x10a674], the phase-2 hang drift is
ported). `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` is back to #[ignore]
on the leg-1f hood-underside blocker. Plan: clear it from the game's rule [M] (collision, the
lane geometry [D]), walk leg 1f (the lane south over the flush lid, the connector west to the
#301 drop), the ch 0x47 wait, then legs 2..6 to the exit on real input only; un-ignore when
green end to end, else back to #[ignore] with the exact blocker; every other walkreal test
stays green.

## What's done

Setup; this section grows as steps land.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
