# WORKLOG

---
Task ID: level-06-audit
Agent: worker (s/level-06-audit)
Task: 07000006 behaviour audit — every placed object/character vs the game; write docs/levels/07000006-audit.md; fix small gaps with tests; large gaps → report sections.

Work Log:
- Sandbox was reset; re-cloned repo, created branch s/level-06-audit from origin/master (b312d959).
- This entry started before any code change, pushed immediately as the first checkpoint.
- Next: downloads (work-slim.7z, ISO), queue worktree, rust toolchain, then the audit sweep per docs/levels/07000008-audit.md's method.

Stage Summary:
- Branch s/level-06-audit exists on origin at the clean master point; no code changed yet.

---
Task ID: level-06-audit
Agent: worker (s/level-06-audit)
Task: the 07000006 behaviour audit — every placement/script/mission record vs the game; docs/levels/07000006-audit.md; small-gap fixes.

Work Log:
- Re-cloned after a sandbox reset; branch from origin/master b312d959; work-slim + ISO re-downloaded; ALSA headers extracted to a user prefix (PKG_CONFIG_PATH=/home/z/alsa-prefix/... — the alsa.pc needed its prefix rewritten for the linker).
- Re-ran the census probe (census_07000006.rs --ignored): all 813 placements re-dumped from the disc (30 kinds).
- Three research sweeps (decomp per kind; port per kind; scripts/mission) folded into docs/levels/07000006-audit.md: one row per placement (813 rows), per-kind game [M] chain + port side + GAP column, plus the scripts table, the mission record, doc corrections, large gaps, live checks.
- Fixes: (1) the SS operate arm now plays its sample [M 0x1a9948:86-89] — SsList::pending_sounds drained in the Drones tick (the lock's routing); test ss_operate_sound_06_live (the four 0x11b barrel traps [D census]); (2) nis.rs's light-boot comment corrected — SP #808 p0 = 0x06000066 [D census] contradicts the old "no param carries the hash".
- Ran: census_07000006 --ignored, ss_operate_sound_06_live, simplescript_live, cutscene_06_live — all green.

Stage Summary:
- Commit 8053f6e3 on s/level-06-audit carries the audit + fixes. Large gaps documented for tasking: 0xD8 RB physics (well bucket), breakable pane solidity (drone.rs:16209-16224), the dressing tux skin swap, the 007 medal gate (D-004). Latent port bug flagged: hurt.rs reads p5 as one-shot and ignores p6 (decomp: +0x15=p6 one-shot, +0x16=p5 mask [M 0x17e870:25-58]) — inert on 06, wrong on other levels' p5/p6 volumes.
