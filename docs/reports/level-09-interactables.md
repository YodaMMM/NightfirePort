# level-09-interactables — the per-kind interactables battery for 07000009

Branch `s/level-09-interactables` (master 5ae5d60d + the batch-1030w merge bdb8c6e0). Window
2026-10-30 ~18:42–20:42 UTC.

## What's done

**The census** (main map 39, the [D] dump via `census_07000009 --ignored` + docs/levels/
07000009.md): 12× 0x29 Switch (11 `alarmswitchplayer` — 7 on ch 0x7C, 4 on ch 0x7D — plus
`SCRIPTPLAYER_Lasers` #328 at the origin, whose use the game itself cannot reach [D placement at
(0,0,0); the laser state rides the scripts per the doc]), 3× 0xD9 ScriptPlayer (#327 the elevator
panel, #278/#279 the script lift), 2× 0xE2 SS (the LIFTDOORS pair), 3× 0xEA touch plates
(#505 arrival ch 0x51, #529 liftside ch 0xAB, #324 exit ch 0x32), 3× 0xF0 pickups (the armour
kind 3 + two kind-6 "Pickup 007"), 2× 0x33 hint markers, 1× 0xE8 Load Point, 6× 0xDB doors,
17× 0x20 breakable glass, 6× 0x0F guards. Zero player-touchable placements of: locks 0x2E,
monitors 0x30 (none in any of the 40 maps — the security-room wall is baked art), grapple 0x3A,
gadget targets 0x35–0x38, copters 0x2F, turrets/sensors. Not player-touchable: the muxes
0xEC/0xEF, SoundTriggers 0xF9 (audio riding the already-pinned channels), FuseBoxes 0x31 (23
boxes, no params → no channels [D]), rain 0x32/0xDF, skylines 0x2A, CEL rooms 0x47, the rotor
0x71, AI infra 0xE9/0xF5, scenery 0x19, the player start 0x24.

**The coverage map** — already driven before this task: the alarm switches (both mux inputs +
the no-press sister), the glass (the real fire path + the 0x30-filter refusal), the E-doors + the
locked shutter, the armour (take + parked-short), the Load Point's E-refusal, the ch-0xAB plate,
the guard kill — `interact_07000009_live.rs`; the arrival plate, the panel activation
(SP → ch 0x39 → obj 2) and the exit chain (ch 0x32 → 0700000a) — `walkreal_07000009_live.rs`;
the script-lift call stations (light, ride, doors, out-0 silence) — `garage_lift_calls_live.rs`;
the indicator art — `lift_indicators_09_live.rs`.

**The added battery** `crates/nf-view/tests/interactables_07000009_live.rs` (modelled on
`interactables_0700000b_live.rs`, one live test per KIND with sisters, teleports only to reach
spots whose walk is walkreal's separately-proven route):

1. **Kind-6 BOND_BONUS pickups** (#525 lobby, #526 car park [D]): the gold-medal-gated creation
   [M Pickup_Create 0x18dda0:93-95; the 0x0700005/9/11 group's rank-1 gold bit, L 0x2bc920 row 5]
   — with the medal word both bonuses exist at their disc spots and the real walk-over consumes
   the car-park one [M Pickup_Handler 0x18e650:234-236], the lobby one still sitting (sister);
   with the fresh-boot zero word NO kind-6 is created while the armour kind-3 row still exists
   (the gate's own absence, positive control in-test).
2. **The 0x33 hint markers** (#539/#540, label 0x01000291, no sound, no stage gate [D params]):
   both register with the disc label [M Hint_Create 0x1b3248; parsemap case 0x33 0x1d04f0:194-198]
   and a real walk into the west marker's sphere fires it once — the INFO pane holds label
   0x01000291 [M Hint_Update 0x1b3380:24-42, Text_AddMsg life 0xb4] — while the east marker 2.9
   away stays unfired (sister).
3. **The LIFTDOORS SS pair's positive arm** (#259/#260, wait ch 0x39, spline open [D params (0:0x40,
   2:1, 3:0x39, 6:1)]): the real E pulses at the panel write ch 0x39 [M 0x1dbda8:221-229] and both
   door matrices slide apart, leaving wait state 0x40 [M SS_Update 0x1a9948 state 0x40→1]; the
   in-test sister pins the load pose + ch 0x39 off before the press. (The E-refusal negative was
   already pinned by `interact_07000009_live` [M 0x1a9ea8 mask&4].)

**No port gap found.** The port already carried every kind the census found (the hint markers in
`player.rs::hints` [M-cited], the kind-6 gate in `pickup.rs`, the SS channel gate in `ss.rs`);
the battery went green on the untouched port — no game-code change, no test weakening, no
decompile contradiction met. The two suite runs that would share my assertions all stayed green.

## How it was verified (the touched binaries, --test-threads=1, slim ISO)

| binary | result |
|---|---|
| interactables_07000009_live (NEW) | **4 passed; 0 failed** (3.1–3.2 s) |
| interact_07000009_live | 16 passed; 0 failed (14.5 s) |
| walkreal_07000009_live | 4 passed; 0 failed (129.9 s) |
| garage_lift_calls_live | 2 passed; 0 failed (2.2 s) |
| lift_indicators_09_live | 2 passed; 0 failed (1.5 s) |
| security_alarm_09_live | 2 passed; 0 failed (7.3 s) |
| walk_07000009_live | 3 passed; 0 failed (4.0 s) |
| interactables_0700000b_live (the model battery) | 9 passed; 0 failed (16.9 s) |
| census_07000009 (--ignored, the [D] dump) | ok |

42 passed, 0 failed across the family. The new battery was re-run on the final merged tree after
the `origin/master` merge: 4 passed; 0 failed.

## The fast runner — what happened

The runner could not run as specified, for two sandbox reasons, both hit and documented this
window:

1. **Background processes do not survive tool calls here.** Three launches in the first minutes
   (`nohup python3 tools/test/suite.py`, with the cargo PATH fix, then `setsid`+`disown`): each
   was alive at launch and gone by the next tool call (empty `suite.out`, `pgrep` finds nothing).
   The first launch also exposed a second trap: `cargo`/`pkg-config` are not on the default PATH
   and ALSA lives in a user prefix (`PKG_CONFIG_PATH=/home/z/.local/usr/lib/x86_64-linux-gnu/
   pkgconfig`, `LIBRARY_PATH=/home/z/.local/usr/lib/x86_64-linux-gnu` for the `-lasound` link).
2. **The disk cannot hold the full binary set.** The relink of ~900 deleted-by-cleanup test
   binaries consumed the 3.6 GB freed for it and died with `No space left on device` mid
   `cargo test --no-run` (quoted in the session; 0 bytes free at death).

So the runner's coverage was delivered as per-binary foreground chunks over the task's family
(the table above), each binary under its own timeout, every total quoted. The one-number full
suite total line is **not** available this window; the next window can run
`tools/test/suite.py` on this branch once a background-capable sandbox (or a larger disk) is
available.

## What's left

- Nothing on the task itself: the census is complete, every kind is driven live with sisters,
  and the port needed no fix.
- For the next window (if wanted): the full four-crate suite run for the one-line total, and the
  `interactables_07000009` battery against any master movement since bdb8c6e0.

## Live checks for the main agent

None required: no visual work shipped (the tests are sim-side), and no sim-vs-game divergence
was left unresolved.

## Usage

not visible to me — no token, credit or context-window figures are exposed anywhere in this
session's interface (no input/output token counts, no context-used meter, no cost line).
