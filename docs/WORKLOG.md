
---
Task ID: level-08-audit
Agent: worker (s/level-08-audit)
Task: the 07000008 behaviour audit — one row per placement/script/mission record in docs/levels/07000008-audit.md (game [M/D] chain vs the port, GAP column); fix small gaps with tests; large gaps → report sections.

Work Log:
- Branch s/level-08-audit created from origin/master (30598272). Sandbox intact from the prior session (work/, ISO, ../nf-queue present). Master already carries the per-kind census (07000008.md §7, 308 placements) and the 996-level-08-decomp-audit entry sweep — my table is the per-placement expansion of those.
- docs/WORKLOG.md started; pushing this checkpoint before any other work.
- Next: suite.py in background; run census_07000008 (tests/census_07000008.rs --ignored) for the full per-placement dump; reuse the gen_audit kind knowledge base (05/06 method) for the per-row table; audit the level-bin ride script streams + mission record 3; small-gap fixes if they fit.

Stage Summary:
- Branch s/level-08-audit exists on origin at the clean master point; no code changed yet.

---
Task ID: level-08-audit
Agent: worker (s/level-08-audit)
Task: (continuation) the per-placement audit assembled; the SS operate-sound gap fixed.

Work Log:
- Ran census_07000008 --ignored: all 308 placements re-dumped (32 kinds; rainbox 14, low-cover 25, all four muxes 0xEC AND-type).
- gen_audit08.py -> docs/levels/07000008-audit.md: the 308 per-placement rows appended after the 996 audit's entry-sweep content (the file keeps both parts), + the scripts section (13 level-bin kind-7 scripts + 0x06000077 in pack 07800008 + 0x06000078 in pack 07900008, all decoded with dump_sp_script [D]) + the mission-record section (record 3 @0x2a4354, objectives @0x2a3e88 [L]).
- Fixed the SS operate-sound gap [M 0x1a9948:86-89]: SsList::tick plays param 4 at the 0x40 arm through the sim's sound sink; 0xffff stays silent. Two unit tests green.
- Corrections found: spawner groups 4/3/5 + 1/2; trigger #80's ch 43 has no reader; all 26 drones are placed (9 no-wait, 9 wait-42, 7 wait-44, the ally).
- Suite restarted on the final tree; running.

Stage Summary:
- docs/levels/07000008-audit.md complete (the 996 sweep + 308 placement rows + scripts + mission records); ss.rs/drone.rs carry the sound fix; suite in flight.

---
Task ID: level-08-audit
Agent: worker (s/level-08-audit)
Task: (continuation) the 08-relevant tests validated on the final tree; report drafted.

Work Log:
- Ran the prebuilt 08 test binaries directly (lock-free): simplescript 2/2, fix_safe08 5/5, pickups08 5/5, fix_fusebox_cut 4/4, decomp_audit_07000008 1/1, hurt_07000008 2/2, walk_07000008 3/3, walkreal_07000008 4/4 (153.7s), cutscene_08_actors 2/2 — the SS operate-sound change is validated on every 08 path that touches ss.rs.
- The full suite (tools/test/suite.py) build ok (811 binaries, 1059s); the run phase is alphabetical and was still in the c's at 04:26 — it cannot finish inside the window; the report carries the partial + the direct-run list.
- docs/reports/level-08-audit.md drafted (Usage: not visible to me).

Stage Summary:
- The audit, the fix and the validation are all on the branch; finish steps next (push, report, finish.sh, ntfy).

---
Task ID: level-08-audit
Agent: worker (s/level-08-audit)
Task: (final) finish steps.

Work Log:
- Suite partial at the window's close: 270 binaries completed, 1650 passed / 4 failed (the 3 disc_golden + identity_disc on the trimmed ISO) / 82 ignored; net_join_live CRASH exit -9 (the OOM class); no TIMEOUTs; no 08 path failed (the 08 binaries ran directly green).
- The 'spare' scripts row corrected: 06000027/4f/52/7c4 are the game's hard-coded effect scripts [M 0x136ce8:93, 0x180498:436, 0x1afcc8:45]; only 0600006f is a true spare.
- The drone wait-split corrected (9 live / 11 wait-42 / 6 wait-44).
- Report finalized with the partial tally; finish steps follow.

Stage Summary:
- The deliverable (docs/levels/07000008-audit.md, 308 placement rows + scripts + mission records) is on the branch; the SS operate-sound fix + 2 unit tests merged into the tree; the report carries the large gaps (the sky +0x4a word, D-004 medal gate, the ride/intro per-instance extras, the per-instance follow-up tests) and the live-check list.
