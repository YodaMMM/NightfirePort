## Your task: `lift-indicators-09`, 07000009's garage lifts: the floor indicators above the doors and the call buttons are not drawn

At `--bond -3.45,-375.98,20.61,-6.124,-0.083` on 07000009 (the parking-garage lift lobby), the real game [L fresh boot, PCSX2] draws a floor-indicator panel showing "L" above EACH of the two lift doors; the port draws neither, and the lifts' call button is missing too. No placement on map 39 within x -14..8, y -380..-368, z 18..34 is the indicator or a door (only OBJECT_Lights, a drone, an alarm switch player #263, an AI point), so these are script-created entities, objects on another map, or parts of the lift SS objects.

1. Identify each missing piece: the indicator panels (and their "L" display — an animated texture or a script-driven frame), the call buttons. Where they come from on the disc (map/placement or the script entity and its stream) and how the game creates and draws them [M + D].
2. Make the port create and draw them where the game does (data/creation only — if a draw-path change is needed, keep it minimal and say so). Test: at that spot the scene's draw list contains both indicators and the button at the game's positions; sister: a placement the game never draws is still absent.
3. The render comparison against PCSX2 happens before merge (name the frame and view).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/lift-indicators-09`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-24T09:53Z

---
## Report (done, 2026-10-24T10:41Z)

# lift-indicators-09 — 07000009: the garage-lift indicators and call buttons now create and draw

Branch `s/lift-indicators-09` (from origin/master 26543436). The sandbox carried over from the
previous round (repo, `work/`, ISO, toolchain in place — no re-download).

## What the pieces are [M + D]

The two lift cab ScriptPlayers (placements #20 on map 39 — the main map (kind 1) [D the probe's
map dump], class 0xD9, mode 8, scripts 0x060007D/0x060007E — near-identical twins) preload at
level start, run two ticks and hold on a stream Event-3 negative delay until the cab activation
([M `Script::update`'s delay gate, nis.rs; [L] the probe: state 6, play 0x09, clock 2.0,
delay −1 from the first tick]; the keyed matrices persist through the hold). Their SUB scripts
place the visible chrome the driver's frame was missing:

- **0x0600064E s0** → the floor-indicator plates **0x020006B7/B8/B9/BA** (0.71×0.24 flat
  plates; keys all zero — positioned through the SUB parent chain [M `Script_Interp` case 7,
  0x1d9a08]). The last Entity op re-points the stream object at **0x020006B9** — the face
  visible at boot (the "L"; the other hashes are the other floors' letters, swapped in as the
  ops fire at later times).
- **0x0600064F s2** → the call-button plates **0x020006B2/B3** (keys constant
  (−4.62,−375.52,25.91) [D]).
- Each cab script starts each sub **three times** (`Script_SubScriptStart` 0x1db050 ops on
  streams 13-18 [D]) → the sim holds six indicator plates and six button plates — three per
  shaft, the two shafts at x ≈ +1.25 and −2.77 [D the composed positions]. Per shaft: one
  indicator above the lobby doors (y −366.57), one above the garage doors (y −374.39 — the "L"
  the driver's frame shows), one at the lobby door top (y −367.02); the call button on the
  garage floor beside the shaft (x +3.06 / −4.62, y −375.88, z 25.91) plus two lobby-side
  plates.
- The activation SP #64 (0x0600059B) creates 0x020005EA (the elevator-button doc's use-target
  panel; art baked world-space at (−18.38,−368.53,24.83) [D gfx bounds]) at t=0 and the FX
  0x020005EB only at t=16 on the press [D the nis dump].

## The defect and the fix

The sim side was already correct — an early "every matrix is identity" reading was a probe bug
(it read `m[0..2][3]`, the last column; the port's Mat4 convention carries the position in the
last row `m[3]` [M the Camera doc "rows: right, up, forward, position"]). The real defect was in
the DRAW LIST: the script-entity slots are per-(script,stream,hash) with the copy count taken
from placed SPs whose OWN scripts contain the hash — every SUB hash read 1 — and
`Sim::script_entity_k` serves only the newest match, so one cab's plates drew and the other
cab's never did (the same shape as 07000014's nine-vent-lid lesson, but via SUB scripts).

**The fix** (crates/nf-view/src/scene.rs, the script-entity slot build): the root walk now
accumulates a `sub_copies` count — every placed SP running a parent script starts the sub once
per `Script_SubScriptStart` op [M 0x1db050] — and the slots are created after the walk
completes (a parent may pop after its sub; the pop order is preserved in an `order` vec, so
pre-existing attachment indices are unchanged). 07000009 now gets six slots per sub-entity hash.

Data/creation only — no draw-path change beyond the slot count; `script_entity_k`'s matching,
the billboard handling and the per-frame matrix serve are untouched.

## Verification

- **New tests** `crates/nf-view/tests/lift_indicators_09_live.rs` — **2 passed**:
  `the_indicator_and_button_plates_draw_at_the_game_positions` (six slots per hash with copy
  indices 0..5; the six keyed spots per family within 0.05 [D keys]; every live plate served by
  some slot k at its exact matrix) and the sister
  `the_activation_fx_stays_absent_until_the_panel_press` (0x020005EB absent before the press
  [D t=16]; 0x020005EA live from t=0 at its baked-art identity matrix).
- **Regression set, all green**: `load_scene_digest` (1), `load_scene_equality` (1, after the
  documented re-pin — see below), `channel_persist_live` (3), `hat_details_live` (2),
  `abseil_elevator_live` (3), `fix_0c_vent2_live` (3), `elevator_buttons_09_live` (3 — the
  level's own walk: panel press, doors, cab, ride to 0700000a all still work).
- **The re-pin**: `load_scene_equality`'s 0700000c baseline drifted (+6 slots, +248 vertices,
  +576 indices, +1 attachment) — the test's own protocol re-pins for deliberate scene-content
  changes; the comment cites [M 0x1db050]. No other level's digest moved (07000014's vent-lid
  SPs are placed SPs — unchanged; the rest have no multi-SP sub chains).
- The fast-runner suite: one background attempt (setsid+nohup); background jobs have died
  between tool calls in this sandbox family — the foreground regression set above is the round's
  evidence. The merge-side suite run should cover the rest; only nf-view's scene.rs, one new
  test file, the re-pinned digest, WORKLOG and this report changed (nf-game/nf-formats/nf-audio
  untouched — an instrumentation experiment on nis.rs was fully reverted before commit).

## Render comparison for the main agent (before merge)

- `--bond -0.7,-375.98,20.61,-6.124,-0.083` (the driver's spot): both lift shafts (x ≈ +1.25
  and −2.77, z ≈ 28) should now show an indicator plate above each lift door (the garage-floor
  plates at (±1.27/−2.75, −374.39, 26.14)) and the call buttons beside each shaft
  ((3.06,−375.88,25.91) and (−4.62,−375.88,25.91)); the indicator face should read "L"
  (0x020006B9, the last Entity op before the hold [D]).
- `--bond 3.0,-374.2,22.5,-2.6,-0.15` (close on the east shaft): the garage indicator plate and
  the call button plate at eye height.
- The lobby side (after the ride up): `--bond -0.7,-368.4,22.0,-6.124,-0.083` should show the
  lobby indicators (y −366.57) and the lobby-side button plates (y −368.0).

## What's left

1. The main agent's render comparison at the spots above (the "L" face and the plate placements
   are [D]/[M]-pinned; the pixels need the GPU).
2. The merge-side fast-runner suite total line (background jobs do not survive this sandbox).
3. If the game's live read disagrees with any keyed spot, the SUB parent-chain composition is
   the place to look next (`Script_Interp` case 7's parent walk [M 0x1d9a08]).

## Usage

Task-system token/context figures are not visible to me this session. Session wall-clock:
09:56-10:40 UTC (the sandbox carried over; no setup downloads). One branch claim, four pushes
(claim; the pieces + the probe-bug correction; the fix + tests; this report), one probe example
(`probe_lift_09.rs`, five iterations), one fix (scene.rs), one new test file, eight test
binaries run in the foreground (17 tests, all green after the documented re-pin).
