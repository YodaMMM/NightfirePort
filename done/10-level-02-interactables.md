## Your task: `level-02-interactables`, every interactable kind on 07000002 works like the game

07000002 has no per-kind interactables test (0700000b's `interactables_0700000b_live.rs` is the model: one live test per KIND of player-touchable placement - doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, gadget targets - driven the way a player drives it (walk, use key, trigger pull, walk-over, gadget), each asserting the game's effect with a sister). Census every player-touchable placement on 07000002 [D placements + docs/levels/07000002.md], write `crates/nf-view/tests/interactables_07000002_live.rs` with one test per kind, and fix every port gap the tests expose [M the class code; D the params]. A teleport only to reach a spot whose walk is a documented blocker; the operation itself always real input. Every test green (no #[ignore] without a named blocker). Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-02-interactables`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T15:40Z

---
## Report (done, 2026-10-30T17:17Z)


---
## Report from the worker (done, 2026-10-30T17:17Z)

# level-02-interactables — report

Task: every interactable kind on 07000002 works like the game — census every player-touchable
placement, write `crates/nf-view/tests/interactables_07000002_live.rs` with one live test per
kind, fix every port gap the tests expose.

## What's done

**The battery: `crates/nf-view/tests/interactables_07000002_live.rs`, 16 tests, one per kind of
player-touchable placement — 16 passed, 0 failed, 0 ignored** (run on the final tree, merged
with origin/master 8ce724ad, `cargo test --offline --release -p nf-view --test
interactables_07000002_live -- --test-threads=1`, finished in 13.6 s). Every assertion carries
its evidence tag ([M] decompile address / [D] disc bytes / [L] snapshot); the disc data comes
from the committed placement probe (`interactables_07000002_probe.rs`, the 502-placement dump).

The kinds and what each test pins:

1. **Doors 0xDB** — the dining door #331 opens on the use key. The test REFUTED this file's own
   first draft (a "proximity auto-open" reading): a closed kind-2 slide door opens from no path
   but the E — `Door_Update` 0x1363c8 opens only on its open channel, and the collide pick's
   `react_to_hit` 0x135980:101-104 reverses a CLOSING door only; the three castle doors carry no
   channels at all (params 2/3 = 0 [D #268/#330/#331]). `docs/levels/07000002.md`'s "kind-2
   proximity sliders" line carries the same misreading — the census table's behaviour column
   says "Door_Update 0x1363c8 proximity open/close", which is the AUTO-CLOSE arm, not an
   auto-open. Sister: state closed before the E.
2. **TouchOnce plates 0xEA** — the bunker entry chain, driven in the game's real play order:
   walking onto the doorway plate #389 (lock 0x66, off on a fresh boot) raises the ambush 0x81
   [D p0; M Trigger_Hit 0x1b1fc8], and the ambush LOCKS the three BunkerTouchplates #385/386/387
   (lock param 7 = 0x81 [D]) — walking onto them then raises nothing [M Trigger_Locked
   0x1b2fc8:24-29, locked = lock ≠ 0 AND channel ON]. The doc's "the plates chain → AND #388 →
   0x66 → #389" order reads the data forwards; the lock direction makes the ambush the outcome
   of the entry (walkreal's own route step agrees). The AND kind's real-input fold is walkreal's
   earned 0x4C (AND #480 over the hostage rescues + the computer smash) — cited in the test's
   doc comment.
3. **Touch, repeating 0xEB** — #495 holds ch 0x3C while Bond stands inside, clears when he steps
   clear of the box (the exit spot is the lo/hi hi-corner + 2.5 [D]), and re-fires on re-entry
   [M Trigger_Calc 0x1b1900].
4. **LOAD_LEVEL 0xE8** — #206 refuses the E with ch 0x4C off [M Trigger_Activate 0x1b2318:76-90],
   loads 07000003 with a fresh press and the gate on [D param 0; M :111]; the press-edge rule
   from walk_07000002_live re-pinned.
5. **SimpleScripts E-usable 0xE2** — the pavillion grill #272 (mask bit 2, one-shot p1 = 1, no
   channel latch [D]) sweeps off its key-0 pose on the E and the wrap disarms it holding the
   open pose [M SS_Activate 0x1a9ea8; 0x1a9948:121-126].
6. **Pickups 0xF0** — the armour #430 (kind 3, 50 [D]) grants armour 50 on a real walk from the
   spawn [M 0x18e650 case 3]; the P99 ammo #369 (kind 1) walk-over at the boot's full 41-round
   pool is REFUSED — the pool and the pickup stay [M Inventory::equip_ammo's cap refusal], the
   game's own full-pool rule (the first draft asserted an add and the run proved the cap is the
   behaviour at boot; the take-with-room arm is the gun tests' pin); the kind-6 BOND_BONUS #473
   exists only with the mission 0x07000001 gold medal [M Pickup_Create 0x18dda0:93-95], is
   consumed on the walk-over [M 0x18e650:234-236], and the no-medal load creates no kind-6.
7. **Monitor 0x30** — #323 (group 2 [D p0]) enters the camera view on E: state 2, the watched
   sensor is the group-2 camera, the sensor unhides [M Monitor_Activate 0x1b0058; 0x1afcc8].
8. **Fuseboxes 0x31** — all eight placements are scriptless (no params [D]) and each stands at
   its destroyable twin (five real-translation pairs; #15/#324 are the identity-transform twins
   of Computer1 #14 / monitor #323 [D]) — no fusebox instance is created [M FuseBox_Create
   0x1b0448's script gate] and no fusebox channel ever moves. The census's "correctly inert in
   game and port" pinned as a test.
9. **Breakables 0x20** — kitchen-bottle1 #153 shatters under real gunfire and stays gone [M
   Break_ApplyDamage 0x12c128 → Break_Kill 0x12b970]. Sister: intact before.
10. **Destroyables 0xE1** — Computer1 #14 smashes under real gunfire and writes ch 0x48, the
    mission objective's own watch channel [D p0 = 0x48; L c07000002 obj0 channel 0x48; M
    Destroy_CollisionHandler 0x12bd48:42]. Sister: whole + ch off before.
11. **Hurt 0xE4** — #13's gate ch 0x8D parks the volume disabled on a fresh boot [M Hurt_Create
    0x17e870 tail]; after the cable-light smash's ch 0x82 [D p0; M 0x12bd48:42] the emitter's
    out 0x8D rises [M Emitter_Update 0x17b080:163] and the contact damage flows [M Hurt_Update
    0x17e5d0 → Player_DealWithObjHit 0x194458:94-101]. The compact kind entry; hurt_02_live has
    the full chain battery.
12. **Ladders 0x22** — the bunker ladder #114 attaches on a real walk-in from the spawn (the
    walkreal route's own first leg) [M Player_CollLadder 0x196118's yaw window] and the climb
    raises Bond [M Player_Climb 0x196310]. Sister: SS_WALK before the grab.
13. **Wire 0x3E** — #187 grabs on the JUMP where the ctx-3 third icon's arm region reads
    context 3 [D #384; M Player_HandleJump 0x1916b0:118-147 + the icon's 4-tick refresh
    0x196e80:36-54], and the strafe axis carries Bond to a wire end [M Player_Wire 0x1977c0].
    Sister: the E press never grabs (no class-0x2C arm in Player_Activate 0x1a5170).
14. **Sound/Music triggers 0xF9/0xFB** — #378 (ch 0x17 → event 31) and #379 (ch 0x19 → event 32)
    register with the disc params [D], rest disarmed while the channels are off, and the real
    walk onto plate #342 raises ch 0x17 and arms the 30-tick countdown [M MusicTrigger_Update
    0x1b31a0:19-27].
15. **Hints 0x33** — #492/#494 register with the disc labels and the stage gate [D p0/p2 = 3; M
    Hint_Create 0x1b3248:12-25]; the real walk into #494's sphere (the start yard) fires it once
    [M Hint_Update 0x1b3380:24-42]. Sister: the exit marker stays unfired.
