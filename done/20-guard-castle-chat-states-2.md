## Your task: `guard-castle-chat-states-2`, the castle chat states without breaking guard vision

Branch from origin/s/guard-castle-chat-states and merge origin/master. Merged alone onto master it turns two tests red that master passes: vision_bit_census::every_game_vision_bit_writer_enter_sets_vision_on (nf-game) and fix_npcs_live::an_idle_gate_guard_sees_and_attacks_bond_on_the_same_floor. Read both messages: the new states' ENTER probably misses the game's vision-bit write, or the redirect catches guards that are not the 07000006 chat pair. Fix the port the game's way [M CastleChatGuard1 0x1612e0, PlayScript 0x15ba68]; never weaken the census. Then run the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-guard-castle-chat-states-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-12T02:17Z

---
## Report (done, 2026-11-12T03:47Z)

