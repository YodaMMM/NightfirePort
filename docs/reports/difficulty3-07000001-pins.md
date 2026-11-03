# difficulty3-07000001-pins — everything difficulty 3 changes on 07000001, pinned

Branch `s/difficulty3-07000001-pins`, cut from `origin/master` cbed0983. The escort-fight and
hall-fight reports were read first; `walkreal_07000001_live` is untouched (level-01-agent-hall-fight-2
owns it); `crates/nf-view/tests/common/` is untouched (git shows no diff there — nothing to
opt-in, nothing other levels' walks can inherit).

## The test

`crates/nf-view/tests/difficulty3_07000001_live.rs` — 4 tests, all green. Each load is
`load_with_carry_difficulty(..., d)` — the front-end's launch path [M C_SBNFDFCTY_Handler
0x216ca8:15 writes the wheel pick into GameState+0x28 (0x2a3790); the rows [L
ee_start_07000001.bin 0x2df4c0/d8/f0: +0xc = 1/2/3, Txt 0x30000a5/a6/a7]] — the menu way: no
health/ammo writes, no teleport. Every assert cites its evidence: [M] decompile address, [D]
disc parse, [L] the level's FRESH-boot EE snapshot (file offset = EE address).

## The list: what difficulty 3 (00 Agent) changes against 1 and 2 on 07000001

**1. The spawned set: NOTHING changes.** Every kind-0x0F placement on the created map (main map
01000093) carries param 2 = 1 [D the disc parse, all 20 records], and `Drone_Create` 0x136be0:49's
gate spawns iff `param2 <= difficulty` [M] — all 20 exist at Operative already. The one kind-0x0F
elsewhere in the archive — map 12 placement 5, param 2 = 2 — is DEAD DATA: the game's
dynamic-create pass parses only the LAST map's 0x1A block [M `parsemap_handle_block_id` 0x1d1020
case 0x1a `DynamicPtr = FileNextBlock`; 0x1d1cf0's per-file loop runs the pass once from that
pointer; `ResetMap_Load` 0x1d6360:146 — map 53 is 07000001.bin's last map, so map 12's record
never reaches the dispatcher]. The spawner's five group-1 templates (#168/#169/#261/#298/#299,
param 4 = group 1 [D]; spawner #297 param 1 = 1) ride in the roster at every difficulty and
`DroneSpawner_SpawnDrone` 0x137cb0 reads no difficulty — the ch-0xB reinforcement wave is
identical at 1/2/3.

**2. The ONLY roster change: the captain criterion** [M `NDrone2_DefaultInit` 0x14b300:504-535 —
prop 9 on Operative, prop 5 on Agent, prop 8 on 00 Agent, over the placement's behaviour words
[D params 17..32]]:
- #167 captains at EVERY difficulty (prop 9 + 5 + 8 all set);
- #163 joins at Agent; **#165 joins at 00 Agent** — the d3 captain set is {163, 165, 167}.
The captain setup reskins the kind-1 guards to 0x05000038, doubles the health 12 → 24 (×
`DroneCaptain_Mod_Health` [L ee_start_07000001.bin 0x30cb2c = 2.0]), halves the accuracy stat
8 → 4 (× 1/`DroneCaptain_Mod_BulletAccuracy` 2.0 [L 0x30cb28], truncated) and sets +0x100 = 2.0
(the guard's own bullet damage ×2 inside the 3.0 u TooClose band, `Drone_ModBulletDamage`
0x139f50) [M 0x14b300:559]. The contact (#161, skin 0x0500004F) never captains. The mode-0x18
TakeXHits difficulty split [M 0x14b300:728-739] has no object here: no placement runs mode 0x18
[D param 9 ∈ {1, 5, 14, 18, 28}].

**3. The pain scale (damage Bond takes)** [M `Player_HandlePain` 0x1902c8:29-41 — ×
Plr_DMod_Easy on 1, Normal on 2, Hard on 3]: rows 0.5 / 0.7 / **1.2** [L ee_start_07000001.bin
0x30cc50/54/58]. A 10-base pistol round costs 5.0 / 7.0 / **12.0** — and 24.0 inside the 3.0 u
band (× `DroneFiring_TooClose_Damage` 2.0 [L 0x30cb1c]; `Drone_ModBulletDamage` 0x139f50).

**4. The guards' hit chance** [M `DroneWeap_DoBulletAccuracy` 0x179740:48-51 + 81-95 — ×
DroneFiring_Accuracy_Easy/Normal/Hard]: rows 0.5 / 0.7 / **1.0** [L 0x30caf0/f4/f8]. An
accuracy-8 guard at range: 30% / 58% / **60%** per round; an accuracy-4 captain: 50% / 70% /
**80%**; the ×2.0 TooClose accuracy lands before AND after the difficulty factor [M :49-51 and
:97-99] — an accuracy-4 captain inside 3.0 u at d3 multiplies to 300 against Rand_FRand(100):
every round fires.

