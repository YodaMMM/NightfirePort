# level-08-real-health-rockets — report

Branch `s/level-08-real-health-rockets`, cut from origin/master 524924db.

## What's done

`walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage` is un-ignored and
GREEN: the whole 07000008 mission on one fresh load, real health, no health/armour/ammo writes
— end health 76.0, lowest 76.0, no falls, the mission ends into the driving hand-off
0x0730008, in 27969 ticks.

The fights now pick weapon and range the way a player would (all in the 08 test file; nothing
under `crates/nf-view/tests/common/` changed):

1. **The close guards get the gun** (`fight_close_guards`, called before the gondola
   boarding). The descent wave's burst-rifle pair (#263/#264, weapon 0x15, skin
   0x0500002f [D the audit's param dump]) closed to 4.0–6.5 u on the boarding platform and its
   interval-4 bursts landed −1.80/−3.60/−3.60 at frames 24647/24663/24667 in the measured
   death run (docs/reports/level-08-long-range-hits.md). Any live team-1 guard within 12 u
   horizontal / 4 u vertical is now answered with `switch_to_gun` → `stand_and_fire` before the
   walk steps into the car. REAL-DAMAGE ONLY: the fn gates on `tops_up()`, so the topped-up
   sister's fight timeline is untouched (it passed unchanged).
2. **No rocket at a target inside the blast radius**: the AT-420's blast is radius 8, ≈100
   [L ee_start_05.bin @0x2bf150 + 0x2C·0x10C; M Explode_Propagate 0x17c518 case 0] with the
   (int)-truncated linear falloff × the 0.6 level factor [M 0x17c518:69-71;
   Player_HandlePain's table] — the fire gate holds at 10 u and Bond steps back (aim held)
   while the machine is inside that reach.
3. **No rocket down a blocked line — the slit gate.** The long-range-hits task proved the
   death run's four "world" hits (−52.5/−7.5/−22.5/−3.7, frames 26477/26580/26639/26696) were
   Bond's OWN rockets: the ride's end (112.0, 13.5, −179.7) is INSIDE the carriage (the cabin
   floor is the origin's −4.5: boarded y 27.9 = 32.4 − 4.5, arrived y 13.5 = 18.0 − 4.5
   [D stream 1; the ride logs]), a tri-dump I took this session shows the cabin's walls
   (x ≈ 110.4/113.6, z ≈ −176.6/−182.2, wall band top y ≈ 14.31 [D
   `dynamic_tris_near` at the fight]) and the west doorway (the boarding opening, wall band z
   gap −178.25..−180.6) — and the aiming eye (14.24) sits just under the lintel. Rockets fired
   on a blocked line detonate on the cabin walls 0.65–5.5 u from Bond. The fix is the player's
   own discipline, "fire through the doorway, then steer": a launch needs the muzzle ray's
   first 3 u clear of the world (`segment_bullet` — the round spawns at the muzzle and the
   first flight segments are what detonate at his feet) and the machine beyond the blast's
   10 u reach; after launch the GUIDED round steers toward the machine's live position every
   tick [M Bullet_update 0x12e7b0:177-244], so the aim tolerance at the trigger is loose
   (±0.25 rad — the tracker's ±0.08/±0.05 per-tick clamps cannot settle inside a sniper gate
   during the machine's brief low passes through the slit). The fire line also cites
   [M Bullet_update 0x12e7b0's world segment; Collide_Pick 0x1eb320].

Result at the shoot-down: **3 rockets, all three hit the machine (101.0 each — the ≈100 blast
+ the 1.0 contact; 303 > 299 hp [M Copter_Create 0x17d340]), zero self-damage, 1 rocket
spare** (clip 4 + the ride's pickups; the 8-round supply [L record 0x2C +0x92; ammo_data
0x2C69B8 + 0xE·12] was never poked).

## How it was verified

- `walkreal_07000008_live` — all 4 tests green (the un-ignored real-damage walk, the topped-up
  whole-mission walk, both ally sisters), 84.9 s.
- **Every walkreal level binary** run on this branch, `--test-threads=1`: 07000001 (6), 02 (2),
  03 (4+1 ign), 04 (8), 05 (12), 06 (5), 07 (3), 08 (4), 09 (6), 0a (10+3 ign), 0b (4+1 ign),
  0c (2+1 ign), 0c_barrelrow (1), 0c_eastroute (1), 0d (1+1 ign), 11 (1), 11_route (1),
  12 (1), 13 (2), 14 (7), 15 (4), 16 (14), 1b (17), 41 (7), 4a (2), 4a_satchel (2) — **all
  green**; the ignored ones are the documented blockers of other tasks, none new.
- `cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -- --test-threads=1`
  — green except `nf-formats --test disc_golden` (11 passed, 3 failed) — exactly the 3
  disc_golden tests the slim ISO allows to fail.
- `cargo test --offline --release -p nf-view --lib` — 100 passed, 0 failed.

## What's left / sandbox blocker

- The rest of `nf-view`'s integration-test binaries (~200 targets: the probe/fix_* files)
  could not be linked in this sandbox: the disk filled to 100 % (9.9 G rootfs) and the linker
  died with `collect2: fatal error: ld terminated with signal 7 [Bus error]` /
  `No space left on device` on a different binary each attempt (5 attempts; purging stale
  binaries and caches freed ~1.4 G but not enough for the remaining link storm). No code error
  was involved — the same tree links and passes the whole walkreal set, the lib tests, and the
  three other crates. The next sandbox (or the driver's machine) should run the full one-line
  suite once:
  `NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view -- --test-threads=1`
- A render spot-check for the main agent (nothing visual changed; the fight is camera-side
  only): `--bond 111.0,13.5,-179.7` during the Hind fight should show Bond standing inside the
  gondola cabin at the bottom station, the machine visible through the west doorway on its low
  passes.

## Live checks for the main agent

None required — the fix is walk-side tactics; the sim's damage/fight code is untouched.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed (ntfy.sh is not reachable from this sandbox, same as the earlier
sessions):

1. curl: `curl: (28) Connection timed out after 20001 milliseconds`
2. python urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. wget: exit code 124 (the 25 s timeout killed it; no output)
