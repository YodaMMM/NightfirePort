# level-08-audit report

Branch `s/level-08-audit` (from master 30598272). Task: the 07000008 behaviour audit — one row per
placement/script/mission record against the game.

## Done

1. **The per-placement audit table** — `docs/levels/07000008-audit.md` now carries both parts: the
   merged 996-entry-sweep audit (kept verbatim) and the new per-placement part: **all 308 main-map
   placements, one row each** (32 kind sections), each with the game's [M/D] chain (creator in
   `parsemap_create_dynamic_objects` 0x1d04f0 / the static path, update, use/damage/event handlers),
   the port's behaviour, and a GAP column. Re-dumped from the disc this session with
   `census_07000008 --ignored` (308/308, 0 unresolved).
2. **The scripts section** — 13 kind-7 scripts in 07000008.bin + the two placement-named NISes found
   in the packs (`0x06000077` in **07800008.bin** 3880 ticks/30 streams; `0x06000078` in
   **07900008.bin** 342 ticks/15 streams), every live script decoded stream-by-stream with
   `dump_sp_script` [D]: the ride script's ch-83 write at t1410, the death NIS's ch-0x62 write at
   t89, the bottom cinematic's ch-5 write at t341 and its MID-script hand-back at t158, the intro
   NIS's freeze/hand-back set, the lights on/off pair (SP slots [M 0x1dbc08]), SafePickUps's shelf
   re-pose keys, the fusebox's PCB script, the copter's spline flight + the 0xF hand-offs.
3. **The mission-record section** — record 3 [L ee_start_05.bin @0x2a4354; objectives @0x2a3e88]:
   0x0400004 completes on ch 0x4A (revealed 0x49 = plate #237), 0x0400005 on ch 0x4B (revealed 0x4A;
   the writer is copter #45's arrival [M 0x17dbb8]); no per-objective fails; the death chain
   (ch 0x62 -> MissionState 3 [M 0x182c60:22-43, 50-57]) and the end (six Mayhew channels -> ch 100 ->
   `case 0x7000008: LevelToEndTo = 0x7300008` [M 0x182c60:85-116]) pinned with the decomp lines.
   The fresh boot carries no pre-on channels [L @0x26fc90].
4. **Small gap fixed: the SS operate sound** — `SS_Update` 0x1a9948's 0x40 arm plays
   `Sound_Play3D(100.0, ext+0x14 = param 4, at obj+0x80)` before latching state 1 [M 0x1a9948:86-89,
   decomp 001a9948 lines 88-92]; the port skipped it (the 06 audit's finding, still open on master).
   07000008's safe door #205 carries sample 0x1d6 (470) [D census] — the safe opening was silent.
   Fixed in `crates/nf-game/src/ss.rs` (`SsList::tick` takes the sim's sound sink; 0xffff rows stay
   silent = the game's bank miss), the caller updated in `drone.rs`. Two unit tests:
   `the_0x40_operate_arm_plays_the_operate_sound_then_latches`,
   `the_ffff_sound_row_latches_without_a_sound` — both green.
5. **Corrections** (in the audit file, level doc untouched per the shared-file rule): the spawner
   groups are 4/3/5 + 1/2 (§7's "group 2" drift); trigger #80's ch 43 has no reader on the level
   (dead content); all 26 drones are placed (9 live from the load incl. the ally #265 / 11 wait-42 / 6 wait-44).

## Verification

- `cargo test --release --offline -p nf-view --test census_07000008 -- --ignored --nocapture` — the
  308-row dump + the hurt-geometry dump (the audit's [D] base).
- `dump_sp_script`/`list_scripts` over 07000008.bin, 07800008.bin, 07900008.bin — the scripts section.
- `cargo test --release --offline -p nf-game --lib ss::tests` — the new SS sound tests green (the
  module filter ran 11 tests: 11 passed, 0 failed).
- `cargo check --release --offline -p nf-game` — clean (pre-existing warnings only).
- The 08-relevant test binaries run directly on the final tree (all green):
  `simplescript_live` 2/2, `fix_safe08_live` 5/5, `pickups08_live` 5/5, `fix_fusebox_cut_live` 4/4,
  `decomp_audit_07000008_live` 1/1, `hurt_07000008_live` 2/2, `walk_07000008_live` 3/3,
  `walkreal_07000008_live` 4/4 (153.7 s), `cutscene_08_actors_live` 2/2.
- The full four-crate suite via `tools/test/suite.py` on the final tree: build ok (811 binaries,
  1059 s), the run phase was still in progress at the finish deadline — the partial read and the
  caveat are below.

Suite partial at finish (the run phase is alphabetical across 811 binaries and the window closed
first): **270 binaries completed — 1650 passed, 4 failed, 82 ignored**. The 4 failures are the
documented slim-ISO set: the 3 `disc_golden` tests (allowed by the task) + `identity_disc` (the
full-disc identity check — expected on the trimmed onfoot ISO). `net_join_live` CRASHed (exit -9,
the OOM class the 05 session recorded for the net tests). No TIMEOUTs, and none of the 08 paths
failed (the 08 binaries were also run directly — see Verification). The remaining ~540 binaries
were not reached; the next session can re-run the suite to completion.

## What's left (large gaps -> follow-up tasks)

- **The sky +0x4a word** (four 08 rows p4=1; shared with 05): the renderer consumer of the sky
  record's +0x4a is unported/unverified [M 0x1e79a0:19-36]. Needs the sky task.
- **The 007-token medal gate (D-004)**: [M 0x18dda0:~93] gates type-6 pickups on the gold medal; the
  port spawns #279/#280 unconditionally (documented deviation, pickup.rs:557). Needs the BNUS medal
  carry at load.
- **The ride/intro per-instance extras**: the in-car ch-167 destroyables + the 12 ride sounds
  [D 0x0600004C]; the intro NIS's 30-stream body (18 guest anims, one camera) — the arms are pinned,
  the bodies need the ride/cutscene rigs and the main agent's renders.
- **The 08 per-instance follow-up tests** (the kind is pinned, the instance is not): the doors'
  E-open + #122-after-ch-137, the two ladders, the zip ride, the wave releases (42/44/73), the music
  AND-mux (ch 104 after all three plates), the sound triggers #106/#288.

## Live checks for the main agent

In `docs/levels/07000008-audit.md` §"Live checks for the main agent": the safe-door sound (sample 470),
the intro NIS render (plate #237), the bottom cinematic render (the t158 hand-back, the t341 ch-5
write, the copter's spline flight), the three-plate music AND, the spawner waves' screenshots.

## Usage

not visible to me (no token, credit or context-window figures are exposed in this environment; the
session's tool output shows no cost readout).
