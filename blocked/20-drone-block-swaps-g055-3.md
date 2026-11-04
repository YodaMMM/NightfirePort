# drone-block-swaps-g055-3 — the last two real-health walks under the G-055 hunt

Branch from origin/s/drone-block-swaps-g055-2 (merge origin/master into it first) and read docs/reports/drone-block-swaps-g055-2.md. That branch re-lands the three G-055 block swaps and turns four of the six old reds green with cited pins. Two real-health walks still die against the now-faithful hunt:

1. walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage — Bond dies mid west route (~tick 14137, around (-48, 30) after the Kiko band) to the p5=100 hunters (#281/#283). Clear them before the descent legs, or hold the door-wall lane while firing; keep the descent-landing pin (29.99, 0.61, 38.00).
2. walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage — dies at ~9241 on the pit floor to the chaser plus the pit squad. Clear the pit squad (never the chaser) inside the kill loop's backpedal branch.

Real input only; never top up health or force guard state. Shared helpers in tests/common stay opt-in. Done = both green, the four earlier fixes still green, every walkreal_*_live green, then the full suite total line.

---
claimed-by: sandbox1 at 2026-11-04T10:07Z

---
## Report (blocked, 2026-11-04T11:40Z)

---
## Report (blocked, 2026-11-04T11:50Z)
Branch `s/drone-block-swaps-g055-3` (pushed through 2900a2b3; the full report is
`docs/reports/drone-block-swaps-g055-3.md` on the branch). In short: g055-2 merged with
master (one conflict resolved); the two target real-health walks are NOT green yet —
07000016 diagnosed to the route-economy dry-ammo root with a measured attempt chain pushed
(the kill-loop side-fire, the opt-in finish_fights escalation — fights kill true — the
leg-wise hostile clears with corpse walks; next levers in the WORKLOG), 07000014 not
reached (binary 8/1 — its earlier fixes green). The four earlier fixes verified on the
merged sim (nf-game whole: 1051/0/8). Two master-merge shifts documented
(07000003/07000011). The 948-binary nf-view run did not fit the window.