**5. Bond's damage to guards** [M `NDrone2_HitDamage` 0x145558 — × DroneDamage_Easy/Normal/Hard]:
rows 1.5 / 1.0 / **0.8** [L 0x30cab0/b4/b8; the Easy 1.5 is the ELF static — the 01-04 group
writes only Normal/Hard [M ReadTuningVars 0x1d3a80]]. The same 10-base torso hit costs the guard
15 / 10 / **8** — a 24-hp captain takes 3 hits where an Operative mook takes 1.

**6. The player auto-aim is OFF at 00 Agent.** `Autoaim_HardMul` = 0.0 [L 0x30cd64; Easy 1.9 /
Normal 1.0 [L 0x30cd5c/60]], the multiplier `Check_AutoAim` 0x1a73f8 applies to the angle budget
[M; the port's `auto_aim` matches]. `Player_InitWeapon` 0x1a7ef8:24-35 also stores
MAX_AUTO_AIM_DIST_MIDL/NEAR per difficulty (18/12 on 1, 14/8 on 2, 0/0 on 3 [L the d1 snapshot
0x30ccf0/f4 = 12/18 matches the d1 branch]) — those two globals are never read in ACTION.ELF
(the port verified this earlier; the store itself is the [M] fact).

**7. No lock, pickup or ammo amount differs.** `Pickup_Create` 0x18dda0 never reads
GameState+0x28 [M whole-decompile grep — the 19 GameState._40_4_ files do not include it]; the
four ungated pickups (armour #275/#326 kind 3 id 71 amount 50, ammo #324/#325 kind 1 id 6 amount
20 [D params]) create identically at 1/2/3, and the two kind-6 "Pickup 007" bonus pickups are
gold-medal gated [M 0x18dda0:93-95], not difficulty gated — with the medal word carried they
create the same at d1 and d3. No 0x2E lock placement exists on the level [D census], so
`Lock_Create` 0x1ae1f8:67-69's Operative-only auto-unlock has nothing to act on. The level's only
hint #330 (stage param 2 = 3 [D]) passes `Hint_Create` 0x1b3248:12-25's ceiling at difficulties
1..3, and the only sound trigger #319 (gate param 6 = 0 [D]) passes `SoundTrigger_Create`
0x1b2628:24-40 everywhere: both create at 1/2/3 alike. `Copter_Create` 0x17d340 — no 0x2F
placement [D census]. `Player_RamLoad` 0x1957a8's restart boost rows are all 50.0 [L
0x30cc34-3c] — a death-retry serves the same 50 health at every difficulty. `PlrStat_GetScore`
0x1984b0's d3 branch is the score page, not the sim. `Boot_GetPTPData` 0x1c8198 is boot
plumbing. Bond's own load state is difficulty-blind: 100 health, armour 0, hands held (0x47)
with the pistol switch request 2 [M `Player_InitWeapon`'s 07000001 arm] at every difficulty.

Every reader of GameState+0x28 in ACTION.ELF is accounted for above (the full grep: 0x136be0,
0x145558, 0x14b300, 0x179740, 0x17d340, 0x1902c8, 0x1957a8, 0x1984b0, 0x1a73f8, 0x1a7ef8,
0x1ae1f8, 0x1b2628, 0x1b3248, 0x1c8198 + the writers 0x1c7e78/0x1c8560/0x1ffb90/0x20ec00/
0x216ca8). **No reader is missing from the port** — the pin found no divergence to fix.

## The route changes a 00 Agent walker will need

The route geometry is the Agent sister's route unchanged (same spawns, same doors, same
channels); the fights change:

