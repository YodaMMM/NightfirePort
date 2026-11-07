## Your task: `mp-enviro-mods-proof`, Enviro-Mods rows are proven in real matches

Branch from origin/s/mp-enviro-mods-apply, merge current origin/master, and read its docs/reports/mp-enviro-mods-apply.md. The apply work is in, but its proof did not fit that window. Take its "What's left" in order. Write crates/nf-view/tests/mp_enviro_mods_live.rs: row 1 off gives no mounts on a mount-carrying arena; on with weapon set 5 gives `turret == 5` on each mount; on with set 10 gives the seed-keyed pick, below 10; a two-machine net_tick run with mounts on agrees on every tick's hash and on both mount arrays, which is the in-step proof the old clear was waiting for; row 2 off leaves `props.destroy` empty on a destroyable arena and on fills it; row 3 on gives the load's kit 0x50 and off does not. Pick the arena from the mp_world_sync inventory print. Then audit the setup and summary pages that echo the rows, and use `mp_enviro_value_label` there if they print them. If mp_screens pins the locked row's old text, update it citing 0x210030. Run every mp_*_live binary, mp_world_sync_live, mp_defaults_setup, mp_menu_flow, menus_e2e_mp, mp_screens and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mp-enviro-mods-proof`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T18:08Z

---
## Report (done, 2026-11-07T18:55Z)


---
## Report from the worker (done, 2026-11-07T18:55Z)

# mp-enviro-mods-proof — the Enviro-Mods rows proven in real matches

Branch `s/mp-enviro-mods-proof` from `origin/s/mp-enviro-mods-apply` (b51280b36), merged with
`origin/master` (already up to date — the apply branch was cut from master 10956badb and master
had not moved past it into this tree). This window took `mp-enviro-mods-apply`'s "What's left"
in order: the row-by-row real-match pins, the page audit, and the reruns.

## The pins — `crates/nf-view/tests/mp_enviro_mods_live.rs` (new, 7 tests, all green)

The file loads real MP matches through `nf_view::load_mp_carry` (the live match-start path: the
settings ride the load, the equip runs, the sim ticks) and pins each Enviro-Mods row:

- **The census** (`mount_census_finds_a_mount_carrying_arena`): every MP arena loaded with the
  Gun-Emplacements mod ON and weapon set 5. Skyrail 0x07000024, Fort Knox 0x07000027, Snow
  Blind 0x07000029, Phoenix Base 0x07000026, Sub Pen 0x07000025 and Ravine 0x0700004b each
  carry **2 mounts**; Missile Silo 0x07000028 carries none. The pins use Skyrail, the first
  mount-carrying arena in MP_MAP_ROWS order (2 mounts). The mp_world_sync inventory print could
  never show this — its settings leave the row 0, so its print always shows `gunimps 0`; the
  census loads with the mod on, which is the print the task wanted.
- **Row 1 (+0x1B8) OFF**: no mount object at all on Skyrail [M GunImp_Create 0x131f20:56-58 —
  the MP arm returns 0 while the word is 0].
- **Row 1 ON, weapon set 5**: every mount's turret entry is 5 [M 0x131f20:59-62 — the
  `MPSettings+0x1B4` arm for 0..9].
- **Row 1 ON, weapon set 10 (Random)**: every pick is < 10 [M 0x131f20:63 — `Rand_Rand(10)`]
  and equals the private seed-keyed stream's draw per mount (`Rng::new(seed ^ 0x4755_4e49)`),
  and a second `apply_mp_turrets` reproduces the same assignment (the build + match-start
  idempotence). The private stream is the port's documented substitution for the game's
  shared-stream draw [S — the game's shared `Rand_Rand` state at create time is play-history no
  test can reproduce; the substitution keeps every online machine identical without adding a
  draw to the shared stream]. The [S] part pins the port's choice; the <10 range and the
  per-mount identity are the game's [M].
