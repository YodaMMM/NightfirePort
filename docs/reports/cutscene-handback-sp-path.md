# Report: cutscene-handback-sp-path

Branch `s/cutscene-handback-sp-path` (from origin/master @ fcf6f334, then merged `s/cutscene-handback-anim-prev` — see "A note on the base" below). The kind-2 (anim) event-6 hand-back now works in REAL play: the SP runtime path snapshots the anim object's live matrix and Bond comes back at the census's resolution in all three reachable scenes.

## What's done

**1. The SP-path kind-2 object slot — closed as a misdiagnosis, no code needed.**
The previous round's [L] (`NF_SCRIPTLOG`: 0x06000084 `prev kind Some(2) → NO entity`) was taken on the pre-anim-prev tree, whose event-6 arm matched only `Obj::Entity` [M 0x1d94c0:125-134's gate admits kinds {1,2}]. On that arm, a LIVE `Obj::Anim` in the previous stream's slot produces exactly that log line — the snapshot match fails, not the slot. Evidence that the slot is never cleared on the SP path:
- The port's only writers of a dead object slot under a live kind are `kill()` (which zeroes the kind byte with the slot — `kind` would read 0, not 2) and `entity_start`'s type-swap (which sets kind 1) [M crates/nf-game/src/nis.rs, `Script_KillStream` 0x1d9860's port / `Script_EntityStart` 0x1d8d78's port]. A kind-2-with-`None`-slot state has no writer in `nis.rs`.
- With anim-prev's arm on the tree, the real SP path logs the object found: `[scriptlog] event6 hash 06000084 stream 3 prev obj 2 (kind Some(2)) snapshot (-27.50,1.16,32.36)` — and `Script_Free` applies it: `[scriptlog] free hash 06000084 enable at (-27.50,1.16,32.36) mode 0` [L port runtime NF_SCRIPTLOG].
So "find which SP-path kill/freeslot clears it and match the game" resolved to: nothing clears it; the arm was the gap, and the arm is already the game's [M 0x1d94c0:73-84, :125-134].

**2. The runtime `Data` now carries the cut-scene packs' anim sets.**
`nf-view`'s loader (`load`, after the loadout-availability snapshot, before the weapon/g-021 merges) sweeps the level's `078/079/07A-000NN` packs and merges kind-6 skeletons, kind-5 scripts, kind-4 sequences and kind-3 skins — the census's `AnimWorld::for_level` rule, now on the runtime path:
- Game evidence: `LoadableLoad` 0x1d69e8 = `AnimCacheDisable` + `hashtable_delete_hashtype(0x4000000)` (the 0x04-top-byte sequences) + `LoaderLoad(2, pack)` in a loop [M 0x1d69e8 decompile]; registration is first-wins (`hashtable_additem` on a miss, `AnimLoadFile` 0x1bdf40:43-47 [M]). The pack sequences are where the cut-scene anims' root tracks live [D: 06000501's windows 0x04000584/0x04000474 in 07900002.bin, 06000504's 0x04000477/0x04000551 in 07800002.bin, 060006d5's in 0790004a.bin — none in the levels' 07F packs].
- Merge rules: `or_insert` on every table (the level's own entries stay first-wins, as the census does); a pack sequence additionally skips any hashcode ANY bucket already holds, so an appended entry never shifts an existing per-skeleton index and never flips a `by_hash`/`seqs` resolution (Scene builds both after the merge). `share_datums` re-runs after the sweep (`AnimPostLoadInit` 0x1be1a8 runs per skin-file load [M]).
- `// [S] unverified` port choice, out of pass/fail: where a hash would collide, the game holds the PACK's copy during the scene (the 0x1d69e8 sequences wipe) while the port keeps the level's (census-consistent). No collision exists on the pinned rows [D].
- Placement matters: skins merge AFTER the loadout snapshot (the availability gate still sees the level's own set only — the qworm-not-on-0c rule) and BEFORE the cut-scene-actor wanted sweep (a pack skin satisfies its ScriptPlayer placement locally — the pack IS the game's source for those actors).

**3. The real-path test: `crates/nf-view/tests/handback_sp_path_live.rs` (3/3).**
Each scene plays through the real runtime — `Sim::step_bond` per tick, `Scripts::update_player` in the control walk, no skips — and asserts Bond's position at the hand-back (`Script_Free`'s `Player_Enable` → `Bond::stand_at` [M 0x1da880:44-48]) against the census row:
| scene | script | hand-back (real path) | census row | floor [D] |
|---|---|---|---|---|
| 07000002 plc#400 (intro) | 0x06000084 | (-27.50, 1.16, 32.36) | (-27.50, 1.16, 32.36) | 0.05 |
| 07000002 plc#399 (mid-level) | 0x0600007f | (-45.56, 5.92, 8.15) | (-45.56, 5.92, 8.15) | 4.75 |
| 07000003 plc#475 (dragon wedge) | 0x06000080 | (15.09, 5.55, -12.67) | (15.09, 5.55, -12.67) | 4.41 |
All three bit-equal to the census's resolutions (`NF_SCRIPTLOG` lines above; the assert tolerances are 0.6 x/z, 0.9 y — the measured values match the census exactly). Test setup, not asserted as game behaviour: the scenes' start gates stand behind whole-objective chains, so the tests drive the chain roots — 07000002: the captor-death/rescue channels (0x29/0x2A/0x88/0x49) poked + Bond teleported into the repeating bathhouse touch #495 (out 0x3C [D audit]) so the REAL mux machinery writes AND #397 → 0x3B → SEQ #496 → 0x35 (a direct ch-0x35 poke is cleared by the muxes before the SP reads it — they re-write their outputs from the inputs every tick [M Trigger_Update 0x1b1a78's AND/SEQ_AND cases]); 07000003: ch 0x36 poked (its TouchOnce plates #416/#417 write it only on hit and never clear it). Everything from the SP's own start gate on — SP_Update case-0 load, Script_Play, the events, the event-6 snapshot, the enable — is the real path.
A doc correction found on the way: 07000003-audit's "SP #475 … mask 4" disagrees with the runtime parse (mask 1, out 0x36 — the same out-channel family as #399); the parse wins, the test comments cite it.

