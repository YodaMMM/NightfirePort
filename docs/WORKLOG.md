# WORKLOG

## real-health-00agent-02-6 (sandbox5, 2026-11-07 05:1x UTC)

Branch `s/real-health-00agent-02-6` created from `origin/s/real-health-00agent-02-5` (fa1a89882).
Goal: walk 07000002 to the end on 00 Agent with real health; levers per the 02-5 blocked report:
(1) lane-mouth sniper duel aimed-stand window, (2) k=0/k=1 volley headroom vs closing #7, (3) crossing ammo.
Next: merge origin/master, read blocked/50-real-health-00agent-02-5.md + the ignore message, start the fast runner.

### real-health-00agent-02-6 progress (06:2x UTC)

- Run 1 (repro on the merged tree): the lip lottery stand held aim Some at 21 u, 25 rifle rounds,
  ZERO hits (the tap spread 21 [M 0x12c5a0] = ±0.62 u; the wall eats the cone off the slit);
  shot 1 +335, shot 2 +501, dead at the lip.
- Run 2: suppression of the lip fight (dmg.cooldown) + the 8.0 ring + the stair-top shotgun arm:
  the ring REGRESSED the pool (55.2 vs 78.0); ONE shell at 13.2 u (the leg's own engage_in_view,
  exposed=true) took the sniper 12.0 → 2.0. Two shells kill him in the band.
- Run 3: the ring back to 6.0, the shotgun armed after the stair-bottom scoop: the sniper DIED to
  the leg's first in-view shell at 13.3 u, 78.0 held through the lane/door; died later in the
  bunker corridor ambush on a dry gun.
- Run 4: + the corridor rearm (arm_fullest after the band stand): **THE WALK PASSED** — the full
  objective route in 22463 ticks, 78.0 health at the bunker, no falls. Ignore removal next.
