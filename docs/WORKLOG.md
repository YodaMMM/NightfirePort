# WORKLOG

## level-01-interactables-2 (2026-11-01)

Task: every player-touchable interactable kind in 07000001 behaves like the game. Census of 07000001
placement kinds [D], one live test per kind in interactables_07000001_live.rs (model:
interactables_0700000b_live.rs), driving the real walk/E/shoot path, asserting the game's result with
evidence; fix port gaps the tests expose. Branch start: created s/level-01-interactables-2 from origin/master.

### level-01-interactables-2 outcome (2026-11-01)

The three player-touchable kinds the first pass left unpinned now have their live tests in
`interactables_07000001_live.rs` (9 passed, 0 failed, 1 pre-existing ignored): the comms console's
shot arm (health 1.0 from p1 = 0 [M 0x1b08b0:16-21], one bullet → state 3 + the 0x0600027 explode
[M 0x1afcc8:23-45]), the security camera's sight latch (31 consecutive cone ticks → the state-1
track; the p3 = 0 out-channel write stays gated off [M 0x1af618:211-216, 154-157]), and the radio
fax's E restart (SP_Activate → SP_Update's state-1 Script_Play re-runs the page cycle [M 0x1dc338;
0x1dbda8 case 1; D 0x06000016 stream 6]). No port gap needed fixing — all three paths were already
ported (drone.rs's monitor bullet arm, sensor.rs's tick, useobj.rs's player_activate); only the
kind-level pins were missing. The fax's load-state sister needed the [M 0x1dbc08] prime-chain
reading (Play + Update + Stop + the trailing Update leaves bc standing, no bd) before it held.
walkreal_07000001_live 6/0; nf-game 1015/0. Suite: 283 of 1058 binaries (sandbox kills background
jobs, foreground chunk run) — 822 passed, 4 failed (the 3 allowed slim-ISO disc_golden + the
slim-ISO identity_disc sector-count check, both environmental, master's own code), 49 ignored.
