
## level-07-audit (2026-10-26, sandbox expiring ~02:22 UTC)
Branch `s/level-07-audit` = origin/master (b312d959). Task: audit EVERY placement on 07000007's
main map [D] (characters, doors, triggers, scripts/SPs, switches, monitors/sensors, pickups,
destroyables, breakables, ladders, grapple points, lifts, special objects) against
parsemap_create_dynamic_objects 0x1d04f0 / the static path [M]: creator, update, use/damage/event
handlers, port coverage (created/updated/drawn/usable/reacts/sounds). Same for the level's scripts
(every stream's ops, camera streams, hand-backs) and the mission records (objectives, channels,
fail conditions). Table to docs/levels/07000007-audit.md with a GAP column; fix small gaps [M]
with tests; large gaps become report sections for their own tasks.

### level-07-audit — progress (2026-10-26 ~01:57 UTC, window closing)
The audit is WRITTEN: docs/levels/07000007-audit.md — the full dispatch table [M 0x1d04f0 + the
static path 0x1d1998/0x1d00c0], a row per placement (all 23 kinds, grouped only where the game
path is identical), the three scripts' full stream decode (0x0600007A len 23 / 0x06000072 len 3000
/ 0x06000073 len 4326 — locations on disc byte-exact; the reveal ships in 07900007.bin, NOT
07800007; NO event 0x0A in any 07 script — the old "urn uses Break_DoBreak" was wrong), the
mission record @0x2a43a0 (title/hint split corrected), 8 numbered gaps. FIXED GAP-2: the cut-scene
mission-timer pause [M 0x1d94c0:118-121 + 0x199520 + 0x1995a0:9-14, unpause 0x1da880:44-48] —
scene.rs PlayerDisable/Enable now drive stats[0]'s pause byte; new `cutscene_timer_pause_live`
pins the frozen clock through SP #420's real 4326-tick window (green 7.3 s). 07000007.md stale
claims corrected. GAP-1 (guest/Bond cylinder collision — the game blocks Bond [M 0x140e90:66-116 +
0x149ec8 + 0x14b190 <2.0; the port has nothing]) is written up with its full chain for its own
task, plus live-check requests. Suite: the full run could not finish inside this sandbox window
(killed twice by the 10-min tool cap mid-build/run; the build is warm now, 809 binaries);
a 560-s foreground chunk ran 127 binaries with ZERO failures (../suite.log). NEXT MESSAGE: finish
the suite on this tree (`NF_ISO=... python3 tools/test/suite.py` — ~13 min foreground, or two
--only halves), then the report's total line.
