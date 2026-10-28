## Your task: `cutscene-handback-anim-prev-4`, the kind-2 hand-back onto master with the full suite green

Start from `origin/s/cutscene-handback-anim-prev-3` and merge `origin/master`. nis.rs will conflict with master's 07000014 vent-drop fallback (scene 0x06000833, `Obj::Anim.root` + `World::anim_root`): resolve onto YOUR general rule - one per-interp anim matrix (`anim_obj_matrix`), drop `root`/`anim_root`, and let the 0x06000833 fallback read that matrix (keep its IDENT guard: a world without the matrix keeps the strict capture). On that merged tree the full suite has three reds beyond the known frame_cost flake:
1. models_characters_live `g021_the_merge_leaves_gameplay_data_untouched` - skeleton 0 has 416 sequences, the level's own 07F has 402: the loader's 078/079/07A pack sweep adds 14. The game loads those packs at scene time [M LoadableLoad 0x1d69e8 = AnimCacheDisable + hashtable_delete_hashtype(0x4000000) + LoaderLoad per pack], not at level load - port that lifetime (load the pack's anim data when its scene starts, drop it as the game does) or prove from the decompile that the level-load sweep is equivalent and correct the test with the cite.
2. load_scene_equality `the_cut_costs_build_the_same_scene` - the digests of 07000001/02/0c/14 moved with every count equal: follows from item 1; re-pin only with the named mover.
3. bath_hatch_02_live `deep_ladder_climb_opens_the_hatch_on_e_and_exits_into_the_bathhouse` - "the use key never reached the hatch (SS #308 still state 64 at y 5.952)": the test was calibrated on the old 07000002 start; re-derive from the game's real start [L (-27.504, 1.131, 32.348) after the intro] like the -2 round did.
Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-handback-anim-prev-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-28T09:09Z
