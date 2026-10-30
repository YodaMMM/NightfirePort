# WORKLOG

## level-01-interactables (branch s/level-01-interactables)

- start (master 61c7e3ea). Task: every interactable KIND on 07000001 works like the game — census every player-touchable placement [D placements + docs/levels/07000001.md], read what interact_/walk tests for 07000001 already cover, add the missing kinds as `crates/nf-view/tests/interactables_07000001_live.rs` (model: interactables_0700000b_live.rs), fix every port gap the tests expose [M the class code; D the params], every test green, quote touched-binary totals.
- outcome (2026-10-30): interactables_07000001_live.rs — 6 green + 1 ignored (one per kind the earlier tests missed: the RB lamp takes the bullet's class-5 impulse; the comms screen smashes — the smash's channel-0 arm fixed [M 0x1bb60:34-37: no switch_channels write when the tag's channel is 0]; the 0x33 hint marker pins the disc's void parking [D coll-scan]; the ammo pair takes on the walked approach; the bonus medal gate; the music/fusebox/switch load-time census). Ignored: the bonus walk-over take (the west-hall floor is off the walker's reachable set). Report: docs/reports/level-01-interactables.md.
