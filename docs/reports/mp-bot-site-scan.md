# mp-bot-site-scan

Branch `s/mp-bot-site-scan`. This session continued a branch an earlier run of this same task had
started (its commits: the scan port 9ad5cfc52, the live pin 27a65e237, a pre-merge digest re-pin
74781f343), merged it with the CURRENT origin/master, verified everything on the merged tree, and
finished the battery. The sandbox had been reset since: the Rust toolchain, `work/`, the ISO, the
target dir and the ALSA prefix were all rebuilt this session (rustup stable 1.100.0, gcc-14 in a
user prefix, libasound2-dev into `~/my-project/alsa-prefix`, `work-slim.7z` + the on-foot ISO
re-downloaded and extracted).

## What's done

**The port (from the branch's reviewed commits, re-verified on the merged tree).** The game's MP
arm of `NDrone2_FindOpponent` 0x1424e0 has NO early return for a held opponent: every path out of
the hold block reaches the scan at 0x1427c4 [M-dis 0x1426bc-0x1427c0: 0x1426c4 `beqz $a0,0x1427c0`,
0x1426d4 `beq $v1,1,0x142778`, the validity+hold chain 0x1426e4-0x142770 `beqz $v0,0x1427c4` — the
scan with the opponent kept; class 0x3b (the site) skips the validity check 0x1426dc `bne $v1,2`,
and the clear arm's class test `+0xff − 2 < 2` (0x1427a8/0x1427ac `sltiu`) is FALSE for 0x3b, so
0x1427b0 `beqz $v0,0x1427c0` never clears the site]. The port's `find_opponent`
(crates/nf-game/src/mp/bots.rs) now matches: a class-2 opponent survives only while valid and
within the hold (`BOT_getAggressionMul` 0x126468 × 20, unseen ticks), the site pseudo-opponent
(index ≥ OBJ_ROW) is never cleared, and the SCAN RUNS UNCONDITIONALLY for a site holder — the old
`if i >= OBJ_ROW { return }` is gone. A changed pick resets the hold via the opponent-history edge
[`prev_opponent` latch; `BOT_handleOpponentHistory` 0x1257c0:158-162 resets +0x274/+0x270]. No
shared-Rand draws: the port adds none (the scan's pick is the nearest-candidate pass the bot
already ran; the hold math is pure).

**Tests** (all decompile-backed, no [S] in pass/fail):
- `brain_site_holder_scans_for_opponents` (nf-game --lib rigs) — red on the old early-return code,
  green now.
- `demolition_site_holder_turns_to_an_opponent` (mp_bot_objectives_live, 07000023 demolition) —
  the real-bot-match pin: a site-holding attacker turns to a live opponent who comes into view;
  two machines equal.
- `sight_cone_faces_the_way_the_bot_faces` — master's cone test, kept in the merge.

**The merge.** current origin/master (1105786cd) brought the reviewed CanBackoff rework (the probe
1.8 × mul, the `+0xc5 = 0x1e` chain). The conflicted hunks all took master's side; the auto-merge
had additionally left the branch's old `* 0.75` inside the shared `let to` block of `can_backoff` —
fixed to master's `* reach` (commit aecb63b10; `can_backoff` and its test now diff IDENTICAL
against master). `mp_perf_16_live`'s digest re-pinned ONCE for the merged tree:
0x4b0f0d0e0dbee72b (42 kills), reason on the constant (the site holder's unconditional scan +
master's backoff state — decompile-backed behaviour changes, not optimisations).

## How it was verified

- `nf-game --lib` (whole binary, `--test-threads=1`): **794 passed, 0 failed, 4 ignored** — includes
  `brain_site_holder_scans_for_opponents` and `can_backoff_probe_is_18_times_the_mul`.
- `mp_perf_16_live`: 2/2 with the new pin (42 kills).
- **Every `mp_*_live` binary one per tool call** (`--test-threads=1`, NF_ISO set), all green —
  40 nf-view binaries, 159 tests: mp_bot_objectives_live 11 (the site-holder pin inside),
  mp_world_sync_live 8, mp_scenarios_live 19, mp_combat_weapons_live 11, mp_defaults_setup 12,
  mp_enviro_mods_live 8, mp_shot_gate_live 5, mp_core_live 5, mp_site_destruction_blast_live 4,
  mp_site_damage_offline_live 4, mp_hud_real_live 4, mp_pain_options_live 4, mp_bots_live 3,
  mp_bots_moving_live 3, mp_death_effects_live 3, mp_16_slots_live 3, mp_respawn_ammo_live 3,
  mp_bot_voices_live 3, mp_team_aim_marker_live 3, mp_bodies_live 3, mp_fists_skin_live 3,
  mp_match_end_live 3, mp_shot_effects_live 2, mp_hud_16_live 2, mp_pickup_take_live 2,
  mp_pickups_visible_live 2, mp_remote_fx_live 2, mp_death_hands_online 2, mp_host_match_live 2,
  mp_bot_scan_live 2, mp_bot_weapons_live 2, mp_body_anim_live 2, mp_perf_16_live 2,
  mp_bot_armour_online_live 1, mp_bot_body_sounds_live 1, mp_bot_combat_ranges_live 1,
  mp_clock_gate_live 1, mp_status_strings_live 1, mp_menu_lists_live 5;
  plus nf-game's mp_arenas_live 1 and mp_map_objects_live 4.
- The non-live mp bins for completeness: mp_debriefing_flow 6, mp_menu_flow 11, mp_net_lobby 11,
  mp_lobby_one_screen 7, mp_screens 5 (nf-view) and mp_hud_16 7, mp_hud_data 4, mp_hud_pane 6,
  mp_never_mentioned 6, mp_status_bits 3 (nf-game).
- **The fast runner could not run in this sandbox**: background processes are killed between tool
  calls — `setsid nohup python3 tools/test/suite.py` was started (it reported alive for a moment),
  and on the next tool call `pgrep` found nothing and `suite.out` was 0 bytes, so there is no
  `tail -1 suite.out` total line to report; the per-binary list above is the suite's substitute
  (same flags the suite uses, fresh build from this tree merged with origin/master).
- `mp_stress_explosives_live`: **exit 137 (SIGKILL)** mid first test
  (`eight_humans_spamming_explosives_every_tick_stay_in_sync`, 82 bytes of output) — the same
  sandbox-memory kill the base branch's report documents; environmental, not a test failure.

## What's left

- Nothing for the port itself: the scan, the hold, the hold reset and the site-ack-visible turn are
  landed, red-tested, live-pinned, and the battery is green.
- `mp_stress_explosives_live` needs a less memory-starved sandbox to actually complete (three
  sessions in a row now).
- Live checks for the main agent (PCSX2, fresh boot) — carried from docs/reports/mp-bot-backoff.md,
  still the right pins:
  1. Drone+0xc5 of an MP bot mid-match: expect 0x1e [M 0x14c938 case 0x10 → 0x14cbd8:26 — master's
     reviewed chain].
  2. Drone+0xda of an MP bot after spawn/weapon change: expect `BOTWEAP_getWeaponAnimSet`
     0x14..0x1a [M 0x12a028:47].
  3. Drone+0x4f8 bit 0x10 of an MP bot: expect clear while moving [M 0x150218-220].
  4. Drone+0x228 bit 2 of a demolition attacker 2-3 u from the site facing it: expect set within
     ~10 ticks [M 0x175ef0].
  5. NEW for this task: a bot holding a demolition site (Drone opponent = the site object) — watch
     `NDrone2_FindOpponent` 0x1424e0 when a player walks into its sight: the scan at 0x1427c4
     should run every call (no early return) and `NDrone2_SetOpponent` should replace the site with
     the player [M-dis 0x1427c4 onwards; pins the port's unconditional scan].

## Usage

Session context/cost figures are `not visible to me` in this environment (no token or cost readout
is exposed to the worker). What I can see: the task message's own clock (sandbox expiry ~03:37 UTC;
this report written ~02:30 UTC), `git` hashes, and test outputs. No input/output token counters, no
credit balance, no dollar cost were visible anywhere in the session.

## ntfy failed

All three fallbacks failed from this sandbox (same as the previous sessions):
- curl: `curl: (28) Connection timed out after 20000 milliseconds`, exit 28.
- python urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`.
- wget: produced no output, wrapper `timeout 45` killed it, exit 124 (a first combined attempt of
  all three also exceeded the 120 s tool cap and was killed before any output was captured — the
  three attempts above were re-run separately to record each error).
