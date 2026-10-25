
### level-07-party-door-lock (started 02:41 UTC, 2026-10-25)

Task: in the game, after certain points on the party level (07000007) the door back to the main
party room locks; in the port it stays usable. Find the party-room doors [D door placements +
lock/key channel params], what locks them [M switch channels the scripts/triggers/mission
records write at those story points, Door_* lock handling], list the story points that lock (and
any that unlock) with their channels; port any missing writer or lock handling with real-input
tests (before: opens; after: refuses with the game's message/sound; sister: other doors
unaffected).

Plan: branch from origin/master c64b2b80; census the 07 doors; find the lock channels in the
level scripts + Door_* lock handling in the decompile; reproduce through the real input path;
port what's missing; suite; report.
