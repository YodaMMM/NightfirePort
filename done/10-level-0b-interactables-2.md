## Your task: `level-0b-interactables-2`, 0b: every player-touchable kind behaves like the game

`interactables_0700000b_live.rs` has 9 tests. Extend the existing file the same way (one live test per KIND of player-touchable placement — doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, SimpleScripts the player triggers), run the census of 0700000b [D every placement kind], and for every kind a player can touch, add the live test that drives it through the real path (walk/E/shoot) and asserts the game's result [M the class's Update/Activate; D its params], with a sister that fails on the wrong result. Fix any port gap the tests expose, with evidence. Run interactables_0700000b_live, walkreal_0700000b_live and nf-game; quote the totals.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-interactables-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-01T23:36Z

---
## Report (done, 2026-11-02T01:03Z)


---
## Report from the worker (done, 2026-11-02T01:03Z)

# level-0b-interactables-2 — every player-touchable kind on 0700000b behaves like the game

Branch `s/level-0b-interactables-2` (from `origin/master` 6042ee53). The task: extend
`crates/nf-view/tests/interactables_0700000b_live.rs` (9 tests) to one live test per KIND of
player-touchable placement on 0700000b, each driven through the real path (walk / E / shoot /
walk-over) with a sister that fails on the wrong result, after a fresh census of the level's
placement kinds.

## What's done

**1. The census, re-run fresh [D].** `nf-extract placements-all Nightfire-onfoot.iso
0700000b.bin` → 380 placements. Kind histogram (hex): 0x19×169 (static map props, no
dispatcher case), 0xeb×30 (Touch), 0x31×25, 0x47×23 (rooms), 0xea×17 (TouchOnce), 0x32×17
(RainBox, presentation [M 0x1d04f0:188-193]), 0xf9×12 (**Sound Trigger**), 0xe6×11 (cover
nodes), 0xd9×10 (ScriptPlayer), 0x0f×10 (guards), 0xdb×9 (doors), 0xf0×5 (pickups), 0x27×5
(GT turrets), 0x21×5 (RIPPLE_GEN, presentation), 0xef×4 / 0xec×4 / 0xed×3 / 0xee×2
(MultiplexOr/In/SIn/Out muxes [M 0x1d04f0:427-449]), 0x2a×3 (sky objects, presentation), 0xfb×2
(MusicTrigger), 0xf1×2 (AI spawners), 0x28×2 (**Sensor = the security cameras** [M
0x1d04f0:120-125 case 0x28 → Sensor_Create]), 0x2d×2 (PlayerStart), 0xe2×1 (SS), 0xdf×1
(rain_gen), 0x35×1 (PCQWorm), 0x33×1 (**Hint** [M 0x1d04f0:126-131 cases 0x29/0x33 →
Hint_Create]), 0x2f×1 (Copter), 0x2e×1 (Lock), 0x29×1 (Switch), 0x24×1 (Player).

Census notes against `docs/levels/0700000b.md`:
- The doc's type table (line 171-187) omits three player-touchable families that this census
  shows: **0xF9 Sound Trigger ×12**, **0x28 Sensor ×2**, **0x33 Hint ×1**. All three now have
  kind tests (below). The doc's line 189 "no … fuseboxes …" stays true in the param-bearing
  sense: kind 0x2b/0x31 → `FuseBox_Create` [M 0x1d04f0:132-138], but 0b's 25 kind-0x31 rows
  carry **no params at all** [D the placements dump] — they are the level's device-art objects
  (the terminal parts, the lift cab mesh, the keycard-lock art, the camera beam), whose
  E/shoot behaviors bind through params the disc omits here; the port's `FuseBox::create`
  skips script-less 0x31s (the documented class split), and their collision surfaces are
  already pinned by walkreal_0700000b_live's sealed-cage test.
- Breakables (0xd8-class), monitors, rigid bodies: none on 0b [D the same dump].

**2. Five new kind tests in `interactables_0700000b_live.rs` (9 → 14 tests, all green):**

