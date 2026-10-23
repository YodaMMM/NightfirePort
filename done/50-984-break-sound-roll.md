## Your task: `break-sound-roll`, the breakable kill sound's Rand_Rand(100) roll in the game's draw order

`Break_Kill` 0x12b970:65-76: kind 2 plays 0x2E with no draw; every other kind rolls `Rand_Rand(100) < 0x29` → 0x5F2, else 0x5F3. The port (crates/nf-game/src/breakable.rs `kill`) always plays 0x5F3 and skips the draw, because inserting the draw shifted the shared RNG stream and broke escort_follow_07000001's timing (task 959 measured 7.0 s vs 18-46 s stalls).

1. Establish where the draw sits in the game's per-frame RNG order: who else draws from `Rand_Rand` on the frames a vase/bust dies (the bullet pass, debris create 0x1ed548 / 0x1aaa00, the drone AI). Cite every consumer between the bullet hit and the kill in frame order [M].
2. Put the roll in at the matching position so the port's stream matches the game's. If escort_follow then stalls, find out why its timing depends on the stream (it should not depend on a vase sound) and fix that, with evidence.
3. Tests: kind 0 kill → 0x5F2 or 0x5F3 by the roll (seeded), one draw consumed; kind 2 → 0x2E, no draw. If a live check is needed, say which counter to read in PCSX2 (Rand seed address) across a vase kill on 07000007.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/break-sound-roll`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-23T15:32Z

---
## Report (done, 2026-10-23T17:27Z)

Closed by the driver: the report is on `s/984-break-sound-roll` (docs/reports/984-break-sound-roll.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-23T17:44Z)

