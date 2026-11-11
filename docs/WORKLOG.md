# WORKLOG

## guard-castle-chat-states (branch s/guard-castle-chat-states)

Task: the castle's chatting and interrogating guards act as in the game. Port
CastleChatGuard1 [M 0x1612e0] and InterogateWalk [M 0x162338] (named unported in
docs/reports/guard-noise-attack-states.md on origin/s/guard-noise-attack-states), with their
message handling, in the game's order, keeping the Rand count. Red test first through real
input, pin via real play on a level that uses them, then every walkreal_*_live for the levels
that use them, one level per tool call, repairs gated to level. Run ai binaries + nf-game.

- Checkpoint 0 (11:59 UTC): sandbox was reset — full redo of setup: clone from origin/master
  (784ba2d99), work-slim + ISO re-downloaded (ISO size 369823203 exact). Branch cut, WORKLOG
  started. Next: read both handlers in full with addresses, find the levels' placements [D],
  red test, port.
- Checkpoint 1 (13:06 UTC): the port landed — the DoModeSettingsNEW start-script specials [M
  0x14c600:130-158] (the entry path the unreached.md audit missed: its census covered placement
  params 15/16, not the param-1 specials), the PlayScript mode-0x18 redirect [M 0x15ba68:26-30],
  CastleChatGuard1 0x36 [M 0x1612e0] and InterogateWalk 0x3d [M 0x162338] with their msg handling;
  zero Rand draws in both handlers [M]. Red first: 0 passed / 2 failed on the old code; green 4/4.
  [D] census: 07000006 is the only level carrying the specials (0x06000124 + 0x06000126, the dock
  chat pair). nf-game lib whole 786/0/4i. Open: the pair does not spawn at level start (event-
  gated — the live probe is #[ignore]d with the dump evidence); the full suite died on ENOSPC
  (6.2 GB target on a 9.9 GB disk).
- Checkpoint 2 (13:48 UTC): report written, branch pushed.
