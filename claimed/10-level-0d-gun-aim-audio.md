## Your task: `level-0d-gun-aim-audio`, the Chain Reaction mounted laser gun's aim line and fire sound

On Chain Reaction Part 2 (0700000d) the deck's mounted laser gun now draws its beam while firing and clears it on release (s/gunbeam-0d), but two things are still wrong in play: the beam is OFF AXIS from where the gun points (depending on the angle), and the fire audio stutters ("pretty jittery at first then actually plays"). Fix both from the game: the beam's start and direction are the barrel transform's muzzle and its fire direction [M GunImp_Update 0x1320d8:394-407 PositionBeam(muzzle, ray hit) — the muzzle point, the barrel matrix the aim drives, the ray it casts], so derive both from the same matrix the gun draws with; the fire sound is the TurretGuns entry's +0x18 sound (1307 for the laser [D]) — find whether the game plays it once per shot, loops it while the trigger is held, or restarts it each tick [M the Sound_Play3D / looping-handle calls in GunImp_Update], and play it the same way. Tests: the beam's origin is at the drawn muzzle and its direction matches the barrel's forward axis at several aim angles [M]; the fire sound starts once and holds (or repeats) on the game's cadence while firing.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-gun-aim-audio`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T02:54Z
