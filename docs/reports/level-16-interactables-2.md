# level-16-interactables-2 — every player-touchable kind on 07000016 behaves like the game

Branch `s/level-16-interactables-2` (from origin/master 8f27b95e; master had not moved when re-checked,
so the branch is the final tree). Session window 01:54–03:32 UTC.

## What's done

1. **The census re-run [D].** `census_07000016.rs --ignored` re-dumped all 707 placements across the
   64 maps (matches the doc's 707), **38 distinct kind values**; main map m63 carries 26 of them and
   360 placements. The doc's §7 header says "39 kinds" — the recount finds 38 buckets; a few of the
   doc's per-row counts mix all-map and m63 scopes (e.g. 0xdb doors: ×8 on m63, ×9 all-map — m5's
   `tower_door1` template copy is the ninth; 0x0f drones: ×21 all-map, ×20 on m63; 0xf0 pickups: ×19
   all-map, ×17 on m63). Every kind row's existence and wiring in §7 checked out; I did not edit the
   doc (the wiring rows are the authority and all confirmed).
2. **The kind-coverage audit.** The 8 existing tests already pinned: doors (0xdb, incl. the locked
   door[0] + the SecurityStation activate path), the switch kind (0x29 KikoDeathButton → ch 0x90 →
   mux → MissionState 6), pickups (0xf0 walk-over), destroyables (0xe1 shot smash), hurt volumes
   (0xe4, the configured-inert pair), the ladder (0x22 climb), the spawner kind (0xf1 group-1 off the
   spawn plate), and model resolution. The census says there is **no fusebox kind, no monitor kind, no
   rigid-body kind and no 0xE2 SimpleScript placement on this level** — those items of the task's kind
   list have no disc referent here. The two player-touchable kinds with no live test were:
   - **the SimpleScripts the player triggers** (kind 0xd9 ScriptPlayers behind the player's inputs);
   - **the lift** (ScriptPlayer_ShuttleBayLift #694 — the level's only rideable platform).
3. **Two new live tests** in `crates/nf-view/tests/interactables_07000016_live.rs` (now 10 tests):
   - `the_door_panel_e_starts_the_corridor_graphic_script_players` — the mask-0x04 panel #684
     (3.36, 0.84, −141.50, out 0x27 [D]) E'd through the real use dispatch raises ch 0x27; the two
     ch-0x27 Door_corridor_graphic ScriptPlayers (script 0x06000999; the census's #432/#501 = the
     port's main-map SP indices #85/#154) start playing — create state 1 (the mode-0x40 create load
     beat [M 0x1db9a8:81-86]) → state 2 [M 0x1dbda8 case 1] — and door[3] #364 (lock p3 = 0x27 [D])
     opens on the same E. Sister: ch 0x27 off and both SPs at state 1 (created, stopped) before.
   - `the_launch_chain_rides_the_shuttle_bay_lift` — the real walked touch of the launch-room plate
     #602 (2.37, 1.04, −176.41, out 0x81, mask 8 [D]) runs the chain, the launch script's stream-21
     event writes [80, 1] [D 0600090a s21 — dumped], the lift SP #694 (script 0x060009e9, mode 1,
     out 0x50 [D]) un-holds [M 0x1dbda8 case 2 = SP_UnPause], the cab entities (stream 1 hash
     0x02000a79, stream 2 hash 0x02000a7a [D — dumped]) descend their stream-1 keys from y 1.60 to
     −21.22 [D 060009e9 s1, 38 keys dumped], bottom out at **−21.22 exactly** and rest, and the lift
     script's own stream-7 event writes [80, 0] [D 060009e9 s7 — dumped]. The bay drone #634 (skin
     0x500005e, wake p3 = 0x50 [D]) wakes (state 1 → 28) and rides the cab down from y 0.74 to the
     bay floor −22.08. Sisters: ch 0x50 cold, the cab at its top key (y 1.61), the drone dormant at
     state 1. Instrumentation prints kept: the pulse ticks (3207→3388) and the per-100-tick descent.
4. **No port gap surfaced.** Both kinds already behave — `nis.rs`'s Player state machine, the
   create-load beat, `Script_Interp`'s stream keying and the drone wake all carried the disc behavior
   without a fix. Two evidence notes from the dumps:
   - The doc's "the lift's own script 060009e9 writes [80, 0] at its end" is right about the WRITER
     (stream 7 [D]) but the clear lands +181 after the summon in the port (the cab is mid-flight,
     y −4.31, at the clear; the descend continues to −21.22 and holds). The stream-7 event's
     in-script timing (a command-only stream) is what sets it — the doc's "at its end" phrasing is
     loose, not a port deviation, as far as the port's own dumps show.
   - The graphic SPs' live matrices do not move (their animation is not a translation) — the pin is
     the state machine (create 1 → playing 2), not a pose delta.

## How it was verified

- `interactables_07000016_live`: **10 passed, 0 failed, 0 ignored** (`cargo test --offline --release
  -j1 -p nf-view --test interactables_07000016_live -- --test-threads=1`).
- `walkreal_07000016_live`: **13 passed, 0 failed, 1 ignored** (the pre-existing real-damage leg,
  documented in its own ignore note).
- `nf-game`: **1033 passed, 0 failed, 9 ignored** (100 binaries).
- Fast full suite (`tools/test/suite.py`, NF_ISO set, background): see the total line below.

## Suite total

(to be filled from suite-16.out before the final push)

## What's left

- The walkreal real-damage leg (the pit-squad phase kills Bond at ~48 health) — pre-existing, the
  next leg is squad-phase cover tactics (walkreal's own ignore note).
- GAP-16-1 (the 0x3b display pairs' draw-side re-sync) — pre-existing, untouched here.
- Apocalypse #654/#655 stay undrivable on-disc (no writer for ch 0xf4/0xf5 outside the script packs
  [D census]) — unchanged from the doc's live-check note.

## Live checks for the main agent

1. **The lift cab's descent is visible**: after the launch cinema (walk into the launch-room plate at
   ~(2.4, 1.0, −172) facing south), render frames from the lift shaft (e.g.
   `--bond 6.0,-18.0,-156.0, toward the shaft`) during the descent window and confirm the cab mesh
   (hash 0x02000a79 art) moves down to the bay floor (y ≈ −21.2 at (2.48, ·, −159.93)), and that
   guard #634 rides it.
2. **ch 0x50's clear timing**: watch `switch_channels[80]` (0x26fd1d + 80) through the launch — the
   port writes [80,1] at the launch script's stream-21 event and [80,0] +181 ticks later (the lift
   script's stream-7 event) while the cab is still descending (y ≈ −4.3). Confirm the game clears it
   at the same in-script beat.
3. **The door graphic SPs' animation** at door[3] (the corridor graphics at (3.41, 1.14, −141.81) and
   (1.43, 1.14, −141.95)): E the panel #684 and confirm the door graphics visibly animate in the game
   while ch 0x27 holds (the port pins the state machine; the visual is render-side).

## Usage

not visible to me
