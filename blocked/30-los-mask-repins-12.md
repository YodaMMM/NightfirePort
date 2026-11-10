## Your task: `los-mask-repins-12`, the 07000012 walks green again under the game's guard sight

Branch from origin/master (823ff190b turned on the game's guard sight: per-object masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push, FellOutMap). Read docs/reports/los-mask-repairs.md on origin/s/los-mask-repairs (reports do not go to master). These walks were ignored because the game-correct sight moved their fights: walk_spawn_to_the_shaft_exit_real_damage_operative. Read each ignore message. FIRST a behaviour check: door plc 543 at (9.9, 1.1, -14.5) holds state 0x2 at 0.50 for ~200 ticks after the E press; find why (the game's door anti-crush with a body in the leaf, or a port bug) [M Door_Activate 0x135be0 and the door mover], fix it if the port is wrong. LIVE FACT (PCSX2, 2026-11-10, fresh load, guards asleep): Bond placed at (9.9, 1.1, -13.0) facing south, Use pressed, stick held forward 240 frames: he ends at (6.21, 1.13, -25.40), 11 u past the door, so the game's door lets him through on that approach. If the port's leaf still holds at 0.50 there with no guard in it, that is a port door bug. Then re-route each walk with real input so it passes, and remove its ignore. Never lower damage or health, no top-ups; helpers gated to 07000012 and the difficulty. Run walkreal_07000012_live (and its sister binaries) and interactables_07000012_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-12`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-10T21:22Z

---
## Report (blocked, 2026-11-10T22:01Z)


---
## Report from the worker (done, 2026-11-10T22:01Z)

Closed by the driver: the report is on `s/los-mask-repins-12` (docs/reports/los-mask-repins-12.md); the worker didn't run finish.sh.
