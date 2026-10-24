## Your task: `bond-hitbox-pose`, Bond's hit boxes follow his pose

Master tests guard rounds against Bond's 16 hit boxes (`HumanCollisionBones` 0x2c6be8 via `Collide_Jointy`, skin 0x05000007; `crates/nf-game/src/damage.rs` `bond_boxes`), but posed from one fixed standing idle pose (`BOND_IDLE_POSE`, measured from three level-start snapshots) rotated by Bond's yaw. The game poses Bond's palette every frame from his third-person animation [M — find where Bond's AnimObject palette is updated and which anim runs: standing, crouched, walking, wall-hug, ladder, zip-wire, swimming].

1. Find, from the decompile, what pose the boxes use in each player state [M], and the [L] palette for at least crouched and standing-walk (a snapshot or a gdb read of Bond's palette at 0x16c9b60-class addresses in `work/ee_start_*.bin`, or a fresh capture).
2. Pose the boxes by state (at minimum crouch vs stand) [M + L], with a test: a round passing just above a crouched Bond's head misses, the same round hits a standing Bond.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/bond-hitbox-pose`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


