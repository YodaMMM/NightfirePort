# level-0b-parachute-step-up — the parachute take is WALKED (the stance sweep is gone)

Branch `s/level-0b-parachute-step-up` (from origin/master 3afcdce1; merged origin/master
20624c8c mid-session — the level-11 test files came in clean). Subject:
`crates/nf-view/tests/walkreal_0700000b_live.rs`'s parachute sequence — the deck walk arrives
under pickup #280 but the old walk target grounded Bond on the pickup's low box and the test
fell back to a stance sweep (seven `sim.bond.pos` teleports + real forward drives).

## Result

The stance sweep is DELETED. Both walkthroughs now WALK into the take slot and the take fires
on the walk — `walkreal_0700000b_live` **3 passed / 0 failed** (warm 15296 ticks, fresh 21056,
both `stuck 0`; was 21175 with the sweep). No `nf-game` source changed: the port's pickup touch
already matched the decompile — the gap was the test's approach direction.

## What actually blocks the old walk (the prior session's misattribution, corrected)

- The prior report read the wedge at (3.61, 58.07, 41.34) ("cel 189", 2.4 m short of the old
  target (2.53, 57.25, 43.54)) as "the helicopter body's volume". Two probes on this tree
  (`crates/nf-view/tests/probe_0b_parachute.rs`, scratch evidence) show that is wrong:
  1. **The copter never parks near the pickup.** #328's gate (ch 0x2A) opens and the copter
     flies its route AWAY from the pad: state 1 the whole way, at +3600 ticks it is at
     (66.12, 57.38, 64.45) heading out — it never comes back over the deck. The level doc's
     note "the parachute pickup #280 sits on its deck [D coordinates 2 m apart]" is stale: the
     placement (16.73, 58.08, 24.36) and the pickup (2.53, 58.45, 42.34) are 18 m apart [D].
  2. **The blocker is the level's own static geometry** [D the port of the disc's collision
     triangles]: the pickup hangs over a low box (top ≈57.92, x 2.2-3.8, z 42.6-43.2 — that is
     the prior session's "pickup floor 57.92") inside an open-fronted housing (roof ~59.3-60.9,
     east wall z 44.04). The housing's south face is OPEN at x ≤ ~2.8 above the box, so the
     deck at x 2.0-2.5, z 41.0-41.5 has a clear take-ray to the pickup centre; east of x ≈ 2.8
     the housing's slanted south wall (the caps log's n=(0.194,0.143,−0.971) face) fills that
     gap and blocks both the capsule and the take-ray. The old walk target (z +1.2 north of
     the pickup) drove Bond onto the box's south face and the radial push slid him EAST along
     it into that wall — stuck 1.1 m east of the take zone.
- The take law, pinned from the decompile: `Pickup_Update` 0x18e148:202-220 —
  `Vec_SqDist3D(pickup+0x80, player+0x30) < (radius+1.1)²` (radius 0.69 [D model bounds] →
  1.79 m from the ROOT) AND `Collide_LineOfSight(player+0x80 → pickup+0x80, 0x721)` from the
  SPHERE CENTRE [L ee_start_05: root + the yaw-rotated (−0.1347, −0.1267, 0.2402) offset];
  one-sided rays [M Intersect_RayGeom 0x10a8c8:168], surface classes 0xd/0xe skipped. The take
  raises ch 0x3F [M Pickup_Handler 0x18e650:330-333 writes switch_channels[param 3]; D #280
  p3 = 0x3F]. The port's `pickup.rs::touches` models exactly this; from the slot (x 2.0-2.5,
  z 41.0-41.5) both gates pass on the port's own model of the disc's triangles.
