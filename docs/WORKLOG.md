# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## real-health-00agent-15

2026-11-07 06:20 UTC, branch `s/real-health-00agent-15` (from origin/master 22d5836b7).

Task: the 15 00 Agent walk (low priority). The Agent walk is green on
`s/level-15-agent-doors-2` (docs/reports/level-15-agent-doors-2.md); its 00 Agent sister is
ignored because the route's damage budget exceeds 100 health. Measured costs and levers are in
docs/reports/real-health-00agent-12-13-14-15.md on `s/real-health-00agent-12-13-14-15`: the
opening burst from #15 behind the spawn, and the #526 rim-shot blast, which the door[32]
threshold lever with that guard's dropped 0x1a may avoid. Re-route the way a player would.
Real input only; no top-ups; helpers opt-in. Done = the sister green, or ignored with a
measured reason; every walkreal_*_live; the full suite total line.

- [x] setup: repo cloned, branch created from origin/master 22d5836b7, pushed.
- [06:45 UTC] Both reports read (level-15-agent-doors-2, real-health-00agent-12-13-14-15);
  the sister's ignore message names the levers. The #526 lever implemented in the test body
  (d3-gated): the door[32] guard's own dropped 0x1b (the AR's launcher pile [M Pickup_Handler
  0x18e650's AR drop arm]) lobbed from the threshold at (26.5, 71.2) — 15.2 u off the mine, both
  blasts read zero at (int)d ≥ 15 and ≥ 8 [M Explode_Propagate 0x17c518's truncation]; the fuse
  law pinned TIME 210 ticks [M Bullet_init 0x12c5a0; grenade.rs], bounces never explode
  [M 0x12ce80]. The rim walk/shot now skip when the mine is already gone (a read — the d1/d2
  paths byte-identical).
- [07:05 UTC] The crossing burst measured on this master (the first full d3 run): the fights
  killed #15 at 867 for 63.0 in 9 hits (the 12-13-14-15 ledger reproduced exactly), then death
  at 3610 at (35.11, 41.34, 25.20) to the plank pair #7/#8 (26.3) — the d3 stand at
  door[38]-north can never fire (both guard lines are walled by door[39] from there — geometry,
  not aim).
- [07:35 UTC] The 0xf6 plate probed from the port's own trigger bounds: touch box x 80.31..81.31,
  z −1.18..1.35 [L]. The aperture's south half (z −1.6) passes cold. The sneak run: the close
  fight at 3.6 u killed #15 for 21.0 — then his own grenade popped at Bond's feet for 79.0,
  75 ticks after the kill (total 100.0, dead at 992). The head-shot finisher moved to the
  aperture west side (the line to his advanced spot (76.7, −4.6) is 4.5 u): the run died at 1046
  with the leg's stop drifting into the plate box — the finisher never got its kill.
- [07:55 UTC] Budget closed: crossing 63.0 (or 21.0 + the 79.0 grenade trap) + #462's own-shot
  blast 10.7 (80·(1−13/15) at the ≤14 u shot law) + the planks 26.3 ≥ 100.0 BEFORE #526 on every
  path tried. The sister stays ignored with the ledger and the next levers in the ignore message
  (the coded #526 lob, the sneak + finisher, the plank stand at door[39] north (34.6, 28.0),
  the #462 shot at (int)d ≥ 14). Pushing the WIP + the report; the other walkreal binaries did
  not fit the window (no lib code changed — the diff is one test file, d3-gated blocks only).