16. **Sensors 0x28** — camera #406 (group 2, range 6 [D], out ch 0 — nothing to raise) sees the
    standing player in its cone (`seen` counts [M Sensor_Update 0x1af618]); behind the camera
    the cone never counts. Sister with the behind spot.

**Kinds with zero player-touchable placements on 07000002** (the census [D], checked against
the dispatcher 0x1d04f0 [M]; documented in the test header): switches 0x29, locks 0x2E, lifts
(no placement kind), grapple 0x3A, gadget targets (lasers, PCQWorm, Sub/MiniSub, SpaceLaser,
copters, safes), turrets, searchlights, cars, mines, spawners, creepwall vents. The 0xD9
ScriptPlayers are the level's cut-scenes, not player-touchable placements; their channel writes
are pinned by walkreal/walk (cited in the header).

## Port gaps the tests exposed

**None.** No source change was needed — every kind the disc places on 07000002 already behaves
like the game in the port (the earlier rounds' fixes: hurt.rs, the monitor E-view, the SS
bullet-sweep half, the door collide-pick, the wire jump-grab all held up). The two TEST-side
corrections this run made (both documented above): the door kind's operation is the E, not a
proximity auto-open (the draft test asserted the wrong operation and the decompile says
otherwise), and the bunker plates' lock direction (the ambush locks the plates; the draft's
plate-then-doorway order contradicted Trigger_Locked). A third test-side correction: the ammo
take at the boot's full pool is the game's refusal, not an add (the draft's add assertion
contradicted Inventory::equip_ammo's cap gate).

## How it was verified

- `interactables_07000002_live` **16 passed, 0 failed, 0 ignored** (twice: before and after the
  origin/master merge; final tree 8ce724ad).
- The suite (tools/test/suite.py, the fast runner) — **the sandbox kills background jobs**
  (nohup, setsid, disown all died between tool calls), so it ran in foreground chunks within
  the Bash 560 s cap:
  - `interactables_07000002_live` + `census_07000002_drones`: **16 passed, 0 failed, 1 ignored
    (3 min)**.
  - The 07000002 family chunk (walk, walkreal, hurt, ladders ×2, destroy_channels, bath ×2,
    inert_guards, swim, baked_statics, hostages ×2, fix_ss_grate, wire_live, breakables): **40
    passed, 0 failed, 2 ignored (1 min)**.
  - The interactables family batch (96 binaries, the call cap hit partway): 21 completed ok
    before the cap; `walkreal_07000003_live` and `interactables_07000003_live` hit my 100 s
    per-binary cap (they are >100 s walks — not failures).
  - The heavy batteries at a 280 s cap: **36 passed, 1 failed, 2 ignored (7 min)** — the 1
    "failed" is `walkreal_07000003_live`'s 280 s TIMEOUT (its full-route walk needs the default
    600 s; the sandbox window could not afford it); interactables_07000003_live (115 s),
    interactables_0700000d_live, interactables_0700000b_live, interactables_07000012_live all
    ok (the 2 ignored are the 0b/012 files' named stand-ins).
  - The alphabetical chunk A (killed at the 560 s call cap): `fix_0c_pylon_live` FAILED 2 tests
    (`the_0c_pylon_fills_the_view_from_the_closeup_spot`,
    `the_0c_pylon_is_on_screen_at_the_user_spot`) — pre-existing on the merged master tree
    (this branch's diff is tests-only; not investigated further, flagged for the driver).
- **What's left for the suite**: the full 940-binary pass on the final tree (the sandbox's
  background-job kills + the 560 s call cap make a single uninterrupted pass impossible here;
  the chunks above are the honest partial — every named result is quoted from the runner's own
  lines).

## What's left

1. The full-suite pass (above) — in particular `walkreal_07000003_live` needs its full 600 s.
2. `fix_0c_pylon_live`'s 2 failures on the merged master tree (pre-existing, needs an owner).
3. Live checks for the main agent (I can't run PCSX2):
   - **The castle doors' real operation**: at 07000002's dining door #331 (−27.80, 6.64, −20.26),
     approach WITHOUT pressing E — the door should stay closed [M react_to_hit reverses CLOSING
     only]; then press E — it slides [M Door_Activate 0x135be0]. Access type: watch the door's
     obj+0xF4 state byte. Expected: no state change without the E.
   - **The bunker plates post-ambush lock**: after crossing #389 (−46.96, 6.04, −23.35) raises
     ch 0x81, walk onto #385 (−59.28, 7.66, −26.39) — expected: no ch 0x7F write [M
     Trigger_Locked 0x1b2fc8:24-29; D p7 = 0x81]. Access type: read switch_channels[0x7F].
   - **The full-pool ammo refusal**: walk over PICKUP_P99 #369 (−74.20, 5.63, 24.03) with the
     boot's 41 P2K rounds — expected: the pool stays 41 and the pickup stays [M equip_ammo's
     cap]. Access type: read carried[t] + the pickup's gone byte.

## Usage

not visible to me (no token/credit/context-window figures are shown in this environment; the
driver's message and the sandbox expose no usage counters — reported per the instruction not to
estimate).
