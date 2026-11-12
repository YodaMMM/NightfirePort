## Your task: `guard-castle-chat-gate-2`, rebase the castle chat pair work and finish it

Start from origin/s/guard-castle-chat-gate. Merged onto current origin/master it does not compile: crates/nf-view/tests/walkreal_07000006_live.rs then defines walk_the_castle_interior_to_the_party_real_damage_agent, walk_the_castle_interior_to_the_party_real_damage_00agent and the_wellhead_seals_the_shaft_on_real_input twice (master already carries them). Rebase onto origin/master keeping master's versions of those three, then finish what the branch's own report leaves open (the chat pair still breaks off early; the ch 0x11 raiser census in the NIS event layer). Before pushing run, on the final tree: nf-game vision_bit_census, nf-view fix_npcs_live (both went red with the earlier castle-chat-states branch), walkreal_07000006_live, then the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-guard-castle-chat-gate-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-12T03:33Z
