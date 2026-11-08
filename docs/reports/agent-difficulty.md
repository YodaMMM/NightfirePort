# agent-difficulty: Operative / Agent / 00 Agent audit

## The game's difficulty readers
ACTION.ELF has 19 functions that touch the difficulty global GameState+0x28 (0x2a3790). I found them
with a decompile grep for `GameState._40_4_` and checked against an ELF scan for loads at offset
0x3790. The difficulty table at 0x2df4c0 holds three rows with values 1/2/3 [D].

| Reader | What it changes | Port |
|---|---|---|
| Drone_Create 0x136be0:49 | guard placement gate (param 2 <= difficulty) | scene.rs build + drone creates |
| NDrone2_DefaultInit 0x14b300:504-535, 728-739 | captain criterion per difficulty; mode-0x18 TakeXHits 1/2 | drone.rs captain_skin, :2210 |
| NDrone2_HitDamage 0x145558 | Bond's damage to guards × DroneDamage_Easy/Normal/Hard | damage.rs:116 |
| DroneWeap_DoBulletAccuracy 0x179740:81-95 | guard hit chance × DroneFiring_Accuracy_* | drone.rs hit_chance |
| Player_HandlePain 0x1902c8:28-49 | Bond's damage taken × Plr_DMod_*; 4 → ×2 | bullet.rs damage_factor |
| Check_AutoAim 0x1a73f8:127-146 | auto-aim cone × Autoaim_Easy/Normal/HardMul | drone.rs auto_aim |
| Player_InitWeapon 0x1a7ef8:24-35 | MAX_AUTO_AIM_DIST_* (never read) | not needed |
| Player_RamLoad 0x1957a8:21-33 | restart health floor (50 on all three) | player.rs continue_boost |
| PlrStat_GetScore 0x1984b0:294-300 | score multiplier | progress.rs |
| Copter_Create 0x17d340:76-94 | Hind HP and DiffMod 1/1.5/2 | copter.rs |
| Lock_Create 0x1ae1f8:67-69 | Operative auto-unlock | lock.rs:151 |
| SoundTrigger_Create 0x1b2628, Hint_Create 0x1b3248 | placement gates | sound.rs, player.rs hints |
| Boot_*PTPData, Menu_RunMiniMission, MP confirm, bootup, C_SBNFDFCTY | writers / driving plumbing | out of scope |

The per-level rows (Plr_DMod, DroneFiring_Accuracy, DroneDamage) come from ReadTuningVars 0x1d3a80.
The port generates them (tuning.rs), and damage_factor's hard-coded table matches them on every
on-foot level. Pickup_Create and the objective and timer code never read the global, so pickups,
ammo, armour amounts, objectives and time limits are the same on every difficulty [M].

## Divergence found and fixed
With dev mode on (the launcher toggle), the app's level load took `load_with_carry_dev`, which
hard-codes difficulty 1 and a zero medal word. That load covers every mission launch, level change
and restart. So an Agent or 00 Agent pick played the Operative guard roster and the Operative
damage, accuracy, auto-aim and Hind rows, and the medal-gated pickups were missing. The app now
loads through `load_front_end` in both modes, carrying the picked difficulty and the profile's
medal word.

Test: crates/nf-view/tests/difficulty_dev_load_live.rs. It loads 07000001 at difficulties 1, 2 and 3
through both arms and checks that the difficulty and the guard roster match between them, and that
the three rosters differ. With the dev arm forced back to the old path, it fails with `left: 1 right: 2`.

The random stream does not move: non-dev play is unchanged.

## Not changed
The dev menu's cross-level weapon give reloads the level without the medal word, so kind-6 pickups
drop after that reload. This is dev tooling only, not a game path.
