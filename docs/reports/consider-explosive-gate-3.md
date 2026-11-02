# consider-explosive-gate-3 — the gate-2 throw/flee rules land with every walk green

Branch `s/consider-explosive-gate-3`, cut from origin/master c3e3135f with origin/s/consider-explosive-gate-2
6e5eef87 merged in first (clean). Sandbox window 21:01–23:00 UTC 2026-11-02; this report written 22:38.

## What's done

1. **Do 1 (07000015) — the fence chain, resolved with the game's own laws; the walk fixed, the port untouched.**
   The failing pin was `walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage` at tick 2640:
   "#385 goes on #383's blast (cause 2)". Attribution, all measured on this branch:
   - The faithful throw-roll cadence [M DroneWeap_CanThrowGrenade 0x17a798 disasm — the 50% Rand(2) precedes
     the dist check] consumes rolls on every aim-stand 0x68 tick of 07000015's grenade holders: the NF_AILOG
     gate print shows guard 0 holds 3× stun 0x35 and guard 6 holds 3× frag 0x34. The shared RNG stream shifts.
   - The shift retimes the crossing fights (killed at ticks 656/1379/1768/2177 this run), and the route's
     fallen guards drop ARs that the walk-over take auto-equips [M Player_IsBetterWeapon 0x1a6eb0 — the AR
     0x1a is rank 3 of BEST_WEAPON, the held P2K 0x02 rank 16]. Bond reaches the fence mark with the AR.
   - The bullet spawns from the camera–gun-bone midpoint for an fp-model weapon [M Player_WeaponInitBullet
     0x1a4bf0:226-245; the port's scene.rs shot_from], so the AR's muzzle rides the eye→#383 line onto a
     different fence tri. The line passes inside ALL THREE fence pick spheres (measured graze table printed
     by the walk: #383 closest 0.25, #384 0.61, #385 0.65; sph_r 2.18 each), so which mine's tris take the
     round first is the game's own first-solid-node rule [M Collide_Update 0x1ea398:216-229]. From the AR the
     round takes #385 (cause 2 BULLET) and its case-0x4B blast then takes #383/#384 [M Explode_Propagate
     0x17c518] — **the game would do the same**. From the P2K — the gun the walk's own doc pins and the
     sister test fires — it takes #383 first (the sister test `a_p2k_round_from_10_units_explodes_the_door38_fence`
     still pins that chain with the guards held).
   - Fix: the walk re-draws the P2K through the real switch request before the shot (its own stated tactic,
     no health/ammo writes). File 5/5 green. Instrumentation added (env-gated, NF_AILOG): the player-bullet→
     mine hit log and the consider_explosive flee log in nf-game/src/drone.rs.
2. **Do 2 (07000016) — both scenario walks UN-IGNORED and green, plus the real-damage finale walk.**
   - `walk_the_chasers_defeat_completes_the_mission` (top-up): the blocker was never the AND — the measured
     run showed ch 0xc0 ✓ 0xbf ✓ 0x82 ✓ (trigger #286, kind 3, list [191,192], lock 0, gone false) with the
     lift-out's `disabled` never observed. Root cause: AND #286 (0xbf·0xc0 → 0x82 [D]) fires the moment both
     inputs latch, and under the faithful cadence ch 0xbf lands AFTER ch 0xc0 @7808 [D stream 20] (measured
     +8293/+8438), so 0x82 latched while the walk's cut-scene skip was still up and the walker's fast-forward
     played the whole 0x06000632 cinema inside one tick (its disable window, event 4 at t0 to event 6 at
     t1057 [D], fell between samples). Fix: the skip comes off before the squad phase (the mission's own
     transition always plays out). The walk completes: the cinema hands Bond back at the bay key, the button
     fires ch 0x90, MissionState 6, the driving hand-off 0x7300017.
   - `the_chasers_punch_lands_the_record_damage` (no-return-fire real damage): the unanswered corridor
     approach died at tick 2362 (measured 47 AR hits for 100.0 — the AR guards #0/#2/#3 skins 0x500008c,
     #4/#5 0x50000b1 [D p9 = 26]; last hit d 1.9). The no-return-fire premise is the PUNCH WINDOW's, not the
     approach's — a player fights through — so the walk raises the walker's own pub `dmg.return_fire` for the
     corridor legs and drops it at the hand-back. **No common/ change** (the field is per-walker opt-in, set
     by this test only). Green, and the punch pins stand.
   - `walk_the_chasers_defeat_completes_the_mission_on_real_damage` (never ignored, suite-blocking): rebuilt
     on the measured attrition — her kill FIRST (39.6 of the run's 90.6 attrition — one band breach + approach
     stars — against the pit shooters' ~8; the old shooters-first order kept her plinking through the whole
     ~2700-tick clearing), a 300-tick futile-stand cap in the shooters' clearing (a 900-tick stand poured the
     AR into a shooter it never dropped), the existing head-hunting opt-in, and a corpse-to-corpse loot walk
     over the fallen guards' drops (the walker's resupply is blind here twice over: its trail filter holds
     9 u around NAV nodes, and its success check reads only the held gun's rounds while the 0x14 drops file
     into the inventory without equipping [M Player_IsBetterWeapon 0x1a6eb0]). Measured: Bond survives the
     whole finale, the squad-phase audit green, ch 0xc0/0xbf/0x82 latched, the cinema hands him back.
   - **Lesson recorded the hard way**: removing the "useless" re-arm pass from the shared
     `finish_after_the_chaser` broke the top-up walk's pass — the pass's WALKS consume the pit phase's
     guard-AI ticks and shift the stream. The pass is back (verbatim) and the whole file is 14/14 in one
     `--test-threads=1` run. A 3-step re-arm ladder in the squad loop was likewise tried and reverted for the
     same reason (its extra wheel cycles shifted the measured stream); the revert is commented in place.
