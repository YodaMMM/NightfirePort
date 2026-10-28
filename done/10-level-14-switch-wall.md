## Your task: `level-14-switch-wall`, what the player sees at Countdown Part 1's sabotage switch

On 07000014 the E-use switch oDroneTrigger1b (touch out 0xA0, mask 4) sits at (-60.30, 0.61, 38.37); the port draws nothing there, so a player cannot tell where to press E. Find what the game shows at that spot [D the placements/objects/glists within a few units, any 0x3b channel-show pairs, ScriptPlayers whose scripts draw there; M the touch's own draw, if any] and the M.Hatch indicator ScriptPlayers #947/#950/#952 (script 0x06000708, triggers 0x85/0x54) - which objects, frames and channels. Port any missing data/sim link (a placement the scene skips, a show channel not applied, a script object not spawned). List exactly what must DRAW (object hash, position, frame) in the report so the main agent can render and compare with the PS2. Test the data/sim side. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-switch-wall`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-28T12:55Z

---
## Report (done, 2026-10-28T14:44Z)

