
## keypad-entry

Branch `s/keypad-entry` from `origin/master` (10956badb). Task: Bond can type a code on a
keypad lock and it opens — `lock.rs` ports the `Keypad` machine (`Lock_Update` 0x1ae3d8
cases 3/4, `C_KEYPAD_Handler` 0x216810) but nothing calls `Keypad::digit` from input; port
the page's input (cursor over the digit grid, confirm types the digit, cap wrap, display,
beep, close), census every level with a keypad lock and its code [D], pin through real
input on two census levels (right code opens + writes the out channel, wrong code does
not, closing leaves the lock usable), run the lock binaries, the census levels' walkreals
and nf-game. Entry started.
