# level-14-vent-drop-2 — the vent-drop hand-back is ported and pinned; the delta walk's leg 2 found the next blocker (a west door)

**Status: the fix level-14-vent-drop scoped landed and is pinned green; the full
`walk_the_infiltration_to_the_delta_exit` is back to `#[ignore]` with the exact blocker
(a facility west double door that will not open on the walk).**

## What was implemented (the -1 report's "The fix", as scoped)

1. **`Obj::Anim` gained `root: Mat4`** (own field line, `crates/nf-game/src/nis.rs`), seeded
   `IDENT` in the one production constructor (`Script_AnimStart`'s `anim_start`) and the one
   unit-test constructor.
2. **`interp` case 2** stores the body's current world root each tick:
   `w.anim_root(*skin, *script, *frame)` → `*root = m`.
3. **New `World::anim_root`** (default `None`; implemented for `drone.rs`'s `Data`): the posed
   skeleton's **bone 0** world matrix — `script_frame` → `seqs.get(q)` → `fits` →
   `local_pose`/`world_pose` → bone 0. This is the root-track matrix the game keeps in the anim
   object's +0x30 (`AnimGetFirstTrans` 0x1c7348 seeds it, `AnimFrameResolve` 0x1c35f8:428-432
   advances it) [M]. **`discrete0` could not be used**: the player body skin 0x0500005e has no
   discrete parts (`sk.extra` is empty — verified with an env-gated `NF_PARTS_DEBUG` probe that
   showed `seq_in true, fits true` and still an empty parts vec), so `discrete0` read `None` at
   every frame and the first cut captured `IDENT`.
4. **Event 6** (`Script_EventHandler` 0x1d94c0 case 6): after the strict previous-stream capture,
   if `player_at.is_none()` and the pack is `0x0600_0833` [S unverified — the game-side
   discriminator is still the -1 report's live check 1], the fallback scans `objs.list` newest
   first for `Obj::Anim { script: 0x0600_0857, .. }` and takes its `root`. Packs WITH a marker
   entity keep the proven Kiko path untouched.

The chain then runs on existing proven rails: `Script_Free` 0x1da880:44-48 →
`Request::PlayerEnable { at }` → the view's drain (`scene.rs` `PlayerEnable` arm) →
`Bond::stand_at(m[3][0..2], yaw from m[2])` [M].

## The hand-back position: what the data actually says

The -1 report's "(27.60, 0.98, 43.56)" is the **root track's seq end**, not the hand-back pose.
The body stream s18 starts at t294 and the scene ends at t1399, so the capture reads the anim at
frame ~1106, and the anim's later frames carry the body further. The live capture in the port
(the body's bone-0 matrix at event time) is **(30.09, 1.06, 38.73)** — inside the facility, floor
height, past the mezzanine wall (z 38.21 [D walk-4]). The pinned test asserts the proven
essentials (inside the facility: y ≈ 1, x 25..35, z < 39.5) and prints the exact pose; the
game-side confirmation is live check 2 below.

## What ran

- `walkreal_07000014_live` (nf-view, the whole binary): **3 passed; 0 failed; 1 ignored**
  (`walk_the_infiltration_to_the_delta_exit`), including the new un-ignored
  **`the_vent_drop_hands_bond_into_the_facility`** — the walked legs 1a..1f (the crossing, the
  trench/corridor/stairwell, the shaft Q-Claw reel, the vent lane, the mezzanine connector), the
  SP #276 cut-scene writing ch 0x47, objective 1 completing, and the hand-back inside the
  facility. The original symptom (Bond left on the mezzanine) is gone.
- `cargo test --no-run` for nf-game/nf-view/nf-audio/nf-formats: all four crates build clean.
- The full suite did NOT fit the window (a full-disk cleanup + a 4-crate rebuild consumed most of
  it; the background runner is killed between tool calls in this sandbox, so everything ran in
  the foreground). Nothing else was pushed untested: only nis.rs, drone.rs and the one test file
  changed, and the touched binary ran green.

## The leg-2 blocker (exact, for the next session)

The nav walk west reaches (-52.79, 1.14, 39.33) and wedges on the facility's **west double door
at (-52.2, 1.2, 40.1)** — two door records at the same spot, kind 2, state 0x2, pos01 0.00,
`ch_open` 0, `ch_lock` 0x85, `lock_flag` 0, `approach` 0. The walker's "opening door" interaction
does not move it; `walk_to` hits the 6000-tick cap with the target (-49.0, 1.0, 38.4) 3.90 away
(the #381 band x −51.97…−46.39 [D] sits just east). Next session: how the game opens this door —
a switch channel (which placement writes 0x85?), the approach side, or an interaction the port
lacks — then legs 2..6 (unchanged code) to the Delta exit #944.

## Live checks for the main agent (unchanged from the -1 report, now sharper)

Fresh boot 07000014, walk west into the #262 band on the mezzanine:

1. gdbwatch the player obj+0x30 through SP #268's 1400 ticks and `fieldprobe.py` SCRIPTINFO+0x40
   of the active script player at event-6 time: does si+0x40 hold the mezzanine pose (strict
   fallback) or the body root (≈ (30.1, 1.06, 38.7) at t1399 per the port's read of the pack's
   anim)? Also watch obj+0x90 during the scene.
2. Where does control return one second after the scene? If ≈ (30.1, 1.06, 38.7) the body-root
   capture is proven; if the mezzanine, the game moves Bond some other way.
3. Does the game's Bond ever walk/jump the mezzanine's south edge himself, or is the drop
   exclusively cut-scene-carried?

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the
session.