1. **Two new captains on the west side.** #165 (−12.2, 6.5, 17.2 — the west lane by the gallery
   door #2 wedge) and #163 (−24.5, 6.4, 5.4 — the bunker/exit-plate side) join #167 (the hall's
   (9.05, 17.01) guard, a captain at every difficulty). Each: 24 health, accuracy stat 4 (80%
   base hit at range, every round inside 3.0 u), ×2 bullet damage in the TooClose band — their
   pistol rounds cost 24.0 a hit inside the band. The walker's kill order must take captains
   first wherever a fight overlaps them, and no approach may dip under 3.0 u — the escort
   report's "approaches stop at 8" rule is now survival-critical, not a health optimization.
2. **The hall door lure is mandatory.** At Agent the door-phase lure held the hall AND phase at
   zero damage; at 00 Agent the same volleys land at 1.2× pain and 1.0× accuracy — the funnel
   trade the escort report measured as losing at Agent loses ~2× faster. The hall-fight
   report's sound-lure design (hold south of the pre-opened door, kill each hunter as it
   crosses one at a time) is the only viable shape; the hold line must re-check the damage band
   (3.0 u) against its 2.5-4 u stand-off.
3. **Bond's DPS is 0.8×.** The 32-round SMG pool the meet hands over (clip 32 spare 0 [L the
   hall-fight probe]) kills a 24-hp captain in ~3 torso hits instead of Agent's ~2 (the SMG's
   1.5 base [L the same probe]); the pre-clears and sweeps must budget rounds for three 24-hp
   captains + the 12-hp mooks at 8/hit, and the pickups (two 50-armour plates, two 20-round
   ammo boxes — difficulty-blind) are worth a detour more than ever.
4. **The reload economy is unchanged**: the restart boost is 50.0 on all rows — a death costs
   no more at 00 Agent than at Operative.
5. **No autoaim**: a human tester at 00 Agent gets no snap assistance (HardMul 0.0); the
   walker's own converge-and-tap aiming is unaffected (it never used auto_aim's snap), but any
   new helper must not assume the correction exists at d3.

## How it was verified

- `cargo test --offline --release -p nf-view --test difficulty3_07000001_live -- --test-threads=1`
  — **4 passed / 0 failed** (the spawned set row-exact at 1/2/3; the rows + the three reader
  selections through `damage_factor`/`hit_chance`/`hit_damage`; the no-delta sweep including the
  medal cross at d1 and d3; Bond's blind load).
- `cargo test --offline --release -p nf-view --test interactables_07000001_live --
  --test-threads=1` — **10 passed / 0 failed**.
- `cargo test --offline --release -p nf-game --lib` — **656 passed / 0 failed / 5 ignored**.
- The fast runner (`tools/test/suite.py`) cannot run in this sandbox: background processes are
  killed between tool calls (a detached `setsid` canary died the same way — the same constraint
  the escort-fight report documented). The per-binary foreground runs above are the stand-in;
  the branch touches NO shared file (only the new test + WORKLOG/report), so the blast radius is
  exactly the three binaries run.

## What's left

- Nothing on the pin itself. If the driver wants the walkreal-level 00 Agent walk (the real
  health walk at d3), that is a separate task on top of level-01-agent-hall-fight-2's landing —
  the route notes above are its design brief.

## Live checks for the main agent

1. A FRESH-boot 00 Agent boot of 07000001 (the difficulty wheel's third row) with gdbwatch on
   GameState+0x28 (0x2a3790) = 3 at level start, and on guard #167's Drone fields: +0x19
   (captain) = 1, +0xac health = 24.0, +0xb4 accuracy = 4 — settling the prop-8 behaviour-word
   read on hardware. The port pins it from the decompile + the disc words; a live glance makes
   it [L].
2. The same boot, guard #165 (−12.2, 6.5, 17.2): +0x19 = 1 only on the 00 Agent boot (0 on the
   Agent boot) — the d3-only captain flip as hardware evidence.
3. Optional: the tuning block at 0x30cc50/54/58 (pain rows) and 0x30caf0/f4/f8 (accuracy rows)
   read identically on a d3 boot — they are level-group values, difficulty-independent; the
   snapshot already pins them at d1.

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (nothing to copy).
