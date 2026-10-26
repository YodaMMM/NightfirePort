# level-1b-audit — the 0700001b (Equinox) per-placement behaviour audit

## What was done

- **Census** (`crates/nf-view/tests/census_0700001b.rs --ignored`, added by this task): all **304
  main-map placements** of 0700001b.bin (map 26 `01000152`) by kind — 20 drones 0x0f, 22 statics
  0x19, 1 start 0x24, 2 sky objects 0x2a, 129 zero-param 0x31 dummies, 1 hint 0x33, 1 SpaceLaser
  0x38, 8 SpaceMissiles 0x39, 1 DynamicObject 0x3b, 1 room 0x47, 8 Apocalypse 0x4a, 74
  ScriptPlayers 0xd9, 6 destroyables 0xe1, 2 hurt volumes 0xe4, 7 AND-mux triggers 0xec, 7 pickups
  0xf0, 8 spawners 0xf1, 1 AI volume 0xf5, 5 sound triggers 0xf9 — plus all **79 scripts**
  (77 in 0700001b.bin + 1 + 1 in the pack bins) decoded op by op, plus the mission records.
- **The audit table** `docs/levels/0700001b-audit.md`: one row per placement/script/record, the
  game's chain [M/D/L] per kind (re-verified this session against work/decomp/ACTION.ELF's
  `parsemap_create_dynamic_objects` 0x1d04f0 jump table @0x304b20, the param_7 create gate passed
  only by ResetMap_Load 0x1d6360:146, and the `control_funcs` table @0x26f890 — 85 rows keyed by
  obj+0xff, consumed by control_movement_object_handler 0x1336f8:62-63 / Collide_Update 0x1ea398 /
  control_delete_object 0x133178:16-17), the port's per-kind coverage with file:line, and a GAP
  column.
- **Doc drift fixed** in docs/levels/0700001b.md §7: the two 0x2a placements are the SKY objects
  (`View_AddSkyObj` 0x1e79a0 — SKY_StarField/SKY_Earth), NOT starts and NOT the SpaceMissiles (the
  missiles are kind **0x39**, 8 placements); hurt volumes 0xE4 and the Apocalypse 0x4a state
  machine ARE ported; the SpaceLaser 0x38's creator is ported (the update is not); 74 SPs on the
  main map (not 76); the stale missile-tick citation (drone.rs:15493-15501, not :10489).

## How it was verified

- Every kind→creator case was re-read in the decompile this session (0x2a → `View_AddSkyObj`
  0x1e79a0 @0x1d0ce0, 0x38 → `Create_SpaceLaser` 0x1aa8b8 @0x1d06fc, 0x39 →
  `Create_SpaceMissile` 0x1a9ef0 @0x1d0718; 0x19/0x47 route to the jump-table default — static
  path only). The SpaceMissile state machine was re-derived from the disassembly (Ghidra dropped
  its jump table @0x2fcc30): state 3 writes `switch_channels[p6]` (the per-missile 21..28) with
  f6=0 and `switch_channels[p5]` (the fail 199) with f6=1 — matching the disc's 8 p5 = 0xc7 and
  p6 = 0x15..0x1c [D census].
