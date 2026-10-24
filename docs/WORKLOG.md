### ai-dstates-uncited (started 21:18 UTC, 2026-10-24)

Task: 16 ACTION.ELF functions never mentioned in `crates/*/src`: NDrone2_DSTATE_Prone 0x166940,
ProneFire 0x166a28, Interogator 0x1729f0, Alerted1stEncounter 0x173178, CombatTooClose 0x173338,
HoldItRightThere 0x173570, SeenOpponent 0x173b38, SeenExplosive 0x173bb8, FailMission 0x173f68,
HangUp 0x1743d8, WaitForever 0x174430, Tester1 0x16db90, Tester2 0x174150, Tester3 0x1741c8,
Tester4 0x1742d8, initDTYPE_SniperAlert 0x14e580. For each: find its state id(s) in
NDrone2_StateFuncs 0x29b300 and who enters it; then exactly one of (a) port already implements
it under another name (add name+address, test if none), (b) reachable on-foot and missing (port
it + failing test), (c) never entered on the shipped on-foot path (prove it, cite in drone.rs).
calltree.py from each ported state must report no never-mentioned callees.

Plan: fast suite in background; dump StateFuncs table; map state ids -> addresses; grep port
drone.rs state machine; classify each of the 16; check ai-dmode-inits branch research for
SniperAlert/Interogator rows.

- (21:18 UTC) Sandbox had repo/work/ISO from previous session: skipped setup, branched
  s/ai-dstates-uncited from origin/master at 805d51c8, pushing this checkpoint.
