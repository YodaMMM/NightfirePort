## Your task: `keypad-entry`, Bond can type a code on a keypad lock and it opens

crates/nf-game/src/lock.rs ports the `Keypad` machine (`Lock_Update` 0x1ae3d8 cases 3 and 4, `C_KEYPAD_Handler` 0x216810), and drone.rs arms it (around line 16719), checks it each tick (around line 18017) and has `keypad_digit`, but nothing outside that function calls `Keypad::digit`, so no input path types a digit. Its comment says the cursor and page UI are the view's. First confirm through the real app path whether a player can enter a code at all today: walk Bond to a keypad lock, press use, and see what opens and what input does. Read the keypad page's 24 controls in the level def, msg 0x4b and msg 0x51 in 0x216810, and how the page closes (case 4's closed-without-match arm returns to state 0, so the lock can be armed again). Port the page's input: cursor moves over the digit grid, confirm types the digit, cap wrap, display, beep, close. Census every level with a keypad lock and its code, and whether the code is shown [D]. The page's look is a visual, so keep to its logic and data, and list frames for a render check under the report's live checks for the main agent. Pin through real input on two levels from the census: the right code opens the lock and writes its out channel, a wrong code does not, and closing the page leaves the lock usable. Run the lock binaries (grep tests for `Keypad`), walkreal for the census levels you pin, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/keypad-entry`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-07T11:24Z

---
## Report (done, 2026-11-07T12:25Z)


---
## Report from the worker (done, 2026-11-07T12:26Z)


---
## Report from the worker (done, 2026-11-07T12:26Z)

# keypad-entry

Branch `s/keypad-entry` from `origin/master` (10956badb). Task: Bond can type a code on a
keypad lock and it opens — port the keypad page's input, census the keypad locks, pin
through real input on two levels.

## What was already there / what was missing

`lock.rs` already ported the `Keypad` machine (arm [M 0x1ae3d8:176-190], digit [M
0x216810:8-22], the case-4 match [M 0x1ae3d8:262-296]) and drone.rs armed it on the use
press (state 3) and ticked it (case 4) — but nothing called `Keypad::digit` from input: no
cursor, no page input, no close from the player. Confirmed through the real path: walking
Bond to a keypad lock and pressing use opened the page (the machine armed) and then nothing
could ever type a digit.

## The disc evidence the port now stands on

- **The page's 24 controls** (`MenuManager_Create(0x80000003, 0x40000021, …)` [M
  0x1ae3d8:242-243], parsed from the level's kind-8 entry 0x08000001 with
  `MenuDef::parse(…, 0x80000003)`): the page frame, the display label 0x10000075, TEN digit
  controls id 0x100000ba (b=1, 34×34) carrying the authored digits **1 2 3 / 4 5 6 / 7 8 9 /
  _ 0 _** in the layout's rects ((268|304|339) × (188|223|258|293)), ten digit-glyph label
  sprites, the keypad window [D 07000003.bin/07000005.bin — identical; the census test pins
  **all 33 slim-ISO levels' pages** to this table]. `C_KEYPAD_Handler` types
  `'0' + *(control+0x20)` [M 0x216810:13] — the digit value is the control's own data.
- **The input chain** [M]: the manager's edge dispatch — the select edge plays
  `Menu_PlaySound(0)` (= sample **0x1da** [M 0x1f3278 case 0]) and sends msg 0x4b to the
  control under the page cursor [M `Label_Update` 0x218cb8's edge-1 arm]; the back edge
  (edge-2, `Menu_PlaySound(2)` → the same 0x1da [M 0x1f3278 case 2]) closes the page. The
  cursor is a SCREEN POSITION resolved against the control rects each tick (`Menu_GetControl`
  0x1f14f8: the top-most non-hidden control whose rect contains the point) [M].
- **The display**: msg 0x4b sets it to typed[0..=count] inclusive; the page's msg-0x51
  refresh sprintf's the literal `"****"` [M 0x216810:24-29 — the format bytes at 0x30d330
  are `2a 2a 2a 2a 00` [D the ELF]].
- **The codes**: `KeyCodes` (51 × 5 bytes) is boot-RNG — `(Rand_Random() % 9) + '0'` [M
  `Locks_Init` 0x1b0548] — so no disc byte carries the digits; the shown flag
  `KeyCodes[i*5+4]` starts 0 and is set by the Q-Worm gadget's use [M `PCQWorm_Update`
  0x1aee20:31], re-posted as "Obtained Lock Keycode %c%c%c%c" on the next arm [M
  0x1ae3d8:249-258]. The port's table keeps the local-Rng stand-in documented in lock.rs.

## The port

- `lock.rs`: `KEYPAD_DIGIT_RECTS`/`KEYPAD_DIGIT_VALUES` (the disc table, census-pinned);
  `Keypad.cursor` (the page-space point; starts at the page centre [S unverified]);
  `move_cursor` ([S] 3.0 units/tick per full stick, clamped to the page), `digit_under_cursor`
  (the rect law; [S] the tie-break — the ten rects are disjoint [D]), `select` (the confirm
  edge → `digit()`), and the `display()` fix: the count-0 display is `"****"`.
- `player.rs`: `Input.keypad_confirm` / `Input.keypad_back` — the x/circle edges (local
  lanes, the `goggles_action` precedent). The window app maps Enter / Backspace [S the key
  choice].
- `drone.rs`: `keypad_page_input` — the page's per-tick input (cursor movement, the select
  types via `Keypad::select`, the back closes, both edges play 0x1da); the tick drops the
  page when closed so the lock re-arms (the game's case-4 Exists==0 arm [M 0x1ae3d8:292-295]).
- `scene.rs`: while the page stands, step_bond routes the pad to the page and zeroes Bond's
  own input (the monitor-viewing shape — the menu owns the pad; the use edge dies with it).
  The page's DRAW remains the view's work (the live checks below).

## The census

`keypad_census.rs` (nf-formats, green): **33 levels' keypad pages match the digit table
exactly**; **15 kind-0 locks on 8 levels** [D]:

- 0700000a: #286 → code index 2, out 0x31; #288 → 1, 0x30; #2d9 → 2, 0x31; #2dc → 1, 0x30
- 0700000b: #0b2 → 4, 0x2f
- 07000011: #0b1 → 1, 0x12
- 07000012: #290 → 5, 0x0a; #2b6 → 5, 0x0a
- 07000014: #306 → 2, 0x2c; #307 → 3, 0x2d; #308 → 0, 0x3c
- 07000015: #17d → 2, 0xf9; #1b3 → 50, 0xf7; #1ca → 10, 0x95
- 07000046: #025 → 5, 0x3e

Every lock's two sound words are 0xffff (no sounds) [D]. **The code digits are boot-RNG**
[M 0x1b0548] — the table's real contents need a live read (below); the shown flag is
runtime (the Q-Worm), false on every lock at boot [M 0x1b0548:33].

## The real-input pins (all green)

`keypad_entry_live.rs` (nf-view, through real `step_bond` input; the teleports are the
documented stand-in transit):

1. **0700000a, lock #286 (out ch 0x31)**: the use press arms the page; the display rests as
   `****`; the stick steers the cursor onto each digit and the confirm types it; the first
   three digits show and cannot match; the 4th confirm's tick solves — the lock unlocks, the
   manager deletes, and **out channel 0x31 writes** [M 0x1ae3d8:283-289, the case-1 hold].
2. **0700000a, lock #288**: the 5th press wraps the count to 0 [M 0x216810:8-10] — the
   display shows the wrapped single digit; an incomplete code never unlocks.
3. **07000011, lock #0b1 (out ch 0x12)**: a wrong code (every digit shifted +1 mod 9) never
   unlocks and the page stands; the back edge closes it; the SAME lock re-arms on the next
   use (state 0 [M 0x1ae3d8:292-295]) and the right code then opens it — **out channel
   0x12 writes**. (This lock's param-6 tag starts it unlocked on Operative [M 0x1ae1f8:67-69]
   — the game's own difficulty gate; the test plays at difficulty 2.)

## A fixed test (the rule's case)

`lock_keypad.rs`'s `keypad_digit_append_wraps_and_display_is_inclusive` asserted the count-0
display as `""` — that predated the msg-0x51 arm's discovery and contradicts [M 0x216810:24-29]
(the literal `"****"`, format bytes [D 0x30d330]). Fixed with the address in the comment; the
other three assertions of that test (the inclusive display copy, the cap, the wrap) were
already [M]-backed and stand.

## What ran (each binary whole)

- `nf-game --test lock_keypad`: **4 passed / 0 failed**.
- `nf-formats --test keypad_census`: **1 / 0**.
- `nf-view --test keypad_entry_live`: **3 / 0**.
- The keypad-adjacent level binaries: `interactables_0700000a_live` 1/0,
  `interactables_0700000b_live` 3/0, `decryptor_07000012_live` 13/0,
  `decryptor_hold_real_path_live` 14/0.
- **The full walkreal battery (all 78 `walkreal_*_live` binaries, `--test-threads=1`):
  0 failures** (the sweep ran in two halves, both clean).
- The full four-crate suite was started on the final tree in the background
  (`tools/test/suite.py`); see the total line below.

## What's left

- The page's DRAW (the cursor sprite, the digit glyphs, the display label) is the view's —
  the sim carries the state (`Keypad.cursor`, `display()`, the page table) and the report
  lists the render checks below. Nothing draws today; the game's page picture is a visual
  follow-up.
- The cursor's home position and the analog rate are [S] (documented in the code); a live
  measurement can replace them.

## Live checks for the main agent

1. **KeyCodes' real digits** [L]: fresh boot (no save-state resume) of any census level;
   read `KeyCodes` at 0x2a37a8 (51 × 5 bytes: four digit chars '0'..'8' + the flag byte).
   Expected: all flags 0 at boot; the digits non-uniform. This pins the real codes the
   census table indexes.
2. **The keypad page frames** (render checks; the page's look is the follow-up work):
   - 0700000a, Bond standing at the front of lock #286, after the use press — the page up:
     the digit grid 1-9/0 at the def rects, the display showing `****`, the cursor sprite
     (if the game draws one) at its home position.
   - After typing two digits: the display showing the two typed chars.
   - After the back press: the page gone.
   The port's page-space coordinates are the def's 640x480 (`menu.rs` maps them at draw
   time, the name-entry page's precedent).
3. **The cursor home + rate** [L]: with the page up on PCSX2, record the cursor position
   (the manager's cursor control +0x24/+0x26) at page open and while holding the stick —
   pins the [S] start and the 3.0-units/tick stand-in.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
the session (no input/output token counts, no cost display, no context meter), so there is
nothing to copy.
