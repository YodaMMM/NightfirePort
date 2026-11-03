# WORKLOG

## real-health-agent-difficulty-01-07 (2026-11-03)

Task: finish levels 01-07 on real health at Agent difficulty. The real-health walks
(walk_..._real_damage in walkreal_070000{01..07}_live) run at the default difficulty
(Easy: Plr_DMod_Easy 0.4 damage scale). Do: (1) add an Agent-difficulty sister of each
real-health walk for 01..07 — set the sim's difficulty the way the menu does at level
start, real input only, no health writes; (2) where Bond dies, read the damage log; the
port must match the game's per-difficulty damage and stats [M the difficulty multipliers
and the stats the difficulty changes]; fix the port where it differs, otherwise improve
the walk's tactics (cover, pickups, kill order); (3) every new sister must pass; run
every walkreal_*_live, then the full suite. Shared helper changes opt-in per level.

Branch cut from origin/master 6f16146e. Sandbox carried from the previous session (repo,
work/, ISO present); no setup redo needed.
- The seven Agent sisters landed and the difficulty pin: [M] the menu's `difficulty[]` global
0x2df4c0 rows carry GameState values 1/2/3 with Txt 0x14d/0x14e/0x14f = "Operative"/"Agent"/"00
Agent" (menu.rs) — Agent = 2 = Plr_DMod_Normal; Player_HandlePain 0x1902c8 scales Bond's pain
(Easy 0.5/0.6 vs Normal 0.7), DroneWeap_DoBulletAccuracy 0x179740:74-88 the guards' hit chance
(Easy 0.5/0.8 vs Normal 0.8/1.1), Drone_Create 0x136be0:49 the min-difficulty spawns,
Lock_Create 0x1ae1f8:67 the Operative-only auto-unlock. Each walk body is parametrized; the
difficulty-1 call sites keep the old load args byte-for-byte (plain `load`/`load_with_carry`
pass the same 1 to load_impl → Scene::build), the sisters ride load_with_carry_difficulty [M
the front-end's launch path] and assert Drones::difficulty == 2 at load.
- Sister results: 02 PASS (18201 ticks, exit fired), 03 PASS after the opt-in keep_off/back_off
4.0 tactics (the first run died at tick 40108: 52.1 in 16 shotgun hits from 2.8 u — under 3.0 u
every round hits [M 0x179740:113-115]; ends 64.3 health), 04 PASS (92.6 + 50 armour), 05 PASS
(57.6, exit to 06), 07 PASS (5905 ticks, objectives [4][4], carried into 08). 01 DIES at tick
4023 (100 health: 42 in 4 hits from an UNARMED drone #9 melee + 30.4 in 21 hits grinding #13 —
keep_off 4.0 was already on; needs fight-geometry study). 06 WEDGES (not a death): Bond floats
on the guardroom's upper threshold at (28.55,41.5,-19.16), floor 1.8 below, grounded=false —
the Agent RNG's fight displacements land him on the upper level; the mouth-step tactic did not
clear it; needs the guardroom's level-transition geometry probed. Masters 01-07 re-run green on
the branch (04's deleted `_real_damage` wrapper restored).
