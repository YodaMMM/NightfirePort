# level-14-cascade-servers-3 — the door-spam fix landed and verified; the walkreal sweep + the suite left; a master regression found on walkreal_07000014

Branch `s/level-14-cascade-servers-3` = origin/s/level-14-cascade-servers-2 (2fae2cc4) + origin/master
(01917076) merged (601ee4df). The sandbox survived the previous task, so only the branch setup ran.

## Done: the door-spam fix (the -2 draft, re-applied and extended with the missing [M] piece)

`crates/nf-game/src/drone.rs`:

1. **`door_bound: Option<u16>`** on Drone (its own line; both literals) — the route's door binding, the
   game's `+0xBA8` door side (the game's one slot is shared with the kick, which the port already keeps
   apart as `kick_sp`). Port value = the NODE whose door was bound (the node⇔door mapping is 1:1 per
   `NDrone_InitDoorNodes` 0x139a70:24-70, so the node comparison is the game's door-object comparison).
2. **The same-node no-op** in `route_reached_special`'s door arm before the bind:
   `if self.door_bound == Some(node) { return true; }` — a re-arrival at a door node whose door already
   stands in the binding returns immediately, no re-bind, no 0x81 re-entry, no +0x5A6 clobber
   [M `NDrone2_ReachedDestNode` 0x153358:151-155 — `if (iVar11 == *(int *)(iVar2 + 0xba8)) return 1;`].
3. **The clears**: the door-open arrival's tail clears the binding next to the working door
   [M 0x153358:170-176]; the flag-4 node arms clear it [M 0x153358:119-121 + the :175 fall-through].
4. **The missing piece the -2 draft lacked** — `DSTATE_OpenDoor`'s ENTER checks `NDrone2_DoorIsOpen`
   FIRST and returns to +0x5A6 at once: no anim, no `NDrone2_OpenDoor`, no timer-C re-arm
   [M 0x1681f8 case 1 → LAB_00168400]. The -2 session's draft only had 1-3 and did not verify; the
   bisect this session (below) shows why: the spam's re-binds come back through the route after the
   tail cleared the binding, and each re-bind landed in the port's full ENTER (anim + attempt + timer C)
   — the game's ENTER makes an open-door re-bind cost nothing.

**The reproduce → fix measurement** (`no_guard_spams_the_opendoor_after_cascade`,
`crates/nf-view/tests/destroy_wreck_07000014_live.rs`, third test):

- Reproduced on the unfixed tree: **108 OpenDoor (0x81) ticks in the 1500-tick window** (the -2 session
  measured 96 in 1200 — same magnitude). The entry trace: every spam ENTER carried `hit_door Some(n)
  open true` — guards re-binding at doors that were already open (the cascade pair 45/46 AND door 23 on
  a looping route).
- After the fix: **9 OpenDoor ticks total — 0 at the cascade pair** (the pair is never re-worked), 1 at
  an open door (the exit boundary of door 23's legit open: the door flips to state 4 one tick before
  the case-3 UPDATE's DoorIsOpen read exits [M 0x1681f8 case 3]), the rest the legit closed-door open.
- The test pins the [M] rule itself: (a) `open_door_ticks <= 4` — OpenDoor at an open door is the exit
  boundary of a legit open only; (b) `pair_ticks <= 8` — at most one enter+exit per door of the pair
  [M 0x153358:151-155]; (c) the total stays at the one-open-per-door scale (≤16); (d) the doors stand
  open (pos01 > 0.99 [L the -2 run]); (e) at least one guard combat-moves west past x −52.4
  [L probe14_guards' crossings 60/1055/1468 — measured this run: 528 guard-ticks west]. The door
  indices are hoisted BEFORE the unlock (the unlock clears ch_lock to 0x0 [L the -2 session]).

## Runs on the final tree

- `destroy_wreck_07000014_live` **3/3** (the two racks-stay-drawn pins from the -2 branch + the new
  door-spam pin) · `door_wait_07000014_live` **1/1** · `interactables_07000014_live` **10/10** ·
  **`nf-game` 655 passed, 0 failed, 5 ignored**.
- The door-path sample of the other levels' walks (my change is universal, so door-heavy walks ran
  first): `walkreal_0700000d_live` 1+1i, `walkreal_07000016_live` **14/14**, `walkreal_07000011_live`
  1+1i — all green with the fix.
- `walkreal_07000014_live`: **4/8 — the 4 failures are a MASTER regression, not this branch's** (see
  below). The passing 4: the east-duct route, the vent family.

## The master regression (found, bisected, NOT fixed here — the next session should)

`walkreal_07000014_live` fails 4/8 on this tree: `the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door`,
`walk_the_infiltration_to_the_delta_exit` (+ its real-damage sister), `walk_the_tripbomb_route_pick_up_place_and_shoot_it`
— all the same wedge: `walk_to the corridor south to the leg-2 spot: 6000-tick cap`, Bond at
(30.33, 0.55, 35.44) with the approach door at (32.0, 2.1, 35.8) OPEN (state 0x4, pos01 1.00, ch_open 0)
and a 0.17 u segment hit blocking the walk ray.

Bisect proof it is not the door fix: with `crates/nf-game/src/drone.rs` swapped back to the -2 branch's
pre-fix version (the -2 session never touched drone.rs, so that file is master's verbatim in this
merge) the same test still wedges. The -2 branch's own walkreal_07000014 was 8/8 [L the -2 report]
BEFORE the master merge — so the break entered with the merge delta (master c0450ef6 → 01917076). The
delta's behaviour-touching commits: `stun-release-no-stun-anims` (63e76f06/fc76a9d3), `burst-fire-held-trigger`
(fd1f5f32), `sniper-spawn-stats-2` (7343fb0e/cbd181f5/b6b15d9c/34809726 — the stats_table seeding in
`scene::build` touches every level's spawn), plus the later batch-1103e merges. cbd181f5 claims 14 8/8
green at its own point, so the break is in a LATER merge. The window ran out before the per-commit
bisect; the next session should run walkreal_07000014 on master and walk the delta commits.

## What's left

1. The full `walkreal_*_live` sweep on the final tree (lesson 7) — the door-heavy sample ran green
   (above); the sweep itself did not fit the window.
2. The full-suite total line (`tools/test/suite.py`) — did not fit.
3. The master-regression bisect on walkreal_07000014 (above).

## Live checks for the main agent (PCSX2, fresh boot)

1. After the cascade, watch a door-waiting guard for ~10 s (gdbwatch Drone_tag+0x10C): expected — 0x81
   entered at most one enter+exit per door of the pair [M 0x153358:151-155 + 0x1681f8 case 3], no
   sustained OpenDoor at the open pair, then combat-move arms walking west; an already-open door's
   re-bind should show 0x81 entered and left within one frame [M 0x1681f8 case 1 → LAB_00168400].
2. The corridor door at (32.0, 2.1, 35.8) on the infiltration route: does Bond pass it at the leg-2
   spot on a real boot? (The port's wedge dump above — if the game passes cleanly, the master
   regression is port-side; if the game also stops, the route/door data reading moved.)

## Usage

- GitHub token: the driver message's `x-access-token:` PAT (HTTPS password for clone/push only; never
  committed).
- Context window / token / cost figures: not visible to me.
