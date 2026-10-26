# WORKLOG

## level-4a-audit

Task: full behaviour audit of level 0700004a (Nightfire's underwater/grate section) — every placement on the
main map [D] (characters, doors, triggers, scripts/SPs, switches, monitors/sensors, pickups, destroyables,
breakables, ladders, grapple points, lifts, special objects), every script stream (ops, camera streams,
hand-backs), every mission record (objectives, channels, fail conditions).

Plan:
- Build the placement census from the disc map data (0700004a) via the repo's map-parsing tools.
- For each placement: creator in `parsemap_create_dynamic_objects` 0x1d04f0 / static path [M], update + use/damage/event handlers [M], port side (created? updated? drawn? usable? reacts? sounds?), GAP column.
- Same treatment for scripts and mission records.
- Write `docs/levels/0700004a-audit.md`.
- Fix small gaps directly with a test each (evidence-tagged: [M]/[D]/[L] only; [S] stays out of pass/fail).
- Large gaps documented in `docs/reports/level-4a-audit.md`.

Prior level-4a work to respect (do not undo): branches s/level-4a-cutscene-autoaim, s/level-4a-lift-*,
s/level-4a-rappel-vanish, s/level-4a-satchels-lift.

Status: started, census next.

Progress (same session, ~90 min in):
- Census: `nf-extract placements-all` — 294 placements, all on map 41, 12 kinds (0x0f 38, 0x19 7,
  0x24 1, 0x31 1, 0x33 1, 0x47 57, 0xd2 2, 0xd9 4, 0xdb 162, 0xe9 2, 0xea 13, 0xf1 6) [D].
- Creators re-verified per kind in the 0x1d04f0 decomp (0xd2 = `Shooter_Create` 0x17d030 — new
  detail for this level).
- NEW decode: the door mechanism — `Door_Create` 0x136100 param map, `Door_Update` 0x1363c8's
  one-shot unlock block (p3, message 0x2000003 suppressed on 0x7000014/0x700004a), `Door_DoMsg`
  0x135f18's E-refusal (group word 2 ≠ 4 → text 0x200000b), `Doors_Calc` 0x135fe0's twin sync.
  The 162 doors = 81 pairs; exactly 6 carry p3 (unlock ch 0x3d/0x40/0x3f/0x45/0x3e/0x4f — the
  fight-floor plates + the lift's ch-0x40 write); 75 pairs are sealed forever.
- Scripts: 9 in-archive NISs + the 2 companion cut-scenes (060006cf in 0780004a.bin — camera
  true, writes ch 0x22 at t575, `Drone_EnableAll` at t595; 060006d3 in 0790004a.bin — camera
  true, ends `0x12 [0x0720004a]` → movie-code table → 07000013). Event semantics from
  `Script_EventHandler` 0x1d94c0. Reconciled 0700004a.md §2's "11 scripts" = 9 + 2.
- Mission record re-confirmed (1 objective, ch 0x4a cross-level writer, enable 0x4d = plate #277).
- Small gaps fixed with tests: `levels_0700004a_census_live.rs` — the shooter fire-edge (one
  weapon-61 bullet at #286 on the 0x22 edge, never a second) and the door unlock map (the six
  channels open exactly their pairs incl. the twin; sealed pairs and spawner channels open
  nothing). Both green.
- No new draw-only gaps (mesh resolution 0 unresolved; statics draw).
