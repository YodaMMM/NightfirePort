
---
Task ID: level-0c-audit
Agent: worker (s/level-0c-audit)
Task: the 0700000c behaviour audit — one row per placement/script/mission record in docs/levels/0700000c-audit.md (game [M/D] chain vs the port, GAP column); fix small gaps with tests; large gaps → report sections.

Work Log:
- Branch s/level-0c-audit created from origin/master (30598272). Sandbox intact from the prior session (work/, ISO, prebuilt examples, ../nf-queue present). Pushing this checkpoint before any other work.
- Next: the 0700000c level doc + census probe (a 0700000c census test is not on master yet — adapt census_07000008.rs), the suite in the background, the per-placement table via the gen_audit knowledge base, the scripts dump with the prebuilt dump_sp_script/list_scripts, the mission record from the EE snapshots.

Stage Summary:
- Branch s/level-0c-audit exists on origin at the clean master point; no code changed yet.

---
Task ID: level-0c-audit
Agent: worker (s/level-0c-audit)
Task: (continuation) the audit table assembled + two small-gap fixes; the suite restarted on the final tree.

Work Log:
- census_0700000c.rs written + run: 575 placements, 0 unresolved, 36 kinds (the doc said 574 and missed 0x2d/0x71/0xe6×78).
- docs/levels/0700000c-audit.md: 575 per-placement rows (35 kind sections) + the scripts section (13 kind-7 scripts; the SP subs via the 0x1B ops) + mission record 11 [L @0x2a4508, objectives @0x2a4080] + the corrections.
- Fix 1 (re-applied from the 08 branch, unmerged on master): the SS operate sound [M 0x1a9948:86-89] — 0c's hatch #138 carries sample 470. Two unit tests.
- Fix 2 (new): the RAW 0xDC rows read the direct Trigger_Create map (out=p1, mask=p9 with the 0->2 default, lock=p10, hide=p11) [M 0x1b1720:61-91, re-derived against the 0x1b2b88/0x1b2c68 wrapper stores] — the port had the skylight plates' out/mask swapped (ch 2 walk-in instead of ch 0x1b bullets). triggers.rs apply_raw_dc_map + a unit test.
- Validated on the fixed tree: triggers:: 25/25, ss::tests 11/11, fix_level_0c 4/4, walk_0700000c 4/4, walkreal_0700000c 2/2, laser_watch_0c 2/2.
- Corrections: #281's p2=5 is the OUT (ch 5 = MusicTrigger #95's gate — the doc's '[S] level script' writer note wrong); the skylight plates' map; record 11 @0x2a4508; the ch-0xb4 plate #103 has no reader.

Stage Summary:
- The deliverable is complete and pushed; the suite runs for the final partial tally; report + finish steps next.
