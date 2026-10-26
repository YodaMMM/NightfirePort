# level-14-camera-view — "Press Action for Camera View" works on 07000014

Branch `s/level-14-camera-view`. The security consoles on 07000014 prompt "Press Action for Camera
View" but Action did nothing in the port. This task ports the sim side of the game's camera view:
the enter on Action, the input routing while viewing, the exit, and the per-tick active-view camera
exposure for the renderer.

## The system, disentangled

The task pointed at the ThirdCam path; the decompile says the security console is a different
system. **The console is a Monitor (placement kind 0x30, class 0x38) and the feed is the watched
Sensor's own world matrix. ThirdCam (`ThirdCam_Create` 0x1bc510, `ThirdCam_GetPos` 0x1bc590) is the
creepwall/wire rail camera — consumed only by `camera_tracking` 0x1ba5e0 in cam mode 0xb, never by
the security view.** No `Player_SetCamMode` value is involved in the security view at all.

The disc chain on 07000014's east wing [D `docs/levels/07000014-audit.md`]:

- console `SWITCH_Consoles_Evilbase` #1166 at (32.50, 1.54, −17.80), param 0 = 2 (the monitor
  group) — kind 0x30 [D :539];
- hint plate `Hint Small Plate` #1237 at (32.38, 0.61, −17.75) carrying label 0x2000057 (the
  "Press Action for Camera View" text; the strings live in USATxt.dat, not the ELF) [D :629];
- four `camera(a)-lens` sensors #1035–#1038 at (32.55, 4.36, 15.23), (32.60, 4.36, 10.15),
  (32.55, 4.36, −6.24), (32.60, 4.36, −11.32), each param 6 = 2 (the same monitor group), kind 0x28
  [D :430-433].

The game's chain [M `work/decomp/ACTION.ELF/`]:

1. `Player_WeaponFiring` 0x1a5460 polls the Action button (action 0xE, mask 4 = newly-pressed edge,
   :396) → `Player_Activate(player)` 0x1a5170.
2. `Player_Activate`'s main arm (:126-175) switches on the use-sphere hit's class byte `obj+0xFF`;
   class 0x38 → `Monitor_Activate(monitor, player)` 0x1b0058 [:149-151].
3. `Monitor_Activate` (gate `monitor->0xF4 == 0` :12-13 — the ENTER only): walks the global
   `CameraList` 0x319930 from the HEAD (:17-36), takes the first listed sensor of the monitor's
   group (`+0x5E` == `monitor->0xF6`) that is not destroyed (`obj+0xF4 == 3` skips), hides its body
   (`obj+0xF8` display mask = 0, :26 — you're inside its eye), sets state 2 (:40), and:
   `Camera_SetUpdator(slot, player, 0)` :44 **detaches the player's `Player_PositionCamera`**
   (so the sensor matrix becomes the view's only writer), `Input_SetAction(slot, 0xE, 1)` :45
   (consumes the activating press), `Player_Disable(player, 0)` :46 (0x195f70:16-22: `BL+0x94A = 0`
   control freeze; `obj+0xF0 |= 0x10` hides the body and the weapon model), HUD pane 16 (SecCamPane)
   ON, panes 0/1 OFF (:47-49).
4. `Monitor_Update` 0x1afcc8, state 2, per tick:
   - a watched sensor that died (`+0xF4 == 3`) is unhidden (:75) and the walk re-picks FORWARD from
     its CameraList node (:76-93), falling back to the list head (:94-114); none → exit (:114);
   - the Action edge is polled INSIDE this update (`Input_Action(slot, 0xE, 4)` :117-118): the
     watched camera's display mask restores to 1 (:125), the walk goes FORWARD from its node's next
     — NO wrap (exhausting the list yields 0) — and the next camera's mask clears (:143);
   - `piVar3[1] == 0` → the EXIT arm (:159-168): `Camera_SetUpdator(slot, player, 0x18f5d8)`
     re-attaches `Player_PositionCamera`, `Player_Enable` 0x195498 (unfreezes, zeroes velocities,
     `Input_ClearAllActions`), ext player = 0, state = 9, HUD panes restored;
   - else the view tick (:169-177): `switch_channels[sensor+0x5C] = 1` gated `if (ch != 0)` — the
     decompile's own channel-0 skip — **`Mat_Copy(watched_obj+0x90 → viewer+0x120)`** (the feed IS
     the sensor's swept pose; `Sensor_Update` 0x1af618:174-179 interpolates the framelist into
     `obj+0x90`), and the sensor's `+0x60` being-monitored byte sets;
   - the cooldown drain is a SEPARATE switch arm (:50-66): 9→8→…→5 one per tick, 4→0, then the
     `== 0` gate re-opens. So the exit tick itself leaves state 9; the drain starts the next tick.
5. The sensor's placement order is the CameraList order (`LList_Add` 0x1cca28:13-25 tail-appends at
   creation), so the group's first camera is #1035 [M + D].

