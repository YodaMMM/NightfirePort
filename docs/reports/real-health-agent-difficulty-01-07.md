# real-health-agent-difficulty-01-07 — report

Branch `s/real-health-agent-difficulty-01-07`, cut from origin/master 6f16146e.

## What's done

**The Agent pin [M].** The menu's difficulty wheel writes the row value of the `difficulty[]`
global (0x2df4c0, ACTION.ELF; rows read in the ELF: +0xc = 1/2/3, +0x10 = the lock flag) into
GameState+0x28 [M C_SBNFDFCTY 0x216ca8:15], and Txt 0x14d/0x14e/0x14f = "Operative"/"Agent"/"00
Agent" [M menu.rs:846]. So Agent = GameState 2. The damage/stats the pick changes, all already
in the port with the game addresses: Player_HandlePain 0x1902c8 scales Bond's pain by
Plr_DMod_Easy/Normal/Hard per value (01-04 group: Easy 0.5 → Normal 0.7; 05-08 group: Easy 0.6
→ Normal 0.7, tuning.rs); DroneWeap_DoBulletAccuracy 0x179740:74-88 scales the guards' hit
chance (01-04: Accuracy_Easy 0.5 → Normal 0.8; 05-08: 0.8 → 1.1); Drone_Create 0x136be0:49's
min-difficulty gate now spawns the param-2 placements; the captain criterion reads prop 5 at
difficulty 2 [M 0x14b300:504-535]; Lock_Create 0x1ae1f8:67-69's auto-unlock is Operative-only.

**The seven sisters.** Each of walkreal_070000{01..07}_live's real-health walk got an
Agent-difficulty sister (`..._real_damage_agent`, 02's `..._real_health_agent`). The walk body
is parametrized with the difficulty; the difficulty-1 call sites keep the old load expression
byte-for-byte (plain `load`/`load_with_carry` pass the same difficulty 1 to the same
load_impl → Scene::build), the sisters load via `load_with_carry_difficulty(..., 2)` — the
front-end's launch path [M], the menu way: no health writes, real input only — and each body
asserts `Drones::difficulty == difficulty` at load [M GameState+0x28 0x2a3790 → Drones::difficulty].
06's carry `RamSave` rides the same value (the game's difficulty global survives a level change
untouched [M player.rs RamSave::difficulty doc]). Shared helpers: NONE touched
(crates/nf-view/tests/common/ is unchanged on this branch — git shows no diff there), so every
other level's walks are untouched by construction; the masters 01-07 were still re-run green on
the branch.

**Per-difficulty damage verified where the walks hurt.** No port divergence found: every hit in
the death logs is the game's own pipeline (bullet base × part → Player_HandlePain ×
Plr_DMod_Normal; melee through DroneWeap_CloseCombatImpact 0x1789c0 → the same pain path). The
deaths that happened are difficulty being harder, i.e. tactics.

## Sister results (Agent, real health, real input)

- 01 `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent` — **FAILS (death)**.
  Bond dies at tick 4023: 100 health in 31 hits — 42.0 in 4 hits from an UNARMED drone #9
  (melee punches, ~10.5 each at Agent scale) and 30.4 in 21 hits grinding drone #13. keep_off /
  back_off 4.0 were already on. Left: the contact-escort fight geometry needs a smarter
  clear-the-escort strategy (kill order / keep the contact moving) before Bond survives the
  punchers.
- 02 `walk_the_castle_all_objectives_real_health_agent` — **PASSES**: the full objective route
  in 18201 ticks, the exit fired to 07000003.
- 03 `walk_the_full_route_and_complete_every_objective_real_damage_agent` — **PASSES after one
  opt-in tactics fix**: the first run died at tick 40108 to a shotgun guard at 2.8 u (52.1 in
  16 pellet hits — under 3.0 u every drone round hits [M 0x179740:113-115]); the sister now
  sets the 02 walk's keep_off/back_off 4.0 (gated `if difficulty >= 2` inside the level's own
  test file) and completes the route at 64.3 health, 19 return-fire fights.
- 04 `walk_the_full_mission_start_to_finish_real_damage_agent` — **PASSES**: the assassin dead,
  the banner held, ends 92.6 health + 50 armour.
- 05 `the_door_band_exit_chains_to_07000006_walked_real_damage_agent` — **PASSES**: the door
  #112 E fires, the level exits to 07000006 at 57.6 health.
- 06 `walk_the_castle_interior_to_the_party_real_damage_agent` — **FAILS (wedge, not a
  death)**: Bond ends up floating on the guardroom's upper threshold (28.55, ~41.5, −19.16;
  floor_below 40.0, grounded=false) and the door #59 legs hit the 6000-tick cap. The Agent RNG
  (extra spawns → different fight displacement) lands him on the guardroom's upper level where
  the Operative run never goes; an added mouth-step waypoint did not clear it. Left: probe the
  guardroom's upper→lower transition (the ramp/step the player walks down) and add the
  descent leg for the sister.
- 07 `walk_the_party_start_to_finish_real_damage_agent` — **PASSES**: start → hall → Drake's
  reveal → exit in 5905 ticks, objectives [4][4], carried into 07000008.

Masters 01-07 re-run on the branch: all green (01 5.4s, 02 19.1s, 03 25.8s, 04 1.5s, 05 6.9s,
06 7.7s, 07 4.1s). One self-inflicted break found and fixed: an early 04 edit deleted the
`walk_the_full_mission_start_to_finish_real_damage` wrapper — restored and re-run green.

## What's left

1. 01's Agent sister: the escort-fight tactics (unarmed punchers grind Bond down).
2. 06's Agent sister: the guardroom upper-threshold wedge (needs the level-transition
   geometry probed, then a descent leg).
3. The full 4-crate suite run (`cargo test --offline --release -j2 -p nf-game -p nf-formats
   -p nf-audio -p nf-view -- --test-threads=1`) did not fit the window; the branch touches
   only the seven 01-07 walkreal test files (plus WORKLOG/report), the masters were re-run
   green individually, and no shared helper changed, so the blast radius is those binaries.
   The next window should run the sisters 01/06 after their fixes and then the full suite.

## Live checks for the main agent

None filed — the difficulty scaling is fully pinned by code addresses [M] above and the walks
exercise it headlessly. If the 06 wedge persists after the geometry probe, a PCSX2 watch on
Bond's y at the guardroom upper threshold (which surface catches him) would settle the descent
leg; ask then.

## Usage

not visible to me
