
### level-14-empty-panels (started 07:23 UTC, 2026-10-25)

Task: 07000014 openings and panels with nothing behind them — at the two panel spots (vent/hatch
54.21,1.03,46.15; wall panel -88.58,-1.10,46.15) dump what the port draws and collides and what
the level data places there [D], find the missing piece (undrawn object, missing cel, LOD/variant
not swapped, script entity not created) [M for the creator], port it, pin draw lists + collision
with tests, name headless renders in the report.

Plan: branch from origin/master 38f1bc93; start the fast suite in the background early; census
the placements around both spots; find the creators; port what's missing; touched tests + suite;
report.

### level-14-empty-panels (started 07:23 UTC, 2026-10-25)

Task: 07000014 openings and panels with nothing behind them — dump [D] what the port draws and
collides at the two spots, find the missing piece [M], port it, pin with tests, name renders.

- (07:23-07:30) Branch from origin/master 38f1bc93, pushed. The previous task's full suite was
  still running in this persistent sandbox and FINISHED at 07:24: 3284 passed, 23 failed, 194
  ignored (104 min) on the same product code — the 23 = the documented environmental families.
- (07:30-08:10) Two parallel research agents mapped the port's cel/draw/collision pipeline
  (gpu.rs orphan force-draw, find_cel's scene.cels index space, build_scoped's bits) and the
  game's cel draw path (View_AddCels 0x1e6ba0 room+member chains, build_link_objects_to_rooms
  0x1c9150 scoping, no render-side bit-0x20 test). New example dump_07000014_panels: placements,
  materials with the exact cull verdict, portal traces, AABB region census, cel owners.
- (08:10-08:25) FINDING: both spots' statics all DRAW (the exact cull keeps everything near them;
  the pit cells 222-225 are portal-orphans → force-drawn). The missing art is script-spawned:
  the lid script 0x06000A3B's ladder art 0x02000751 spawns only on the SP replay (state 1, mask
  4 = the use activation), and the fusebox script 0x06000A1B's wire entities 0x02000A61/62/73
  are the documented stand-in.
- (08:25-08:45) panels_07000014_live: the route lid's replay PINNED GREEN through the real use
  path from inside the duct (the ladder art spawns at the lid's base); the fusebox wires
  confirmed missing (fail-first repro, #[ignore]d). The 14 walk family still 2/2. Report written.

### level-14-fusebox-wires (started 12:13 UTC, 2026-10-25)

Branch from origin/s/level-14-empty-panels (1112fc5b), merged origin/master (d8334b70).
Task: 07000014 the opened fusebox panel shows its wires — run the fusebox script 0x06000A1B Entity streams (wire sub-objects 0x02000A62 / 0x02000A61 / 0x02000A73, type 0x31, streams 0/1/3) through the SP-entity path (SP_Update 0x1dbda8, SP_Activate 0x1dc338) replacing the fusebox.rs one-wire-per-group stand-in; un-ignore and green panels_07000014_live::the_fusebox_use_spawns_the_wire_sub_objects_behind_the_panel with real input; keep walks + fusebox/objective tests green.

Work log (12:57 UTC):
- Reproduced the ignored repro: empty pre AND post — then traced it: the fusebox SPs prime (mode 0x40, one-tick play), the script entities DO spawn, and the groups register — under the real [D] hashes 0x02000AA2/AA1/AB3/AA4 (streams 0/1/3/5, obj_type 0x31; dump decode + raw cmd bytes: rev-fold big-endian, e.g. stream 0 bytes 07 a2 0a 00 02 → 0x02000AA2). The task/report hashes 0x02000A62/61/73 are a transcription slip (dropped 0x40 bit) — the repro was a false fail.
- Verified the full chain is already ported and pinned: wires exist from the preload [M 0x1dbc08:46-54], group 0 = stream 1 wire 0x02000AA1, group 1 = stream 5 wire 0x02000AA4 [M 0x1d94c0:172-175 → 0x1b0970], the cut is the hit path [M 0x1adfe8:88-123] (fix_fusebox_cut_live 4/4 incl. the 07000014 laser sabotage → 0x8C), E is inert [M 0x1a5170:95-160 no 0x3C arm; mask 1 no bit 4]. All four wire glists resolve to models (meshes exist).
- Rewrote the ignored test to pin the decompile truth (correct hashes, preload as the spawn agent, the E press pinned inert through real input, the behind-the-panel geometry, the registrations, the glist resolution) and un-ignored it: panels_07000014_live 2/2 green; walk_07000014_live 2/2; fusebox_06_live 4/4.
- Updated the two stale passages in docs/levels/07000014.md (the stand-in/E-cut notes).