## What the port does now

- `crates/nf-game/src/monitor.rs`: `Monitors::activate` is the state-0 enter only (head walk, group
  match, display-mask hide, state 2). `Monitors::tick(…, monitor_edge)` is `Monitor_Update`'s state-2
  body: death re-pick (forward, then head), the press edge (forward cycle, no wrap, exit arm state 9),
  the channel latch with the channel-0 skip, the `+0x60` unhide, and the drain as a separate arm
  (exit tick leaves 9; 9→…→5→4→0). `Monitor_ApplyDamage`'s restore arm unchanged.
- `crates/nf-game/src/sensor.rs`: `Sensor.display_mask` (obj+0xF8, init 0x1F) — the monitor view
  drives it; the draw-side consumer (hide the watched camera's mesh) is pending.
- `crates/nf-game/src/drone.rs`: the use dispatch passes the monitor enter to `Monitors::activate`;
  the viewing press edge is staged on `Drones.monitor_edge` (scene.rs, per tick) and consumed
  inside `tick_inner` for the monitors' tick — the game consumes it inside `Monitor_Update`, not
  `Player_Activate` (behind the freeze `Player_Update` 0x193600:80-87 returns early).
- `crates/nf-view/src/scene.rs`: while a monitor views, movement/aim/fire input is zeroed (the
  `Player_Disable` freeze) and the press edge rides `drones.tick`; the exit zeroes the velocities
  (the `Player_Enable` arm). `Sim::monitor_viewing()` and **`Sim::monitor_camera()`** expose the
  ACTIVE VIEW CAMERA: `(pos, yaw, pitch)` from the watched sensor's matrix — position = the matrix
  translation (obj+0x90 row 3), direction = row 2 (`Mat_GetDir`'s axis; the game reads the same row
  for the use pick at 0x1a5170:101), yaw = `atan2(d.x, d.z)`, pitch = `asin(d.y)`.
- `crates/nf-view/src/lib.rs`: the app camera takes `sim.monitor_camera()` while the view stands
  (before the script-camera pick), so the renderer's active view camera is the watched sensor's
  swept pose every tick.

Port choices marked unverified: the 2.2-unit monitor reach (the door/SS sphere rule — the game's
`Player_Activate` sphere test) and the restore-side bookkeeping the port derives from the state
(the freeze end). Both are `// [S] unverified` comments at the sites; nothing pass/fail hangs on them
beyond the live tests passing.

An independent audit (subagent, read-only) compared the diff against 001b0058/001afcc8 line by line:
no behavioral divergence on reachable single-player paths, merge-safe, every test assertion
citation-checked. Two documented deviations remain, both below the observable bar:
- the freeze is input-level (the port still advances the body with zeroed input; the game's
  `Player_Update` returns before ANY body processing, so a mid-air enter would hang in the game
  but coast to a landing in the port) — the live tests pre-ground Bond, and the audit rates it
  unobservable on the console's reach;
- the yaw/pitch conversion in `monitor_camera()` (atan2/asin of matrix row 2) is the port's
  convention (the same one lib.rs's missile-cam arm uses) — `// [S] unverified` as a convention,
  while the matrix itself is [M 0x1afcc8:175].

The viewing press edge is a staged `Drones.monitor_edge` field consumed (cleared) in
`tick_inner` — `Monitor_Update` polls the edge itself, and keeping the 2-arg `Drones::tick`
signature untouched avoids churning the ~30 test harnesses that call it.

## Tests (all green on this tree)

Unit (`cargo test --offline --release -p nf-game --lib -- monitor:: sensor::` — 9 passed):
- the enter picks the group head and hides it; the viewed tick latches the monitor-view channel
  (with the decompile's channel-0 skip — the test previously pinned a channel-0 write, which
  contradicts `if (uVar2 != 0)` at 0x1afcc8:171; fixed per the rule that the test is wrong);
- the edge cycles forward, no wrap; past the last camera the exit runs and the exit tick leaves
  state 9; the drain 9→8→…→5→4→0; the console re-arms at the head;
- a dead watched camera hands the view forward (or exits); the idle monitor never writes; the shot
  destroy path.

Live through the real input path (`crates/nf-view/tests/monitor_view_07000014_live.rs`, NF_ISO,
3 passed): aim + E pulses through `Walker::tick` at the group-2 console from the task's spot
(33.71, 0.65, −18.11):
- **Action at the console enters the camera view with the game's camera id [M + D]** — state 2,
  watched = the CameraList head = camera(a)-lens #1035 at (32.55, 4.36, 15.23) [D :430]; its display
  mask clears; `monitor_camera()` equals the sensor's matrix translation and row-2 direction;
  the player is frozen (movement + look do nothing) [M 0x195f70 → 0x193600:80-87];
- **the edge cycles forward through #1036/#1037/#1038 and past the last camera the view exits** —
  state 9 after the exiting tick, watched = None, `monitor_camera()` back to None (the player view),
  the last camera's mask restored, the controls answer again [M `Player_Enable` 0x195498], the
  cooldown drains and the console re-arms at the head [M 0x1afcc8:55-65, 0x1b0058:17-36];
- **without Action the console stays idle** — no state, no watched camera, no view.

An intermediate draft routed the viewing edge through a re-routed `Player_Activate` and drained the
cooldown in the exit tick (a live test caught state 8 vs the pinned 9). The decompile settled it:
the edge is polled inside `Monitor_Update` (:117-118) and the drain is a separate switch arm — the
port now mirrors that structure, and the decompile file was re-read line by line to confirm
(work/decomp/ACTION.ELF/001afcc8_Monitor_Update__FP7obj_tag.c).

## Crate tests run before the pushes

- `cargo test --offline --release -p nf-game --lib -- monitor:: sensor::` (9 passed) after every
  code change;
- `cargo test --offline --release -p nf-view --test monitor_view_07000014_live` (3 passed);
- the full fast suite (`python3 tools/test/suite.py`) on the final tree merged with origin/master —
  started 21:25 UTC, results below.

## Full suite (final tree)

The sandbox reaps every detached/background process at tool-call boundaries, so the full ~111-minute
suite cannot complete inside this session's window (the 21:57 UTC background start was killed with
0 bytes of output; a controlled `setsid nohup … & disown` probe confirmed the reaping). The test
binaries were instead run through the same fast runner (`python3 tools/test/suite.py --only …`) in
the foreground on the FINAL tree (merge of origin/master batch-1026j + this task's commits):

| binaries | result |
|---|---|
| `nf_game` (lib — 585 unit tests incl. monitor::/sensor::/mission::) | 580 passed, 0 failed, 5 ignored |
| `nf_view`, `nf_formats`, `nf_audio` (lib) | 208 passed, 0 failed, 0 ignored |
| `monitor_view_07000014_live` (the task's live tests) | 3 passed, 0 failed |
| the 8 binaries that failed the earlier pre-merge baseline suite | 52 passed, 6 failed |

843 passed, 6 failed across the binaries run (each binary green-unless-noted; no TIMEOUT/CRASH).
The 6 failures are OUT OF TASK and pre-existing (triaged on the merged tree):
- `disc_golden` ×3 (bigf header/tables/directory tree) — expected on the slim ISO per the task rules;
  the reads disagree with the golden TSVs around `/DRIVING/MIS01.MUS` (BIGF "too small" + 1499
  manifest + 159 dir-tree diffs) — a disc-layout/BIGF reading issue owned elsewhere;
- `identity_disc::the_real_disc_passes_the_identity_check` — the disc is read fine (NIGHTFIRE SLUS-20579
  NTSC 1.00) but the sector count ≠ docs/build-identity.md's 1,191,072 (env/doc mismatch, not code);
- `fix_0c_pylon_live` ×2 — the 0C pylon view checks; 0C rendering, untouched by this task's diff
  (monitor/sensor/drone-edge/scene-freeze only).
For contrast, the earlier pre-merge baseline (suite.out of 20:08 UTC) had 8 failed binaries; the
master merge fixed 5 of them (level_reload_live, mp_world_sync_live, use_key_census_live,
walk_07000005_live, gate_guard_truck_live now all pass).

## What's left (draw side, a separate task)

- The SecCamPane overlay (7 sprites, `HUD_UpdateSecCamPane` 0x1a1538; the interlace flicker wash
  `0x7f7f7f78`/`0x7f7f7f00` by `GameState+0x34 & 0x1f`) and panes 0/1 disabling during the view.
- The draw-side consumers of the sim state: the watched camera's mesh hide (`display_mask` 0), the
  player body + weapon hide (`obj+0xF0 |= 0x10`, `BL+0x7E8`), the sensor lens-glist draw.
- The viewer anchor bookkeeping (`viewer+0x04 = 0` on enter/cycle/exit) if the draw side needs it.
- The group-1 (west wing) and group-3 consoles work through the same code (verified by the unit
  tests' group parametrization); a live walk of the west wing would be a nice extra.

## Renderer observation spots (for the main agent's render)

- `--bond 33.71,0.65,-18.11,<yaw toward the console>` (aim at (32.50, 1.54, −17.80)): the console
  mesh in front; the "Press Action for Camera View" plate is the draw side's pending overlay.
- Press Action (in-game): the view jumps to camera(a)-lens #1035's eye at (32.55, 4.36, 15.23),
  slowly sweeping the east-wing corridor northward per the sensor's framelist; Bond is not visible.
- Each further Action press: the feed steps to (32.60, 4.36, 10.15) → (32.55, 4.36, −6.24) →
  (32.60, 4.36, −11.32) — always the next camera southward, never back.
- The press past the last camera: back to Bond's own view at the console, controls live again.

## Live checks for the main agent

1. **The enter pick** — fresh boot 07000014, walk to console #1166, press Action:
   breakpoint/monitor `Monitor_Activate` 0x1b0058; expect `monitor->0xF4 (obj+0xF4) == 2` after, and
   the picked sensor's `obj+0xF8 == 0`; the picked object should be the group-2 head — the sensor
   whose placement is #1035 (watch which camera the on-screen feed shows: the north-most, (32.55,
   4.36, 15.23)).
2. **The view matrix** — while viewing, watch `glb_viewer[slot]+0x120` each tick (gdbwatch) and the
   watched sensor's `obj+0x90`: expect equality every tick (`Mat_Copy` 0x1afcc8:175).
3. **The cycle order** — press Action 4 more times: expect the feed to step #1035→#1036→#1037→#1038
   and the 5th press to exit (`obj+0xF4 == 9` on the exiting tick, then 8 on the next) with the HUD
   ammo/health panes back (`HUD_Enable` pane 0x10 off).
4. **The freeze** — while viewing, push the stick: expect no movement and no camera change (the
   `Player_Update` early-return behind `BL+0x94A == 0`).
5. **The label** — confirm USATxt.dat label 0x2000057 reads "Press Action for Camera View" (the
   prompt's disc text; the port's draw side will bind it via the Hint plate #1237).

## Usage

My own session's token/context/cost figures are not visible to me (the environment shows me no
token, credit or context-window counter for this conversation). Figures the environment DID show,
copied exactly from the subagent results' footer ("Tokens ->" lines):
- independent-audit subagent: prompt 112335, completion 4942, total 117277;
- sandbox-availability probe: prompt 7608, completion 437, total 8045;
- suite monitor probe: prompt 8158, completion 882, total 9040;
- suite restart probe: prompt 18185, completion 1176, total 19361;
- targeted-binary suite runs: prompt 36466, completion 1175, total 37641;
- remaining-crate-lib suite run: prompt 9897, completion 377, total 10274.
No cost figures were shown anywhere.

## ntfy failed

The required ntfy notification could not be delivered: ntfy.sh returns HTTP 429 (the topic's daily
message quota is exhausted) on all three fallbacks, tried in order at 22:28 UTC 2026-10-26:

- `curl -sS -m 20 -H "Title: NightfirePort" -d "level-14-camera-view: done — the 07000014 security
  console enters the game's camera view on Action; report at docs/reports/level-14-camera-view.md"
  https://ntfy.sh/nightfire-yoda-8k3x` — exit 0 but the response body is the API error:
  `{"code":42908,"http":429,"error":"limit reached: daily message quota reached; increase your
  limits with a paid plan, see https://ntfy.sh","link":"https://ntfy.sh/docs/publish/#limitations"}`
- `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x',
  data=b'level-14-camera-view: done', headers={'Title':'NightfirePort'}), timeout=20)"` —
  `urllib.error.HTTPError: HTTP Error 429: Too Many Requests`
- `wget -q -O- --post-data "level-14-camera-view: done" https://ntfy.sh/nightfire-yoda-8k3x` —
  exit code 8 (server error response), no output.

The one-line result the notification would have carried: "level-14-camera-view: done — Action at
the 07000014 security console enters the game's camera view (Monitor/Sensor chain, the feed is the
watched sensor's matrix; ThirdCam is the climb-rail camera, not this view); the edge cycles forward,
the exit restores the player view, the active view camera is exposed per tick for the renderer;
targeted suites on the final tree: 843 passed / 6 out-of-task failed (3 disc_golden expected on the
slim ISO)."
