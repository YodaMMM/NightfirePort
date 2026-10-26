## Your task: `capsule-cull-and-los-masks-6`, the mp digest the cull branch moves

Start from `origin/s/capsule-cull-and-los-masks-5` (merge it, then origin/master; the door-state filter is landing separately as `door-state-collision` — drop it from this branch if it lands first). On origin/master mp_net_slots_post_move_live `four_human_session_digest_is_pinned` is GREEN; on this branch it reads 0xbc5e2155a3786e9c. So the divergence is this branch's cull/LOS/gt.rs content, not master's: diff the per-tick component hashes (crates/nf-view/tests/mp_digest_diag.rs, NF_NET_HASHDBG) between origin/master and this branch to the first divergent component and tick, name the change that moves it, and either fix it to the game's rule or prove the new value from the game [M] before re-pinning. Every other test that round 3/5 made green must stay green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-26T23:33Z
