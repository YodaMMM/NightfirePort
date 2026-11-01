# WORKLOG

## level-0d-interactables-2
Started 2026-11-01 19:52 UTC. Branch s/level-0d-interactables-2 from origin/master 6042ee53.
Task: interactables_0700000d_live.rs has 11 tests; using interactables_0700000b_live.rs as the model,
add one live test per KIND of player-touchable placement in 0700000d (doors, switches/consoles,
fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, player-triggered SimpleScripts),
each driven through the real path (walk/E/shoot) with a sister that fails on the wrong result; fix any
port gap the tests expose, with evidence. Run interactables_0700000d_live, walkreal_0700000d_live and
nf-game; quote the totals.
Plan:
1. Census of 0700000d placements [D every placement kind].
2. Map each kind to the port's class Update/Activate; find what 0b's tests cover that 0d lacks.
3. Add live tests per kind + sisters; fix port gaps with [M]/[D] citations.

Work log:
- (19:52) branch created, WORKLOG started; setup in progress (work-slim + ISO download).
- (20:20) setup redone (sandbox reset): clone, work-slim, ISO, rustup, alsa prefix (alsa.pc
  prefix made absolute + libasound.so repointed at the system runtime — pkg-config filtered the
  default-dir -L and the -dev symlink dangled); suite.py started in the background.
- (20:35) census check vs docs/levels/0700000d-audit.md + §7: the 11 existing tests cover 0xEA
  walk-in/use, 0xDB doors, 0xE2 SS (channel half), 0xD9 SPs, 0x3A grapple, 0x3E wires, 0x22
  ladders, 0xFD cams, 0xE1 destroy, 0xF0 pickups, 0xEC mux (via the c5 headline). Added 6 tests
  for the remaining player-touchable kinds: 0xEB re-fire plate #510 (fires in-band, Trigger_Calc
  clears on exit 0x1b1900:50-71), 0xFB MusicTrigger #512 (created ch9→event 13; armed-29 while
  ch 9 on, drains to 0 after; 0x1b31a0), 0x3B DynObject button pair #113/#114 (hidden-at-create
  unhide + mode-0 self-delete on the real E; 0x18dbf8/0x18d3b8), 0x33 Hint #535 (fires at its
  marker after the real C5 shot-open + corridor plate, enable ch 0x28 / blocker ch 0x1e;
  0x1b3380), 0xF9 SoundTrigger #516 (created, gated silent forever — ch 103 has no writer [D
  scan]), 0x3F CreepWall #375 + 0xFA icon #537 (ctx-6 arms from the walk pick, the context E
  starts the hug, ride ch 0x71 held; 0x1a5170:66-97/0x1969d0:85-88).
- (20:52) interactables_0700000d_live 17/0 (was 11), walkreal_0700000d_live 1/0, nf-game 1028/0.
  Rigid bodies (0xD8 #89-91): no sim object in the port (rigid_body.rs is init-math only) — the
  at-rest draw is the census's GAP-0d-2, reported, no fake test.
