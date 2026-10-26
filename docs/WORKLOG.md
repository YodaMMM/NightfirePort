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
