## Your task: `camera-beam-satchel-draw`, the security cameras' sight beam, the satchel charges, the turret wreck

Master draws the security cameras' body and mount and the gun emplacement's gun from posed slots (`crates/nf-view/src/scene.rs`, `lock_slot` and the slot loops after the breakables). Still undrawn (`docs/systems/cameras-turrets.md` §5, the `// draw hand-off:` comments):

1. **The camera sight beam.** The lens proxy (param 0, 0x2000360-class) is posed at the camera's matrix every tick and stretched to the beam's hit length (`Sensor_SetBeam` 0x1af398:57-75: lens obj+0xE8 = range, or the `Collide_RayIntersect` hit distance). Find how the draw applies obj+0xE8 to that object [M — the view/draw path for objects carrying +0xE8] and draw it the same way.
2. **The 4a satchel charges.** The charges are drone bullets of the 0x3b family (`crates/nf-game/src/bullet.rs`, the fuse fields). Check whether the projectile-mesh slots already draw their mesh (weapon+0x5C, 0x20007b3) in flight and once landed on the lift cab; if not, draw them. Then the two countdown digit sprites (`SatchelDigits`, `Bullet_DoTrails` 0x12d890:485-553) and the per-digit beep 0x447 [M 0x12e290-0x12e2b4].
3. **The GT turret wreck** (glist 0x2000039 puff + the destroy burst, the events in `drone.rs` at the GT destroy arm).

