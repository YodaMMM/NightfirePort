# level-12-interactables-2 — every player-touchable kind behaves like the game

Branch `s/level-12-interactables-2` (from `origin/master`). The census and the per-kind game
behaviour live in `docs/levels/07000012.md` §7 and the audit (`docs/levels/07000012-audit.md`,
every main-map placement with params [D]); this task covered the kinds the interactables file
did not yet drive.

## What the census says vs what was covered

The file had 9 tests (one is the parse-level census pin `every_placement_resolves_its_model`,
one the documented-blocker-ignored doors test). Covered kinds before this task: doors 0xdb
(walk + the ch 0xa edge), keypad locks 0x2e (the Q-Decryptor burn), the console/lift-call E +
the inverter 0x41, the shutter switch → SS pair 0xe2, the monitor 0x30 + the watched sensor
0x28, the hack computer 0x35 → the Q-Worm SP, the pickups 0xf0 (the Scorpion walk-over take,
the Briefcase as the untouched sister), the breakable glass 0x20 (the shot), and the model
sweep. After this task the file carries 11 tests.

## The two kinds this task added

1. **0xea TouchOnce plates** — `the_touchonce_plate_latches_its_channel_on_the_walk_in`:
   walking into plate #30 ("New Trigger4 (Once)" at (13.41, 1.22, 28.59) [D params (0x5, 0x8)])
   latches ch 0x5 ([M] `Trigger_TouchOnce` 0x1b2b88 keeps slot 1 = OUT = the placement's param 0;
   `Trigger_Update` 0x1b1a78 case 1; the port's `from_params` TOUCH_ONCE row out = q(0)]), the
   channel HOLDS after the toucher leaves (the kind's distinction from TOUCH's per-frame
   clear-then-OR in `Trigger_Calc` 0x1b1900 [M]), and the sister (a fresh load, nobody walks)
   keeps ch 0x5 off [D the channel census: no other writer].
2. **0xeb laser TOUCH triggers** — `the_laser_beams_write_no_channel_while_crossed`: all 20
   "laser" rows carry only param 7 = 0 [D], so out = param 0 = 0 and `Trigger_Calc` 0x1b1900's
   clear-then-OR lands the kind's writes in the unused channel 0 — the beams write nothing real
   (inert by data [D the audit's kind row]). The test crosses the #496-#503 stack through the
   real walk path and pins the ENTIRE channel board (1..=255) at its pre-walk values; the
   sister-half pins the same KIND's real-out row (#683, out 0x1f [D]) whose hold drives
   walkreal's exit leg.

## The kinds deliberately not re-driven (with the reason)

- **0x3a GRAPPLEPOINT** (#130 SkyBridge1 [D]): the level's loadout never owns the Q-Claw
  (weapon 0x50/0x51 [L the bank records; D the 12 pickup census has no claw item and the loadout
  line prints 4-5 owned weapons without it]) — the kind is not player-reachable on THIS level;
  the port's pick/range lives in `gadget.rs` (`pick_grapple_point`). The audit's "no live drive"
  note stands.
- **0xf9 SoundTriggers** (#747/#750 [D]): channel-driven emitters (`SoundTrigger_Update`
  0x1b2790 plays while the ON channel is set) — not player-touchable; the ON channels ride the
  script/alert paths.
- **0x27 frames / 0x33 hint plates / 0x71 hologram / 0xfa icon / 0x2a skyline**: display-only,
  no sim behaviour [D params; the audit's rows].
- **0xe8 the gated exit**: already driven end-to-end by `walkreal_07000012_live` (the REAL load
  into 0700004A with the carried RamSave — that test passes on this branch too).
- **0xe1 ExplosionBox**: the Destroy-family shot behaviour is the same class the 07000001 file
  pins (`shooting_the_comms_room_screen_smashes_it`); 12's box rides the heli script [D].

## Verification (the named binaries, every total)

- `interactables_07000012_live` — **10 passed, 0 failed, 1 ignored, 0 measured**
  (`--test-threads=1`, 22.04 s; the ignored one is the doors test's pre-existing documented
  blocker: the west-wing walk the router cannot leg — its mechanism is proven by the keypad
  test's door[19] edge).
- `walkreal_07000012_live` — **1 passed, 0 failed, 0 ignored** (5.22 s; the full route: spawn →
  the Q-Decryptor burn → the console E → the shutters → the shaft exit → the REAL 0700004A load
  with the carried save, both objectives completing).
- `nf-game` — **953 passed, 0 failed across 98 test binaries** (`--test-threads=1`).

## The full suite (the documented command, `-- --test-threads=1`)

Run in foreground chunks this window (the sandbox kills background jobs between tool calls):

- `-p nf-formats -p nf-audio`: **136 passed, 3 failed** — the 3 are exactly the slim-ISO's
  allowed `disc_golden` set (`bigf_header_fields_behave_as_documented`,
  `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`).
- `-p nf-game`: **953 passed, 0 failed**.
- `-p nf-view`: **736 passed, 0 failed across 172 test binaries** and still walking when the
  window closed — the run was inside `front_end_chain_01_to_05` (a master test this task's diff
  does not touch; the binary count includes the lib tests). No FAILED line appeared anywhere in
  the covered set. The tail binaries (the front-end chains and the long movie harness tests)
  remain for the next window's suite pass.

## What's left

- The nf-view suite's uncovered tail (the front-end chain binaries) — expected green (master
  tests, zero diff in their path); run to completion next window.
- The doors test's ignored blocker (the west-wing route) is unchanged and documented.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for the
session (no usage API or status line was available; nothing to copy verbatim).
