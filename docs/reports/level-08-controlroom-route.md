# level-08-controlroom-route — the control-room glass, walked in and shot

## What was done

`interactables_07000008_live::glass_pane_breaks_on_the_first_bullet` is un-ignored and green: Bond
now walks into the 07000008 lower-station control room on real input and breaks a control-room pane
with the first aimed bullet.

1. **The game's route into the control room (found, walked).** The route does NOT pass through the
   start-#8 spot at all: from the y-33.5 hall the west walkway (x 110-114, y 33.7 [D floor scan of
   the level collision]) runs south past the stairwell to the corridor at z −118, where the free
   control-room door #145 Door_Station_9 (107.75, 33.5, −119.37; p3 = 0 = never locked [D params;
   audit doors table]; the lock law is `Door_Create` 0x136100's p3 [M door.rs ch_lock]) opens on E
   into the room. The panes #170/#176/#177 (kind 0x20, params (2,0,0) [D]) resolve to live breaks
   over the y-33.5 floor: the west window (x 105.25 plane, z −122..−120.1), the south row (z −122,
   x 105.25..109.25) and the east window (x 109.25), all y 34.25..36.37.
2. **The start-#8 wedge resolved: real disc geometry, not a port bug — routed around.** The wedge
   ("the walk input moves nothing from (118.7, 34.6, −113.2)", the level doc §"The kill floor") is
   the start pocket's south edge: the y-33.5 floor stops at z ≈ −113.3 and the pocket's south wall
   (z −114.5 plane, x 117..120.25, y 34.25..37.88 [D tri dump of `CollisionWorld::tris_near`])
   stands over a floorless recess (no floor tri anywhere between the edge and the wall in the same
   dump). A southbound walk off the edge is caught in the recess (measured: the body lifts to
   y 35.09, slides east, `grounded=false`). No door-state bug either — every door on the stretch is
   free (p3 = 0 [D]). Start #8 itself is fine: the walk NORTH into the hall leaves normally
   (104 ticks in the probe). Since the game's own route is west and the disc walls the pocket off,
   the test routes around; the wedge text in `docs/levels/07000008.md` now records the resolution
   (§"The start-#8 wedge and the control-room route").
3. **The first-bullet pin.** The rewritten test spawns Bond at the disc's start-#8 placement
   (118.66, 35.0, −113.22 [D #8; both 08 starts are gate 0, so the port's `Player_Start` 0x190fd0
   pick takes #0 — the test spawns at the #8 spot, the game's own start spot, not an arbitrary
   teleport]), walks every leg on real input (north to the hall, the hall west, the walkway head,
   the walkway south ×2, the corridor west), asserts the three control-room windows stand, E-opens
   door #145 and asserts the door leaves its closed key (pos01 0.5 → swings [M door.rs pos01;
   Door_Activate 0x135be0]), walks into the room, aligns the aim on the west pane (turn/look only,
   no fire; asserts the aim settled), then fires exactly ONE shot — the trigger is held until the
   clip decrements once — and the pane dies from that bullet (health 0 dies on the first hit
   [D #170/#176/#177 param 0 only; M Break_Update 0x12bfd8 → Break_Kill 0x12b970]). Sister: the
   other control-room windows are still standing after the shot; sister before: the three windows
   stand pre-walk.

## How it was verified

- `interactables_07000008_live`: **14 passed, 0 failed, 1 ignored** (the remaining ignore is
  `the_kill_floor_ends_the_fall` — the death-NIS hold, a different task's blocker, untouched).
- `walkreal_07000008_live`: **3 passed, 0 failed** (the whole-mission walk is unaffected by the
  route work; no game code changed).
- Full suite (resumable chunked runner, `tools/test/suite.py`'s exact per-binary law with
  `--test-threads=1`, because this sandbox kills background jobs between tool calls): reached
  **446 of 1074 test binaries — 1435 passed, 0 failed, 129 ignored** before the window closed; the
  one non-pass is `mp_stress_explosives_live`, killed at the 560 s per-binary wall-clock limit
  (suite.py's own limit is 600 s — it is a long MP stress binary, the tail the previous sessions'
  runs also never reached). The branch diff touches one test file and one doc — no game code — so
  it cannot be this branch's regression; the follow-up is a solo 600 s run of that binary.
- The probe runs behind the findings (floor grid, tri dumps, the wedge repro, the route scout) were
  scratch tests, deleted before the commit; the numbers above were read from their output.

## What's left

- `mp_stress_explosives_live`: confirm it passes under suite.py's 600 s limit (or is a real hang) —
  needs a solo run with the full window.
- `the_kill_floor_ends_the_fall` stays ignored for the death-NIS terminal event (its own task).

## Live checks for the main agent

- 07000008, `--bond 107.2,34.7,-121.0` facing west at the control-room west window: the pane stands
  after the walk in (draw parity for the interior room + windows; the port renders the panes while
  alive and the shatter on death).
- The start-#8 spot `--bond 118.7,34.6,-113.2`: the view north should show the hall; walking south
  in the real game should stop at the pocket's south lip (the wedge's disc geometry), not fall.

## Usage

not visible to me (no token, credit or context-window figures are exposed in this environment).

## ntfy failed

All three notification attempts failed:
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-08-controlroom-route: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 urllib.request.urlopen(...)` → `OSError: [Errno 101] Network is unreachable`
   (socket.connect to ntfy.sh), wrapped in `urllib.error.URLError`.
3. `wget -q -O- --post-data "…" https://ntfy.sh/nightfire-yoda-8k3x` → timed out after 60 s
   (killed; no output).
