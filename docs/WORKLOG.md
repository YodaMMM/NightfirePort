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

- (21:18-22:45 UTC) Setup skipped (repo/work/ISO present); branch pushed 21:19. Suite runner fought the
  sandbox: background processes die between tool calls and the box is 3 GB/2-core — the build was warmed
  in foreground chunks (alsa-sys needs PKG_CONFIG_PATH=/home/z/.local/alsa/.../pkgconfig here), then
  suite.py reached "build ok: 788/789 binaries" before being killed; after the drone.rs edit all test
  binaries rebuilt clean (0 errors) and 6 drone-critical bins ran via --only: 19 passed / 0 failed.
  Two parallel subagents re-derived the evidence: (1) StateFuncs 0x29b300 verified per-id [L ×4
  snapshots byte-identical], full entry-channel census (SetState literals ×319, +0x5A2/A4/A6 stores,
  msg+0xcd8, anim-end arg4, DTYPE rows) — found ONE doc error: Tester4 0xba IS produced (Tester3's
  CallAnim end-state 0x1741c8:26) but stays transitively unreached; fixed unreached.md (Tester4 row +
  Tester1's row-8 census now closed: mode 8 unreachable). (2) Mode-0x19 reachability PROVEN: 43
  placements on 0700004a/0700000c/07000016/07000006 create as alert snipers → DroneTypeSettings row 25
  → initDTYPE_SniperAlert 0x14e580 runs at creation [M 0x14cbd8:118-121]. Classification: initDTYPE_SniperAlert
  = (a) already implemented (name-keyed stationary dispatch drone.rs:2052; its disasm-verified only
  observable effect is +0x4F8|=0x10 — the DCVfromOBJ stack result is never read); the 15 DSTATEs = (c)
  never entered, cited as a block in drone.rs state module with per-id proofs. New test
  dtype_sniper_alert_live: 0700004a's garrison spawns mode 0x19/stationary/start 0x2a (green),
  abseil pair negative-checked. calltree.py from all 16: TOTAL never-mentioned reachable 0.
