# stream-batch — the joined gas+guard+muzzle tree, merged and first-measured

Started 2026-11-06 11:49 UTC. Branch `s/stream-batch` cut from `origin/s/gas-guard-repin`
ee3a7dade, merged in order: `origin/s/muzzle-on-master-2` 3dbc1a3c7, `origin/s/guard-repin-master`
0cc4409fd, `origin/s/guard-repin-master-2` c6b6851dc (landed), then `origin/master` d6245281a
(already contained in every parent — "Already up to date"). **Every gas and muzzle draw is kept**:
the merged `gas.rs` carries both the gas-init draw test (`init_ex_consumes_the_games_draws_in_order`)
and the muzzle tests (`gt_shot_smoke_draws_six_and_lands_the_values`, `trail_big_puff_draws_six_...`);
`drone.rs` carries the muzzle-flash orientation draws [M DroneWeap_FireWeapon 0x178f68:189-213] and
both guard lever sets; no game-code draw was dropped or reordered in any merge.

## Done

1. **The four merges landed with every lever.** Five content conflicts (gas.rs, ai_live.rs,
   enemy_accuracy_live.rs, walkreal_07000014/15) + the guard-repin-master merge's five
   (ai_live, enemy_accuracy, walkreal_07000001/15/16) + WORKLOG add/add resolved. The
   "most recent tree" rule per walk, as applied:
   - `ai_live.rs` `guard_takes_cover_like_the_game`: **seed 16 kept** (the muzzle tree's
     measurement, tip commit 3dbc1a3c7 at 11:09 UTC) over gas-rng-draws' seed 4 (~03:35 UTC);
     both re-pick comments kept in the chain. The joined tree differs from both parents, so the
     seed re-pins from the joined run (not yet done).
   - `enemy_accuracy_live.rs`: taken **wholesale from guard-repin-master** — its harness carries
     the fixed-stream `measure()` reset (`Rng::new(0xacd_1b6a)`, guard-batch-repin's bisect) and
     its pins go with that harness. The muzzle branch's no-reset pins (11/11/17 …) ride a
     different harness model and are superseded by any future re-measure; the muzzle GAME-code
     draws stay in the tree, so no lever is lost. Kept the muzzle comment as a sister note.
   - `walkreal_07000001_live.rs`: **gas lineage kept** (far_field_guards/far_field_sweep +
     un-gated lip fights, last touched 07:56 UTC; answers the d3 4163 far-pair death). The guard
     side's post-AND poll loop is common ground in both versions.
   - `walkreal_07000014_live.rs`: **gas side kept** (`nudge_west_of_the_door_jamb`) — the gas
     tree is the newest tree with a full 26-binary battery on record (202/23/15). NOTE: the
     joined run's red below died exactly on this leg, so the muzzle alternative
     (`try_walk_to(29.4, 0.8, 36.7)`, measured 11:09 UTC) is the first lever to try next window.
   - `walkreal_07000015_live.rs`: **UNION** — the guard-lineage skin-based crossing kill
     (guard-radii-finish) AND the muzzle tree's post-fight `nav_until_clear` re-clear (different
     levers, both kept; the muzzle side's hard `shoot(0)` is replaced by the skin kill as before).
   - `walkreal_07000016_live.rs`: **gas side kept** (`walk_to_the_chaser_fight(&mut w, true,
     false)` full route economy, a1d200d3c) — its rationale is conditioned on the gas draws,
     which the joined tree carries.
   - `docs/WORKLOG.md` was add/add (the file does not exist on master): the gas-lineage file kept
     whole, guard-repin-master's entry appended at the end, nothing rewritten.
2. **One real compile break fixed.** `voice_triggers_live` referenced `Drones::explosive_queue`,
   which gas-draws-8 (ffcb70a5d) removed when the HandleExplosives broadcast became a per-frame
   live-grenade scan [M 0x1435f0:20-52 + 0x138b10:18-19]. The test now pushes a live 0x34 frag at
   rest beside the guard (same pattern as ai_live's flee harness); the test compiles and the
   flee-through-live-objects mechanism is what runs.
3. **MP digests re-measured once on the joined tree** (task step 2):
   - `mp_perf_16_live`: digest moved 0x986f_90e8_e5d6_b12f → **0xbbb6_911c_6656_9440** (kills 35,
     p50 0.446 ms, p99 0.696 ms — perf itself unchanged). Re-pinned in the test with the
     attribution comment. **How I know the change comes from the batch's levers:** the run is
     fixed-input deterministic (2 scripted humans + 14 NEUTRAL bots, fixed 60 Hz, no wall clock),
     so the fold moves iff the sim moves; `git diff origin/master..HEAD -- crates/nf-game/src`
     shows the only sim changes are the three lever families (gas init draws [M Gas_Init
     0x1aacc8:72-80], muzzle-flash orientation draws [M 0x178f68:189-213], guard arrival
     radii/glass [M 0x152548:174-181]), and the MP arena exercises all three (bots fire, smoke
     pops, bots use the shared arrival radii). A per-family bisect was NOT run — that is the
     honest limit of the attribution: the fold's move is attributable to the batch as a whole,
     not to one family.
   - `mp_net_slots_post_move_live`: **2 passed / 0 failed, green** on the joined tree (the
     six-human loopback per-tick hashes still agree — nothing to re-pin; the slot-move digest is
     an agreement property, not a pinned value).
4. **The walkreal battery, as far as it fit** (task step 3). Whole runs, joined tree, real ISO,
   `--test-threads=1`:

   | binary | result | the red(s) |
   | --- | --- | --- |
   | walkreal_07000001_live | 7/1 | `..._contact_real_damage_agent` — death 5664 at (6.26,6.38,8.88), guard #6 skin 0x5000036 shotgun 0x1c at d 2.9 (the door-lip fight line) |
   | walkreal_07000002_live | 2/1+1i | `..._real_health_agent` (walkreal_07000002_live.rs:759) |
   | walkreal_07000006_live | 5/1+1i | `..._party_real_damage_agent` — walker.rs:296 death |
   | walkreal_07000008_live | 5/1 | `..._real_damage_00agent` — walker.rs:296 death (the station-volley red the guard battery predicted) |
   | walkreal_0700000d_live | 1/1+1i | `walk_arrival_to_the_c5_door_real_damage` (walkreal_0700000d_live.rs:836) |
   | walkreal_07000014_live | 8/1+1i | `..._delta_exit_real_damage` — death 5063 at (31.73,1.76,33.83), guard #1 skin 0x5000087 weapon 0x14 at d 4.2 — the descent leg where the merge kept the gas nudge |
   | walkreal_07000015_live | 5/1+1i | `..._omega_sector_exit_real_damage_agent` — death 7435 at (12.26,42.17,71.20), last hits from skin 0x5000087 at d 28.8 (~1200 ticks before death) |
   | walkreal_07000016_live | 11/3+2i | `bond_is_armed_at_the_chaser_fight_after_the_15_handoff` + `..._on_a_direct_load` — the cold load carries weapon 26 (`[1,2,17,26,52,55,78,80,84]` vs pinned `[1,2,17,52,55,78,80,84]`); `the_chasers_punch_lands_the_record_damage` |
   | walkreal_0700001b_live | **17/0+2i, GREEN** | — (holds its gas-lineage count) |

   **Total line from these whole runs (not a full run — see "what's left"):**
   `61 passed / 10 failed / 9 ignored across 9 walkreal binaries whole + mp_perf_16 re-pinned + mp_net_slots 2/0`
   Every failure so far is in a real-damage / real-health / 00-agent variant or the 16 armed pins;
   the plain (tops-up) walk halves passed everywhere they ran, which matches the task's expected
   ordering (re-pin plain walks first — they don't need it on this evidence so far).

## Triage so far (task step 4 — incomplete)

- **16's armed pins** (NOT re-pinned — an attempted re-pin was reverted): the two `bond_is_armed`
  tests fail differently depending on test ORDER. In the whole-binary run both actuals measured
  `[1, 2, 17, 26, 52, 55, 78, 80, 84]` (9 weapons); filtered to just the two tests, the handoff
  arrival measures `[1, 2, 17, 52, 55, 78, 84]` (7) and the cold load `[1, 2, 17, 52, 55, 78, 80, 84]`
  (8) — the original pins. So the binary carries cross-test state (or an order-dependent shared
  cache) that the whole run exposes: **a look-item in itself** (which test leaks what into the
  later tests — the "loadout: switch request N" lines differ 7 vs 8 vs 9 owned). The re-pin must
  wait for that to be understood; the pins stay as they were.
- **14's descent death at (31.73,1.76,33.83)** is on the exact leg where the merge chose the gas
  nudge over the muzzle lane-leg; the muzzle variant is the first lever to try.
- **01/06/0d/08 deaths** sit in the same families the guard-repin battery already classified
  (guard rounds through glass, the bay-sniper class, the station volleys) — the guard-repin-master
  report's levers (stand_shooters_down sweeps, kill-dwell) are the candidates, unchanged by this
  merge.
- **15's late death (7435) at (12.26,42.17,71.20)** is past the crossing (the skin-kill + re-clear
  both ran); the last hits are from the crossing guard at d 28.8 long before the death tick — needs
  the fight-ledger read before deciding walk-tune vs behaviour.

## What's left (the next window continues from this branch)

1. The rest of the battery on the joined tree: 03 (752 s alone last window), 04, 05, 07, 09, 0a,
   0b, 0c ×3, 11, 11_route, 12, 13, 41, 4a, 4a_satchel + `npc_doors_live`, `fix_guard_misc_live`,
   `ai_live`, `enemy_accuracy_live`, `grenade_live` and the full four-crate suite. This window ran
   9 of the 26 walkreal binaries whole (2-core, 3 GB sandbox; five concurrent sims OOM-kill, three
   fit; the sandbox dies at 13:49 UTC and re-runs after 13:29 were barred by the driver's rule).
2. The re-pin pass on the joined tree, in the task's order (plain walks first — currently all
   green where run — then real damage, then Agent, then 00 Agent), starting with the levers named
   above (14's muzzle lane-leg; 16's weapon-26 pin) and the four look-items (0a laser/alert
   channel 0x12; 03 servant 0 WAIT_SWITCH/HOSTAGE; 0d the ~10 m fall; any 6000-tick cap).
3. The per-family digest bisect for mp_perf_16 if the main agent wants family-level attribution.
4. No [L] pin was loosened, no ignore was added, no game code was changed to suit a walk. The one
   game-adjacent edit is `voice_triggers_live`'s driver (the removed queue's test), which follows
   the gas-draws-8 game change rather than papering over it.

## Live checks for the main agent

None requested this window — the failures above are all reproducible in-sim; no EE-side fact is
in doubt.

## Usage

Token/credit/context figures: not visible to me. (No token, cost or context-window display is
available in this sandbox; nothing to copy.)
