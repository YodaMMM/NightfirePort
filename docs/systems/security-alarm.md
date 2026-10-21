# The security-subsystem alarm (SSys) — SSys_Init 0x1b01b0, SSys_Monitor 0x1ad998, SSys_Msg 0x1ad878, SSys_RamLoad 0x1b0208

The four Phoenix Building security subsystems (laser 1, laser 2, camera 1, camera 2) and their
escalating alarm countdown. Ported to `crates/nf-game/src/ssys.rs` (task unported-misc-4).

## Reachability [M]

- `SSys_Init` ← `ResetMap_GameInit` 0x1d5080 (every level load).
- `SSys_Monitor` ← `Game_Run` 0x1c9790, after the `GS_IsPaused(-1)` gate and behind its own
  `ScriptCam == 0` check (docs/atlas/object.md "Update order").
- `SSys_RamLoad` ← `Player_RamLoad` 0x1957a8 (checkpoint loads).
- `SSys_Msg` ← `SSys_Monitor` only.

## State [M/L]

`SSysItems` 0x2befd0 (40 bytes): 5 × `{ f32 timer; u8 channel; u8 state; }`, item 0 unused.
`SSysDelayTimer` 0x30cd94. Thresholds [L ee_snapshot @ 0x30cd84..0x30cd94]: MIN3 180.0, MIN2
120.0, MIN1 60.0, MIN01 6.0. Init [M 0x1b01b0] items 1..=5 = {−1.0, 0xff, 0}; [L ee_start_05 @
0x2befd0] matches (items 1..4 = (−1.0, 255)).

## Monitor flow [M 0x1ad998]

Per item 1..=4, `ScriptCam == 0`:
1. channel == 0xff → skip.
2. channel off: state != 0 → state 0, `Msg(i,0)`, timer −1.
3. channel on: timer ≤ −1 → arm. Item 2 (laser 2) arms through the shared 8 s `SSysDelayTimer`;
   at its first expiry with item 1 still live it plays `Msg(1,0)`, forces item 1's channel on and
   disarms it (channel = 0xff), rewrites the delay to 8.0, and arms through the shared path at the
   next expiry. The shared arm: timer = 240.0 (0x43700000), state 6, `Msg(i,6)`.
   Otherwise timer −= dt (1.0 at the fixed 60 Hz).
4. Countdown: 6 → 5 at < 180 (`Msg 5`), → 4 at < 120, → 3 at < 60, → 2 at < 6, and state 2 at
   ≤ 0 → `Msg(i,1)`, the item's channel switched off, state 0, timer −1.

## Msg gates and sound tables [M 0x1ad878 / L]

With `GameState == 0x0700000b && switch_channels[0x76] == 0` only laser 2 (type 2) sounds play;
on 0x0700000a camera 1 (type 3) never plays. Each type reads `table[id].sound` (the u32 at
table + id·8 + 4, volume 100.0f) [L ee_start_05]:

| type | table | EE addr | id 0..6 sounds |
|---|---|---|---|
| 1 | `Laser1.134` | 0x2bef60 | 0x2a4 0x2a3 0x298 0x297 0x296 0x295 0x2a2 |
| 2 | `Laser2.135` | 0x2bef98 | 0x431 0x43d 0x298 0x297 0x296 0x295 0x438 |
| 3 | `Cam1.132` | 0x2beef0 | 0x2a1 0x2a0 0x298 0x297 0x296 0x295 0x29b |
| 4 | `Cam2.133` | 0x2bef28 | 0x2a1 0x2a0 0x298 0x297 0x296 0x295 0x29b |

## RamLoad [M 0x1b0208]

Items 1..=4 copy from the SSYS save block, skipping item 2 on 0x0700000a; on 0x070000b item 1 is
forced back to {−1.0, 0xff, 0}; `SSysDelayTimer = 0`.

## Tests

`crates/nf-game/src/ssys.rs` — 6 tests, each with a sister assertion that can fail: the
thresholds/arm-time bit patterns [L], the sound tables [L], the msg gates [M], the full
countdown with the channel-off shutdown, the laser-2 8 s double delay (7+1+8 ticks after the arm
tick — the game rewrites `SSysDelayTimer = 8.0` at every expiry), and the ram-load skips.

## Draw side

Nothing to draw: the system produces state and sound ids only (the HUD readouts the security
room shows are the monitor consoles, monitor.rs — a different system).