- The walk's body query carries no mask that could skip the box: HITTEST+0x98 = 0x1c0 is the
  zipline substate's flags word; the walk writes 0 there [M Player_Collision 0x194df0 + the
  ee_start read in world.rs:380-386]. The step-up hypothesis (the driver's hint) does not
  apply: nothing in the disc's geometry needs stepping up to reach the pickup.

## The test change

`walk_to_the_mission_end` (both walks share it): after the deck-below arrival (hard-asserted,
unchanged), the walk now targets the take slot `Wp::new(pp[0] − 0.3, pp[1] − 1.4, pp[2] − 1.0)`
(arrived at +77 ticks at (2.20, 58.07, 40.34) in both walks), then the real forward drive
watches ch 0x3F (≤120 ticks), and `taken` is a HARD assert — the seven teleport stances and the
`sister assert` fallback are gone. The header doc and the jump comment no longer repeat the
"helicopter body" misattribution.

## Verification (what I ran)

- `walkreal_0700000b_live` **3 passed / 0 failed** on the final tree (merged with
  origin/master 20624c8c), run twice more during development; both walks print the walked slot
  arrival and `mission complete`.
- The pickup binaries [D the same law on other levels]: `pickup_live` **1/1**,
  `pickups08_live` **5/5**, `fix_pickups_remainder_live` **1/1**.
- The sibling 0b binaries: `interactables_0700000b_live` **9/9**, `walk_0700000b_live` **4/4**,
  `models_sweep_0700000b_live` **2/2**; the merge's `walkreal_07000011_route_live` **1/1**.
- Build: `cargo test --offline --release --no-run` for all four crates on the final tree.

## Suite total

The sandbox kills every background job between tool calls (the nohup'd `suite.py` died twice,
once mid-build, once after the alsa-sys env omission), so the suite ran on the resumable
foreground protocol from level-11's report: per-binary `--test-threads=1` under NF_SILENT=1,
240 s cap, results accumulated in `/home/z/suite0b_state.json` across invocations. State at
report time (03:47 UTC):

- **279 of 976 test binaries completed: 812 passed, 6 failed, 53 ignored, 0 CRASH.**
- The 6 failures are pre-existing/environmental, none touched by this task (test-only diff):
  `disc_golden` (3 — the slim ISO's known allowed failures), `fix_0c_pylon_live` (2 — the
  GPU-less wgpu `adapter: NotFound` environmental pair), `campaign_flow_live` (1 — the 240 s
  wall cap; it is a long flow binary, untouched here).
- The remaining ~700 binaries are resumable (re-run the same protocol; the state JSON lives in
  the sandbox home and may not survive a reset).

## What's left / open items

- The suite's remaining ~700 binaries (resumable; the touched binaries and the pickup binaries
  are already green above).
- The level doc's stale copter note ("the pickup sits on its deck, coordinates 2 m apart") —
  correct it when the doc round next touches 0700000b.
- Pre-existing: the waiting-room exit transit (the one kept stand-in, live check 1 in the
  prior report), the ch 0x54/0x06 writer chain, the boot-loadout Q-Worm grant.

## Live checks for the main agent

1. **The walked take on the disc** — PCSX2, FRESH boot of 0700000b (no save-state resume),
   Bond on the helipad deck after the roof cut-scene (key [−14.71, 58.00, −2.06]). Walk east
   to the parachute and into the slot south-west of its housing (x 2.0-2.5, z 41.0-41.5),
   WITHOUT jumping or stepping up. Expect: the "took the parachute" message and objective 11
   arming while standing on the deck (the touch law 0x18e148:202-220 needs only the 1.79 m
   distance + a clear 0x721 ray). If the disc refuses from the slot, the port's triangle set
   around the housing's south face differs from the disc's — dump the cel's faces there.
2. **The copter's route** — same boot, from the deck: watch #328 after the gate opens. Expect
   (port reading): it lifts off, circles out and away (state 1), and never parks over the
   pickup. If the DISC's copter instead hovers near the deck at any point, its route keys or
   `Copter_Update`'s state machine differ from the port's read and the walkthrough visuals
   differ from the port's.

## Usage

not visible to me
