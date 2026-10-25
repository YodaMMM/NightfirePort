## Your task: `level-11-rope-guard-hitbox`, 07000011: rounds pass through the rope guards; shell casings hang in the air

Real play in the 07000011 stairwell (Bond (20.73, 28.17, 4.83, yaw -2.635) frames 1732/1772; (17.46, 28.17, 4.81, yaw -3.580) frame 2089): the crosshair sits dead on a guard hanging on the rope, the assault rifle fires, and the bullet marks appear on the wall BEHIND him — the rounds pass straight through his body. In the same frames, the rifle's ejected shell casings hang motionless in mid-air, drawn several times too large (a dozen brass cylinders floating around the well).

A test build that kept these WAIT_SWITCH guards `enabled` (the port's WAIT_SWITCH ENTER clears `enabled`, and the bullet pass skips disabled drones, drone.rs ~13617) did not make them hittable — so a second gate stops the round: the guard's hit boxes on the rope pose [M — the per-bone HITDATA boxes, NDrone2 hit-part tests, Drone_CollisionHandler 0x140968; note a held branch found Bond's own boxes need skin 0x05000007 — check the rope guard's skin/pose has its boxes], the abseil state's damage arms [M NDrone2_DSTATE_AbseilSlide 0x169730 / AbseilHang 0x169940], and whether a disabled drone is pickable in the game [M Collide_PickObj 0x1eae80:26-30, obj+0xF0 & 0x20].

1. Drive the real level to the stairwell with real input, wait for the rappel, aim at a guard on the rope and fire; log the round's contacts per tick (which objects/boxes it tests, why the guard is not hit). Fix it [M]; real-input test: a round at a rappelling guard damages him (sister: a miss still hits the wall).
2. The shell casings: find the casing spawn/physics/draw [M — the weapon's eject, the shell object's update and scale] and why the port's casings freeze in the air and draw oversized; fix; a test on a casing's motion and scale after ejection.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-rope-guard-hitbox`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
