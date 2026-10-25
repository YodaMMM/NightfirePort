## Your task: `level-14-cutscene-door-2`, the 14 cut-scene door leaves without changing other levels' scripts

Start from `origin/s/level-14-cutscene-door` (merge it, then origin/master). Its finding stands: script 0x06000833 streams 10/11 key the door leaves 0x02000891/0x02000892 [D], and the port never applied them to the placed door pair #116/#117. But the branch also carries the broad Script_CameraStart identity rule (the whole script base reset), which master deliberately does NOT have — master applies the identity to the CAMERA interp only (`nis.rs` case 4, branch cutscene-camera-base). Merged with master the branch turns these red (each green on master):
- walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact
- walkreal_07000008_live (the ally noise path; the gondola ride)
- zoe_gondola_realpath_live::zoe_rides_the_carriage_and_never_targets_bond
- interact_07000001_live (the armour pickup + ch 13; the entry sliding door)

1. Keep master's camera-only rule; land only the door-leaf keying (the entity stream drives the placed door object's pose/state) with its [M] cite. If the leaves need a world-space base, derive it for THOSE entities only, from the decompile.
2. All of the above green plus the door test; run the full suite on the merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-cutscene-door-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T22:57Z
