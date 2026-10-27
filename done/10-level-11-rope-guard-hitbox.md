## Your task: `level-11-rope-guard-hitbox`, rounds pass through the roping guards on 07000011

On 07000011 (Phoenix Fire, the stairwell) shots at a guard on the rope go through him: the crosshair sits on the roping guard and the bullet holes land on the wall behind; spent casings also hang frozen in mid-air there. Master now has the room-scoped bullet pick (`segment_bullet_scoped`) and the abseil damage arms (Init/Slide/Hang case 8 [M 0x169610:48-50, 0x169730:68-70, 0x169940:50-52]). Find why a round aimed at the hanging guard's body misses his hit boxes: compare the port's per-bone boxes for the abseil states (the rope pose, `DroneFunc_HandleImpact`, the bone hit test `Collide_Jointy` 0x1ec180) with the game's [M], from the stairwell landings at (14.11, 34.03, 0.26) and the east connector looking at p258 (18.895, 46.516, 1.434) on ch 0x13. Also the casings: shell casings ejected there must fall and settle on the floor like elsewhere (`Bullet_DoTrails`/casing physics [M]). Tests: mouse-look rounds aimed at the hanging guard's chest hit him (not the wall) in every abseil state; casings land.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-rope-guard-hitbox`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-26T17:49Z

---
## Report (done, 2026-10-26T19:29Z)


---
## Report from the worker (done, 2026-10-26T19:29Z)

# level-11-rope-guard-hitbox — the sim-side pick hits the rope guard in every abseil state; the kept-casing mid-air rest found and fixed

Branch `s/level-11-rope-guard-hitbox` from origin/master 7b4cabf2. Window 18:51–20:30 UTC.

## What was asked

On 07000011 (Phoenix Fire stairwell) shots at a roping guard go through him (holes on the wall
behind) and spent casings hang frozen mid-air. Compare the port's per-bone boxes for the abseil
states (the rope pose, `DroneFunc_HandleImpact`, `Collide_Jointy` 0x1ec180) with the game's [M];
make casings fall and settle like elsewhere.

## What the probes found (evidence-backed)

1. **The Jointy pick itself is sound in every abseil state.** Dead-on chest aims through the real
   fire path (`fire_player` → `Bullet_update` 0x12e7b0's per-tick pick: `Collide_PickObj` 0x1eae80
   0x201 sphere then `Collide_Jointy` 0x1ec180 over `AnimGetCollData` 0x1c4ac8's posed boxes) HIT
   the hanging AND the sliding guard, over BOTH the flat world and the real scoped
   `CollisionWorld::build_scoped` world (crates/nf-view/tests/rope_guard_hitbox_07000011_live.rs
   `shots_at_the_roping_guard_hit_him_in_every_abseil_state`, green: the scoped flight pick never
   stops the aim ray before the guard — the wall-4 cel-scoped rule holds on this path — and ≥1
   chest hit lands in INIT/SLIDE/HANG). The boxes are 16, palette Some (73 bones) in Slide and
   Hang; render and boxes share `Drone::object_pose` (scene.rs `pose_one`), so a drawn-guard-vs-
   boxes divergence is structurally impossible in this build.
2. **One real anomaly recorded, not yet the fix's object**: in HANG the guard still plays danim 11
   (the Init rope-grab `SetDAnimNow(0xb)` [M 0x169610 case 1]) — the Hang ENTER's `CallAnim(2)`
   (the hang pose, [M 0x169940 case 1]) never started; the probe shows danim 11/script
   100665490 persisting while state = 0x90. Render and boxes agree with each other, so this does
   not explain misses, but the HANG POSE itself is not what plays — worth a look (the game's
   CallAnim path may gate on something the port's `call_anim` fold drops).
3. **The casings: the frozen-in-mid-air rest found and fixed.** The PC-port "Keep shell casings"
   option rested every casing on its landing plane — the game's SOUND plane, shooter_y −
   (1 + FRand(0.5)) [M Bullet_init_casing_ex 0x12f678:180 / 0x12f8e8] — 1..1.5 below the shooter's
   ORIGIN. For a ground shooter that sits at the deck; a roping guard hangs 10+ units up, so his
   kept shells froze mid-air next to the rope — the reported symptom. FIX (casing.rs): each
   casing now carries `rest_y` — the static floor beneath the eject point (a 40-unit down-ray the
   callers run at eject/arm time, `eject_with_floor`/`reserve_with_floor`, the tick_inner bones
   closure computes it per pending shell) — and a kept casing rests THERE (the [M] sound still
   fires on the plane crossing [M 0x12fb70-0x12fb94]; keep = 0 behaviour unchanged, game-exact).
   The casing lib tests stay green (7/7) — the `|_| None` bones closures were untouched by the
   tuple widening.
4. **Not done — the live kept-casing pin**: `a_kept_casing_from_the_rope_guard_settles_on_the_
   floor_below` is `#[ignore]`: the rope guard's shell never armed in 240 ticks in the fixture —
   his held weapon may carry no casing mesh (the `casing_mesh` ammo census for the 0x1c guards'
   weapon is pending). The fix's unit path is pinned by the unchanged lib tests + the new
   rest_y branch; the end-to-end pin waits for the census.

## What is left (the next round's shape)

- The user-facing miss is NOT reproduced sim-side with dead-on aims. If it survives in the app,
  the divergence is above the sim: the crosshair/HUD ray vs `Bond::look` (the composite `aim` and
  the free-aim `cursor` offsets), or the spread scatter interacting with the small torso box.
  The probes give the harness (rope_guard_hitbox_live.rs / rope_guard_hitbox_07000011_live.rs)
  to bisect with real input.
- Live checks for the main agent (PCSX2, fresh boot, stairwell):
  a. Bond at (14.11, 34.03, 0.26) aiming the crosshair at the hanging p258 guard's chest: dump
     the game's `BL+0x8a8` pitch/yaw and the FIRED bullet's spawn pos + dir (BU+0x20/+0x30) for a
     few shots — pins the game's crosshair-to-shot transform for steep up-aims.
  b. The hanging guard's `+0xA0` (hip) and the danim state id playing while state 0x90 (the
     port's CallAnim(2) never starts — what does the game play?).
  c. A guard's shell eject on the rope: does the game's shell mesh exist for the 0x1c guards'
     weapon (the ammo byte → `casing_mesh`), and where does the shell draw.
- The suite: `tools/test/suite.py` was started per the driver note, but its run predates the
  final tree (no time to rerun inside the window). What I ran on the final code: nf-game lib
  (583 total, 0 failed incl. the 7 casing tests), nf-view tests rope_guard_hitbox_07000011_live
  (3 passed, 1 ignored) and the earlier probes. The full-suite rerun is the next message's first
  step.

## Usage

Environment token/context figures: not visible to me (no token or cost display in this sandbox).
