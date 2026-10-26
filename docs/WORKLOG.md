## level-09-audit (2026-10-26, sandbox expiring ~04:21 UTC)
Branch `s/level-09-audit` off origin/master (30598272). Task: the 07000009 behaviour audit — every
placed object/character vs the game: per placement, the creator in parsemap_create_dynamic_objects
0x1d04f0 / the static path, update + use/damage/event handlers [M], the port's coverage; the level's
scripts (streams, camera, hand-backs) and the mission records. Table to docs/levels/07000009-audit.md
(one row per placement/script/record, GAP column); small gaps fixed [M] with a test each, large gaps
as report sections. Tests pin only [M]/[D]/[L]-evidenced behaviour.
Plan: fresh sandbox — clone+branch pushed at start; work-slim + ISO downloads next; suite runner early;
placement census from the disc data + the kind→creator map from 0x1d04f0 + per-kind port grep; then
scripts/mission records; then fixes. Prior art: docs/levels/07000002-audit.md, 07000008-audit.md.
