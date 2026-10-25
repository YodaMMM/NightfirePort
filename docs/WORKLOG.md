
## level-14-cutscene-fall

Branch `s/level-14-cutscene-fall` (from origin/master). Window 09:17-11:17 UTC.

Task: 07000014 — the broken cut-scene camera and the fall below the map after it. Bond spots:
the broken cut-scene at (40.87, 2.98, 40.85, -3.179, 0.210) frames 1931/2300 (camera inside
rock, flat blue void); the camera-view console at (33.71, 0.65, -18.11) frame 9909 ("Press
Action for Camera View" shows, nothing happens); panels at (54.21, 1.03, 46.15) frame 5739
(vent/hatch, nothing beneath) and (-88.58, -1.10, 46.15) frame 25538 (wall panel, nothing
behind); the fall: (-49.65, 0.65, 36.85) frame 20889 (black screen) then (-107.07, -70.14,
73.28) frame 21799 — Bond below the map after a broken cut-scene, objective "Sabotage base
security and escape from room". (1) Find which script plays at the cut-scene spot (trigger/SP,
its camera stream) and why the port's camera sits inside rock [M Script_UpdateCamera 0x1db518,
the SP's base matrix, the parent chain, the camera key interpolation]; fix it. (2) The fall:
find the cut-scene's hand-back (the marker entity that re-enables Bond, Player_Enable
0x195498 — the matrix origin placed directly [M :57-68]) and why the port's hand-back pose
lands off the floor; fix it [M]. (3) Real-input tests from the trigger to the hand-back: the
camera stays in the room, Bond is back on the floor, the walk continues.

Progress:
- (09:20) Repo + work/ + ISO survived; branch created from origin/master 8de76e31 and pushed;
  WORKLOG started.
- (09:40) [M root cause] The script base matrix (SCRIPTINFO+0x00) resets to IDENTITY at every
  camera start: `Script_CameraStart` 0x1d9140's first act — before the +0xaa8 flag test — is
  `Mat_IdentityT(SCRIPTINFO)` [M 0x1d916c: the jal's a0 still holds param_1; the decompiler
  dropped the argument; `Mat_IdentityT` 0x1e34c8 zeroes 0x40 bytes and writes the 1.0 diagonal;
  nothing restores it — `Script_CameraEnd` 0x1db348 kills the stream only; the Mat_IdentityT
  caller census has no other Script_* hit]. The base is seeded from the placement transform at
  load [M 0x1d8498 View_RotTransMatrix(obj+0x50, obj+0x30, SCRIPTINFO) + 0x1e5168], and case 4
  (camera) concats it [M 0x1da054-68 flag=1 + 0x1da838-44 Concat(SCRIPTINFO, out)], case 1
  (entities) composes parents + base [M 0x1d9b30-74]. So scripts WITH a camera op key WORLD
  space; scripts WITHOUT one key LOCAL offsets. [D] proof: 07000014's three cut-scene scripts
  (0x06000833/0600062f/06000848, camera-flag 1 [D header]) have keys that sit on real level
  geometry only RAW (the security-centre room at (25.9, 2.16, 36.07); the Kiko hand-back key
  (−55.50, 1.55, 42.01) beside cover nodes #432/#433), while the COMPOSED spots are disc-data
  voids — zero placements and zero glists in (70..105, 70..105) and (−120..−90, 60..85)
  [D census_07000014-style dumps]. The no-camera scripts (wires 0x06000a1b, SafePickUps
  0x060009e2, truck 0x06000a22 — flag 0) keep the composed placement base. This also explains
  the [L]-measured 0700004a landing hand-back (0x060006cf, flag 1: the raw key IS the enable
  spot) — the hash-0 kind-0x19 special case was a workaround for the same missing reset.
- (09:50) Fix: nf-game/src/nis.rs `Script::run` op 0x0d sets `self.matrix = IDENT` at the
  camera start [M 0x1d916c]. The countdown camera test re-pinned to the raw keys (the old
  expectations contradicted the decompile — the TEST was wrong, fixed with the address).
- (10:00) NEW real-input pair cutscene_fall_07000014_live 2/2: the vent scene (trigger #301)
  camera stays in the security-centre room with floor below every sampled tick and completes
  (ch 0x47); the Kiko scene (trigger #381) hands Bond back at (−55.49, 1.10, 42.02) ON the
  floor (the old composed spot fell to y −70 in real play), he stays standing 180 ticks and
  walks west. Regression set green: nf-game lib 547/547; countdown 2/2; exit_aim 2/2; landing
  1/1; ledge 2/2; cutscene_guard_models 3/3; walk_07000014 2/2; walk_07000005 2/2;
  walk_07000016 3/3; walk_0700004a 1/1; walk_07000002/06/07/08 9/9. Final tree = merge
  origin/master (batch-1025c) — 07000014 batch re-run green on it.

## level-14-cutscene-fall-2 (start)
- (11:23) Branch s/level-14-cutscene-fall-2 = origin/master + origin/s/level-14-cutscene-fall
  (c2d9fbc8). Mission: the identity base reset at camera start re-keys 01/08 camera-flag
  scripts whose entity keys are local offsets (6 red test groups on the merged tree). Plan:
  pin the exact rule [M 0x1d916c, case 4 @0x1da054 vs case 1 @0x1d9b30, per-SCRIPTINFO reset
  when a player runs several scripts], fix nis.rs, all named tests green, full suite via
  tools/test/suite.py.
