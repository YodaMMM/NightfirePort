# WORKLOG

## level-14-interactables (branch s/level-14-interactables)

- start (master 61c7e3ea). Task: 07000014 has no per-kind interactables test — census every player-touchable placement [D placements + docs/levels/07000014.md], write `crates/nf-view/tests/interactables_07000014_live.rs` with one live test per KIND (doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, gadget targets) driven by real player input, fix every port gap the tests expose [M the class code; D the params], every test green, quote touched-binary totals.
- outcome (2026-10-30): interactables_07000014_live.rs — 9 green (one per kind; no lift kind on this level). Census corrections: 0x33 = Hint markers (fix-context-hints), 0x30 = Monitor consoles, 0x28 = Sensors. The 0x20 windows take the blast, never gunfire [M 0x1ece30 filter ~0x20|0x106; D param 2:0x20]. Report: docs/reports/level-14-interactables.md.
