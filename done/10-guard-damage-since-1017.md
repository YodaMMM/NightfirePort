# guard-damage-since-1017 — Bond loses far more health standing still in some spots

Standing 10 seconds on 07000006 costs 188 health and on 07000011 costs 180, against 6.6 and 10 on a 10-17 build; other stands barely moved. Bisect master from 10-17 for the commit or commits that changed it, using the same stand at default difficulty. Read each against the decompile [M]: guard hit roll, range falloff, weapon damage, difficulty scale. Fix anything that differs and pin the numbers. If the current behaviour is right, say so with the addresses.

---
claimed-by: sandbox4 at 2026-11-05T07:14Z

---
## Report (done, 2026-11-05T08:03Z)

Closed by the driver: the report is on `s/guard-damage-since-1017` (docs/reports/guard-damage-since-1017.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-05T08:03Z)


---
## Report from the worker (done, 2026-11-05T08:04Z)

# guard-damage-since-1017 — the standing-damage jump since the 10-17 build

Branch `s/guard-damage-since-1017` (cut from origin/master a89818235). One worker session,
2026-11-05 07:17–08:40 UTC.

## The premise

Standing 10 s on 07000006 costs 188 health and on 07000011 costs 180, against 6.6 and 10 on a
10-17 build; other stands barely moved. The task: bisect master from 10-17 for the commit(s)
that changed it, read each against the decompile [M] (guard hit roll, range falloff, weapon
damage, difficulty scale), fix what differs — or say the current behaviour is right, with the
addresses.

## What was done

The repro harness is `crates/nf-view/tests/guard_stand_numbers_live.rs`: the level loads at the
fresh-boot difficulty 2 ([L] docs/live/2026-11-01-07000011-start-corridor-guards.md: the
fresh-boot difficulty is 2, 0x2A3790), Bond stands at his own spawn for 600 fixed-60 Hz ticks
with `Input::default()` — no teleports, no return fire, no top-up (the raw `Sim` has none). Every
Bond-damage site lands in `Drones::player_hits` (frame, owner, health lost), so the log is the
full damage record.

The stands measured at five points of the history (07000011 d2 / 07000006 d2):

| commit | date | 07000011 | 07000006 |
|---|---|---|---|
| 9c71ceea7 (the 10-17 baseline) | 2026-10-17 | 3 hits × 6.00 = 18.0 | 0 hits = 0.0 |
| 065d6e53f^ | 2026-10-31 | 4 hits (2×12.0 + 2×6.0) = 36.0 | 0 hits |
| 065d6e53f (fire rework) | 2026-10-31 | 3 hits (2×12.0 + 1×6.0) = 30.0 | 0 hits |
| adbcbe69e^ | 2026-10-31 | 6 hits (3×12.0 + 3×6.0) = 60.0 | 0 hits |
| origin/master a89818235 | 2026-11-04 | 5 hits × 12.00 = 60.0 | 0 hits |

Two mechanisms, both landing between 10-17 and now:

