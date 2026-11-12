## Your task: `guard-castle-chat-gate-3`, the castle chat pair without touching other guards

Start from origin/s/guard-castle-chat-gate-2 merged with origin/master. The chat pair itself checks out against the game [L PCSX2 fresh 07000006: both pair drones in CastleChatGuard1 0x36 from frame 0, unchanged for 2400 frames with Bond idle at the start, ch 0x11 0; the port on this branch matches]. But the full suite on master + this branch has 4 reds that pass on master: guard_parity_live::a_tower_guard_outside_his_civilian_states_can_attack_on_sight and guard_ai_other4_live::investigate_stands_down_to_idle_when_nothing_is_found (generic guard AI - the change reaches guards outside the chat pair: find which shared arm, e.g. look_for_opponent's sight-bit gate or the enemy_alerts route, and keep it to what the decompile says for those states), rook06_realpath_live::the_switch16_guard_fights_through_the_door47_doorway, and walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage. Each red gets a cause with the address; a test that was wrong against the decompile gets corrected with the address; nothing is weakened. Then the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-castle-chat-gate-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-12T05:51Z