- The kind-level verdicts: **16 of 19 kinds have no gap** (0x0f/0x19/0x24/0x31/0x33/0x39/0x47/
  0xd9/0xe1/0xe4/0xec/0xf0/0xf1/0xf5/0xf9 + the 0x3b sim side). The gaps:
  - **0x38 SpaceLaser — LARGE (report §below):** `SpaceLaser_Update` 0x1aa3f8 is unported
    (space_laser.rs:18-20 states it); the laser never arms on ch 19, never fires the 10 s beam
    (Bullet_init + the beam object + Emitter + sound 0x46a), never writes ch 99, never disables on
    ch 14 — and the hurt volume BigLaserBeamHURT (0xe4 #259, gate ch 18) can never arm.
  - **0x2a sky — DRAW/SIM:** the sim's RotatingSky/Sky::tick_group (sky.rs:193-352) has no
    production caller; both placements carry rotation speed param[2] = 0 [D census], so the
    placement-faithful state matches today — but the G-019 rotation the base doc claims ([S])
    is unproven on both sides.
  - **0x39 missiles — the HUD pane:** `HUD_UpdateSpacePane` 0x19d3b8 (the eight MissileDeploy
    lamps, Equinox-only @0x19d3b8:12-21) is unported; missile_deploy is maintained
    (space_missile.rs:164/198/203) but read only by tests.
  - **0x4a Apocalypse — the whiteout:** the tick computes `fade` (world_fx.rs:769) with no
    renderer consumer; the rumble is a documented no-op (world_fx.rs:359-362). The countdown
    state machine itself is ported.
  - **0x3b DynamicObject — FIXED this session** (below).
- **The fix (3370abef):** the 0x3b gated dynamic object now rides a posed Character-layer slot
  (one per 0x3b placement, `WorldObjects::create` order, after the sensors in `lock_slot`'s slot
  chain) that **collapses once the sim deletes the object** — `DynamicObject_Update` 0x18d3b8:26
  (mode 0, armed B) sets obj+0xFE bit 0 and the port's delete pass retains !dead; on 0700001b the
  control-room glass (p1 = ch 10, Drake's gate [D PL#0]) vanished from the SIM but kept rendering
  as a build-time static instance. Test `dynobj_glass_live`: the slot poses at the placement
  matrix with vertices, ch 10 fires → the sim's list empties → the palette slot collapses
  (y < −9000). The scene-equality pins for the four other 0x3b-carrying levels were re-pinned
  (3adea8de, the pins' own [S port engineering] convention, reason in the comment).
- **Suite** (sandbox: background jobs die between tool calls — the nohup suite.py died in 8 s
  again, verified — and the 9.9 GB rootfs cannot hold the whole 818-binary build at once, so the
  per-binary runner shape ran through `scripts/wave_suite.py` in waves, each binary
  `--test-threads=1` under NF_SILENT=1 with a wall-clock limit, results in a resumable state
  file):
  - **slots wave — the scene.rs change's blast radius: 28/28 binaries, 84 passed / 0 failed**
    (every test that reads skin_palette/char_verts/door_base, plus the new dynobj_glass_live),
    after the scene-equality re-pin.
  - **game wave: 89/89 nf-game binaries — 344 passed / 0 failed / 3 ignored** (the sim side the
    audit crosses; no nf-game change this session).
  - Touched-crate direct runs: `cargo test --offline --release -p nf-view --test dynobj_glass_live
    -- --test-threads=1` → 1/1; `--test census_0700001b -- --ignored` → the census dumps (both
    green, assertion-free); `--test load_scene_equality` → 1/1 after the re-pin.
  - Not run: the remaining ~670 nf-view test files and nf-formats/nf-audio (no nf-formats/
    nf-audio change this session; the wave runner's state file resumes them: `python3
    scripts/wave_suite.py scripts/wave_state_1b.json <wave>`). The image note: this sandbox's
    slim ISO was deleted for disk last session; NF_ISO is the injected byte-superset image (only
    the 20 movie records differ — every disc-content test on this level ran green on it).

## Large gaps → their own tasks

1. **SpaceLaser_Update 0x1aa3f8 (kind 0x38).** What: the arm/fire/damage state machine — states
   0–4, the 10 s re-fire timer (data+0x38), the charge script+sound 0x5de, the player raycast +
   `Bullet_init` (weapon def @0x2c6154) + the beam object (control idx 0x10) + `Emitter_Create` +
   3D sound 0x46a, the ch-99 (data+0x40) state-3 write, the ch-14 disable → obj+0xfe kill. Where:
   the creator is ported (nf-game/src/space_laser.rs:54-68, stored drone.rs:12637-12642); the
   update is not (space_laser.rs:18-20 admits it; nothing ticks `space_lasers`). The [M] chain:
   dispatcher case 0x38 @0x1d06fc → control_funcs row 0x50 → `SpaceLaser_Update` 0x1aa3f8. The
   gameplay stake: the laser never threatens the player and BigLaserBeamHURT (0xe4 #259, gate
   ch 18) can never arm; the disable-by-Drake's-death (ch 14) never runs.
2. **HUD_UpdateSpacePane 0x19d3b8 (the missile lamps).** What: the Equinox-only HUD pane pairing
   MissileDeploy[i] (@0x30cd70, values 1/2/3) with switch_channels[i+1]: blinking missile icon
   (sprite 0x3000177) in flight, red X (0x300017a) once resolved, alpha 0x80/0xff by the resolve
   path. Where: the sim state exists (space_missile.rs missile_deploy, drone.rs:12558/:15497);
   the HUD pane does not. The [M] chain: 0x19d3b8:12-21 (the GameState==0x700001b gate), the
   sprite/alpha reads :37+.
3. **The Apocalypse whiteout + rumble.** What: Apocalypse_Update 0x18d4f0's viewer+0x200 shake
   (param[3]) and the Player_SetFlashBang whiteout `remaining·382/total` (param[4]) + the
   controller rumble ramp (param[2]). Where: the port computes `fade` (world_fx.rs:769) with no
   renderer consumer; rumble is a documented no-op (world_fx.rs:359-362). The 8 placements'
   latch channels 0x83..0x8a have no disc writer (the same family as the base doc's ch-199 note),
   so this only matters once a writer exists — [S] the critical-countdown script.
4. **The rotating sky (0x2a + G-019).** What: whether the station sky rotates in play and by
   what — the placements' own speed is param[2] = 0 [D census], so any rotation comes from the
   sky module/script the base doc labels G-019 ([S]). Where: the sim's RotatingSky
   (sky.rs:193-319) + Sky::tick_group (:333-352) have no production caller; the sky pass draws
   static (scene.rs:7756-7757). Needs the main agent's render to arbitrate (lesson 6).

## Live checks for the main agent

1. `NF_ISO=<retail image> python tools/test/suite.py <repo> <target> --only dynobj_glass_live,
   load_scene_equality` → both green on the retail image (the pins above are image-shape
   sensitive only through the shared level data, which is identical).
2. PCSX2 fresh boot → Equinox: walk to the control-room glass (the 0x3b, inside the station
   core), trigger the Drake fight chain, and confirm the glass DISAPPEARS the tick the fight
   starts (the ch-10 write). A save-state resume from mid-fight skips the boot path — not a
   valid measurement.
3. PCSX2 fresh boot → Equinox, let Drake's fight start WITHOUT killing him: does the station
   sky visibly rotate (G-019), and does the SpaceLaser (PL#130, the pit's "BigLaserBeam Fire
   Marker") ever fire at Bond in the retail fight? If it fires, record the interval (the 10 s
   timer) — that pins SpaceLaser_Update's arm path for the follow-up task.
4. The missile-lamp HUD: fresh boot → the missile countdown (needs the retail launch chain) —
   do the eight lamps blink in flight and turn to red X on resolve? (Only reachable on the
   retail disc; the disc-data chain is dead-ended at ch 19 on every image.)

## What's left

- The remaining nf-view waves (~670 files) and nf-formats/nf-audio — no change touches them;
  the wave runner's state file resumes.
- The four large gaps above → their own tasks.

## Usage

Not visible to me — this session's environment shows no token, credit, context-window or cost
figures anywhere I can read, so there are no numbers to copy.