## A note on the base

The task text says anim-prev is "on master"; it was not — `origin/master` @ fcf6f334 carries the census (4cd6b875) but NOT the anim-prev work (the `Obj::Anim` matrix, the event-6 kind-2 arm, `AnimWorld::for_level`'s pack sweep — all only on `s/cutscene-handback-anim-prev`). This branch merged that branch first (single conflict: docs/WORKLOG.md add/add, resolved keeping both entries) and built on it. No anim-prev code was modified.

## How it was verified

Foreground runs, `--offline --release`, `NF_ISO` set (this sandbox kills background jobs between tool calls — the `suite.py` fast runner died twice with an empty output file before its build phase printed; same constraint the previous two sessions recorded):

- **The new test**: `handback_sp_path_live` 3/3 (repeated green across four runs, including with `NF_SCRIPTLOG=1`).
- **The census**: `nis_handback_census` 1/1 (26 hand-back rows, the four kind-2 rows hard, 0 VOID — unchanged).
- **Lib suites**: nf-game **578/0/5 ign**, nf-formats **91/0**, nf-audio **23/0**, nf-view **96/0** (`--test-threads=1`; the parallel attempt was OOM-killed, signal 9, the sandbox's documented ceiling).
- **The cut-scene-sensitive integration set**: walkreal_07000003 1/1, fix_kimiko_handback_live 1/1, capture_placement_07000014_live 1/1, simplescript_live 3/3, exit_aim_01_live 1/1, interact_07000001_live 10/10, cutscene_side_strips_live 2/2.
- **walkreal_07000002_live — FAILING here, exonerated by bisect.** It fails the Mayhew-computer shot assert (line 182, ch 0x48 never rises within the shot cap) **4/4 in this sandbox**: twice with this task's loader merge, once with the merge stashed (the plain anim-prev tree), once with `NF_SILENT=1`. The walk itself reaches the desk normally; the shot waits out. The identical code ran green 4 of 5 times upstream (the anim-prev report documents the same assert failing once there, "one-off, no identified mechanism"). With and without the task's changes failing identically, the failure is not this task's — recorded here for the next reader with the bisect recipe. The intro (SP #400) and the mid-level route inside that test DID play through, including the real SP #400 hand-back.
- **Not run**: the remaining ~850 integration bins and the 3 slim-ISO `disc_golden` tests (the window; the families above are the ones this task's surfaces touch — the loader merge + the census + the cut-scene set).

## What's left

1. The full four-crate suite on a sandbox where the background runner survives (or a longer window) — in particular the rest of the integration bins against the loader merge (the pack sweep runs on EVERY SP-level load; the families above are green, the long tail is unexercised).
2. The walkreal_07000002 computer-shot flake: reproduce on a machine where it fails WITH master (no anim-prev) to split "sandbox FP drift" from a real regression — this sandbox's bisect only covers anim-prev ± the loader merge. If it reproduces on master, the shot's FP path (the bullet vs the prop bounds −49.37..−48.35 / 6.24..6.65 / −24.05..−23.22 [L port props]) is the place to look.
3. Live checks for the main agent (the anim-prev report's item, still open): fresh-boot 07000002 (SP #400 at level start, SP #399 mid-level) and 07000003 (SP #475 at the dragon wedge) — where does Bond stand when each scene releases him? The port now predicts the census rows above; a [L] of Bond's position at each scene end would pin the kind-2 rule in the shipped game end-to-end. (PCSX2: `--bond` not applicable — these are cut-scene hand-backs; the positions are where the camera hands back.)
4. The `[S]` collision rule (pack-vs-level duplicate sequence hashes): if a live check ever shows a cut-scene anim playing the level's copy where the game plays the pack's, flip the merge order for kind-4 under a `LoadableLoad`-faithful wipe model.

## Usage

not visible to me

## ntfy failed

All three fallbacks ran; the ntfy.sh daily message quota rejected every attempt (HTTP 429):
- curl: `{"code":42908,"http":429,"error":"limit reached: daily message quota reached; increase your limits with a paid plan, see https://ntfy.sh","link":"https://ntfy.sh/docs/publish/#limitations"}`
- python urllib: `urllib.error.HTTPError: HTTP Error 429: Too Many Requests`
- wget: `HTTP request sent, awaiting response... 429 Too Many Requests` / `ERROR 429: Too Many Requests.`
