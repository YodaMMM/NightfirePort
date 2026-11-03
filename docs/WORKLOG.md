# WORKLOG

## level-14-cascade-servers-4 (started 2026-11-03 17:00 UTC)

Branch = origin/s/level-14-cascade-servers-3 + origin/master merged. Task: the door-spam fix
without breaking guard door crossings — the driver's correction says walkreal_07000014_live is
GREEN on master and the 14 wedge is THIS branch line's own `door_bound` change; merged onto
master the -3 tree turns 11 tests red (npc_doors, fix_guard_misc, walkreal_03 ×2, walkreal_0a
×3, walkreal_14 ×4). Re-check `door_bound` against [M 0x153358:151-155, 0x1681f8 case 3,
DSTATE_OpenDoor's ENTER + NDrone2_DoorIsOpen]: when the game clears +0xBA8 and when a fresh
door is bound.
- 17:05 UTC: the decompile re-read. THE GAME HAS ONE SLOT (+0xBA8) and its lifecycle is:
  bind at the pair's near node (0x153358:151-163, the node's +0x28 door OBJECT, no-op when
  already bound); the state machine (0x1681f8) NEVER clears it; the clears are (a) the far-side
  arrival tail gated on `NDrone2_DoorIsOpen` [M 0x153358:170-176 — which reads the BOUND object:
  0x14aaa0 returns `bound+0xf4 == 4` for a class-0x1c door, TRUE otherwise], (b) the kick arms,
  and (c) **`NDrone2_ControlSTANDARD` 0x1490f0:233-244 — the stale+distance self-heal the port
  already models: 60 ticks after the last `NDrone2_OpenDoor` stamp (+0xBB0), a bound door
  farther than 2.0 u is FORGOTTEN (`+0xba8 = 0`), and one within 2.0 u with DoorIsOpen false is
  RE-ATTEMPTED.** The port's per-tick self-heal (drone.rs, the ControlSTANDARD block) cleared
  only `hit_door` — the -3 session's new `door_bound` no-op key (the SAME +0xBA8 split in two)
  LEAKED: after any interrupt that takes the guard >2 u from the door, every later route pass
  no-ops on the stale `door_bound` while `hit_door` is gone — no re-bind, no door_node, no
  re-attempt: the guard strands before the closed door. That is the 11-test regression.
- THE FIX: the stale+distance clear now wipes BOTH stand-ins (one line + the [M] comment) —
  the game's single-slot lifecycle restored. Next: build, the 11 tests + the spam pin, the
  walkreal sweep, the suite.
- 19:05-19:15 UTC (continued session): the second [M] piece landed at 50bcbeca — the ENTER
  deferral. DSTATE_OpenDoor's ENTER ran BEFORE the port's door-node resolution could land, so
  the -3 DoorIsOpen early-exit read the stale pre-bind latch and bounced EVERY fresh binding
  back to +0x5A6 (the driven guard stood in 0x63 with the reach consumed, no re-attempt). The
  game's order is bind-then-ENTER [M 0x153358:149-163 → 0x1681f8 case 1]: a fresh binding
  defers the ENTER; Drones::tick re-delivers it right after the resolution. Verified 19:14:
  **ALL 11 previously-red tests GREEN** — npc_doors_live 6/6, fix_guard_misc_live 6/6,
  walkreal_07000003_live 5/5 (both route tests), walkreal_0700000a_live 10/10 (all three),
  walkreal_07000014_live 9/9 (all four, incl. the delta-exit wedge and the tripbomb route) —
  AND the pin kept: no_guard_spams_the_opendoor_after_the_cascade green (destroy_wreck
  3/3). Next: the full walkreal sweep, the full suite total line, the report.

## level-14-cascade-servers-2

- Start: branch `s/level-14-cascade-servers-2` from origin/master c0450ef6. Task: on 07000014 finish the destroyed-model swap for the cascade's objects (racks must draw their destroyed models), find why guards spam the door outside the room after the cascade, add real-input tests; run interactables_07000014_live, walkreal_07000014_live, full suite. Previous attempt's research: origin/s/level-14-cascade-servers-and-door-guards (wreck-side swap in, two OpenDoor dispatch fixes in, crossing bisect + suite run left open).
- The racks: the no-swap case landed. [D] obj#273 DESTROY_BoxSatellite glist flags 0x30 (bit 0x10) → Destroy_Create 0x12c318:57-59 zeroes tag[0] → Destroy_Smash 0x12bb60's *piVar1==0 arm skips the swap → the intact glist stands; the port's old census hid the racks behind the per-frame dead gate (they vanished). Now: 0x10-flagged leaves join neither destroy_centres nor the wreck bakes. destroy_wreck_07000014_live 2/2, door_wait 1/1, walkreal_07000014_live 8/8, interactables_07000014_live 10/10, destroy regressions (0d keys 3, channels 1, blast_debris 1, blast_scripts 3) all green.
- The door spam: REPRODUCED with a real-input test (E press → cascade → 0x85 → doors 45/46 unlock and fully open; then 96 OpenDoor 0x81 ticks in a 1200-tick window). Root cause identified [M 0x153358:151-155]: the game's re-arrival at a door node whose door already stands in +0xBA8 is a no-op (return before any state change); the port re-enters OpenDoor on every arrival. Drafted fix (persistent door_bound = the +0xBA8 stand-in, the same-door no-op, the tail clear) did NOT verify within the session — reverted, analysis in the report.

## level-14-cascade-servers-3 (started 2026-11-03 12:26 UTC)

Branch = origin/s/level-14-cascade-servers-2 (2fae2cc4) + origin/master merged (601ee4df; the -2
session's racks no-swap case rides along). Task: land + verify the drafted door-spam fix (the
`door_bound` +0xBA8 stand-in [M NDrone2_ReachedDestNode 0x153358:151-155 + 0x1681f8 case 3]),
`no_guard_spams_the_opendoor_after_the_cascade` green, the racks pin kept, every walkreal_*_live
green, the full suite total line. Sandbox survived the previous task: repo/work/ISO intact, only
the branch setup was needed.
- 12:30-14:00 UTC: the door-spam fix LANDED. The -2 draft (door_bound + the no-op + the clears) alone
  did not drop the count (108 = the -2's 96 shape); the bisect found the missing [M] piece —
  DSTATE_OpenDoor's ENTER checks DoorIsOpen FIRST [M 0x1681f8 case 1 -> LAB_00168400] and an open-door
  re-bind must cost nothing (no anim/attempt/timer C). With it: 108 -> 9 OpenDoor ticks (the pair 0,
  the exit boundary 1); the pin no_guard_spams_the_opendoor_after_the_cascade green with the [M]-shaped
  bounds (the at-open ticks <= 4, the pair <= 8, the total one-open-per-door scale, the west crossing).
  The racks pins stay green; door_wait 1/1; interactables 10/10; nf-game 655; the door-heavy walkreal
  sample 0d/16/11 green. FOUND: walkreal_07000014 is 4/8 on MASTER itself (the corridor wedge at leg 2;
  bisected — the -2 drone.rs in the merged tree still wedges; the -2-only tree was 8/8 [L the -2
  report]) — the delta c0450ef6..01917076 (the stun-release/burst-fire/sniper-spawn-stats/1103e merges)
  carries it; the per-commit bisect is the next session's first item. The full walkreal sweep + the
  suite total did not fit the window. Report: docs/reports/level-14-cascade-servers-3.md.
