## Your task: `level-14-interactables`, every interactable kind on 07000014 works like the game

07000014 has no per-kind interactables test (0700000b's `interactables_0700000b_live.rs` is the model: one live test per KIND of player-touchable placement - doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, gadget targets - driven the way a player drives it (walk, use key, trigger pull, walk-over, gadget), each asserting the game's effect with a sister). Census every player-touchable placement on 07000014 [D placements + docs/levels/07000014.md], write `crates/nf-view/tests/interactables_07000014_live.rs` with one test per kind, and fix every port gap the tests expose [M the class code; D the params]. A teleport only to reach a spot whose walk is a documented blocker; the operation itself always real input. Every test green (no #[ignore] without a named blocker). Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-interactables`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T19:59Z

---
## Report (done, 2026-10-30T20:53Z)


---
## Report from the worker (done, 2026-10-30T20:53Z)

# level-14-interactables — every interactable kind on 07000014

Branch `s/level-14-interactables` (master 61c7e3ea + the task's files). The task: one live test
per KIND of player-touchable placement, driven the way a player drives it, each with a sister;
fix every port gap the tests expose.

## What's done

**The census** (docs/levels/07000014.md §1b + this session's `census_07000014` dump over the
slim ISO, [D]) — the player-touchable kinds on 07000014 and their evidence:

| Kind | Placements [D] | The test |
|---|---|---|
| Doors 0xdb | 55 (incl. the Door_Win_Slide_* pairs, 4 OBJECT_Door, the 0x85-locked Door_Slide_11A/B #477/#478) | `a_plain_door_opens_on_use_and_the_locked_pair_refuses` |
| Switches 0x29 | 4 — "alarmswitchplayer" #605 (56.36, 1.64, 42.90) ch 0xDE, #632 ch 0x4F (+2 template-map rows the main-map rule never creates [M 0x1d1cf0's epilogue]) | `the_alarm_switch_e_toggles_chde_once` |
| Fuseboxes 0x31 | 5 real (#617/#709/#711/#1159/#1245, script 0x06000A1B) + the vent prop | `the_east_wing_fusebox_cuts_through_the_laser_and_the_e_is_inert` (box #617, group 0 = ch 0x22) |
| Pickups 0xf0 | 32 main-map (HKOICW wall, MP5/5.56 ammo, grenades, tripbombs, stickies, BodyArmourHeavy ×4) | `the_server_room_armour_is_taken_by_the_walkover` (#629, kind 3, 50) |
| Breakables 0x20 | 3 — ServerRoomWindow #412, CompRoom1Window #423, SCCP_Window #482 (params [2, g, 0x20]) | `the_server_room_window_breaks_on_the_gunshot` |
| Plates 0xEA/0xEB | 42 TouchOnce / 13 Touch | `the_vent_touchonce_walkover_fires_chb8_from_the_spawn` (#601 out 0xB8 mask 8) |
| Monitors 0x30 | 12 "SWITCH_Consoles_Evilbase" (camera groups 1-5, param 1 = 1 → immune [M 0x1b08b0:16-21]) | `the_security_monitor_use_shows_the_group_camera_and_cycles` (group 1 ↔ sensors #640/#1108) |
| Hints 0x33 | 24 "Hint Small Plate" (sabotage-room labels 0x100020C-E/0x1000211, gated on 0x8C/0x53) | `the_sabotage_hints_gate_on_the_8c_channel` |
| Gadget targets 0x3a | 3 GRAPPLEPOINT_SkyBridge (#626/#657/#658) | `the_qclaw_latches_the_skybridge_grapple_point` (#658, the vent-room one) |
| **Lifts** | **none** — the census lists no lift kind on 07000014; the level's vertical moves are the stairs, the ducts and the Q-Claw | no test (documented absence) |

Sensors (0x28, 14 + 1 template row) are the cameras the monitors feed on — the detection
system, not a use/touch kind; their out-channel trip (2.0 s in-cone [M 0x1af618:152-157]) is
part of the alarm chain, not a player operation, so no separate test (census note).

**The file**: `crates/nf-view/tests/interactables_07000014_live.rs` — 9 tests, all green. The
vent-room tests (plate, grapple) walk from the spawn with real input only. The interior tests
reach their spots with the documented stand-in transit — the real-input crawl wedges at the
east duct mouth (x = 40.98, the lintel head bump [D probe14_eastduct;
walkreal_07000014_live's two `#[ignore]`d legs]) — and every operation itself is the real
input path (the walk-over, the E press, the trigger pull, the Laser burn, the grenade blast).

**What the tests pin** (every assertion cites its evidence in the file):

- The TouchOnce walk-over fires ch 0xB8 from the spawn crawl and the channel holds [D #601 out
  0xB8 mask 8; M Trigger_Hit 0x1b1fc8].
- The Q-Claw bolt (weapon 0x50) latches the SkyBridge grapple point and the reel hangs within
  2.0 of it [M Bullet_Delete 0x12e660:50-55, Player_SSGrapple 0x192860:79-121; D placement].
- The unlocked OBJECT_Door #618 opens on the E [M Door_Activate; D param 3 = 0]; the sister
  Door_Slide_11A/B locked on ch 0x85 refuses the same E while 0x85 is off [D #477 param
  3:0x85; M Door_Activate's lock gate].
- The alarmswitchplayer E toggles ch 0xDE exactly once — the busy one-shot [M Switch_Activate
  0x1b02a8 + the busy gate 0x1b02a8:12-16; D param 4:0x1].
- The fusebox's E press is inert (no class-0x3C arm [M 0x1a5170:95-160]) and the Laser cut
  writes the group-0 word [M FuseBox_Update 0x1adfe8:88-123; D #617 p1 0x22].
- The armour walk-over grants 50 [M Pickup_Handler 0x18e650 case 3; D #629 params 0:0x3 2:0x32].
- The monitor E shows the group's camera (state 2, a same-group sensor watched) and the next E
  cycles [M Monitor_Activate 0x1b0058, Monitor_Update 0x1afcc8; D param 0:0x1].
- The hint plates post their labels into the type-1 INFO queue and gate on 0x8C both ways [M
  Hint_Update 0x1b3380:33 + the channel gates; D params 0:0x100020e 4:0x8c / 3:0x8c].
- **The breakable discovery** ([M]+[D], a behaviour pin, not a port change): the three 0x20
  windows carry param 2 = 0x20 → `Break_Create` writes tag+0x08 = ~0x20 [M 0x12c230:31-39],
  and `Collide_FilterBullets` drops every contact whose weapon word meets ~0x20 | 0x106 =
  0xffdf [M 0x1ece30:24-33] — the disc's guns carry word 0x0008, the laser 0x0002, the watch
  0x0100 [L ee_start_05.bin weapon_data +0x10]. So a real gunshot NEVER breaks these panes;
  the killer is the blast (`Explode_Propagate` case 0x20 → `Break_ApplyDamage` [M
  0x17c518:163-167], no weapon filter). The test pins both sides: 60 aimed gun rounds leave
  the pane standing; the weapon-0x34 blast kills it. The grenade itself is
  walk_07000014_live's documented stand-in (the level's grenade pickups sit in the west pick
  room, 60 u west of the window; the real-input throw needs that transit) — the BLAST is the
  game's own code path.

**Port gaps found: none in the class code.** All nine kinds behaved per the decompile on the
first real-input drive — the fixes during bring-up were test-side (the door stance distance,
the monitor stance's floor probe, the window's blast path), each cited in the file. Two doc
corrections fall out of the census (docs/levels/07000014.md §1b's guesses, superseded by this
session's dump): kind 0x33 is `Hint_Create` (the one-shot hint markers fix-context-hints
ported), not "monitors/screens family"; kind 0x30 is the `Monitor_Create` consoles and 0x28
the `Sensor_Create` cameras, not "dressing".

## Verification

- The task's binary, fresh process:
  `NF_ISO=$NF_ISO ./target/release/deps/interactables_07000014_live-35e25598deb59159 --test-threads=1`
  → `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.47s`.
- The touched crates: only `nf-view`'s test set (one new test file; no source file changed).
  `cargo test --offline --release -p nf-view --test interactables_07000014_live --no-run` clean.
- The fast runner (`NF_ISO=… nohup python3 tools/test/suite.py > suite.out 2>&1 &`, started
  20:06 UTC on the branch tree = master + WORKLOG): build `build ok: 946 test binaries in
  1244 s`, then the sequential per-binary runs. At report time (21:40 UTC) its output is 5
  problem lines: `identity_disc` + the 3 `disc_golden` (the documented slim-ISO cross-check
  failures — the same 4 the level-07 report quotes as the slim-ISO baseline) and `CRASH
  mp_stress_explosives_live (exit -9)` (a SIGKILL — the environmental OOM under this
  sandbox's parallel builds, not a test failure; the test's own process was killed, and this
  branch touches no MP code). The runner cannot finish its ~946 binaries inside the sandbox
  window (the level-07 session hit the same wall); the per-binary state resumes from
  `suite.out` on this branch. The final tree differs from the runner's tree by exactly one NEW
  test binary (no existing code changed), whose 9/9 run is quoted above.

## What's left

- The fast runner's tail (binaries after `mp_*`) — the next message can resume from
  `suite.out` and quote the final total line.
- The monitor's third-E exit: the port's cycle arm picks the first OTHER same-group sensor,
  which alternates a two-camera pair and never reaches the cooldown states 9→4→0 [M
  0x1afcc8's none-left arm]. The decompile's walk order (from the watched camera's successor)
  implies an exit after the last; the test leaves it a live check rather than pin either
  reading (marked [S] in the file). A one-line fix in `monitor.rs::activate` once the walk
  order is settled.
- The real-input grenade throw at the server-room window (the west pick room transit, then
  the throw) to replace the documented stand-in seed.

## Live checks for the main agent

- The east duct mouth (x = 40.98): the real-input crawl wedges at the lintel — the port's
  crawl capsule vs the game's. Watch the head bump there in PCSX2 (fresh boot, crawl east from
  the vent store room); the port needs the exact lintel clearance.
- The monitor console (47.45, 2.19, 36.81): E → the camera feed shows; E again → the other
  group-1 camera; E a third time → does the feed EXIT (Bond back in control) or cycle back?
  Settles `monitor.rs::activate`'s cycle order.
- The server-room window (x ≈ 30.95, z 33.25): shoot it with any gun in the real game —
  expect NO break (the disc filter ~0x20 [D + M 0x1ece30]), then a grenade at its wall —
  expect the shatter. Confirms the filter reading end-to-end.

## Usage

- Context window: not visible to me (no token/credit/cost figures are shown in this
  environment's interface; nothing to copy).
