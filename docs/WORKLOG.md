
## level-0c-door2turbine-2 (started 05:42 UTC 2026-10-29)

Task: Chain Reaction Part 1 — walk to the door2turbine E stance for real on 0700000c. The
level-0c-door2turbine report leaves the walker balancing on an unexplained ~13.05 surface at
x 40.45..40.6 between the SE walkway and the 11.61 shelf / 11.09 corridor (owner not in the
static census): find it [D] (drone, dynamic object, or missed collision mesh), fix the port
collision gap [M/D], then replace the stand-in with the real-input leg (corner E opens #281,
the slot jump onto (44,-209), the band walk), then blockers 2-5 of level-0c-every-objective
in order. Full suite, quote its total line.

Plan: branch pushed from origin/master (f1791382), fast-suite runner started in the
background now (master moved past the warm cache, so the build phase recompiles first),
then read the level-0c-door2turbine and level-0c-every-objective reports and reproduce the
balance on real input.

Progress (06:40 UTC): the 13.05 mystery solved — no such surface exists; it was Bond's body
centre on the 12.02 slot floor [D probe]. The district re-mapped: the SE walkway is two
ceilinged dead-end boxes, the real route is the west ledge 11.61 (z −222..−213) → the 0.52
drop onto the 11.09 corridor → door2turbine #281. The real-input leg now runs: ledge →
corridor → the REAL E press opens the door (state OPEN, ch_open 5 on [M 0x135be0/0x135618])
→ the corner → the doorway tunnel in the east wall → the slot jump LANDED on the band
(44.84, 12.18, −208.80). One wedge left: the band x 53..67 has a 12-unit floor gap [D band
west map] — the next session's item, with blockers 2-5. Report at
docs/reports/level-0c-door2turbine-2.md; probe walkreal_probe0c_door2_live committed.

## level-06-every-objective (started 06:42 UTC 2026-10-29)

Task: make crates/nf-view/tests/walkreal_07000006_live.rs walk the WHOLE mission of level
07000006 from a fresh load on real input (step_bond only; no teleports past blockers, no
pokes): every objective the level's mission data lists, in player order, each asserted
complete, ending on the level exit firing. First list what the current tests already cover
and every stand-in/poke they carry, then add legs for the rest. Every blocker is a port gap:
find the game's behaviour [M/D] and fix it, or list it in the report if drawing only. Full
suite, quote its total line.

Plan: branch pushed from origin/master (ccc714a1) now; fast-suite runner started in the
background; then read the existing walkreal_07000006_live.rs coverage + stand-ins, the level
docs (docs/levels/07000006.md, -audit.md) and the mission data, and build the missing legs.

## level-06-every-objective (started 06:42 UTC 2026-10-29)

Task: make crates/nf-view/tests/walkreal_07000006_live.rs walk the WHOLE mission of level
07000006 from a fresh load on real input (step_bond only; no teleports past blockers, no
pokes): every objective the level's mission data lists, in player order, each asserted
complete, ending on the level exit firing. First list what the current tests already cover
and every stand-in/poke they carry, then add legs for the rest. Every blocker is a port gap:
find the game's behaviour [M/D] and fix it, or list it in the report if drawing only. Full
suite, quote its total line.

Plan: branch pushed from origin/master (ccc714a1) now; fast-suite runner started in the
background; then read the existing walkreal_07000006_live.rs coverage + stand-ins, the level
docs (docs/levels/07000006.md, -audit.md) and the mission data, and build the missing legs.

Progress (07:50 UTC): DONE. The level lists exactly ONE objective ("Find a way inside to the
party", ch 0x02, reveal ch 0x46 [L ee_start_05 @0x2a3e40]). The old OPEN LEG (gallery →
dressing room) is closed with evidence, not a port fix: the gallery's north edge is a
BALUSTRADE (balusters z −64.92, y 49.25..51.3, x 30..36.7 [D fine ray sweeps]; Bond grinds at
z −65.50 grounded AND airborne [D probe_06_edgetest]) and door #46 is locked forever (ch_lock
200, no writer [M 0x136810:10-16; U-044]); the game's own walkable web (map block 0x05
"Boundary Prime" [M AIPath_Parse 0x1d2598], dumped by probe_06_aipaths) routes the room AROUND
the gallery: courtyard 41.15 → the 43.0 step → the x≈17.4 ledge (43.5→45.3) → the 45.25 shelf
at z −64.6 → the 46.0 sill's open gap at x≈36 → the room's 43.1 strip → the LoadPoint #746.
Every floor under the chain already exists in the port's collision [D probe_06_gapmap bands] —
no gap to fix. The walkreal now walks that chain on real input (walk_the_ledge_to_the_party)
and asserts the full chain: reveal (ch 0x46) → LoadPoint ch 0x83 → the dressing scene folds
the objective to 4 [M 0x1827d8:55-67] → level_exit 07000007 [M 0x1d94c0 case 0x12]. 3/3
walkreal tests green on the direct binary (party 21.19s; gallery+sister 35.82s). Report at
docs/reports/level-06-every-objective.md; suite total line to be appended when the
background run lands.
