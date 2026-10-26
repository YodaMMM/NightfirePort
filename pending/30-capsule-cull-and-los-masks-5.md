## Your task: `capsule-cull-and-los-masks-5`, the open door leaf and the mp digest

Start from `origin/s/capsule-cull-and-los-masks-4` (merge it, then origin/master) and read its report. Round 4 nailed the walkreal_07000008 stall: the port puts every glist door's leaf mesh into the collision set unconditionally (drone.rs `set_dynamic` doors.extend), and the OPEN Secure Room door #10's leaf (natural pose = the 5x6 panel at x 155.003, z -118..-123) blocks the walk. Land its fix recipe — door parts collide by the door's state the way the game's zone rule does (CLOSED blocks; OPEN/OPENING/CLOSING leaves do not) [M Doors::update's zone, 0x135738/0x1336f8] — review the SS parts (the safe door #205, the bolt #217) the same way, and run the whole door family (fix_npc_doors_live, every walkreal) before reporting. Then the mp digest (mp_net_slots_post_move_live `four_human_session_digest_is_pinned`): the round-4 method (NF_NET_HASHDBG on a7afd219 vs the tree) to the first divergent component, fixed or re-derived with its cause.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

