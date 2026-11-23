# s/touch-law-b

**The random stream does not move with this branch as committed** (`drone::TOUCH_LAW_LIVE` is
false, so Bond keeps the stand-in box contact). Switching it on DOES move it: the 0a hatch walk
needs a new lever, the 0b Operative recording runs 3 ticks longer, and the 0d real-damage roll
changes.

Based on `s/touch-ref-point-p`. Scope: levels 08, 0a, 0b, 0d, 011 and 4a.

## The switch

- `drone::TOUCH_LAW_LIVE` (false) picks `game_touchers` over `box_touchers` in
  `human_touchers`. `drone::touch_law_live()` is the same switch for tests whose pins differ
  between the two contacts.
- Recordings made under the game's contact sit beside the old ones as `*_operative_law.bin`
  (0a, 0b). `walkreal_0700000a/b` and `replay_parity_0700000a/b` pick the file by the switch.

## Per test, with the switch ON

| Level | Test | Result |
|---|---|---|
| 08 | walkreal (6), interactables (15, cable-car floor #236 included), walk (3), zipline | green, no change needed |
| 0a | walkreal hatch walks (4) | re-pinned: lever 2000/60 under the law (2000/30 kept for the box) |
| 0a | walkreal recording | new `0700000a_operative_law.bin` |
| 0a | interactables armour pickup | the walk waits up the office block for guard #8 to clear the z 5.5 corridor (both contacts green) |
| 0b | interactables lift_shaft_plates_fire_on_the_touch | re-pinned: from the roof deck key Bond walks north into #245 (a closed box, every face out; a stand inside touches nothing) |
| 0b | interactables sound_triggers ..._real_writes | re-pinned: walked from just inside the terminal room door onto #163 |
| 0b | walk files_parachute_and_mission_complete | re-pinned: the same walk onto #163 |
| 0b | walkreal fresh-spawn real damage | new `0700000b_operative_law.bin` (17788 ticks, was 17785) |
| 0d | interactables c5 / walk c5 corridor / walkreal arrival | re-pinned: #433's flat top is just above the standing HITTEST top (behind its up-facing plane), and its slope face meets Bond on the corridor ramp. Bond walks down the ramp |
| 0d | walkreal walk_arrival_to_the_c5_door_real_damage | **still red under the law**: Bond dies on the ladder #435 top (t4548-13397 over 20 stop levers); not re-pinned |
| 011 | interactables sound_triggers_created_and_played_per_the_disc_gate | re-pinned: plate #235 hangs 0.8 over the standing reach; sound 331 is now pinned on the vent fall (`the_vent_route_reaches_the_storeroom`) |
| 011 | walkreal walk_spawn_to_the_lower_floor_load_point (+ real damage, + 00 Agent) | **still red under the law**: the stairwell descent runs slower (the east connector takes 319 ticks, was 219) and Bond reaches flight B's bottom 0.9 high; not traced |
| 4a | interactables the_riders_first_steps... | re-pinned: under the law the bare stand fires #277. The standing HITTEST bottom rests 0.48-0.54 over its flat quad, inside r 0.55. At the real spawn it fires at tick 5. The sister now asserts that the stand fires exactly when the law is on |
| 4a | walkreal (4), satchel (2), walk (1) | green, no change needed |

08 cable-car floor: the boarding walk fires ch 4 under the law, so the rider contact does not
need the containment branch on this route. Whether the game's ch 4 comes from the floor face or
from the walk into the doorway was not separated.

## Tests run

- With the switch on (local build): every binary in the table, all green except the two rows
  marked red (0d real damage, 011 walkreal x3).
- With the box contact (the committed default), all green: walkreal_0700000a (15), interactables
  0a (14), 0b (14), 0d (11), 011 (17), 4a (6), walk_0b, walk_0d (3), replay_parity_0a/0b,
  walkreal_0b fresh-spawn real damage, walkreal_0d arrival. The 0d walk's stand sister and the
  4a bare-stand sister assert only under the law, since the box reaches those plates from the
  stand.
- With the switch on: replay_parity_0a/0b against the `_law` recordings, green.

## Unproven

- 4a #277 firing on the bare spawn stand rests on the port's standing rest height (0.006 inside
  r at the spawn). It has not been checked live.
- 011's slower descent and 0d's ladder-top death under the law are not traced to a cause.