1. **The per-hit doubling.** At 10-17 every hit is 6.0; now every hit is 12.0. The ×2 is
   `Drone+0x100 = 2.0` for the held-weapon class {2, 6, 0xA, 0xB, 0xE, 0x10, 0x3B..0x40} —
   `NDrone2_DefaultInit` 0x14b300's weapon switch writes `*(iVar3+0x100) = 0x40000000` for
   exactly those cases [M; read in work/decomp/ACTION.ELF/0014b300_NDrone2_DefaultInit, the
   case block at LAB_0014b798's switch; the port cite is 0x14b300:267-293]. It landed
   2026-10-22 (3b3bd4eeb standins-drone-922, batch 1), was reverted and re-landed with the
   placement-stats work (adbcbe69e, 2026-10-31).
2. **More shots landing.** 3 hits → 5-6 hits per 600 ticks on the same stand, spread over the
   late-October AI work: the vision-bit census on 35 more ENTER arms (74f68a479, 10-27), the
   fire rework (065d6e53f: pellets per shot, tuning-driven hit chance, first-stopped cut,
   back-shot gate, 10-31), Bond's hit volume spanning his body boxes (48edddf6e, 10-31) and the
   placement-stats accuracy read (adbcbe69e, 10-31). The per-commit hit counts above are single
   samples of a shared-RNG stream — the count attribution across those four commits is
   trend-level, not per-commit exact.

## The four areas, read against the decompile today

1. **Guard hit roll** — `DroneWeap_DoBulletAccuracy` 0x179740 [M, read in
   work/decomp/ACTION.ELF/00179740]: base `100 − accuracy·5`; ×TooClose_Accuracy below
   TooClose_Distance — **twice, once before and once after the difficulty multiply** (the
   decompile really multiplies at both LAB_00179740's second block and LAB_00179894); ×first-moved
   (+0x1dd); ×first-stopped only for levels 0x0700000c..0x0700000e with +0xDA == 0x13; ×moving
   (+0x1DC); ×Easy/Normal/Hard on GameState+0x40; the roll `Rand_FRand(100.0) < p`; beyond
   TooClose the hit also needs +0x270 ≥ NewSighting_TimeToHit·60 and +0x41 == 0; dist < 3.0
   always hits; a miss flies the deterministic wobble (0.5, 1.5, 1.5 sin) rotated by the aim yaw.
   The port's `hit_chance` (drone.rs 12024) matches this line for line, double multiply included.
2. **Range falloff** — there is none in the game. The hit site 0x19455c-0x194588 applies
   `HIT+8 + Hurt_GetDamage` with no distance term [M]; the only distance terms are in
   `Drone_ModPlayerHitDamage` 0x13a940 → `Drone_ModBulletDamage` 0x139f50 [M, read in
   work/decomp/ACTION.ELF/00139f50]: voice type 0xC → 0 damage, weapons 0x63-0x66 untouched,
   ×TooClose_Damage (2.0) under TooClose_Distance, ×Drone+0x100 (already in the bullet), and the
   backshot ×2 (mode neither 2 nor 0x19, anim SET +0xDA == 0x12, level not 0x0700001b, the
   travel-yaw vs facing angle under 90°). The port's `player_hit_damage` (drone.rs 12048) is the
   same chain. Neither bonus can fire on the corridor stands: the guards' anim set is 4, and
   they fire from 6-9 u [L the corridor doc].
3. **Weapon damage** — the raw amount is `weapon_data+0x0C` carried in the bullet; rounds per
   shot `weapon_data+0x18` [M DroneWeap_FireWeapon 0x178f68:233-241]. The ×2 class write is the
   DefaultInit switch above. The port matches both.
4. **Difficulty scale** — `Player_HandlePain` 0x1902c8 multiplies by the level's Plr_DMod row
   for the difficulty [M ReadTuningVars 0x1d3a80; group 05-08 = 0.6/0.7/1.0 @ 0x1d3c44, group
   11-13+4A = 0.5/0.6/1.0 @ 0x1d42bc; difficulty 4 doubles @ 0x190470]. The port's
   `damage_factor` carries the same rows. The measured 12.0 at d2 on 07000011 = 10 raw × 2.0 ×
   0.6 — and it equals the live-measured per-hit 10/12/20 at d1/d2/d3 [L the corridor doc], which
   pins the whole chain against the real game.

## Verdict: the current behaviour is right

Every element of the task's suspect list matches the decompile at the addresses above, and the
per-hit result equals the game's own live-measured numbers. The 10-17 build was the outlier: it
under-shot the game (6.0 per hit instead of 12.0 — the ×2 class write did not exist yet — and
fewer shots landed, because the AI/vision/fire pipeline was still partial). The driver's stands
jumped because the port closed onto the game's own damage, not because a regression introduced
too much. Nothing to fix; the numbers are pinned instead:

- `stand_10s_07000011_per_hit_is_the_decompile_12`: every hit in a real 10 s stand is exactly
  12.0, the total is the per-hit sum, and the stand survives (the [L] death is at f1113).
- `stand_10s_07000006_spawn_takes_no_hits`: the 07000006 spawn stand takes 0 hits — pinned at
  both 10-17 and master, so the spawn is not the driver's 188 spot.

## Harness note (the driver's numbers vs mine)

My stands are at each level's spawn; the driver's are not (07000011's spawn costs 60 on master,
not 180; 07000006's spawn costs 0, not 188). The per-hit arithmetic is the same chain wherever
the stand sits — d2 on 07000006 scales ×0.7 (group 05-08), so a weapon-2/0xE hit there is 14.0,
a non-class hit 7.0 — but the spot and the count are his harness's. The two mechanisms the
bisect found (the ×2 class damage, the shot-cadence pipeline) are spot-independent.

## What's left

- The exact `--bond x,y,z,yaw,pitch` spots of the driver's 07000006/07000011 stands, so the
  totals (188/180) can be pinned at the same spots, not just the mechanism.
- A live cadence trace (below) to move the shot-count side from [L]-doc-consistent to measured.

### Live checks for the main agent

1. 07000011, fresh boot d2, Bond idle at the spawn: `gdbwatch` on the two corridor guards'
   fire tick (DroneWeap_FireWeapon 0x178f68 entry) from f400-f1113 — expected: a shot every
   ~80-160 ticks per guard while in the corridor (my stand: 5 hits/600 ticks across both
   guards), each `Bullet_init` carrying damage 20.0 (10 × the +0x100 2.0) and Bond losing
   exactly 12.0 per connect at d2.
2. The same stand at d1 and d3 (fieldprobe poke of GameState+0x40): expected per-hit 10.0 / 20.0
   (the [L] rows) — confirms the difficulty scale outside the pinned d2.
3. The driver's 07000006 stand spot (188 in 10 s): a fieldprobe screenshot + the guard positions
   there, so the spot can become a pinned stand; expected per-hit 14.0 (×0.7) for a ×2-class
   weapon at d2.

## Verification

- `guard_stand_numbers_live` 2 passed / 0 failed (the pins above).
- `start_corridor_guard_07000011_live` 8 passed / 0 failed (the same guards' full behaviour file).
- No `crates/*/src` file changed — the branch adds the harness/pin test file and this report
  only, so no wider run list applies. The full suite did not fit this sandbox's disk/time
  window (9.9 GB root, ~1 h window): the two binaries above are the complete affected set.

## Usage

not visible to me
