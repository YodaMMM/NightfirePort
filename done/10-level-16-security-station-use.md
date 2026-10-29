## Your task: `level-16-security-station-use`, Countdown Part 3's security station works when the player uses it

On 07000016 the security station does not work for a player: walking up and using it does not switch its screens (and whatever else it should do). The existing test `dynobj_glass_live::the_16_screens_swap_with_their_channels` only SETS the station's channel 0x24 directly - it never proves the player can raise it. Reproduce on real input (`step_bond`: walk to the station, face it, press Use - no channel pokes, no teleport past a blocker): find the station's placements and what raises 0x24 [D the SecurityStation / its touch or interactable, mask, out channel; M Player_Activate 0x1a5170 and the activate path for that class, Trigger_Activate 0x1b2318], what the game shows/does when it is used (the screens' 0x3b objects, any text, objective, doors), and port every missing link. Test end to end from a fresh load on real input: the use fires, ch 0x24 rises, the screens' show state swaps, and anything downstream the game does. List exactly what must DRAW for the main agent. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-security-station-use`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T11:14Z

---
## Report (done, 2026-10-29T12:37Z)

Closed by the driver: the report is on `s/level-16-security-station-use` (docs/reports/level-16-security-station-use.md); the worker didn't run finish.sh.