Each item: a test that the draw lists carry the piece at the right pose (fails before), and a headless `--shot` (or `--cam`) render per item named in the report with its path, so the result can be compared with the game.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/camera-beam-satchel-draw`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox3 at 2026-10-24T18:51Z

---
## Report (done, 2026-10-24T20:09Z)


---
## Report from the worker (done, 2026-10-24T20:09Z)

# camera-beam-satchel-draw — the security cameras' sight beam, the 4a satchel charges, the GT turret wreck

Branch `s/camera-beam-satchel-draw` from origin/master (6e004921). All three undrawn pieces of
docs/systems/cameras-turrets.md §5's hand-off now reach the draw lists, each with a draw-list test
(`crates/nf-view/tests/camera_beam_satchel_wreck_live.rs`, 3 tests, green — each fails on the
pre-task tree).

## 1. The camera sight beam — DONE

The [M] chain the driver asked for: the lens proxy (param 0) is a gfx object created at
`Sensor_Create` 0x1af010:97-107 (`hashtable_set_object_to_entity_gfx`, +0xE8 seeded with the range
:103, the **OnlyScaleZ** bit OR-ed into obj+0xFA :102); `Sensor_SetBeam` 0x1af398 poses it at the
camera's own matrix (`Mat_Copy(lens+0x90, obj+0x90)` :58) and rewrites obj+0xE8 every tick — range
by default (:62,72), the mask-8 `Collide_RayIntersect` first-hit distance (HITDATA+0xC) on a hit
(:78); SetBeam runs inside `Sensor_InCone` (0x1af4f0:54, so every sweep/track tick) and again in
the return state (0x1af618:238). The draw is the object's PLAIN glist path (`View_DrawObjectGlist`
0x1e6780:43-45 — no billboard bit; the Camera_Beam glist flags are 0x241 [D probe]) with the object
scale riding the mesh's length axis (OnlyScaleZ), and the **Camera_Beam mesh 0x02000360 is
unit-length along local +z** (z −0.0085..1.0283, x/y ±0.16, "Camera_Beam" [D probe,
`beam_satchel_probe.rs`]; the disc lens hash on 07000014 is param 0 = 0x02000360 [D probe] —
docs/fields/sensor.md's "lens 0x02000FE0" row does not match the census levels and should be
re-checked). State 3 clears the lens display mask (0x1af618:248-250) — no beam while disabled.

Port: `world.rs::line_of_sight_hit` (the distance-returning masked walk, same drops), `sensor.rs`
`beam_len` field + `beam_ray` (in_cone writes it per tick; state 2 refreshes it), `scene.rs` third
camera slot (body/mount/beam) posed in `lock_slot` at `sn.matrix` with the z row scaled by
`beam_len` — the same row-2 stretch the port's tripbomb/laser beams use.

Test: `the_camera_sight_beam_draws_at_the_pose_stretched_to_its_hit_length` — sim side (0 <
beam_len ≤ range [M :72,:78]) and draw side (palette rows 0/1 == the camera pose, z row keeps the
forward direction with length == beam_len, the slot's verts are the unit-length Camera_Beam mesh
[D]).

## 2. The 4a satchel charges — DONE

The shooter/record reconciliation [M 0x17d0c0:29-37 + D bullet_range_census]: the 4a shooters
(#286/#287, arm ch 0x22/0x4f) fire **record 0x3d whose family byte (+0x02) is 0x3b** — both
statements in the brief hold. The in-flight mesh (weapon+0x5C = 0x020007b3 "OBJECT_SatchelCharge"
[D probe]) already drew; the gaps were:

- **The landed mesh collapsed**: `Bullet_update` case 3 zeroes the flight dir [M 0x12edec] and the
  draw's `align2dir` of the zero vector zeroed the rotation rows. The landing now captures the
  surface normal (`Bullet::stick_normal`, [M Bullet_CollisionHandler 0x12ce80:256-266's −normal
  stick alignment]) and the draw aligns the landed mesh to −normal.
- **No countdown digits for drone-fired charges**: `Bullet_DoTrails` 0x12d890:418-553 spawns
  `SatchelDigits[sec/10]`/`[sec%10]` (sec = fuse/60, :421-422) EVERY state < 2 tick, but the port's
  digit draw read only the thrown-grenade pool. `tick_charge_fuse` now writes `Bullet::digits`
  every tick and the digit draw (kinds 4/5) covers both carriers. The LED meshes
  0x02000791..0x079a resolve in 4a [D probe].
- **No per-digit beep**: Bullet_DoTrails 0x12d890:435-443 — on the quarter-second change
  (qs = fuse·0.06666667 ≠ BU+0x100) at the rate gate `(qs&3)==0 || ((qs&1)==0 && sec<10) || sec<5`,
  `Sound_Play3D(100.0, −1.0, −1.0, 0x447, obj+0x30)` [M 0x12e290-0x12e2b4 is inside Bullet_DoTrails;
  the symtab has no separate symbol there]. Ported via `Drones::pending_beeps` flushed after the
  bullets pass (play3d + `Sound { id: 0x447, inner: 100.0, outer: −1.0 }` — the decomp's values;
  the grenade's own 96.0 site left as it was).

Test: `the_4a_satchel_draws_in_flight_landed_with_digits_and_beeps` — the real arm path
(ch 0x22 → the shooter's `Bullet_init`), digits written in flight, the 0x447 beep in
`drones.sounds`, and the landed chain (mesh at the charge, z == −normal; the charge is teleported
onto the floor for the landing leg — it falls at ~4 u/s [L this run's trace], a 40 s natural drop
does not fit the test window).

## 3. The GT turret wreck — the smoke DONE, the burst a [S] stand-in

`GT_Update` 0x180498 case 6 (DESTROYED) smokes every 10th game frame:
`Gas_Init(0.05, 0.25, cel, obj+0x30, rot-euler, 0, glist 0x2000039, 7, 0x48403080, 1)` [M case 6,
decompile L440-451]. Ported as `GasPool::init_gt_smoke` (the searchlight site's 6-draw kind-7
shape, life 128 — the decompile-dropped 11th arg; both sibling kind-7 sites read 0x80 [M
0x1cbce0 / 0x1a43f8-0x1a442c]) emitted INLINE in the per-turret loop (the game's RNG position),
replacing the empty `GTEvent::Smoke` arm. The draw side needed nothing: mesh 0x02000039
"smoke_1_dust" is already pooled (64 gas slots, billboard + the rgba/255 colour slot).

The **destroy burst** is `Emitter_Create(10.0, 0, −2.0, barrel+0xC0, up)` [M 0x180498:249-257,
L199-200] — no emitter-def mapping is located in the port yet, so `DestroyedBurst` presents a
one-shot gas puff `// [S] unverified` (shape/life are the port's stand-in; the event itself stays
pinned by gt.rs's `death_gate_and_wreck_smoke`). The barrel-spin half of §5's hand-off
(ext+0xd4/d8) is untouched — `spin` is still never integrated (gt.rs sets only `spin_vel`); named
as the remaining piece.