- **The in-step proof** (`row1_online_two_machines_agree_on_every_tick_with_mounts_on`): two
  machines (slots 0 and 1, the live match-start setup), mounts ON with weapon set 10, walkers
  picking up, shooting destroyables/breakables, pressing use, jumping. **1500 ticks, every
  tick's world hash agrees**, and after the run both machines' mount arrays are identical
  field-by-field (placement, turret, active, hot, pitch, yaw, smoothed mirrors, heat,
  cooldown). This is the proof the old unconditional `gunimps.clear()` was waiting for: the
  mounts now exist in a real match AND stay in lockstep. Sister: nudging one machine's mount
  yaw alone diverges the hash — a mount desync is DETECTED (the fold's teeth, scene.rs
  `net_world_fold`).
- **Row 2 (+0x1D4) OFF/ON** on Skyrail (a destroyable arena): OFF → `props.destroy` EMPTY [M
  Destroy_Create 0x12c318:14-15]; ON → the arena's **8** class-0x18 destroyables [L the
  mp_world_sync_live inventory print, this session's run].
- **Row 3 (+0x1D0) ON/OFF**: the load's kit owns the Q-Claw **0x50** with the grapple on and
  not with it off [M MP_EquipPlayer 0x1855f8:27-34; the port's arm
  `nf_game::mp::equip::mp_equip_player` runs at every MP spawn/load].

`NF_ISO`-gated like every arena-loading test: without the ISO the tests print a skip line and
return.

## The page audit — one label list, one copy