3. **Do 3 — the runs.**
   - **Every `walkreal_*_live` level binary** (26): 01 (6), 02 (2), 03 (5), 04 (8), 05 (12), 06 (5), 07 (3),
     08 (4), 09 (6), 0a (10+3 ign), 0b (4+1 ign), 0c_barrelrow (1), 0c_eastroute (1), 0c (2+1 ign), 0d (1+1 ign),
     11 (1+1 ign), 11_route (1), 12 (2), 13 (2), 14 (8), 15 (5), 16 (14), 1b (5), 41 (22), 4a (+satchel) —
     **all ok** (the listed ignores are other tasks' pre-existing ones). corridor-11 dedicated file
     `start_corridor_guard_07000011_live` 8/8.
   - `cargo test -p nf-game --lib`: **655 passed**, 5 ignored.
   - The grenade test files (grenade_rattle, stun_grenade, explode, fix_grenade_release/throw/types,
     consider_alerted, probe_11_grenades) and `grenade_live` (nf-game integration) did NOT complete this
     session — the linking overran the window — and the full suite did not run. The gate-2 session's runs of
     exactly these (all green on the same nf-game diff, report §4) plus this session's 655-strong lib run are
     the standing evidence; the next message should run the grenade set + the full suite first.

## What's left

- The grenade test set + the full suite (`NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-game -p
  nf-formats -p nf-audio -p nf-view -- --test-threads=1`; expect only the 3 disc_golden failures on the slim
  ISO and the known mp_stress_explosives OOM).
- The 07000016 live checks from the gate-2 report stand (the pit squad's lethality under the faithful
  cadence; the corridor stun lob).
- One open port question surfaced by the work, not fixed here: the port's DRONE bullets never test the mines
  (only the player's bullet pass does, nf-game/src/drone.rs update_player_bullets), while the game's
  `Bullet_update` 0x12e7b0 picks a mine for any bullet's flight segment (mode 0x201, mask 0) → Collide_Pick
  0x1eb320 → Collide_PickObj 0x1eae80. The 07000015/16 walks never needed it; a guard spray landing on a
  mine fence is the [M] gap to close in its own task.

## Live checks for the main agent

1. 07000015's garrison endowments [D]: confirm guard placements hold p11 = stun 3× (the port's guard 0) and
   frag 3× (guard 6) — the roll-cadence attribution rests on them.
2. 07000016, faithful cadence: whether the real game's pit-squad clone #12 (death ch 0x1a) can spawn at FULL
   100 health north of the launch flame (the port measured the pit clones flame-damaged to ~20-25 and this
   one at 100), i.e. whether the mission's flow expects the player to have ammo for it.

## Usage

not visible to me

### ntfy failed

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` → exit 28,
   `curl: (28) Connection timed out after 20002 milliseconds`.
2. python urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`.
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x` → killed at the 40 s tool
   timeout with no output (no rc line printed).