Test: `the_gt_wreck_smokes_the_case6_gas_init` — `apply_damage(101)` → DESTROYED, then a kind-7,
scale-0.05, rgba [72,64,48,128], life-128 pool entry on the %10 gate, its gas draw slot's billboard
at the puff and the colour slot carrying rgba/255.

## What ran (rule 5)

- `cargo test --offline --release -p nf-game --lib`: **547 passed, 0 failed, 3 ignored**.
- New tests, one binary: `camera_beam_satchel_wreck_live`: **3 passed, 0 failed**.
- Sibling regression binaries built and run: `gun_turrets_live` 2/2, `gt_los_live` 2/2,
  `worldobj_live` 5/5, `fix_satchel_tripbomb_live` 6/6, `walkreal_0700004a_satchel_live` 2/2 —
  all green.
- **The full fast-runner sweep did NOT run**: this sandbox kills background processes (the
  runner died detached, suite.out empty) and a foreground full run does not fit the remaining
  window beside the builds. The crates my tests touch were run as above; the final-tree full
  sweep is left to the driver (same as the hatch-8 round's note).

## Renders for the main agent (no GPU in this sandbox — not produced here)

1. **The beam**: `--cam 27.5 4.2 31.5 4.1 0.35 --frame 40` on `07000014` — the security-centre
   camera at (26.31, 3.54, 28.99) [D probe], lens 0x02000360 range 5: expect the translucent
   Camera_Beam quad from the lens out ~5 units along the camera's forward, at constant
   cross-section, drawn in the camera pose.
2. **The satchel**: `--channels --frame 120` on `0700004a` at the #286 ceiling shooter
   (x≈3, z≈−1 [L trace]): the charge mid-fall with its body mesh and, once landed, the two LED
   countdown digits (15 → 0) riding it; after landing the body sits flush to the surface.
3. **The wreck**: needs a staged destroy (no CLI) — the pool/palette assertions above are the
   verification; a `--bond` spot near wall gun #636 (9.84, 2.28, 17.48 [D gun_turrets_live])
   after `apply_damage` would show the puffs.

## Live checks for the main agent

1. gdbwatch `Collide_RayIntersect` 0x1ec690 during 07000014 with a camera sweeping: the mask-8
   ray from the lens — the first hit's HITDATA+0xC should equal the drawn beam's end distance
   (the port's `beam_len`); expect the value to shorten when the sweep crosses the wall.
2. gdbwatch the lens proxy's obj+0xE8 (0x02000360-class object): range constant while the ray is
   clear, the hit distance otherwise [M 0x1af398:72-78].
3. PCSX2 0700004a: the landed charge's digit sprites — tens/units == the remaining seconds, and
   the 0x447 beep cadence (1/s; 2/s under 10 s; 4/s under 5 s) [M 0x12d890:435-443].

## What's left

- The destroy-burst's real emitter-def mapping (the [S] stand-in above).
- The GT barrel spin integration + renderer consumption (§5's other half).
- The full fast-runner sweep on the final merged tree.
- docs/fields/sensor.md's lens-hash row (0x02000FE0 vs the census 0x02000360) — [D probe]
  disagrees; left uncorrected here since the doc's probe source is not re-derivable in this window.

## Usage

not visible to me — this session exposes no token, credit or context-window figures to copy; the
session's per-subagent task calls reported (prompt/completion): 190932/5695 and 129907/12033, read
from the Task tool results.
