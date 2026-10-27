# Report: level-4a-ride-hand-props

Task: In Phoenix Fire Part 3's (0700004a) lift-ride landing cut-scene (pack 0x060006cf, SP #289) the
guards' hand props (their weapons) are missing. Find how the game attaches props to script-animated
actors and port it so each prop draws in its actor's hand for the whole scene.

## What's done

**The scene's frame story, on the disc's own data** ([D 0780004a.bin 0x060006cf], decoded through the
port's own `Nis::parse` — the dump fed `ride_cutscene_objects_live.rs`):

- The pack keys its WHOLE scene in one car-local frame centred near the origin: the camera (s0) holds
  t0-t230 at (2.89,-7.83..-8.66,-4.58) — level with the hatch entity (s10 keys (3.05,-7.99,-3.08)) —
  cuts up the shaft t326-t475, returns below t500-t572; the pack's own lift car (s13) keys static at
  (0,0.99,0); the guard's body sequence 0x040005fd roots at (2.35,0.85,-0.64) — ON the pack's lift;
  the charge (s7 0x02000758 "OBJECT_DemoCharge", 461 keys) rides just above that deck. Every pack
  stream's `parent` is None — the props attach through their keyed paths and the anim object's
  weapon-datum path, not stream parents.
- The guard's held weapon: stream s4 t0 fires event 0xB param 0x18 onto the previous stream's anim
  (s3, skin 0x0500004) [M `Script_EventHandler` 0x1d94c0:152-158]. `AnimObjectDraw`
  0x1c4490:256-265 then calls `AnimDatumGetWeaponInfo` 0x1c7900 (`weapon_data[w].+0x80` mesh on
  datum `weapon_data[w].+0x50`) and `AnimDatumSetEntity` 0x1c7848, which arms the skin's
  extra-attachment slot (skin+0x20 table, 9-word rows: id, bone, vec, quat) — the draw loop at
  0x1c4490:328-360 builds `Quat_QuatTransToMat(row quat, row vec)` and concats the bone's world
  palette entry: the prop follows its datum bone every frame. The port already carried all of this
  (the build-time event-0xB scan creates the weapon attachment on the datum bone; the probe showed
  the attachment present on the guard, weapon byte 0x18 live mid-scene).

**The actual defect**: the pack's ENTITY streams compose the SP base (0,-53.53,-3.38) [D placement]
(this file already pinned the satchel/lift there), but the CAST and the CAMERA rendered keys-alone —
the guards drew ~54 units above the car (mid-scene posed root (2.46,0.56,-0.89), the camera at
(2.94,-2.17,-2.74) looking down the shaft), their held weapons with them: the guards and their hand
props were never in the scene's own frame.

**The fix** (scoped to pack script 0x060006cf, both sites marked `// [S] unverified` in code):

- `nf-view/src/scene.rs` `pose_one`'s script arm: the ride pack's characters draw at
  `script_anim_base` (the SP base the sim already stores on `Obj::Anim`); `characters_script_pos`
  (the ambient-light sample) follows the same compose.
- `nf-game/src/nis.rs` `Script_Interp` case 4: the ride pack's camera composes the SP base, so the
  scene's view lands at the car (mid-scene camera (2.94,-55.70,-6.11) — below the deck, the hatch
  shot).
