# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## pane-collision-pairing

2026-11-07 08:10 UTC, branch `s/pane-collision-pairing` (from origin/master 22d5836b7).

Task: intact glass panes stop Bond and bullets, and shot-out ones do not.
drone.rs (~line 19300) builds `_pane_parts` for the placed 0x20 panes but never adds
them to the collision world: the shape-key/matrix pairing is unresolved, and with them
in the set, walks/guard sight/dropped items on 01/09/0a/0b/14 stopped at shapes where no
pane stands. Both dead_pane_live.rs tests are ignored. Read done/30-dead-pane-collision-2.md
in the queue first, settle the pairing from the game (Break_Create 0x12c230, Collide_PickObj
0x1eae80, Break_Kill 0x12b970:60), turn the parts on with the death-tick grace, respect each
pane's bullet filter (Collide_FilterBullets 0x1ece30), remove both ignores, pin through real
input. Keep one rule for script-created panes (entity arm) and placed panes.

- [x] setup: branch created from origin/master 22d5836b7, pushed.

- [x] the pairing settled from the game: Break_Create 0x12c230 takes the caller's glist
      (Control_SetGList 0x134960; the placement path passes pCelList[index] / the hash hit,
      NULL and 0xa000-flag rows never create [M 0x1d1b68:77-96]), the quat-0 placement
      matrix, Collide_PickObj's per-cel walk + admission gate, Break_Kill 0x12b970:59-63.
- [x] the census: pane_pairing_census.rs prints 01/09/0a/0b/14 (0b has ZERO 0x20 rows) —
      120/120 parts on their placements, all game-created, no doubles; the all-SP assertion
      passes.
- [x] the ride wired: world.rs PANE_PART + the ray-set skip (the per-cel stand-in) + the
      node-pass flag; drone.rs rides the kind-2 panes with the death-tick grace behind
      PANE_RIDE; BREAK_WORLD_STOP tied to the switch. Ride-on: dead_pane 2/2, glass_pass 3/3,
      walkreal 09 8/8 green; the 01 real-damage trio dies in a guard-melee cascade (master
      green, bisected) — the switch lands OFF, master's walks stay green.