- `qworm_console_installs_on_the_gadget_shot` — the 0x35 PCQWorm console: the real gadget cycle
  (`Player_WeaponChange` 0x1a6f80 mode 1, the d-pad's own path) draws the Q-Worm 0x58, then the
  real trigger pull installs and writes ch 0x4C [M PCQWorm_Update 0x1aee20; D #297 param 1].
  Sisters: ch 0x4C off + the PC in state 0 before the shot. Observation kept in the test: the
  shot lands from the low aim band (pc+0.1); the high band (pc+0.55) that walkreal's route
  uses missed from the teleported stance (the desk's screen collision truncates the dart) —
  walkreal_0700000b_live's own objective-7 shot still passes on this master.
- `sound_triggers_play_on_their_channels_real_writes` — the 0xF9 kind: `SoundTrigger_Update`
  0x1b2790 starts a trigger's sound on its ON channel [M sound.rs]. Driven by the same real
  walks the plate kinds pin: the walked-over terminal plate #163 (out ch 0x4D [D]) starts sound
  1158 (#330 [D]). Sisters: neither id plays on a fresh load. The kind's single-global-voice
  law [M 0x1b2790:19-28] is documented as a live check, not asserted (the headless port's
  voices end only on a stop, so the 1158/1159 conflict's disc outcome depends on the PS2
  driver's sample-length answer).
- `exec_camera_sight_fires_its_out_channel` — the 0x28 Sensor camera kind: 31 consecutive
  in-cone ticks alert, 2 s of sight writes the OUT channel [M 0x1af618:154-157]. Camera #172
  (out ch 39 [D]) fires on the real sight path (the stance enumerates the sweep's cone);
  sisters: both channels off at load and camera #164's ch 40 stays silent while #172 fires.
- `lift_shaft_plates_fire_on_the_touch` — the lift kind's channel plates: #245
  lift_stop_trigger (0xEB, out ch 0x75) and #171 mid-shaft (0xEA, out ch 0x53) [D], the Touch
  laws [M triggers.rs], with off-before sisters. The cab ride itself is
  walkreal_0700000b_live's pin (the documented no-teleport ride); whether a deck-walking
  player alone touches #245 stays the level doc's open live check.
- `the_message_hint_carries_the_disc_params` — the 0x33 Hint kind's parse-model pin (the live
  firing path — Hint_Update 0x1b3380's one-shot TXTMSG — is not in the sim; the same port gap
  interactables_07000013_live documents). Params [D #354]: label 0x02000021, sound 0, stage 3,
  no on-channel, off-channel 79 = the objectives' AND ch 0x4F [D #299]. Sister: the stage gate
  refuses stage 3 on difficulty 4 [M 0x1b3248:12-25].

No port code changes were needed: every law the new tests probe already held — the tests
expose no port gap to fix.

**3. Coverage statement.** With the census above, every player-touchable kind on 0700000b now
has a live test in `interactables_0700000b_live.rs` (switch 0x29, lock 0x2E, PCQWorm 0x35,
copter 0x2F, ScriptPlayer 0xD9 via the vent's mode-64 pair, doors 0xDB, SS 0xE2, TouchOnce
0xEA, Touch 0xEB, pickups 0xF0, MusicTrigger 0xFB, SoundTrigger 0xF9, Sensor 0x28, the lift's
shaft plates, Hint 0x33 as the parse-model pin) — except: guards 0x0F and turrets 0x27 (actors,
pinned by the drone/GT tests the census row cites), the muxes 0xEC/ED/EE/EF (channel logic,
exercised by walkreal's objective chains), and the presentation/AI kinds (sky, rain, ripples,
spawners, rooms, static props) which no player input reaches.

## How it was verified

- `interactables_0700000b_live` **14 passed, 0 failed** (20.02 s, --test-threads=1).
- `walkreal_0700000b_live` **4 passed, 0 failed** (18.52 s) — the end-to-end route still green
  on the same tree.
- `nf-game` **1028 passed, 0 failed, 9 ignored** (`cargo test --offline --release -j1 -p
  nf-game -- --test-threads=1`, summed over its binaries).
- The full suite (`tools/test/suite.py`) could NOT complete in this sandbox window: the build
  phase hit the known disk-full linker failure twice (100% disk, `cc`/LLVM link crash in
  probe_drone4 / guards13_warm_probe / walkreal_probe_fb9 / fix_grenade_release_live —
  environmental, not test failures); two strip+prune passes recovered ~1 GB each but the
  -j4 build of all four crates' test binaries was still linking at the deadline. The next
  message can run `NF_ISO=<abs> python3 tools/test/suite.py` on this branch to completion.

## What's left

- The full-suite run + its total line on the final tree (above).
- Optionally: teach `docs/levels/0700000b.md`'s type table the three families the census added
  (0xF9/0x28/0x33) — left to the driver's merge since the doc is shared.

## Live checks for the main agent

1. **The sound triggers' global voice on the disc** — does sound 1159 (Sound Trigger #331, ON
   ch 0x0D) audible-play when Bond crosses the dome fence AFTER the terminal visit? Address:
   the SoundTrigger objects' handles (the game's `SFXIsSFXPlaying` view); expected: 1158 (#330,
   ON ch 0x4D) may hold the single global voice [M 0x1b2790:19-28] — a fresh-boot playthrough
   walk (terminal room → fence) answers whether 1158's sample ends and frees it.
2. **The deck walk vs lift_stop_trigger #245** — does a player who walks from the roof deck
   key [−14.71, 58.00, −2.06] north into #245's volume (z −1.88..2.15) raise ch 0x75 on the
   disc? The level doc's open question; the port touches a point 0.4 inside [L probe].
3. **Render spots** (visual, needs the main agent's render): `--bond -19.0,47.2,30.8,2.2,0.1`
   (the Q-Worm console with its screens lit), `--bond 9.0,46.8,11.0,1.57,0.3` (camera #172's
   sweep on the exec floor), `--bond -13.7,47.2,6.5,3.14,0` (the hint marker's room).

## WORKLOG

`docs/WORKLOG.md` does not exist on master (dropped as a process file in batch-1101j); this
report is the `level-0b-interactables-2` entry.

## Usage

not visible to me — this environment shows no token, credit or context-window meter
(checked the session UI and the tool output; no figures are exposed to copy).
