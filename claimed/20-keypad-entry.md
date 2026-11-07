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