- [ ] left: settle the 01 capsule/melee cascade, flip PANE_RIDE, un-ignore dead_pane (the
      report's Live checks); the suite run with the ride on.

## pane-ride-on

2026-11-07 11:57 UTC, branch `s/pane-ride-on` (from origin/s/pane-collision-pairing 79dad06ea,
merged origin/master 10956badb).

Task: make the intact 0x20 glass really stop Bond on every level. The pairing report's ride is
wired behind `PANE_RIDE` in drone.rs; with it on everything passes except the three 07000001
real-damage walks, which die to guard melee at spots with no pane (a guard-capsule/melee-timing
cascade under capsule-solid panes). Find which pane first changes which guard's path on 01, tick
by tick, against the switch off; read how the game limits a moving body's contact set
(`Collide_PickObj` 0x1eae80 and the room/cel link of a 0x20 object); if the port's capsule walk
sees panes the game's would not, add per-cel scoping for capsules like the stand-in the rays got;
if guards really brush the sheets, re-route the walk or its fights (level-gated helpers, never
lower damage/health). Then `PANE_RIDE` on, remove both dead_pane ignores, keep the kind-0/1
props out of the set, run dead_pane_live, glass_pass_every_level_live, every walkreal_*_live,
the ai binaries and nf-game.

2026-11-07 13:02 UTC — the bisection is settled: the 01 real-damage cascade's first divergence
is tick 683, guard 13 at (−0.87, 6.34, −0.47), and the pane there is k=27 (placement 136,
object (53,113), the thin kind-2 sheet at z −1.167, x −1.9..−0.1). The game's movement query
reads the room flood (start room + portals within queryR·1.5, ≤126 rooms [M Collide_Pick
0x1eb320:125,313-329; NDrone2_Collision 0x140e90]) and the pane links into the room containing
its sphere centre (build_LinkToRoom's 0 is the straddle flag, not a room index [M 0x12c29c;
build_FindCel 0x1c8cb0]) — the game's own guards brush these sheets, so per-cel capsule scoping
is not the fix and the walk is what changes. Landed: finish_fights for the 01 real-damage walks
(the 07000016 opt-in, gated to !tops_up()) — the Operative sister passes under the ride (3.0
health, deterministic); the Agent sister still dies to the unfought west-door SMG pack (#3/#12/
#17, 64.3 combined with no fight window). PANE_RIDE stays OFF, both dead_pane ignores carry the
updated reason, the three real-damage walks verified unchanged on the landed tree (1/0 each).
Report docs/reports/pane-ride-on.md with the live checks (the guard-vs-sheet deflection is the
pinnable one).

## pane-ride-on-2

2026-11-07 16:14 UTC start, branch `s/pane-ride-on-2` (from origin/s/pane-ride-on 0cfd81885,
merged origin/master — already up to date).

Task: give the west-door SMG pack a fight window (gated to 07000001), reach and fix the 00
Agent sister, then PANE_RIDE on + both dead_pane ignores off.

- [x] setup: the sandbox kept the repo/work/ISO from the previous session; branch created from
      origin/s/pane-ride-on and pushed (the alsa.pc prefix needed re-pointing at the extracted
      user prefix and the poisoned target/ needed a clean rebuild — 21 min).
- [x] the d2 death REPRODUCED under the ride: tick 5774, the pack's ledger exactly the report's
      (#3 26.5/12, #12 23.1/17, #17 14.7/9). Root cause found [M]: `shoot_bounded`'s 540-tick
      flank (walker.rs) walked Bond INTO the walled pocket (flank target (-30.8, 6.3, 9.9),
      d 1.0-4.6 to the pack) — ALL of the pack's 64.3 landed on that flank walk; the mouth
      stand before it bled 63 more with clip+spare frozen (aim None the whole fight: the
      guards' muzzle lines pass the pocket opening, Bond's box rays clip its edge).
- [x] the d2 fix PROVEN under the ride (working tree, not landed): the pocket-hold flag
      (opt-in `Walker::flank_hold_x`, flank suppressed west of x -27, bail after 3 holds) +
      the ambush block's band-edge approach ((-26.5, 6.4, 14.6), the corridor's own z >= 13.4
      band, d 3.7-4.5 = outside the x2 TooClose 3.0 band [M Drone_ModBulletDamage 0x139f50])
      + the fight radius 4.2 + exposed-first order → the Agent real-damage walk PASSES with
      PANE_RIDE on (15.0 health + 50 armour, 23811 ticks, the pack killed).
- [x] the 00 Agent sister REACHED under the ride: the first blocker is the door lip — #5
      (0x5000038 shotgun, 24 hp) is ENABLED BY the door-open script (the log pairs "opening
      door (7.9,5.3,11.1) state 0x2" with his first 8-pellet volley ~30 ticks later), so no
      pre-open lip pass can see him; the step ate 92.8 in 8 at d 1.8 (tick 3336). A crack-
      retreat-lure law (open from 1.1 u, retreat to the hold line, then the lip lure) got the
      d3 run past the door at 92.8 health; the remaining blocker is the accumulated hall-phase
      bleed (5.2 left at the slab, death 5826) — the d3 fight economy needs its own session.
- [x] the ghost-d3 coupling: the d2-gated laws broke the RIDE-OFF d3 (death 17321, west
      corridor) even gated to `difficulty == 2` with d3's path byte-master except the
      door_lip_lure fn extraction; one real order bug found and fixed that way (master samples
      bond_z/doorway_step AFTER the lip lure — the lure moves Bond), one riddle left. NOT
      landed: the working tree was reverted to the landed state and the landed tree verified
      green.
- [x] landed tree verified: walkreal_07000001_live 8/0; dead_pane_live 0 passed / 2 ignored
      (unchanged); glass_pass_every_level_live 3 passed / 3 ignored (unchanged).
- [ ] the flip (PANE_RIDE on, both ignores off) — blocked on the d3 bleed audit + the ghost-d3
      riddle; the next session re-lands the two proven laws (the full recipe in
      docs/reports/pane-ride-on-2.md) and bisects the ghost-d3 coupling.
