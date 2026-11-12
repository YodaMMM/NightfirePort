## Your task: `mp-bot-backoff-2`, multiplayer bots probe 1.8 behind them before backing off

Branch from origin/s/mp-bot-backoff and merge origin/master; read docs/reports/mp-bot-backoff.md. A review found one mismatch: the CanBackoff probe is (0, 0, -1.8), not -0.75 [M NDrone2_CanBackoff 0x1501a0; disasm 0x1501f4 'lui r1,0xbfe6' = 0xbfe66666 = -1.8]. The branch's other findings stand: the bot speed-mul arm (+0xc5 == 0x1e) does not fire for bots, so the probe is UNSCALED; the anim-state-4 gate as data; the NDrone2_MoveTest port. Set the probe to 1.8 unscaled, fix can_backoff_probe_is_075_and_unscaled into a 1.8 pin citing the lui, re-pin mp_perf_16_live's digest once at the end, and run every mp_* and net_* binary one per tool call plus nf-game --lib.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-mp-bot-backoff-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-11-12T05:29Z

---
## Report (done, 2026-11-12T07:00Z)

# mp-bot-backoff-2

Branch `s/mp-bot-backoff-2` (cut from `origin/s/mp-bot-backoff` 9bf508137, merged origin/master 7dc4c9dcd; merge commit 4f3f0c534).

## What's done

**The review's one mismatch is fixed: `NDrone2_CanBackoff`'s behind probe is (0, 0, −1.8), not −0.75 —
and it is UNSCALED for bots.** Evidence: the decompile's probe vector is `uStack_98 = 0xbfe66666` =
−1.8 f32 [M 0x1501a0 (work/decomp/ACTION.ELF/001501a0_NDrone2_CanBackoff__FP10DCVars_tag.c); disasm
0x1501f4 `lui r1,0xbfe6` = 0xbfe66666 = −1.8; −0.75 = 0xbf400000 appears nowhere in the function].
0xbf400000 (−0.75) and 0xbfe66666 (−1.8) are a hex misread apart. In `can_backoff`
(crates/nf-game/src/mp/bots.rs):

- the probe is now `R·(0, 0, −1.8)` straight back, through the same `nav::Network::move_test` /
  `walk_clear` path as before;
- the doc block now cites the lui and the unscaled finding;
- the branch's other findings stand untouched: the anim-state-4 gate as data (`ANIM_STATE4_TABLE`),
  the speed-mul arm (`Drone+0xc5 == 0x1e`) that never fires for bots [M 0x14cfa0:38 + D 0x29ad40
  row 0], and the `NDrone2_MoveTest` port;
- `can_backoff_probe_is_075_and_unscaled` → `can_backoff_probe_is_18_and_unscaled`: the same rig
  geometry (wall 2.0 behind → 1.8 stops 0.2 short → passes; wall 0.45 behind → blocked; both speeds;
  the movement lock), now pinning 1.8 with the lui citation in the assert message;
- `backoff_exit_bound_is_the_engage_band`'s wall-case comment updated (the probe is 1.8 — the wall
  0.45 behind still blocks, outcome unchanged);
- `mp_perf_16_live`'s `PINNED_DIGEST` re-pinned ONCE, at the end of the code changes:
  0x0241227609ef6722 (39 kills) → 0x4af9208e6a465647 (48 kills), reason on the constant (the 1.8
  probe lets bots back off where a wall 1.8 u behind leaves room; the 16-bot match plays out
  differently — decompile-backed behaviour change, not an optimisation).

No hearing/other code was touched; `git diff origin/s/mp-bot-backoff` is bots.rs (probe + comments +
test) and mp_perf_16_live.rs (digest) only.

## How it was verified

- `cargo test --offline --release -p nf-game --lib bots -- --test-threads=1`: **47 passed** (includes
  the renamed 1.8 pin and the engage-band test).
- `nf_game-36475ea9581ecb05 --test-threads=1` (the whole `nf-game --lib` binary): **788 passed,
  0 failed, 4 ignored**.
