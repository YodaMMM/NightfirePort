
## level-01-audit (2026-10-25, sandbox expiring ~22:28 UTC)
Branch `s/level-01-audit` off origin/master (aa94ce02). Task: the 07000001 behaviour audit — every
placed object/character vs the game: per placement, the creator in parsemap_create_dynamic_objects
0x1d04f0 / the static path, update + use/damage/event handlers [M], the port's coverage; the level's
scripts (streams, camera, hand-backs) and the mission records. Table to docs/levels/07000001-audit.md
(one row per placement/script/record, GAP column); small gaps fixed [M] with a test each, large gaps
as report sections. Tests pin only [M]/[D]/[L]-evidenced behaviour.
Plan: reuse sandbox work/ + ISO + nf-queue; branch pushed at start; suite runner early; programmatic
placement census (the 07000014 placement parser, retargeted) + the kind→creator map from 0x1d04f0 +
per-kind port grep; then scripts/mission records; then fixes.
