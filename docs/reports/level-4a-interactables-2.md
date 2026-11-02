# level-4a-interactables-2 — report

Branch `s/level-4a-interactables-2` (from origin/master 8f27b95e). Task: extend
`interactables_0700004a_live.rs` (2 tests) to one live test per KIND of player-touchable
placement on 0700004a, modeled on `interactables_0700000b_live.rs`, with a fresh census
[D every placement kind], sisters that fail on the wrong result, and port-gap fixes with
evidence.

## Done

- **Census re-run [D]** (fresh, this session): `census_0700004a --ignored` — the ALL-MAPS kind
  table confirms docs/levels/0700004a.md §7 exactly. m41 (the playable main map) carries:
  0x0f×38, 0x19×7, 0x24×1, 0x31×1, 0x33×1, 0x47×57, 0xd2×2, 0xd9×4, 0xdb×162, 0xe9×2, 0xea×13,
  0xf1×6. The model list's other kinds are absent from m41: pickups 0xf0 (m6/m14 resource maps
  only), switches/consoles 0x29 (m6 only), monitors 0x2d (m13 only), keypad locks 0x2e (m6
  only), and breakables 0x20 / rigid bodies 0xd8 / SimpleScripts 0xe2 (not placed on ANY map of
  this level).
- **4 new tests** in `interactables_0700004a_live.rs` (2 → 6), each through the real input path
  with sisters:
  1. `the_riders_first_steps_walk_onto_the_top_reveal_plate` — KIND 0xea by the player's own
     WALK: 90 bare ticks standing in the spawn region leave ch 0x4d LOW (hard sister, measured),
     then 84 real steps (turn+forward only) fire plate #277 (out 0x4d, mask 8 [D #277; M
     Trigger_Hit 0x1b1fc8:56]), reveal the objective (states[17][0] = 3 [L record 4a]) and the
     TouchOnce deletes itself (glist-0x10 delete [M 0x1b1d78-0x1b1d90]).
  2. `the_bare_ride_carries_the_player_to_the_landing_hold` — KIND 0x31 + the 0xd9 lift SP: the
     bare ride (no input, no re-seat) descends the key line [D 0x06000571 keys], the t1610 event
     writes ch 0x40 [D keys], the landing scene disables and the hand-back re-enables Bond on
     the held platform [M Script_Free 0x1da880; D 0x060006cf s9/s26], drop 42–50 at the hold
     [D keys]. Sisters: ch 0x40 low + all doors closed at load.
  3. `the_bottom_plate_walk_in_triggers_the_lobby_script_and_the_level_end` — KIND 0xd9, the
     SimpleScript the player TRIGGERS: the ride's end carries Bond into bottom plate #62 (out
     ch 0x84 [D #62]); lobby SP #276 (script 0x060006d3, out 0x84 [D #276]) runs (state 2 [M
     SP_Update 0x1dbda8 case 1→2]) and the mission's LevelEnd follows [M 0x182c60:73].
     Sisters: ch 0x84 low, no exit, #276 waiting at load.
  4. `m41_places_none_of_the_pickup_switch_monitor_lock_breakable_rigid_or_simplescript_kinds`
     — the absent-kinds census pin [D census] (0xf0/0x29/0x2d/0x2e/0x20/0xd8/0xe2 empty on m41),
     with the placed kinds' §7 counts as the sister (a parse/data regression fails here first).
- **No port gap exposed**: all 6 tests pass without any nf-game change (the one failure during
  development was my own test's early loop break eating the mission-state settle ticks — a TEST
  bug, fixed in the test; walkreal's full-84-tick window shape restored).
- **Measurement correction recorded** (not a test change): interact_0700004a_live's comment says
  the plate fires "as soon as the ride ticks" from the spawn-inside-volume; measured here: a
  BARE stand does not fire #277 for the whole 90-tick pre-descent window — the player's own
  walk (or the re-seat poke that interact_'s harness does) moves him onto the volume. interact_'s
  assertions are unaffected (its Ride re-seats); this file pins the true causality.

## Verification (what ran, with totals)

- `interactables_0700004a_live`: **6 passed, 0 failed, 0 ignored** (12.9 s).
- `walkreal_0700004a_live`: **2 passed, 0 failed, 0 ignored** (11.2 s).
- `nf-game` (all 100 test binaries, `cargo test -p nf-game -- --test-threads=1`): **1033 passed,
  0 failed, 9 ignored** — the same totals the previous tasks recorded; no src change, so no
  regression.
- The fast suite (`tools/test/suite.py`): NOT completable in this sandbox — the background runs
  were killed between tool calls 3× (sandbox kills nohup'd children), so it was re-run as
  foreground chunks; the master-relink of nf-view's ~600 test/example targets exceeds the
  window (same outcome the level-14/15 sessions recorded). The binaries my task touched were
  run directly (the two lines above). No src file changed on this branch (one test file only),
  so every other binary is master-identical.

## Left

- One full fast-suite pass on the final tree (the driver's next session: the build cache is
  warm; only nf-view's remaining ~300 example/test relinks + the ~13 min run phase remain).
- The parked drone-fire leg (`a_spawned_drone_takes_the_riders_fire_across_the_shaft`, ignored)
  and its two open live checks — unchanged from the prior task (docs/levels/0700004a.md §7).

## Live checks for the main agent

Carried unchanged from docs/levels/0700004a.md §7 (level-4a-every-objective): (1) gdbwatch the
group-2 templates' first `Drone_Create` (0x136be0) against Bond's y through the −40 floor — does
the real game spawn at the arm tick (ch 0x17's rise) or later; (2) whether the real rider's fire
can land on the group-2 spots (x ≈ −4.3, z −0.55) mid-ride (the port's eye→spot rays are blocked
for the whole descent, 0/2400).

## Usage

not visible to me — this environment exposes no token, credit, cost or context-window figures
for the session (no usage header, no billing panel, no context meter is shown to me anywhere).
