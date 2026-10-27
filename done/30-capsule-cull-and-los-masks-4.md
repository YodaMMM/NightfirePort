## Your task: `capsule-cull-and-los-masks-4`, the last two reds of the head-on cull + LOS masks

Start from `origin/s/capsule-cull-and-los-masks-3` (merge it, then origin/master) and read its report's "What's left":
1. walkreal_07000008_live `walk_the_gondola_to_the_briefcase_and_back` (:164, ch 134 never written) is red under `--test-threads=1` after the master merge: add the env-gated `[burn]` trace at the contact site and `NF_WALK_TRACE` on the fusebox->safe legs, separate a stance stall from the burn, and fix the cause (the push-tail gate is the suspect).
2. mp_net_slots_post_move_live `four_human_session_digest_is_pinned` (:285): build the pin's own commit (a7afd219) in its own target dir, dump the per-tick component hashes on both trees, find the FIRST divergent component and tick, and either fix the port to the game's rule or re-derive the pin from it with the cause cited.
Both must be green on the full suite (and the turret/armour/zoe/level_end groups that round 3 made green must stay green).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-26T18:20Z

---
## Report (done, 2026-10-26T19:06Z)

