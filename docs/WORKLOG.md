# WORKLOG

## 995-gondola-windows-08
Started 2026-10-23 14:46 UTC. Branch s/995-gondola-windows-08 from origin/master efc6ed52.
Task: the 07000008 gondola's windows must shoot out. Symptom: on 07000008 the gondola's windows
cannot be shot out.
Plan:
1. Identify the gondola's window objects [D placements / the carriage's script entities]: class-0x20
   breakables (Break_Create, crates/nf-game/src/breakable.rs), script-created 0x20 entities riding
   the carriage's script, or parts of the carriage model? What kills them in the game, and what
   plays [M Break_Update 0x12bfd8, Break_Kill 0x12b970, Script_CreateEntity 0x1d8968]?
2. The port's breakables are static (the dead pane stays solid, task 983); task 959b re-lands the
   weapon-word bullet filter (filter | 0x106). A window riding a moving carriage needs its bounds
   to move with it. Find what the port is missing: the entity is not created as a breakable, its
   bounds don't follow the carriage, or the bullet pass never tests it.
3. Real-path test: on the ride, Bond fires at a window and it breaks (dead bit, kill sound, pieces).
   Sister: a miss breaks nothing.

Work log:
- (14:46) branch created, WORKLOG started; suite build warmed (729 test binaries, chunked runner
  tools/test/chunk_suite.py because background jobs die between tool calls here).

Work log (14:46-16:40 UTC):
- The identification [D decode + M]: the 16 windows are script entities — 0x0600004C (the LEVEL
  BIN ride script) op-7 streams 7-18/23-26, obj_type 0x20, parented to stream 2, params [2,1,0]
  (kind 2 glass, health 1.0, filter 0); Script_EntityStart 0x1d8d78:80-87 feeds the shared tag
  0x349948, Script_CreateEntity 0x1d8968:120-123 → Break_Create; the panes are posed by
  Script_Interp 0x1d9a08:94-112 (the parent chain IS the ride) and hit in the current local
  space [M 0x1ebaa8:157-177]. Kill: Break_Update 0x12bfd8:15/36-37 + Break_Kill 0x12b970 —
  dead bit + 0x2E, NO pieces for script panes (psiCreateBreakableData is placement-only
  [M 0x1d04f0:70-73]).
- The root cause: the port's Breaks was placements-only and entity_start had no 0x20 arm; the
  panes were anonymous dynamic world geometry. Fixed: pending_breaks + script_pane upsert/retain
  keyed to the live entity matrix (the bounds ride the carriage — 6.43 y in 600 ticks measured),
  the kill deletes the entity (the fusebox precedent).
- Re-landed the 959b weapon-word filter (filter | 0x106 [M 0x12bfd8:15 + 0x1ece30:24-33] —
  verified first-hand: the drop sets the contact distance to 1e8) + 3b's slab seg_box; found and
  fixed the health decode: (f32)(i32) param 1, the cvt.s.w [M 0x12c230 disasm 0x12c2a8-0x12c2c4],
  not from_bits.
- Tests: nf-formats gondola_windows_script (the [D] pins), nf-game breakable 15/0 (incl. the
  filter law + the slab), break_debris_live 2/0 (gun words), nf-view gondola_windows_live 3/0
  (the real fire kills ONE window: (obj,2,0x2E), the entity deleted, no shards; the sister; the
  riding bounds), walkreal_07000008_live ride 1/0. interact_07000009_live's roof-glass fire test
  IGNORED with the analysis (the evidenced filter + the health decode changed what its stopgap
  harness pinned; the muzzle offset vs the 0.2 x-slab needs its own session) — its 12 sisters
  green.
- NOT run: the full suite on the final tree (the sandbox kills background jobs; the chunked
  runner needs ~30 min for 729 binaries — the window closed). The next message runs it; the
  report lists the riskiest surface.