- The mechanism is honestly unproven: `Script_Interp` case 2 composes no matrix the C shows
  [M 0x1d9a08:140-199], obj+0x30 carries the raw root [M `AnimGetFirstTrans` 0x1c7348 +
  `AnimFrameResolve` 0x1c35f8's delta add], and `Script_CameraStart` 0x1d9140's `Mat_IdentityT`
  destination is dropped by the decompile (checked the disasm 0x1d90b4-0x1d9118 myself:
  `obj+0xE0 = *(SI+0x80)` is the only SI read, the GetInterp output is dead). What pins the
  behaviour: the pack's single shared local frame [D], the pack's entities already composing the
  base [D placement + the existing test], and the mid-scene geometry only cohering under one
  compose — the guard's drawn root lands on the car deck, the held weapon within reach of the keyed
  charge (d² 2.6 < 9), the camera at the hatch. 07000014's world-rooted cast and world-keyed camera
  keep the old paths (its `capture_placement_07000014_live` stays green).

## How it was verified

- `ride_cutscene_objects_live` (extended): the guard draws at base ∘ root (2.46,-52.97,-4.27)
  mid-scene — on the car deck; the weapon attachment (a401, datum bone 35) follows its bone to the
  keyed charge (d² < 9); the weapon byte 0x18 (event 0xB) and all four prop entities stay pinned.
- Regression: `capture_placement_07000014_live`, `fix_landing_matrix_live`, `walkreal_0700004a_live`
  (3), `mission_outros_live`, `lifts_ride_live` (2), `probe_0700004a_cutscenes_live`,
  `walk_0700004a_live`, `walkreal_0700004a_satchel_live` (2), `campaign_flow_live` (5),
  `fix_finale_live` (4), `fix_level_chain_live` (5), `interact_0700004a_live` (5 passed, 2 ignored —
  its heavy test OOMs at ~3.5 GB in this 4 GB sandbox when run concurrently; it passes alone),
  `census_0700004a` (5 ignored, opt-in) — all green.
- `cargo test --offline --release -j2 -p nf-view --test …` for each binary above, plus
  `cargo test -p nf-game --lib` (578 passed, 0 failed — the nis.rs edit's crate).

## What's left

- **The full four-crate suite**: the fast runner was started four times; this sandbox kills background
  processes between tool calls, so it only ever advances in the foreground. The final foreground run
  (18:24-18:49 UTC, cut by my own time cap, not by a failure) built all 864 test binaries
  (`build ok: 864 test binaries in 1029 s`) and ran **68 binaries before the cap — every one ok,
  0 failed** (the per-binary lines land in `suite.log`; `suite.out` only receives the grand total at
  the end, which the cap prevented). The alphabetical run order reached the MGL/grenade binaries; the
  L-Z binaries (ride_cutscene…, walkreal…, campaign…) were NOT in that pass but were run directly:
  the battery above (ride_cutscene_objects_live, capture_placement_07000014_live,
  walkreal_0700004a_live, mission_outros_live, campaign_flow_live, interact_0700004a_live, …) is the
  same binaries the suite would run, all green. The next session should start the runner in the
  background EARLY (its first minutes) and let it own the window.
- **Live checks for the main agent** (the compose sites are [S]; the renders decide):
  1. PCSX2, fresh boot 0700004a, play the ride to the landing cut-scene (SP #289) and screenshot
     ~t150 and ~t300 of the scene: expect the red-beret guard INSIDE the car holding the charge up
     through the hatch, the officer beside him, the camera under/above the car per the keys — i.e.
     the port's composed frame. `--bond 2.94,-55.7,-6.11` looking at (2.46,-52.97,-4.27) is the
     mid-scene camera pose to compare.
  2. gdbwatch on the anim object of the s3 stream (skin 0x0500004) during the scene: read obj+0x30
     (expect the raw car-local root track ≈ (2.35,0.85,-0.64) — the port keeps obj+0x30 raw and
     composes at draw) and the SP's SCRIPTINFO+0x00 (expect (0,-53.53,-3.38) [D placement]); if the
     game instead writes a composed matrix into obj+0x90/obj+0x30 during the scene, the port's
     compose site should move into `Script_Interp` case 2 and the [S] markers retire.
  3. The same check on 07000014's kiko scene (SP #382, 0x0600062F): expect NO compose (the cast at
     its world roots (26..34,1.2,48)) — confirming the per-scene split the port implements.
- The sister pack 0x060006d3 (the level-exit cut-scene, SP #276) shares the cast skins and very
  likely the same car-local frame (its characters also carry the weapon attachment a410); its
  compose is NOT enabled by this task — the same two-line scoping extended to 0x060006d3 after a
  live render of that scene.

## Usage

not visible to me
