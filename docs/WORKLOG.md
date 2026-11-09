# WORKLOG

## level-01-d3-walk (2026-11-09, started)

Task: the 07000001 00 Agent (d3) real-damage walk — ignored and red through the whole
guard-pane-drain series — goes green with guards turning on Bond on contact and guard rounds
breaking glass, then the ignore comes off. Branch `s/level-01-d3-walk` cut from `origin/master`
(3efb98f19). Ledger carried in: drain-5's report (docs/reports/guard-pane-drain-5.md on
origin/s/guard-pane-drain-5) — the SPEND ran and still held the trio at z 15.4/18.4/18.1
(states 0x73/0x74), and master's own guards_shut_door_07000001_live pins the nav law: the hall
guards' NavPath (port path 1, 28 nodes) carries only the west door #2 pair; the south door's
pair is kind-8-Mayhew-only, excluded from the A* [M 0x1d2598 AIPath_Parse, 0x1571b0
AINetwork_DoAStarPath] — no armed lure can ever draw them through the south aperture. Next
levers (drain-5's ignore/report): the WEST door #2 peek — stage to the corridor side of #2
(-14.1,5.3,13.1), open by hand, scan stands down the hall's west line, fight SHOOT-ONLY;
second lever the east stage perch (2.97,6.38,16.27) behind the mouth, shoot-only. Then the
escort phase past the AND, never yet run on any tree. Helpers gated to 07000001 + difficulty
3; real input only, no top-ups, never lower damage or health. drain-6's session-start WORKLOG
read (no report file exists on it; no work commits).

Entry started — porting drain-5's d3 test-file work onto this branch, then the west-door lever.

Result (same session): the four door-phase deaths reproduced and removed (the 5925 flank
through the plane, the 6159 fight-creep into the mouth, the 6007 open-aperture stand drill,
the 10017 loot-walk to an in-hall drop) — the port's guard rules compare clean at every point
checked (the standoff law, the open-door fire lines, the side-step drift, the damage rates),
so the fix is the player's movement discipline: opt-in `no_cross_z` + `flank_hold_z` + the
fight-creep off + the return-fire lip hold + the resupply drop gate (all in the shared
helpers, master bit-exact flag-off) + the peek rounds shut the leaf behind Bond (d3-gated
after ungating shifted the d2 walk; the whole binary went 6/1, the gate restored it) + the
d3 mouth strip with a halved wound cap. RESULT: #6/#11 die through the aperture at zero cost
("peek 2 done, health 85.6, guard hp -2.0"), the door phase ends at the AND assert ALIVE;
#7 (SMG captain, 24 hp) is UNEXPOSABLE from the south (10 rounds + the mouth strip found no
part at any drift spot; the armed lure ran dry and never drew him) — the guards_shut_door law
holds, the walk stays #[ignore]d with the ledger and the sole lever (the WEST door #2 peek,
then the east perch) in the ignore message; the escort phase past the AND has still never
run. walkreal_07000001_live whole: 7 passed / 0 failed / 1 ignored. The other walkreal
binaries did NOT run (the clock ran out; every shared change is flag-off bit-exact by
construction; the next session sweeps first). Report: docs/reports/level-01-d3-walk.md.
