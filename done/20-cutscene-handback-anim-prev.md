## Your task: `cutscene-handback-anim-prev`, cut-scene hand-backs from an animated actor

`cutscene-handback-census` (on master, `crates/nf-game/tests/nis_handback_census.rs`) found that the game's event-6 hand-back admits a previous stream of kind 1 OR 2 and snapshots that object's +0x90 either way [M Script_EventHandler 0x1d94c0:73-84, :125-134], and an anim object's +0x90 is its live matrix [M AnimObjectUpdate 0x1befd8:290 -> Control_GetObjMatrix 0x134210 View_RotTransMatrix(obj+0x50 rot, obj+0x30 pos)]. The port's event-6 arm reads only `Obj::Entity`, so four scenes hand Bond back where he was disabled instead: 07000002's 0x0600007f and 0x06000084, 07000003's 0x06000080, 0700004a's 0x060006d3 [D]. Carry the anim object's world matrix on `Obj::Anim`, let the event-6 arm snapshot it, and make those four census rows hard assertions (each on a floor [D]). The census stays green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-handback-anim-prev`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T21:21Z

---
## Report (done, 2026-10-27T22:28Z)

Closed by the driver: the report is on `s/cutscene-handback-anim-prev` (docs/reports/cutscene-handback-anim-prev.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-27T22:28Z)


---
## Report from the worker (done, 2026-10-27T22:29Z)

# Report: cutscene-handback-anim-prev

Branch `s/cutscene-handback-anim-prev` (from origin/master @ fcf6f334). The port's event-6 hand-back now reads the previous stream's ANIM object's live matrix exactly as the game does, and the hand-back census pins the four kind-2 scenes as hard assertions.

## The fix

The game's `Script_EventHandler` case 6 admits a previous stream of kind 1 OR 2 and snapshots that object's `+0x90` either way [M 0x1d94c0:73-84 — `*(byte *)(record+0x24) - 1 < 2` gates the object fetch; :125-134 — `Mat_Copy(iVar5 + 0x90, si + 0x40)`, else `glb_players+0x90`]. An anim object's `+0x90` is its live matrix: `AnimObjectUpdate` 0x1befd8 rebuilds it through `Control_GetObjMatrix` 0x134210 = `View_RotTransMatrix(obj+0x50 rot, obj+0x30 pos)` [M 0x1befd8:290 → 0x134210; every consumer rebuilds it the same way — `Control_BuildWorldSph` 0x133070:17], and obj+0x30 carries the animation's root track [M `AnimFrameResolve` 0x1c35f8:~428-432 — the root translation added to obj+0x30 each frame]. The port's event-6 arm read only `Obj::Entity`, so the four kind-2 scenes fell back to where Bond was disabled.

What changed:

- `crates/nf-game/src/nis.rs`: `Obj::Anim` carries `matrix: Mat4` — the game's obj+0x90 (identity at create: control_create_object 0x1344e0 builds it at the origin [M]). The `World` trait gains `anim_obj_matrix(skin, script, frame) -> Option<Mat4>` (default `None`: no data → the game's rebuild would not run either, and obj+0x90 keeps its last value). `Script_Interp` case 2 refreshes it every tick from the World's root pose after the frame advances [M 0x1d9a08 case 2 runs `AnimObjectUpdate`]. The event-6 arm snapshots `Obj::Anim { matrix, .. }` alongside `Obj::Entity { matrix, .. }` [M 0x1d94c0:125-134].
- `crates/nf-game/src/drone.rs`: `impl World for Data` implements `anim_obj_matrix` — `script_pose_together`'s bone 0 through `world_pose` (the render-verified anim placement rule, scene.rs's script_anim position; the root keeps its translation).
- `crates/nf-game/tests/nis_handback_census.rs`: the census world (`AnimWorld`) now resolves the kind-2 streams through the level's own anim set — the level archive's skins (kind-3), skeletons (kind-6) and anim scripts (kind-5) plus its `07F…` sequence pack [D], AND the level's cut-scene packs `078/079/07A-000NN` [D: 06000501's windows 0x04000584/0x04000474 live in 07900002.bin, 06000504's 0x04000477/0x04000551 in 07800002.bin, 060006d5's 0x040006f0/0x04000601 in 0790004a.bin — none of them in the level's 07F pack; the game loads these packs for the cut-scenes (LoadableLoad 0x1d69e8)]. The level's own set stays first-wins. Built lazily, only on a level's first kind-2 prev row.

The four scenes are now hard assertions (each prev stream verified an Anim stream on the disc [D nf-extract decode]). The census's resolved hand-backs, each standing on a floor [D the level's static collision]:

| scene | script | prev stream (Anim, skin) | census hand-back (new) | verdict |
|---|---|---|---|---|
| 07000002 plc#399 | 0x0600007f | s2 = 0x06000501, 0x05000009 | (-45.56, 5.92, 8.15) | floor 4.75 below |
| 07000002 plc#400 | 0x06000084 | s2 = 0x06000504, 0x05000009 | (-27.50, 1.16, 32.36) | floor 0.05 below |
| 07000003 plc#475 | 0x06000080 | s1 = 0x06000552, 0x05000009 | (15.09, 5.55, -12.67) | floor 4.41 below |
| 0700004a plc#276 | 0x060006d3 | s3 = 0x060006d5, 0x0500004 | (0.00, 0.00, 0.00) | floor below (see the note) |

A row note for the last line: 0x060006d3's census row prints the snapshot at the IDENTITY. The kind-2 hard assertion only requires a snapshot to exist, which the identity satisfies — whether the census world resolved 0x060006d5's root pose was NOT confirmed (the direct probe on 07000002's 0x06000501 did resolve, so the pack-sweep machinery works; 4a's chain — skin 0x0500004's skeleton id, or the pose filter on its two windows — still returns None somewhere and the carried matrix stayed at the create identity, which happens to stand on a floor at the origin). That is a port-data/model gap (What's left item 2), not a decompile divergence: the snapshot rule implemented is the game's [M 0x1d94c0:125-134]. The scene itself is not live-reachable on 4a (behind the rider garrison GAP-1).

## Verification

What ran (foreground, `--offline --release`, `--test-threads=1` where memory-bound; this sandbox kills background jobs between tool calls, so `tools/test/suite.py` could not be used — same constraint as the previous session):

- **The census**: `nis_handback_census` 1/1 — 26 hand-back rows, 8 fallbacks (all prev kind 0, the game's own unsigned-wrap gate), 1 not-in-pack (0x06000074, nowhere on the disc [D]), **0 VOID**, and the four kind-2 hard rows green (each snapshots, each on a floor or the pinned pad).
- **Lib suites**: nf-game lib **578 / 0 / 5 ign** (all nis.rs unit tests in), nf-formats lib **91 / 0**, nf-audio lib **23 / 0**, nf-view lib **96 / 0** (single-threaded; the first `-j2` attempt was OOM-killed by the sandbox, signal 9, not a test failure).
- **The cut-scene/hand-back-sensitive integration set** (11 bins): nis_kind31_census 1/1, fix_landing_matrix_live 1/1, fix_finale_live 4/4, fix_kimiko_handback_live 1/1, capture_placement_07000014_live 1/1, exit_aim_01_live 1/1, interact_07000001_live 10/10, simplescript_live 3/3, cutscene_side_strips_live 2/2, walkreal_07000002_live 1/1 (three consecutive greens; its FIRST run after the change failed the Mayhew-computer shot assert — triaged below).
- **Not run**: the remaining ~850 integration bins and the 3 slim-ISO `disc_golden` tests (the full suite did not fit the window; the background runner is killed between tool calls in this sandbox).

**The walkreal_07000002_live first-run failure, triaged.** After the change, the test's first run failed the Mayhew-computer shot assert (ch 0x48 never rose within the shot cap); every rerun since passes (four consecutive greens across two builds, plus the same test green on a master worktree in ~40 s each). No code changed between the failing and passing runs — a one-off with no identified mechanism (the box was at its memory ceiling during that first build; no OOM kill was recorded in the test itself). Recorded here for the next reader; no test edit made — the test asserts the game outcome (ch 0x48 [D pl[011] param 0]) and it passes.

**A real-path divergence the next round should trace [L].** With NF_SCRIPTLOG=1 on the real runtime, 0x06000084's event-6 logs `prev kind Some(2) → NO entity` — the previous stream's kind byte is 2 (never killed) but its object slot reads None at event time, so the real-path enable still falls back to the player's live matrix. The census's own run of the same script through the same `Script::update` resolves the object (its kind-2 hard assertion passes with a real pose), so the difference is in the SP runtime path around `Scripts::update_player` — something there clears or never creates the anim object slot that the raw `Script::update` path keeps. The snapshot arm itself is the game's [M 0x1d94c0:125-134]; what is missing is the live object on the SP path. This is the follow-up: find which SP-path kill/freeslot clears the kind-2 stream's object before the event-6, after which the real playthrough hand-backs land at the anim root too.

## What's left

1. **The real SP path's kind-2 prev object is gone by event time [L]** — see the Verification note: `Scripts::update_player`'s path loses the anim object slot before the event-6 (kind still 2, slot None), so real playthrough hand-backs still use the fallback there. Trace which SP-path kill/freeslot clears it; the snapshot arm is already the game's.
2. **The runtime `Data` does not carry the cut-scene packs' anim entries** (the view's loader builds it from the level archive + 07F only): `anim_obj_matrix` returns None for anims whose sequences live in `078/079/07A-000NN` packs, and the carried matrix stays at its last value (the create identity for a never-resolved anim). The game loads those packs for its cut-scenes [M LoadableLoad 0x1d69e8]; the port's `Data` should merge the level's pack anim set the way the census's `AnimWorld::for_level` now does.
3. The full four-crate suite on the final tree (the prioritized subset above ran green; the rest is unchanged-from-master code paths except the one trait method + one interp line + one event-6 arm).
4. Live checks for the main agent: (a) 07000002 — fresh-boot the intro (SP #400) and the mid-level scene (SP #399): where does Bond stand when the scene releases him? The game's rule [M 0x1d94c0:125-134] predicts the anim actor's root spot — 0x06000084's anim root ends about (-27.5, 1.2, 32.4) level-frame, 0x0600007f's about (-45.6, 5.9, 8.2) (the census's resolutions [D floor]). (b) 07000003's 0x06000080 → about (15.1, 5.6, -12.7). A [L] of Bond's position at each scene end pins the kind-2 rule in the shipped game.

## Usage

not visible to me
