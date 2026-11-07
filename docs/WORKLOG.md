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

2026-11-07 13:14 UTC — mp-team-aim-marker: branch cut from origin/master 10956badb; WORKLOG restored from the latest running copy (s/pane-ride-on) and entry started. Task: wire the Check_Target 0x1a7810 multiplayer arm — class-3 aim-ray hits ask MP_isObjectOnTeam 0x18b720 and set the marker bytes +0x133/+0x132, class-2 asks about the body behind ext+0xc and decides the hostile latch; the port has mp_is_object_on_team (crates/nf-game/src/mp/rules.rs) with no caller. First: settle the MPSettings gating words the decompile reads (_388_4_ and _408_4_) against the port comment, find the +0x132/+0x133 readers, then port the arm (no Rand draws, state equal on every machine), pin teammate/enemy/team-objective aim through a real team match, and run every mp_*_live binary, mp_world_sync_live and nf-game.

2026-11-07 14:17 UTC — mp-team-aim-marker: the Check_Target 0x1a7810 multiplayer arm landed in the crosshair block (crates/nf-view/src/scene.rs). Settled the MPSettings words: _388_4_ is DECIMAL 388 = 0x184 (master_copy — MP_Start 0x183a50:70 copies the master enable; the struct comment said +0x388), _408_4_ = 408 = 0x198 (friendly_fire); corrected both struct comments. The class-3 arm (live players — Player_Init 0x1910f8:76 / MP_ReSpawn 0x185368:83; the bots ride MP_ReSpawn [M BOT_respawn 0x124e38:77-79]) marks BLData+0x132/+0x133 when master_copy && friendly_fire && mp_is_object_on_team (0x18b720 — now called), and takes NO aim latch (the +0x2a/+0x2c4 stores are class-2-only — the disassembly: 0x1a7aa8/0x1a7ab4 run before the bnez 0x1a7ab0); the ray excludes the aimER (Collide_RayIntersect takes the aiming obj in param_1). Objective models are class 0x35 (MP_CreateObject 0x185de8:122) — no arm. FIXED an inverted SP guard the disassembly contradicts: levels 0x7000005..0x7000008 SKIP the weapon check and always mark (0x1a7ad0-0x1a7adc `bnez $v1, 0x1a7b48` straight to the mark stores) — the port marked never there. Tests: mp_team_aim_marker_live (team match: teammate human/bot → 3, enemy → 2, FF-off → 2, objective → 2, no latch; Assassin gate; CTF base) 3/3. mp_*_live batch + full suite next.

2026-11-07 14:11 UTC — mp-team-aim-marker: the report drafts at docs/reports/mp-team-aim-marker.md. Every mp_*_live binary green (28 binaries, pass counts in the report), mp_world_sync_live 8/8, the nf-game mp binaries green (mp_never_mentioned 6, mp_match_rules 18, mp_teams_flow 4, mp_hud_pane 5, mp_hud_16 7, mp_debriefing 10, mp_status_bits 3, mp_hud_data 4, mp_lockstep 0-on-this-path, mp_arenas_live 1, mp_map_objects_live 4), the SP crosshair tests unchanged (civilian_no_shoot 3, fix_player2 5, fix_player3 6). mp_stress_explosives_live SIGKILLs on this 4 GB sandbox with AND without the branch (verified against the parent commit) — environmental. Full suite running on the final tree.
