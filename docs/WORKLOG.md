
## keypad-entry

Branch `s/keypad-entry` from `origin/master` (10956badb). Task: Bond can type a code on a
keypad lock and it opens — `lock.rs` ports the `Keypad` machine (`Lock_Update` 0x1ae3d8
cases 3/4, `C_KEYPAD_Handler` 0x216810) but nothing calls `Keypad::digit` from input; port
the page's input (cursor over the digit grid, confirm types the digit, cap wrap, display,
beep, close), census every level with a keypad lock and its code [D], pin through real
input on two census levels (right code opens + writes the out channel, wrong code does
not, closing leaves the lock usable), run the lock binaries, the census levels' walkreals
and nf-game. Entry started.

Work (same entry): the page's input landed — the cursor point over the disc's digit grid
(page 0x40000021's ten 0x100000ba controls [D, pinned by keypad_census.rs across 33 levels]),
the confirm edge types via `C_KEYPAD_Handler` 0x216810's msg-0x4b arm [M], the cap wrap, the
`****` display [M 0x216810:24-29 — the existing lock_keypad test said "": fixed with the
address], the 0x1da menu beeps [M 0x1f3278 cases 0/2], the back edge closes and the lock
re-arms [M 0x1ae3d8:292-295]. The census: 15 kind-0 locks on 8 levels; 07000011's #0b1
starts unlocked on Operative [M 0x1ae1f8:67-69]. Real-input pins green on 0700000a
(solve + channel 0x31, wrap) and 07000011 (wrong code, close, re-arm, solve + channel 0x12).
