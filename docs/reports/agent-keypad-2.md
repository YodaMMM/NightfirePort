# agent-keypad-2: the keypad page's input

Branch `s/agent-keypad-2`: the held `s/keypad-entry` squashed and replayed onto master d09f8b2b3,
which now carries the lock re-lock work; the keypad solve and the keyed open both use its state-2
fold (unlocked, burned, the out channel on the tick). No shared
random stream change. The page has no draw yet, so a player sees no change at a keypad: the press
still does nothing visible and Bond stays free, as on master.

## The gate: an undrawn page holds nothing

`Drones::keypad_page_drawn` is false and nothing sets it. While it is false the page takes no
input and `step_bond` leaves Bond's input alone. The page's draw sets it true, and the whole input
side below then runs. The held branch zeroed Bond on every use press with nothing on screen.

## What the page must draw before the flag is set

Page 0x40000021 of file 0x80000003, 24 controls in the level's 0x08000001 entry, identical on the
33 census levels [D `keypad_census.rs`]. All rects go through `FixupResolution` (`fe_pages`) except
the type-0xd window [0], whose own draw was not traced.

- Frame: control [23], button, id 0x10000001, rect 267,163,107,165, def fields f1 0x28, f2 0xb.
- Display backing: control [2], label, rect 271,166,98,21, texture 0x3000042, colour 0x263207ff,
  sub-rect msg 0x2d (0x250013, 0x20002).
- Display text: control [1], label id 0x10000075, rect 274,165,93,17, colour 0x70202ff, showing
  `Drones::keypad_display()`.
- Ten digit keys: controls [3]-[12], buttons id 0x100000ba, rects `KEYPAD_DIGIT_RECTS`, f1 0x27,
  f2 9. `Button_Update` picks the component state: 0x20 on an unfocused key (they all share the
  focused key's id), 0x80 on the focused key, 0x40 while ✕ is held [M 0x1ef948:52-96].
- Ten digit glyphs: controls [13]-[22], labels, 32x32, texture 0x300017c, colour 0x7f7f7fff, each
  with its own msg 0x2d sub-rect [D].
- The focus: `Drones::keypad_focus_digit()`. The manager's cursor sprite sits at the focused
  control's centre with its w/h [M 0x21edc8:139-149].
- When the Q-Worm has shown the code, the existing INFO message already posts [M 0x1ae3d8:246-257].

The world keeps running behind the page: `param_6` of the create is 0, no pause [M 0x21b908].

## The input side

- The cursor is a control, not a point. `Keypad::focus` holds the digit button; `menu::keypad_step`
  runs the existing `menu_find_control` port over the ten rects after `FixupResolution`. Only the
  digits can hold the cursor: the other 13 controls carry msg 0x2b value 6, whose bit 2 stays set,
  so their +0x7a is non-zero and every walk skips them [D, M].
- Home is digit 1 [M]: at open every control's +0x7a is non-zero, `Page_Update` parks the cursor on
  the backup control at (-100, -100), and the next frame `Menu_GetFirstControl` takes the first
  usable control in list order. `Manager_SendMessage(0x22, 0x10000002)` at create finds no such
  control on this page [D].
- Directions are actions 0x1a-0x1d: d-pad or left stick past half [M 0x10cd80:380-410], fresh or
  repeat (mask 0xc), repeat from frame 46 on counts divisible by 8 [M raw words 0x17f0d0-0x17f0f0].
  Order left, right, up, down; page mode 1 is the default and the def sends no msg 0x56.
- Select is `Button_Update`, not `Label_Update`: ✕ mask 4 plays sound 0 (0x1da) and sends 0x4b.
  Held, mask 8 plays the sound again and sends 0x61, which the handler ignores.
- Back is △, action 0x22, the manager's exit action. `MenuManager_Update` plays sound 8 = 0x1d9
  and `MenuManager_Monitor` deletes the manager the same frame [M 0x21c3d8:77-105, 0x21d508:29-37,
  order 0x1c9790:13-14]. Sound flags are 9 on this page [M 0x1f31f0], so □ and ○ are silent no-ops
  and need no lane.
- 0x1f and 0x22 refuse each other while the other is held [M 0x1f1f40].
- Lanes: `Input::keypad_confirm`, `keypad_back`, `keypad_nav`, held levels. Keyboard: Enter,
  Backspace, arrows or W/A/S/D [S the choice]. Pad: `menu::pad_keypad_lanes` merged in `gamepad.rs`,
  which is the one gamepad change, made so a pad can leave the page.
- `use_key_prev` is recorded while the page stands, so a use key held through the close is no fresh
  press.
- The solve equips the code's key [M 0x1b0650:15] and the arm reads the ring [M 0x1aed00:18-33]: a
  sister keypad on the same code opens on the press with no page. 0700000a's two code-2 locks are
  the case that shows it.

## Census [D]

15 kind-0 locks on 8 levels: 0700000a #286/#2d9 (code 2, out 0x31), #288/#2dc (1, 0x30); 0700000b
#0b2 (4, 0x2f); 07000011 #0b1 (1, 0x12); 07000012 #290/#2b6 (5, 0x0a); 07000014 #306 (2, 0x2c),
#307 (3, 0x2d), #308 (0, 0x3c); 07000015 #17d (2, 0xf9), #1b3 (50, 0xf7), #1ca (10, 0x95); 07000046
#025 (5, 0x3e). The digits themselves are boot RNG [M `Locks_Init` 0x1b0548].

## Proof

`keypad_entry_live` against the old `s/keypad-entry` code, with only compile shims added (the flag
field, the `keypad_nav` lane, `keypad_focus_digit` over the old point read, the two pure helpers):
9 of 10 red, including `a_page_nothing_draws_leaves_bond_free`, the steps, the sounds, the held use
key and the pad. `a_drawn_page_holds_bond_until_it_is_left` passes on both, as it should: the old
code held Bond too. With the fix, 10 of 10 green.

Run on the fix, all green: `lock_keypad` 7/0, `keypad_census` 1/0, `keypad_entry_live` 10/0,
`keypad_unlock_lod_live` 1/0, `use_key_census_live` 20/0, `decryptor_07000012_live` 1/0,
`interactables_` 0a 13/0, 0b 14/0, 11 16/0, 12 12/0, 14 10/0, 15 17/0, `walkreal_` 0a 13/0 (2
ignored), 11 5/0, 12 3/0, 14 9/0 (1 ignored).

Re-run on master d09f8b2b3: `keypad_entry_live` 10/0, `lock_keypad` 7/0, `use_key_census_live`
20/0, `keypad_unlock_lod_live` 1/0, `interactables_0700000a_live` 13/0, `decryptor_07000012_live`
1/0, `walkreal_0700000a_live` 13/0 (2 ignored), `walkreal_07000011_live` 5/0.

The `walkreal_*` routes do not press use at a keypad, so they say nothing about this page.

## Unproven

- Home on digit 1 is read from the code, not seen on a running game. One live read settles it:
  page up, manager cursor +0x24/+0x26.
- A page open when Bond dies, or a level ends: the game's order was not traced. Only matters once
  the flag is set.
- Enter/Backspace key choice is the port's own.
- `gamepad.rs` builds with the `gamepad` feature; no real pad was used.
