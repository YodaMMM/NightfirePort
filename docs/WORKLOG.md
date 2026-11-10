
## pretrans-arms-3

Window 1 (21:22-21:30 UTC, sandbox kept from level-03-guard12-door5; setup skipped). Branch
s/pretrans-arms-3 = origin/master 0a9d39435 + cherry-picked s/pretrans-arms-2 616d4b0e1 (the
+0x38 root-motion zero) as 3c6a8e503. One conflict, in the shared damage_log.rs return-fire
stand: master's fire_shield cone (opt-in) and pretrans-arms-2's return_fire_shield_blocks fold
(opt-in) landed on the same spot; both kept — the fire gate already reads `!blocked && !shield_line`.
Pushed. Next: re-run the two red walks of the -2 report on this tree (master's solid guard bodies
0a9d39435 sit under the root zero now) and fix them as a player would.
Window 2 (21:31-22:20 UTC). The named reds are fixed by the rebase; a sweep maps the interaction class:
- walkreal_07000002 00agent real-health GREEN (full route, 25570 ticks; the geisha-2 captain's
  round costs 0 health — armour absorbs what killed Bond on -2). walkreal_07000003 real_damage +
  _agent GREEN (378 s). Master's solid guard bodies changed both engagements. No pin/damage/ignore
  touched.
- Sweep (one binary per call): 21 green (02 1, 03 2, 04 9, 06 6, 07 5, 08 5, 0a 13, 0b 7,
  0c_barrelrow 1, 0d 1, 11_route 1, 12 4, 13 3, 14 7, 15 7, 16 15, 1b 16, 41 15, 41_wall 2,
  4a 4, 4a_satchel 2), nf-game lib 799 green. 6 binaries red (7 tests), all green on both parents
  (-2's walk list; bare-master spot-check for 01): 01 x3 (the AND-touch plate; the escort
  real-damage pair die drained at the stairwell — the hall flank's deeper nav leg wedges on solid
  bodies and the 24-hp captain's planted 8-pellet volley lands 48.0 last; his weapon datum reads
  0x0 so agent_order cannot see him), 05 (crawl-prompt 6000-tick cap), 09 (switch reach), 0c +
  0c_eastroute (wire #282 B-end grab never fires), 11 (south-band doorway wedge). Two 01 fix
  attempts made and reverted (unproven). Levers in the report.
- Report: docs/reports/pretrans-arms-3.md.

## pretrans-arms-4

Window 1 (00:28-00:35 UTC, sandbox kept from the previous round; setup skipped — repo, work/,
ISO, nf-queue all present). Branch s/pretrans-arms-4 = origin/s/pretrans-arms-3 525153962, merge
origin/master 0a9d39435 was already up to date. Read docs/reports/pretrans-arms-3.md. Plan: repair
the six interaction reds one level per tool call, the -2 round's way (per-level walk repairs with
the trace, no sim changes): 01 (volley-carrier-first retarget lever), 05, 09, 0c + 0c_eastroute,
11; then re-run 01 and 14; run the ai binaries and nf-game; remove ignores when green. Started the
baseline suite in the background.