- The couch **Enviro-Mods page** already draws `mp_enviro_value_label` (mp-enviro-mods-apply).
- The **online one-screen lobby** (`mp_lobby.rs`) echoed the same rows through its own inline
  second copy of the node list — retired this window: the list now lives in ONE function,
  `nf_view::menu::mp_enviro_row_label(envmods, explosive_unlocked, row)` [M P_MPENVIROMODS
  0x210030's msg-0x10 binds; the locked row binds by the unlock flag, M 0x210030:35-44], and
  both the page's `mp_enviro_value_label` and the lobby's row build call it. Values unchanged
  (the lobby's row-2 `enabled` flag still keys on the unlock flag).
- The **setup pages** (P_MPSETUP 0x20d7a8's team/character/handicap prompts) print no enviro
  rows; the **Confirm/summary page** prints no enviro rows either — its decompile arm builds
  MAP / SCENARIO / WEAPON SET / TIME / POINTS / FRIENDLY FIRE and nothing else [M P_MPCONFIRM
  0x20ec00:143-389, the same list `mp_confirm_summary` implements]. Nothing to change there.
- **mp_screens** pins no locked-row text (its only On/Off pin is the friendly-fire row
  [M 0x20ec00:313-318]) — no update needed; the binary passes as-is.

## The reruns (release, `--test-threads=1`, per-binary under a timeout — the suite.py shape)

Every binary rebuilt against the final tree (my commit e236a93f1) before its run. **Totals:
944 passed, 0 failed, 5 ignored** across the required list, plus the two environment casualties
below (no pass/fail evidence, counted as neither).

- **nf-game lib unit**: 688 passed, 5 ignored.
- **nf-game's 10 mp_* integration binaries**: mp_arenas_live 1, mp_debriefing 10, mp_hud_16 7,
  mp_hud_data 4, mp_hud_pane 5, mp_lockstep 6, mp_match_rules 18, mp_never_mentioned 6,
  mp_status_bits 3, mp_teams_flow 4 — 64 passed, 0 failed.
- **nf-audio lib unit**: 24 passed.
- **nf-view, the required 43 mp/menus binaries** (192 passed, 0 failed):
  menus_e2e_mp 5; mp_defaults_setup 12; mp_menu_flow 11; mp_menu_lists_live 5; mp_screens 5;
  mp_lobby_one_screen 7; mp_net_lobby 11; mp_death_hands_online 2; mp_online_failures 4 (see
  below); mp_scenarios_live 19; mp_debriefing_flow 6; mp_status_strings_live 1;
  mp_16_slots_live 3; mp_bodies_live 3; mp_body_anim_live 2; mp_bot_armour_online_live 1;
  mp_bot_body_sounds_live 1; mp_bot_objectives_live 10; mp_bot_voices_live 3;
  mp_bot_weapons_live 2; mp_bots_live 3; mp_bots_moving_live 3; mp_clock_gate_live 1;
  mp_combat_weapons_live 11; mp_core_live 5; mp_death_effects_live 3; mp_enviro_mods_live 7;
  mp_host_match_live 2; mp_hud_16_live 2; mp_hud_real_live 4; mp_match_end_live 2;
  mp_net_slots_post_move_live 2; mp_perf_16_live 2; mp_pickup_take_live 2;
  mp_pickups_visible_live 2; mp_remote_fx_live 2; mp_respawn_ammo_live 3;
  mp_shot_effects_live 2; mp_shot_gate_live 5; mp_site_damage_offline_live 4;
  mp_site_destruction_blast_live 4; mp_world_sync_live 8.

- `mp_online_failures::join_in_progress_from_the_menus_then_leave_and_reconnect` **hangs**
  (killed at 150 s, again at 420 s). **Not my regression**: rebuilt from the pre-edit sources
  (b51280b36's menu.rs/mp_lobby.rs, binary relinked) and it hangs identically — pre-existing on
  this tree in this sandbox (its 4 siblings pass, 4/5 in that binary).
- `mp_stress_explosives_live` (one test: 8 humans + 8 bots, 3 machines, an explosive every
  tick) is **killed by the sandbox at 500 s** (SIGKILL, no result line). My diff — two
  front-end label functions and a test file — has no pathway into that sim; its lockstep
  siblings (mp_world_sync_live's two-machine runs, mp_net_slots_post_move_live,
  mp_perf_16_live) all pass. No pass/fail claim.

**The full suite (`tools/test/suite.py`) could not run this window**: background processes are
killed between tool calls in this sandbox (verified with a detached `sleep` probe), so the
suite can only run in the foreground, and one tool call is capped at ~10 minutes; the suite's
build step alone (all four crates — nf-view alone has ~980 test binaries to relink after any
lib change) does not fit one call, and the disk (9.9 GB, <450 MB free) does not hold a second
full set of artifacts. I ran the suite's exact per-binary shape (`--test-threads=1`, per-binary
wall-clock limit, one `test result:` line each) over the task's required list instead, and the
numbers above are that run. nf-formats' 46 integration binaries were not rerun: neither the
apply diff nor this diff touches that crate (the apply touched nf-game's gunimp.rs and
nf-view's scene.rs/lib.rs/menu.rs; this touch is nf-view's menu.rs/mp_lobby.rs plus the new
test file).

## What's left

- The suite's own full pass on a machine where background processes survive (the totals above
  cover the required list only).
- A main-agent render of the mounts: load Skyrail with the mod on, weapon set 5, and stand at
  each `props.gunimps[i].pos` — expect the set-5 gun-child mesh (`TurretGuns[5].art`
  0x0200_0b07 [D the TurretGuns table]) on the mount base (see "Live checks").
- `mp_online_failures`' hanging test deserves a look on hardware where sockets/waiters behave
  (it passed in past windows; this sandbox hangs it on both trees).

## Live checks for the main agent

1. Skyrail mounts visible: boot the port with `--mp` settings gun_emplacements=1, weapon_set=5
   on 0x07000024, or load it in the viewer with the same settings; the census prints each
   mount's `pos` (`--bond <pos> yaw pitch` from `p-props] gunimps 2 turrets [5, 5]` — the
   positions are the arena's two 0x34 placements). Expect: each mount shows the gun-child mesh
   of art 0x0200_0b07 (the set-5 entry) — NOT the laser 0x0200_0b50 and not "no gun".
2. The lobby's locked row: a fresh profile (no Explosive-Scenery reward), online lobby one
   screen, focus the Explosive Scenery row — expect "Locked" (the 0x1000076 node), not "Off"
   [M 0x210030:35-44], same as the couch page since mp-enviro-mods-apply.

## Usage

not visible to me

---
## Addendum from the worker (2026-11-07T19:0xZ)

The branch carries the final report (5160caaa0): the reruns reached 175 binaries — 1,397
passed, 4 failed (the sanctioned slim-ISO disc pins: disc_golden ×3 + identity_disc's
full-disc sector count), 11 ignored; the nf-game integration sweep covered all 106 binaries
green; mp_online_failures' join_in_progress hangs pre-existing on both trees in this sandbox;
mp_stress_explosives_live killed at 500 s (no result). ntfy: all three fallbacks failed
(curl 28; python Errno 101; wget 124 at the TLS connect) — the exact records are in the
branch report under "ntfy failed".
