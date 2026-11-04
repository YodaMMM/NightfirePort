# real-health-14-00agent-4 — the 14 00 Agent walk past the mezzanine lip

Read docs/reports/real-health-14-00agent-3.md on origin/s/real-health-14-00agent-3. The push gate is settled: the ref-guard in Intersect_CylGeom 0x109da0:431-447, not the +0x51 flag. Continue from its "What's left": land the dead-tail reading with the cc fix so the mezzanine west lip (x ~45.1, z ~39.9) stops holding Bond, keep the d1 walk green, then the d2 with_real_damage wrap and the d3 un-ignore. Merge origin/master first. Cite [M]/[D]/fresh-boot [L] for every pin, and run every test that uses changed code.

---
claimed-by: sandbox7 at 2026-11-04T19:12Z

---
## Report (done, 2026-11-04T19:40Z)

The honest ref rule LANDED and is probe-proven: Bond::honest_ref (opt-in, default false = master everywhere) runs the game's push-tail guard faithfully [M Intersect_CylGeom 0x109da0:431-447, decompile re-verified — push + pick bit + best-hit record all inside the arm]; probe14_lip_release shows the mezzanine west lip RELEASES in 28 ticks under the flag vs x pinned 45.1338 under the fallback (the -3 number reproduced; the hold = the lip's west face n=(1,0,0) radial push). The coupled re-land chain (cc fix + drift + 3.6 windows + lid crouch, cherry-picked 4f6352d7a/fbdded0c6 + the d2 wrap) measured WITH the flag: 2/7 (the d1 falls at tick 745 near the spawn — a non-up-facing push loss; the non-_at walkers jam without the flag) and 1/8 (all walkers honest) — parked in history, the tree back at the -3 green set: walkreal_07000014_live 9 passed / 0 failed / 1 ignored whole (197 s, the d1 GREEN), fix_npc_doors_live 1/0, fix_guard_misc_live 6/0, nf-game lib 659/0/5. The d2 wrap + d3 un-ignore stay parked with the chain; next levers + two live checks in docs/reports/real-health-14-00agent-4.md on the branch.
