# agent-0a-door725: 0700000a's security room is entered through door #725

## What opens #725 [D + M]

- Door #725 (type 0xDB, (6.83, 1.10, −1.46)) has p3 = 49 (0x31). `Door_Create` 0x136100 copies
  level_tag+0x38 (p3) into obj+0x68, the lock channel. While +0x68 is set, E refuses the door.
  On the rising edge of switch_channels[+0x68], `Door_Update` 0x1363c8:88-107 posts "Door
  Unlocked", activates the door and zeroes +0x68 for good.
- A scan of every map's placements on 0a finds 49 in exactly three records: the door, and the two
  type-0x2E keypads #646 (east face, (7.01, 1.33, 0.26), yaw +π/2) and #729 (west face, yaw −π/2).
  Both keypads have p0 = 0 (keypad kind), p1 = 2 (KeyCodes row) and p5 = 49 (out channel).
  No trigger, switch or SP placement carries 49. Script event channel writes were not scanned
  (unproven, but nothing on the walked route writes ch 0x31 before the keypad: the test asserts it
  is off on arrival).
- The writer is the keypad. E arms it (`Lock_Activate` 0x1aed00 → state 3). When the four digits
  of KeyCodes row 2 match (`Lock_Update` 0x1ae3d8 case 4), `Lock_Unlock` 0x1b0650 sets state 1,
  and case 1 holds switch_channels[49]. This is the same machine as door #724's keypad (row 1,
  ch 0x30). The interactables test shows that E on the office PC reveals the row's code.
- The room's only walkable entrance is #725. Probe rays find a wall at x 6.75..7.0 from z −2 to
  z 17.5 that splits the lobby and room side from the east corridor. The keyboard's faces point
  into the room (−z).

## Port

The port already did all of this the game's way (door.rs lock channel, lock.rs keypad, the E arm).
**No sim change was needed, so no test here can fail on old port code.** The fault was in the walk
itself. On master it pressed the keyboard from the corridor behind the lift cab's east wall.

## Test change (walkreal_0700000a_live)

- The prologue (`walk_qworm_and_back`, formerly `walk_to_security_keyboard`) now ends in the lobby
  after the Q-Worm install. It no longer presses the keyboard. The short test is renamed
  `walk_office_floor_qworm_install_and_back`.
- New `security_room_visit`, spliced into the ring at the east corridor's west end (9.9, 5.3).
  Every step uses real input:
  1. Walk south to keypad #646 and press E on it. The test asserts that #646 armed, not another
     keypad.
  2. Type row 2's digits. Assert ch 0x31 comes on, the door opens by itself (state 4), and its lock
     channel is cleared.
  3. Walk through the doorway and across the room. Press E at the keyboard's screen from inside
     the room, getting ch 0x3A, then 0x49 and 0x4A.
  4. Walk back. Press E on the closed door from inside and return to the corridor.
- The test asserts ch 0x31 and ch 0x3A are both off when Bond reaches the door, and that Bond is
  inside the room when he presses the keyboard.
- This replaces the corridor press on master. It also makes the in-room placement on
  s/agent-use-pick unnecessary: when that branch merges, its walkreal keyboard hunk should be
  dropped in favour of this route. keyboard_0700000a_live, the focused test, is unchanged here and
  still presses from the corridor on master.

## Tests

- walkreal_0700000a_live: 13 passed, 2 ignored (the same 2 as on master). This includes the three
  real-damage ring variants, which now walk through the room too.
- keyboard_0700000a_live 1/1, interactables_0700000a_live 13/13, walk_0700000a_live 3/3.

## Unproven

- Script event (0xD/0x11) writers of ch 0x31 were not scanned.
- Whether the game lets the keyboard be used from the corridor: the code says no
  (s/agent-use-pick's refine). This needs a live check.
