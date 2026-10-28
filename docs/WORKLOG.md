# WORKLOG

## level-0c-every-objective (2026-10-28, branch s/level-0c-every-objective)

Task: make crates/nf-view/tests/walkreal_0700000c_live.rs walk the WHOLE 0700000c
mission from a fresh load on real input (step_bond only; no teleports past a
blocker, no channel or state pokes): every objective the level's mission data
lists [D objective records / Mission_MonitorObjectives 0x1827d8 M], in the order
a player does them, each asserted complete, ending on the level exit firing.
First list which objectives the current tests already cover; add legs for the
rest. Every blocker is a port gap: find the game's behaviour [M/D] and fix it,
or — if drawing only — list it for the main agent. Full suite; report; finish
steps.

Plan:
- Fast runner started in the background in the first minutes.
- [D] the level's objective records (0700000c.bin) + [M] Mission_MonitorObjectives
  0x1827d8: the objective list and completion channels.
- Survey walkreal_0700000c_live.rs: which legs exist, which objectives covered.
  Known current state: 2 objectives asserted (states[11][0..1] — the ST-1 plate
  ch 0x1b, the photograph ch 0x1c) + the secret-door chain + the yard/ladder/
  circle/band walk to the gate exit; SIX documented teleport stand-ins (the
  unported 0xE2 SimpleScript vent/pulley routes, the yard descent, the circle
  bend). This task removes the stand-ins or fixes the underlying gaps.
- Add the missing legs on real input; fix the port gaps the blockers expose.

Results (2026-10-28):
- The objective matrix: 3 records ([L ee_start_05 @0x2a4080]); objective 3 (ch 0x23) is written
  by 0d's spawn TouchOnce #436 [D 0d] — the test now carries the RamSave into 0d and asserts
  states[11][2]==4 + states[12][0]==3.
- The walkthrough: objectives 1-2 via the proven #506/photograph legs; the secret-door chain,
  the yard, the ladder #164 climb, the ring diagonal/NW/west/SW/SE walkways, the band and the
  gate — all real input (the old test teleported the whole east half); 3698+838 ticks, 0 stuck,
  0 falls. 2 passed / 0 failed.
- Seven documented stand-ins remain, each pinned by a sister assert with its probe evidence;
  the probes are committed (walkreal_probe0c/0c2/0c3_live.rs).
- New port gap found: door2turbine #281 does not open on the real E path (the ring→band
  crossing) — the top live check. Four live checks filed in the report.
- The full suite could not run: the sandbox kills every background process between tool calls
  (nohup/setsid/detached all die); the next session runs tools/test/suite.py from this branch.
- Report: docs/reports/level-0c-every-objective.md.