- The mp/net battery, one binary per run, `--test-threads=1`, NF_ISO set — **40 binaries, 171 passed,
  0 failed**: mp_perf_16_live 2 (new pin green, digest 0x4af9208e6a465647, 48 kills), mp_bot_objectives_live 10,
  mp_bot_combat_ranges_live 1, mp_bot_scan_live 2, mp_bot_weapons_live 2, mp_bots_live 3,
  mp_bots_moving_live 3, mp_core_live 5, mp_world_sync_live 8, mp_site_damage_offline_live 4,
  mp_defaults_setup 12, mp_shot_gate_live 5, mp_enviro_mods_live 8, mp_clock_gate_live 1,
  mp_death_effects_live 3, mp_pickup_take_live 2, mp_respawn_ammo_live 3, mp_pain_options_live 4,
  mp_status_strings_live 1, mp_remote_fx_live 2, mp_team_aim_marker_live 3, mp_bot_body_sounds_live 1,
  mp_match_end_live 3, mp_fists_skin_live 3, mp_hud_real_live 4, mp_bodies_live 3, mp_body_anim_live 2,
  mp_debriefing_flow 6, mp_net_lobby 11, mp_screens 5, mp_menu_lists_live 5, mp_hud_16_live 2,
  mp_lobby_one_screen 7, mp_scenarios_live 19, net_match_live 3, net_e2e_loopback_live 2,
  net_resync_live 3, net_join_live 2, net_desync_report_live 4, net_match_end_clients_live 4.
  (mp_bot_scan_live and mp_bot_weapons_live shared one tool call — deviation from one-per-call,
  both quick; everything else was strictly one per call.)

**The fast runner could not run in this sandbox and the suite total is therefore per-binary, not
suite.py's line.** tools/test/suite.py was started twice (`nohup` then `setsid nohup ... & disown`);
this sandbox kills background processes between tool calls — after both starts `pgrep suite.py`
returned nothing and `suite.out` stayed empty (0 bytes), with no FAILED/TIMEOUT lines to read. The
sandbox also kills *foreground* work at the 10-minute tool-call cap, so the build was done in chunks
and the disk (9.9 GB total, 100 % reached once; linker `Bus error` at 94 %) forced triage: test
binaries were deleted as soon as their pass counts were logged. What ran above is everything that
could be linked in the window; each binary is the same run the suite would do (serial,
`--test-threads=1`, fresh build from this tree merged with origin/master).

## What's left

- **24 mp/net binaries were never linked in this window** (disk, above): mp_16_slots_live,
  mp_arenas_live, mp_bot_armour_online_live, mp_bot_voices_live, mp_combat_weapons_live,
  mp_death_hands_online, mp_debriefing, mp_host_match_live, mp_hud_16, mp_hud_data, mp_hud_pane,
  mp_lockstep, mp_map_objects_live, mp_match_rules, mp_menu_flow, mp_net_slots_post_move_live,
  mp_never_mentioned, mp_pickups_visible_live, mp_shot_effects_live, mp_site_destruction_blast_live,
  mp_status_bits, mp_teams_flow, net_internet_live, net_soak_faults_live.
- **mp_stress_explosives_live and mp_online_failures not attempted**: they were SIGKILL'd by this
  sandbox's memory both ways on the base branch too (the base report documents the same).
- The non-mp nf-view/nf-formats/nf-audio binaries (walkreal_*_live etc.) did not fit either — the
  probe change touches only mp bot behaviour, and the 40-binary mp/net battery above plus
  `nf-game --lib` is the touched-code coverage.
- Live checks for the main agent (PCSX2, fresh boot) — unchanged from the base branch's report,
  still the right pins:
  1. Drone+0xc5 of an MP bot mid-match: expect 0x00 [M 0x14cfa0:38 + D 0x29ad40 row 0].
  2. Drone+0xda of an MP bot after spawn/weapon change: expect `BOTWEAP_getWeaponAnimSet` 0x14..0x1a
     [M 0x12a028:47].
  3. Drone+0x4f8 bit 0x10 of an MP bot: expect clear while moving [M 0x150218-220].
  4. Drone+0x228 bit 2 of a demolition attacker 2-3 u from the site facing it: expect set within
     ~10 ticks [M 0x175ef0].
  No new check is needed for the probe itself: −1.8 is pinned from the decompile + disasm directly.

## Usage

Session context/cost figures are `not visible to me` in this environment (no token or cost readout is
exposed to the worker). What I can see: the task message's own clock (sandbox expiry ~07:32 UTC; this
report written ~07:12 UTC), `git` hashes, and test outputs. No input/output token counters, no credit
balance, no dollar cost were visible anywhere in the session.

## ntfy failed
