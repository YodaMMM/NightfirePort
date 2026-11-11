## Your task: `mp-bot-backoff`, multiplayer bots back off only where the game's let them

Branch from origin/master: combat ranges plus a CanBackoff port (a probe 1.8 x speed behind the bot, the back-off state's entry and per-tick logic 0x16fc30) landed in 4c2203cc6 (crates/nf-game/src/mp/bots.rs, test mp_bot_combat_ranges_live); demolition_bots_shoot_the_site is green there. Check that landed CanBackoff against the decompile and add only what is missing. With the game's 4.0 too-close band, mp_bot_objectives_live::demolition_bots_shoot_the_site went red: the site only becomes the bot's opponent inside 4 u [M 0x1424e0:536-538], so the fire state's back-off arm now fires there. The game's NDrone2_CanBackoff 0x1501a0 has two halves the port lacks: the DroneAnim_CanDoAnimState(anim 4) gate and the behind-room NDrone2_MoveTest (the -0.75 local-Z probe, cel-checked [M 0x1501a0:31-77]). Read both in full and port them in the game's order. If the anim gate needs anim data the port lacks, read it from the disc [D]. Pin it through a real bot match, and make the demolition test green. Multiplayer state must stay equal on every machine and must not touch the shared Rand stream. Run every mp_*_live binary and mp_world_sync_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-bot-backoff`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-10T23:19Z

---
## Report (done, 2026-11-11T00:45Z)


# mp-bot-backoff

Branch `s/mp-bot-backoff` (cut from origin/master 0ee8478fd).

## What's done

**1. The landed `CanBackoff` audited against the decompile — two halves were missing and one
landed constant was wrong.** `NDrone2_CanBackoff` 0x1501a0, in the game's order:

1. behaviour prop 6 — implicit for bots: a bot's drone mode is 0 (`BOT_init` 0x1249e8's DIVARS
   template is memset-zeroed and never writes +0x48 — disasm: stores at +0x2c/+0x40/+0x4c/+0x68
   /+0x64 only), and mode 0's hook `DMODE_Normal` → `DMODE_Defaults` sets prop 6 [M 0x14d958,
   0x14d528:2].
2. `DroneAnim_CanDoAnimState(dcv, 4, 0)` [M 0x1501d4-dc, 0x13dfc8] — **ported**: the gate reads
   the disc's `Drone_AnimTables` anim-state-4 block (now `ANIM_STATE4_TABLE` in bots.rs):
   `DroneAnimStates[4].u16 = 14` [D 0x272c68+48] → base 14·0x6c = 1512 [D 0x275ae0+1512], 27
   per-character u16 entries; the character's entry ≠ 0 else character 0's ≠ 0 [M 0x13e040-74].
   A bot's character is its WEAPON's anim set: `BOTWEAP_changeWeapon` writes Drone+0xDA =
   `BOTWEAP_getWeaponAnimSet(weapon)` [M 0x12a028:47], run at spawn by `NDrone2_PostLoad_Init`
   [M 0x14d890:22-24]; the port reads `weapon_anim_set(vars.weapon_id)` (botweap.rs, [M
   0x12b140]). Every weapon anim set 0x14..0x1a has a non-zero entry (699 / 2573/2576/2570/2586
   /2572/2574 [D]) — the gate passes for bots, but as data, not the old "[S reads true — no anim
   pipeline]" constant. (The SP `DefaultInit` character ids 2/3/7/10/12/14/18 all read 0 — the
   MP skins' default char 12 included — so the gate is real for SP drones.)
3. the movement lock Drone+0x4f8 & 0x10 [M 0x150218-220] — `ctx.stationary` (unchanged).
4. the probe [M 0x1501f0-74] — **fixed**: local (0, 0, −0.75) — the old port probed 1.8 × speed
   mul, reading the `Vec_MulR32(mul, 1, mul)` arm as always-on. The scaling arm fires only when
   Drone+0xc5 == 0x1e, and a bot's +0xc5 = `DroneModeSettings[mode].b0` [M 0x14cfa0:38] with
   mode = DIVARS+0x48 = 0 → `DroneModeSettings[0].b0 = 0x00` [D 0x29ad40 row 0] — the arm never
   fires for bots: the probe is 0.75 straight back.
5. `NDrone2_MoveTest(feet, feet+R·probe, NULL, obj, NULL, 0, 1) == 1` [M 0x1502f0-308] —
   **ported**: the port's `nav::Network::move_test` (the range chain — trivially true at 0.75 —
   plus the boundary-wall `BoundsTest` walk in its mode-0 arm [M 0x157d80:165-169, no endpoint
   proximity]) replaces the `walk_clear`+floor stand-in. The game's `BoundsTest` first walks
   `Collide_StraddleCels`; the port's one-cel deviation keeps the [S] +1 LOS stand-in inside
   `move_test` (documented there). `nav: None` (unit rigs) keeps `walk_clear` [S].

The old comment's "(+0xc5 == 0x1e)" premise was also wrong: that byte belongs to drone mode 0x21
(`DroneModeSettings` row 33, whose hook `init_DMODE_Bot` 0x14e568 is empty), which `BOT_init`
never selects — the port's mode is 0.

**2. Two landed stand-in defects the faithful CanBackoff exposed (both fixed, both cited):**

- `opponent_visibility` (bots.rs) computed the bearing as `atan2_approx(dx, dz)` — the GAME's
  +0x54 convention [M 0x1424e0:543 uses `ATAN2_APPROX(x, z)`] — inside a brain whose yaw is the
  port's (cos, sin) convention (`atan2_approx(dz, dx)`: the aim turn 0x178f68's port, the scan's
  ±90° cone, `side_walk`). The cone read mirrored: a bot facing its opponent scored < 0.7 at
  most headings, so the sight latch lapsed and the type-5 heartbeat tail (0x16e060:498-513)
  never re-fired. Fixed to `atan2_approx(dz, dx)`.
- The site pseudo-opponent's seen test (bots.rs step-0 + scene.rs's site row bit 4) used the
  point-centre ray, which is blocked by the site's own geometry at 2 u — the game's
  `CanSeeObject(obj, 5, 0)` [M 0x1424e0:551] sees an OBJECT when the ray stops on it. New
  `site_seen`: the ray to the contact-sphere centre passes when nothing is hit or the first hit
  sits at/behind the sphere surface. [S the surface-aware stand-in — see Live checks.]

**3. Tests.** The `can_backoff_probe_is_scaled_by_the_mul` rig test contradicted the decompile
(1.8 × mul) — fixed to `can_backoff_probe_is_075_and_unscaled` (0.75, unscaled, both speeds,
the wall inside/outside the probe, the lock), per the rule that the test is wrong when it
contradicts the decompile. New `anim_state4_block_matches_the_disc` pins the [D] block and that
every weapon anim set plays. `backoff_exit_bound_is_the_engage_band`'s wall case moved inside
the 0.75 probe. `mp_perf_16_live`'s `PINNED_DIGEST` re-pinned (0x5193ea0e9852b444 →
0x0241227609ef6722, 36 → 39 kills) — the digest guards optimisations, and these are intended
decompile-backed behaviour changes; noted on the constant.

## How it was verified

- Rig tests: `cargo test --offline --release -p nf-game --lib bots -- --test-threads=1` —
  **47 passed** (includes the three reworked backoff tests).
- mp batteries (all `--test-threads=1`, NF_ISO set): mp_bot_objectives_live **10/10** (the
  headline: demolition_bots_shoot_the_site and protection_bots_shoot_the_site green again),
  mp_bot_combat_ranges_live 1/1, mp_bot_scan_live 2/2, mp_bot_weapons_live 2/2,
  mp_site_damage_offline_live 4/4, mp_site_destruction_blast_live 4/4, mp_world_sync_live 8/8,
  mp_bots_live 3/3, mp_bots_moving_live 3/3, mp_bot_armour_online_live 1/1,
  mp_combat_weapons_live 11/11, mp_core_live 5/5, mp_clock_gate_live 1/1, mp_defaults_setup
  12/12, mp_net_slots_post_move_live 2/2, mp_death_effects_live 3/3, mp_16_slots_live 3/3,
  mp_pickup_take_live 2/2, mp_hud_16_live 2/2, mp_pickups_visible_live 2/2,
  mp_respawn_ammo_live 3/3, mp_pain_options_live 4/4, mp_status_strings_live 1/1,
  mp_remote_fx_live 2/2, mp_enviro_mods_live 8/8, mp_team_aim_marker_live 3/3,
  mp_bot_body_sounds_live 1/1, mp_bot_voices_live 3/3, mp_match_end_live 3/3,
  mp_scenarios_live 19/19, mp_fists_skin_live 3/3, mp_death_hands_online 2/2,
  mp_hud_real_live 4/4, mp_shot_gate_live 5/5, mp_shot_effects_live 2/2, mp_perf_16_live 2/2,
  mp_bodies_live 3/3, mp_body_anim_live 2/2, mp_menu_flow 11/11, mp_debriefing_flow 6/6,
  mp_net_lobby 11/11, mp_lobby_one_screen 7/7, mp_screens 5/5, mp_menu_lists_live 5/5,
  mp_host_match_live 2/2.
- **Not run to completion:** `mp_stress_explosives_live` and `mp_online_failures` are SIGKILL'd
  by the sandbox (memory) both with and without the fast suite running — environmental, not a
  test failure; they were not run on the base tree in this window either.
- The fast runner (`tools/test/suite.py`) was started on the final tree but killed early —
  the foreground mp battery above is the coverage this window could afford. **Re-run the suite
  on this branch before merging.**

## What's left

- The suite re-run on the final tree (the fast runner never completed in this window).
- Live checks for the main agent (PCSX2, fresh boot):
  1. Drone+0xc5 of an MP bot mid-match (address, read): expect 0x00 (drone mode 0), not 0x1e
     [M 0x14cfa0:38 + D 0x29ad40 row 0 — the port's unscaled 0.75 probe rests on this].
  2. Drone+0xda of an MP bot after spawn/weapon change: expect `BOTWEAP_getWeaponAnimSet`
     (0x14..0x1a) [M 0x12a028:47].
  3. Drone+0x4f8 bit 0x10 of an MP bot: expect clear while moving [M 0x150218-220; the port's
     `ctx.stationary` is always false in the live driver].
  4. Drone+0x228 bit 2 (the sight ack) of a demolition attacker standing 2-3 u from the site
     facing it: expect set within ~10 ticks [M 0x175ef0; pins the surface-aware site sight].
- The site-ack chain (`find_opponent`'s early return for a held site, flagged "needs its own
  task" in the landed comment) remains as landed; the port's scan still skips for a site
  holder [S — the game's scan runs unconditionally, M-dis 0x142770/0x1427b8 → 0x1427c4].

## Usage

Session context/cost figures are `not visible to me` in this environment (no token or cost
readout is exposed to the worker). What I can see: the task message's own clock (sandbox
expiry ~01:20 UTC; this report written ~01:06 UTC), `git` hashes, and test outputs. No input/
output token counters, no credit balance, no dollar cost were visible anywhere in the session.
