# WORKLOG

## level-objects-uncited — branch start (2026-10-26)

- Setup: repo + work/ + ISO already in sandbox; created `s/level-objects-uncited` from origin/master, pushed this checkpoint.
- Sandbox note inherited from the previous session: background jobs are killed between tool calls here, so the
  fast suite runner (`tools/test/suite.py` via nohup) is expected to die silently; will attempt once and fall
  back to foreground crate suites.
- Task: for each of the 22 level-object functions never cited in crates/*/src
  (Copter_FireMissile, SpaceLaser_Register, Sunreflection_Update, Bubble_Update, camera_fixed, LS_LoadBonus,
  Sub_Laser, Sub_DeployTorpedoes, Sub_Update, Sub_ApplyDammage, Sub_Collide, Sub_Delete, MiniSub_FireTorpedo,
  MiniSubMove_Forward, MiniSubState_Attack, MiniSub_Update, MiniSub_Collide, MiniSub_Delete, Creature_Delete,
  Creature_Update, Creature_Hurt, Creature_Move): name creator/caller + reaching placement kind/script event +
  levels; port on-foot-reachable simulation with tests; driving-only/unreachable get a proof comment.
- Prior art to build on: docs/systems/uncited-audit.md (Creature_* dead on retail data — no type-0xe0
  placements [D census]; camera_fixed dead — no SetCamMode 4/5/6/0xa writer [M]), docs/systems/space-laser-update.md.

---
Task ID: 50-level-objects-uncited
Agent: level-objects-uncited worker
Task: the 22 never-cited level-object functions — creator/caller + reaching kind/event + levels;
      port the on-foot-reachable ones with tests; driving-only/unreachable get proof citations.

Work Log:
- Branch created from origin/master, WORKLOG pushed at start, census pushed as checkpoint 2.
- Three parallel research passes over work/decomp/ACTION.ELF (Sub/MiniSub; Copter/SpaceLaser/
  Sunreflection/Bubble; LS_LoadBonus/camera_fixed/Creature) + my own caller/table sweeps:
  the per-class wiring is control_funcs 0x26f890 (rows: Bubble 6, Creature 0x23, Sunreflection
  0x19, Sub 74, MiniSub 77, SpaceLaser 0x50); the creators ride parsemap 0x1d04f0 kinds
  0x2f/0x36/0x37/0x38/0xe0.
- New nf-formats/tests/level_objects_census.rs: per-level placement-kind pins over every 07*
  archive (driving included): 0x2F on 05×5/08×3/0B×3/12×2/46×1, 0x38 on 1B×1, 0x36/0x37/0xE0
  nowhere [D]. The 0xE0==0 test is the Creature dead-data proof as a number.
- Ported Copter_FireMissile 0x17d9f0 (copter.rs MissilePod + fire_missile) — draw order,
  cadence 60·DiffMod, pod parity, 0x07000008 homing, weapons 109/98; 4 tests.
- Ported SpaceLaser_Update 0x1aa3f8 + SpaceLaser_Register 0x1aa9e8 (space_laser.rs) — inert
  while ch 19 is closed (the disc condition), the armed fire path, the kill gate, the state-3
  re-raise; 4 tests. FIXED the old param-map test: p5 is the re-raise channel (99), p7 the arm
  gate (19) — the decompile beat the test, recorded in the report.
- Proof citations delivered in docs/reports/level-objects-uncited.md for Sunreflection (no
  creator), Bubble (no creator/symbol), camera_fixed (mode 4; the 21-call SetCamMode sweep has
  no 4/5/6/0xa), LS_LoadBonus (LIVE: SaveIFFBlocks 'BNUS' row, reached from Menu_UpdateMessageBox
  masks 0xff/0x2d/0x25), Sub/MiniSub (driving-only: zero kind rows), Creature (zero kind rows +
  the single class-0x23 writer).

Stage Summary:
- Tests: nf-game 905 passed / 0 failed; nf-formats green except the 3 known slim-ISO disc_golden
  reds. The fast suite runner cannot run here (background jobs die between tool calls).
- Report: docs/reports/level-objects-uncited.md (usage section: not visible to me).
- Left for later: the laser's script-player integration (pinned-decode like space_missile),
  the copter state-1 spot check (unlocks state 2), the delay-slot live check on 0x17df34.
