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
