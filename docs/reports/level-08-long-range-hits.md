# level-08-long-range-hits — report

Branch `s/level-08-long-range-hits`, cut from **origin/s/real-health-walks-08-11-12-14-15**
(3fed43b3), not origin/master: the subject test
`walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage` was created on that
branch and exists nowhere else; origin/master (a1cfe068) is an ancestor of it, so this branch is
a strict superset of master plus that branch's five real-health sisters and their default-off
opt-ins.

## What the death actually is (measured, this session)

Reproduced deterministically (21.5 s wall clock, one command):
`NF_DMG_TRACE=1 cargo test --offline --release -p nf-view --test walkreal_07000008_live
walk_the_whole_mission_on_real_input_real_damage -- --nocapture --test-threads=1` dies at
tick 26696 at (111.02, 13.54, −179.69), 9 hits for 100.0:

| frame | damage | source |
|---|---|---|
| 762 / 1275 | −2.40 / −2.40 | drone #14 / #10 (skin 0x5000013, weapon 0xa) at 5.3 / 11.7 u — the station approach |
| 24647 | −1.80 | drone #19 skin 0x500002f **weapon 0x15** at (104.5, 28.2, −122.6), **d 4.0** |
| 24663 / 24667 | −3.60 ×2 | drone #20 skin 0x500002f **weapon 0x15** at (101.9, 28.2, −122.3), **d 6.5** |
| 26477 / 26580 / 26639 / 26696 | −52.5 / −7.5 / −22.5 / −3.7 | "the world" — Bond's own AT-420 rocket blasts at the bottom station (86.2 total) |

**The driver's "60.5 units, every ~4 frames" premise is a death-log artifact.** The damage
ledger's owner line prints the drone's position *at death-log time* against Bond's *death*
position: drone #20 died (hp 0 → `drop_weapon` cleared `weapon` to 0x0
[M DroneWeap_DropWeapon 0x17a2a0 — the weapon byte cleared]) on the boarding platform at
(100.9, 28.2, −121.9), and Bond died at the bottom station (111.0, 13.5, −179.7) 400+ ticks
later — √(10.1² + 14.7² + 57.8²) = 60.5. The hits themselves landed at the boarding platform
(NF_DMG_TRACE, hit-time positions above): **4.0–6.5 u**, frames 24663/24667 four apart — the
tail of weapon 0x15's interval-4 burst [L weapon_data 0x2bf150 + 0x15·0x10C: interval +0x40 = 4].

## Drone #20 named [D]

Port drone index 20 = placement **#264**, of the **lower-station pair #263/#264**
(docs/levels/07000008-audit.md): `oDrone` at (96.76/96.71, 28.33, −117.74/−115.44), skin
**0x0500002f** (guard kind 2 [M NDrone2_DefaultInit 0x14b300 — kind 2 → 0x0500002f]), weapon
param p9 = **0x15** [D the audit's param dump] — the burst rifle (base 0x14, 3-round burst,
interval 4, damage 3.0, model 0x2000314 [L weapon_data]), waiting on channel 0x2C (44 — the
TouchOnce #81/#94/#98 descent wave), p4 = 3, class p14 = 0. They woke on the descent, closed to
4.0–6.5 u of the boarding platform, and fired two bursts. Not the Hind, not a turret: a placed
guard pair. (Not a spawner copy: the port's drone list is placement-ordered — index 19/20 are
the 20th/21st oDrone placements #263/#264; the trace's #14 = placement #258 confirms the
offset.)

## The game's fire law for it [M] — the port matches

- **Burst/fire rate**: burst 3 (weapon_data+0x2C, no resize for 0x15), interval 4 ticks
  (weapon_data+0x40), burst-delay pause between bursts [M DroneWeap_NextBulletTime 0x179d40].
  The observed 24663/24667 pair is that burst tail.
- **Accuracy with distance**: ONE roll per FireWeapon, `Rand_FRand(100) < 100 − acc·5`, ×
  TooClose (only under 3 u), × first-moved/moving factors, × the difficulty factor — **no
  distance falloff beyond TooClose** [M DroneWeap_DoBulletAccuracy 0x179740]; beyond TooClose
  the roll also needs the 120-tick sight age [M :93-99]. The miss offset is the frame-wobble
  (0.5, 1.5, 1.5) [M :107-124] — distance-independent. At 4.0–6.5 u an alerted pair (sight
  24–28 × alert scale [M DroneVision_OpponentVisibility]) hitting 3 of a burst or two is the
  game's own shape; the walk's return fire had already marked both (hp 10/20).
- **Damage**: raw 3.0 [L weapon_data +0x15·0x10C+0x0C] × Drone+0x100 (1.0 for 0x15
  [M NDrone2_DefaultInit's pistol-class set]) → at the hit × Drone_ModBulletDamage 0x139f50:
  #20's −3.60 = 3.0 × 1.2 backshot (shooter behind Bond, anim set 0x12, level ≠ 0x1B
  [M :24-48]); #19's −1.80 = 3.0 × 0.6 leg part [M hit_damage part table].
- **Range cap**: no fire-path range gate at all beyond target acquisition (sight range scaled
  by alertness) and the 120-tick sight age — [M DroneWeap_DoFiring 0x17a080 → Ready2Fire
  0x178c48 (the 2.0 near-gate and the 20° gun-aim gate only) → FireWeapon 0x178f68].

**Verdict: no port fire-law difference.** The four fatal hits are Bond's own rockets: each
AT-420 launch whose muzzle line was blocked by the bottom-station geometry exploded at the
contact 0.65–5.5 u from Bond (explosion kind 44, radius 8, damage 100 [L ee_start_05.bin
@0x2bf150 + 0x2C·0x10C]), and Explode_Propagate's linear truncating falloff
[M 0x17c518:69-71, the (int) truncation pinned in explode.rs tests] × Player_Hurt's level
factor 0.6 on Easy for 0x05..=0x08 [M Player_HandlePain table; port damage_factor] gives
exactly 100·(1−k/8)·0.6 = 52.5 (k=1), 7.5 (k=7), 22.5 (k=5), and the last blast (7.5 incoming)
clamped at Bond's remaining 3.7. Rocket 2 of 4 reached the Hind (101 damage); 1, 3, 4 hit the
station.

## The fix (tactic, no health writes) — in progress

The walk holds position inside the bottom-station pocket and fires whenever the *view* sits on
the machine; rockets 1/3/4 flew into the station's own geometry. Next: a muzzle-line clear
check before the trigger (the world segment the game's own bullet flight would walk
[M Bullet_update 0x12e7b0]) and a reposition step to the open side when blocked, plus boarding
the gondola only with the platform pair finished or the loss priced in. Details as they land.

## Usage

not visible to me
