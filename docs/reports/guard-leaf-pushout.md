# guard-leaf-pushout — the 07000014 leaf press moves first, then pushes out

## What was done

**The task's premise, grounded.** guard-stuck-14-corner's "What is left" item 1 blamed the
07000014 leaf-face freeze on the port's probe-reject order (the game "moves first and then
pushes out in the contact pass"). I read both cited functions in full and ported exactly that
order — and in doing so pinned the full context the freeze lives in:

- **The game's order [M].** `Drone_CollisionHandler` 0x140968:204-208 adds the pending delta
  (`drone+0x3e0`, the feet-HITTEST's point 0) to `obj+0x30` BEFORE the contact loop — the move
  applies unconditionally — and `Collide_Intersect` 0x1ebaa8:353-373 resolves the capsule
  afterwards: the DeltaP shifts the HITTEST's five points (+0x10/+0x20/+0x30/+0x50/+0x60) out
  along the contact, the into-face part of the step is removed, the along-face part survives.
- **The collide radius [M].** `NDrone2_Collision` 0x140e90:91-93 stamps the guard's collide
  sphere radius into `drone+0x45c`: 0.4 for the standard guard (0x3e99999a = 0.3 only on the
  crouch-mode `+0xC5 == 0x17` arm; 0.55 on the `+0xD8 == 0xC` arm) — the same radius the bounds
  push (`AINetwork_GetBoundsPushVectorForSphere`, 0x140e90's tail) uses. So the game's pressed
  guard rests IN CONTACT: the center at the face minus 0.4.
- **The old port's rest [L].** The probe-reject parked the walker 0.435 west of the 0x1A leaf's
  west face (x 50.378 vs the face 50.803-50.813) — out of contact, so the press the door's own
  collide function reads never registered.

**The fix.** `Drone::leaf_press_rest` (nf-game/src/drone.rs, `move_by`'s reject arm, gated to
07000014 — the `door_leaf_slide` set-growth precedent; every other level's move path is
branch-identical to before): when the probe rejects and `door_leaf_slide` finds no tangent (the
pure face press), the scan is door_leaf_slide's (the same segment pair at the two heights, the
same door id band `0x100_0000_0000..0x200_0000_0000`), and the body moves to the contact — the
face point backed off 0.4 along the face normal — instead of staying put. The step's length
still lands in `blocked_reach` (the pane drain's absorbed-movement measure). No RNG: the change
adds no draw (pure position math; the Rand count is unchanged by construction).

**What the freeze actually is (new facts, all measured this round).**

- The doorway is a double SLIDE door (kind 2), both leaves hinged at (50.82, 1.19, 52.92):
  0x1A covers z 51.66..53.00 (its west face x 50.742..50.892), the twin 0x19 covers
  z 52.84..54.18 [L the posed-tri AABB dump]. The doorway seals shut; there is no lateral
  escape — the predecessor's "rounds the leaf end" expectation was a wall-gap misread.
- `Door_ReactToHit` 0x135980's kind-2 arm reverses a CLOSING leaf only (`state & 0x10 == 0 →
  return`); a CLOSED slide door ignores presses. The port's `react_to_hit` already matches.
- The doors' ch_open/ch_lock are 0 [L the door dump] — no channel opens them; the only openers
  are a route's door-node reach (the special band: dist < 0.4, or 0.4-1.0 with a class-2 peer
  within 0.4 / the player within 0.55 on the node [M `LinkCreep_AtSpecialNode` 0x159dd0]) or
  the player's `Door_Activate` 0x135be0.
- The pressed walker stands 1.13 from its route's door node (50.403 vs 51.537) — outside the
  1.0 band — and no other drone ever comes within 0.4 of the node ([L] the 60 s two-drone log:
  drone#1 strafes 1.05-3.5 off in the combat-step cycle). **So the no-input press is the
  GAME's own behavior too** — the sweep's HIT row is what the real game does with nobody
  opening the door, not a port bug.
- With the door opened, the walker flows through the opening leaves and completes his mission:
  the route advances (idx 5→6→7→8) and he parks in **PressAlarm 0x8a** [M `NDrone2_DSTATE_
  PressAlarm` 0x168ea8] at (55.18, 1.00, 44.48), 1.80 from the goal — the goal (56.06, 0.01,
  42.91) IS the alarm node [L the flow run]. The game's blocked-route face-and-fire arm
  (GoToGoalPosition 0x165640 cases 9/0xc) aims at `+0x170` = `NDrone2_SetOpponent`'s opponent,
  not the door — it never fires here (no opponent, the follow returns MOVING, not 0xc).

## The pin (red on the old code, green on the new)

`guard_stuck_sweep_live::the_countdown_leaf_walker_presses_in_contact_and_reaches_his_goal_once_
the_door_opens` — real play from the level load (no teleport), the game's own alarm path, then:

1. Phase 1 asserts the press: the walker's pressed rest x crosses 50.39 (the face 50.803 [L]
   minus the collide radius 0.4 [M 0x140e90:91-93] = 50.403).
2. Phase 2 performs the player's act — `Door_Activate` 0x135be0 on both leaves (group 1) — and
   asserts the walker crosses the opened doorway (x > 51.3) and reaches his route's end (the
   PressAlarm park at (55.18, 1.00, 44.48)).

- RED on the old tree (the pre-fix drone.rs checked out, verified `leaf_press_rest` absent, the
  binary rebuilt): `the leaf walker's pressed rest x 50.378 never entered contact (the face
  50.803 [L] minus the collide radius 0.4 [M 0x140e90:91-93] = 50.403; the old reject held
  50.378) — the move-then-pushout order [M 0x140968:204-208] is not applied`.
- GREEN on the new tree: the phase-1 press and the phase-2 reach both pass.
- The existing slide pin (`the_countdown_leaf_walker_slides_along_the_leaf`) stays green.

## What ran

All on the final tree (the branch already contains origin/master — the merge at branch creation
was "Already up to date"), `--test-threads=1`, `--release`:

| binary | result |
|---|---|
| `walkreal_07000014_live` | 7 passed / 0 failed / 3 ignored (101.2 s) |
| `walkreal_07000001_live` | 7/0/1i (96.0 s) |
| `walkreal_07000002_live` | 4/0/1i (89.8 s) |
| `walkreal_07000004_live` | 9/0/1i (15.4 s) |
| `walkreal_07000005_live` | 17/0 (74.0 s) |
| `walkreal_07000006_live` | 6/0/2i (31.2 s) |
| `walkreal_07000007_live` | 6/0 (23.2 s) |
| `walkreal_07000008_live` | 5/0/1i (109.6 s) |
| `walkreal_07000009_live` | 9/0 (15.5 s) |
| `walkreal_0700000a_live` | 13/0/2i (60.0 s) |
| `walkreal_0700000b_live` | 9/0 (60.8 s) |
| `walkreal_0700000c_live` | 3/0/2i (39.0 s) |
| `walkreal_0700000d_live` | 1/0/2i (28.1 s) |
| `walkreal_07000012_live` | 3/0/1i (19.9 s) |
| `walkreal_07000013_live` | 3/0/1i (31.6 s) |
| `walkreal_07000015_live` | 6/0/3i (43.8 s) |
| `walkreal_07000016_live` | 15/0/3i (89.3 s) |
| `guard_stuck_sweep_live` (SWEEP_LEVELS=07000014) | 3/0 (7.5 s; the sweep + both pins) |
| `fix_npc_doors_live` | 1/0 (12.3 s) |
| `fix_guard_first_shot_live` | 1/0 (1.0 s) |
| `fix_guard_weapons_live` | 3/0 (99.3 s) |
| `fix_guard_misc_live` | 6/0 (6.7 s) |
| `stuck_guard_combat_05_live` | 2/0 (33.6 s) |
| `nf-game --lib` | 783 passed / 0 failed / 4 ignored (0.98 s) |

The walkreal battery total: **113 passed / 0 failed / 21 ignored over 17 levels — every
`walkreal_*_live` binary ran green** with the fix live. The first `nf-game --lib` run's single
failure was my own call missing `NF_ISO` (the env failed `NotPresent` at world.rs:4149); with
the env set it is 783/0/4i. A throwaway diagnostic (`zz_diag_leaf.rs`) drove the geometry
findings and was deleted before the commits; it never carried assertions.

## Suite

`tools/test/suite.py` needs a background process, and this sandbox kills background processes
between tool calls (re-proven this session: no `nohup`/`setsid` run survives). The per-binary
table above IS the suite coverage for the touched crate (nf-game's lib + every nf-view live
binary the rules name, all run on the final tree); nf-formats and nf-audio are untouched by the
diff (drone.rs + one test file) and were not run.

## What is left

1. **The other levels' leaf presses.** The press law is the game's on every level; the gate
   covers 07000014 only. Grow the set one level at a time with each level's walks re-run (the
   door_leaf_slide precedent) — 07000007's slide level is the natural next (its slide arm
   already runs; the press arm would extend the same scan).
2. **The press's door-side consequence.** The game's pressed guard now rests IN CONTACT, which
   is what `Door_ReactToHit` (0x135980) reads from `obj+0xD0` — the port's
   `doors.collide_step` should now see the pressing capsule every tick. I pinned the position
   law (the contact rest), not the contact-list contents; a white-box pin on
   `doors.collide_step`'s contact for door[26] during the press would close that loop.
3. **A live PCSX2 measurement of the pressed guard** (the predecessor's live checks 1-2 stand):
   whether the game's guard really jitters ±delta per tick at the leaf face, and whether
   `HITTEST+0x10` (the DeltaP) carries the west push while the contact list is non-empty.

## Live checks for the main agent

1. 07000014, guards alarmed, Bond absent: watch drone#0 (skin 0x05000090, GO_TO_GOAL 0x63,
   goal (56.06, 0.01, 42.91) — the ALARM node) at the 0x1A leaf's west face (~50.4, 52.34).
   Expected per this round's reading: the position jitters east/west by the per-tick delta
   around the face − 0.4 rest, the state stays 0x63, the door stays closed — indefinitely
   (the press cannot open a closed slide door [M 0x135980's kind-2 arm]; ch_open/ch_lock are 0
   [L]). Access: read watch, 60 Hz, ~60 s.
2. The same scene, then Bond uses the door (Door_Activate 0x135be0): expected — the leaves
   slide open (0x19 north, 0x1A south), drone#0 walks through mid-opening and completes his
   route into PressAlarm 0x8a at ~(55.2, 1.0, 44.5). Access: read watch + a screenshot at the
   PressAlarm entry.
3. `drone+0x45c` for that guard during the press: expected 0.4 (0x3f000000) every tick —
   confirms the pinned collide radius [M 0x140e90:91-93]. Access: breakpoint read, any tick
   while the guard presses the leaf.

## Usage

not visible to me

## ntfy failed

All three fallbacks in order, with their exact output:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "guard-leaf-pushout: done …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (exit 28)
2. `python3 -c "… urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'guard-leaf-pushout: done …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
   (the first attempt died with `SyntaxError: bytes can only contain ASCII literal characters`
   on the em-dashes in the payload; the ASCII-only retry reached the URLError above)
3. `wget -q -O- --post-data "guard-leaf-pushout: done …" https://ntfy.sh/nightfire-yoda-8k3x`
   → the call was killed at the 40 s tool timeout; no connection established, no output.
