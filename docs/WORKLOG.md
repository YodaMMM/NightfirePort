# WORKLOG

## level-02-interactables (branch s/level-02-interactables)

- start (master 2f1edf31). Task: census every player-touchable placement on 07000002 [D placements + docs/levels/07000002.md], write crates/nf-view/tests/interactables_07000002_live.rs with one live test per KIND of player-touchable placement (doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, gadget targets) driven the way a player drives it (walk, use key, trigger pull, walk-over, gadget), each asserting the game's effect with a sister. Fix every port gap the tests expose [M the class code; D the params]. Every test green (no #[ignore] without a named blocker). Run the touched binaries and quote their totals.
