//! fix-08-collision: an invisible wall that still collided inside
//! 07000008's missile room (`Room_Guardhouse_F2`, graph cell 48) at
//! `--bond 155.19,43.12,-113.38,1.57,0` with portal culling ON.
//!
//! The exterior `Room_Outside_3` (cell 15) wall slabs — fix-08-walls measured their tris across
//! the room interior [D per-tri probe; this file's probe: 64 collision tris inside the room box,
//! bands x 155.0..165.9, y 36.75..57.4, z −125.0..−118.0] — cull visually at master, but their
//! COLLISION still blocks Bond: the port's movement pass calls the FLAT
//! `CollisionWorld::capsule_push`/`segment_hit` (every static triangle, one shared world)
//! [M crates/nf-game/src/player.rs `physics_tick`/`physics_tick_climb`], while the game resolves
//! the whole pass through the actor's own cel: `Player_Collision` 0x194df0 stores the actor's
//! obj+0x20 cel pointer into the BLData+0x740 HITTEST's start-cel field (`*(BLData+0x7c0) =
//! *(obj+0x20)` — the first store of the function), `Collide_Pick` 0x1eb320 walks
//! `Collide_StraddleCels` 0x1eac98's result and tests each straddled cel's OWN object list
//! (`*(cel + 0x18)` list head), and `Player_CollisionHandler` 0x1949a8 runs the floor test
//! (`Player_FeetOnPoint`) off the same HITTEST. So a triangle hanging on another room's cel never
//! blocks indoors — culling and collision share the per-cel attribution.
//!
//! The port already HAS the per-cel machinery: `capsule_push_scoped`/`segment_hit_scoped`
//! (the straddle walk + the chain-owner buckets fix-portal-eyecel's `attribute_scope` re-keyed).
//! These tests pin the switch: the northward walk across the slab face must cross it (fails
//! pre-fix), the room's real walls must still block (sisters), and the scoped query must
//! disagree with the flat one exactly at the slab face (the mechanism pin).
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load_08(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000008.bin", false).expect("load 07000008");
    let idle = Input::default();
    for _ in 0..600 { if !scene.sim.as_ref().unwrap().disabled { break; } scene.sim.as_mut().unwrap().step_bond(idle, false, false, false); }
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(idle, false, false, false); }
    assert!(!scene.sim.as_ref().unwrap().disabled, "the intro never released Bond");
    scene
}

/// Fail-first repro: from the repro spot inside the missile room, walking NORTH (yaw π,
/// the −z facing) hits the exterior slabs' west face (z −118.0 [D probe]) — pre-fix the flat
/// capsule push stopped Bond at z ≈ −117.5 (face + radius) and squeezed him UP the invisible
/// wall (y 43.75 → 44.17 over ticks 59-68) before shoving him over the top; post-fix the slabs
/// hang on cell 15 and never enter the pass [M Player_Collision 0x194df0: the HITTEST's start cel
/// is obj+0x20; Collide_Pick 0x1eb320 tests each straddled cel's own list (cel+0x18)] — Bond
/// crosses the band on the level floor.
#[test]
fn the_missile_room_walk_across_the_slab_band_meets_no_wall() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_08(&iso);
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::new([155.19, 43.12, -113.38], std::f32::consts::PI);
    for _ in 0..90 { sim.step_bond(Input::default(), false, false, false); }
    eprintln!("settle: pos {:?} cel {:?} grounded {}", sim.bond.pos, sim.bond.cel, sim.bond.grounded);
    assert!(sim.bond.pos[1] > 40.0, "Bond stands on the missile room floor (got {:?})", sim.bond.pos);
    let forward = Input { forward: 1.0, ..Input::default() };
    // The floor lip at z −114.41 (a riser whose top edge sits at the floor height) holds the walk
    // for a while before Bond rides up it to y ≈ 43.75: in the game the same placement and walk
    // took ~35 frames at the lip in one run and 60-90+ in two others, crossing z −117.11 → −118.11
    // between frames 60 and 70 in the fastest [L PCSX2 07000008, audits/live-1127-08slab]. So the
    // lip's hold time is not pinned; what is pinned is that the slab FACE (z −118.0) holds nothing:
    // once Bond reaches z −117.0 he is past −118.1 within the walk's own pace (1 u ≈ 10 ticks).
    let mut max_y = f32::NEG_INFINITY;
    let (mut at_face, mut past) = (None, None);
    for t in 0..240 {
        sim.step_bond(forward, false, false, false);
        max_y = max_y.max(sim.bond.pos[1]);
        if at_face.is_none() && sim.bond.pos[2] < -117.0 { at_face = Some(t); }
        if past.is_none() && sim.bond.pos[2] < -118.1 { past = Some(t); eprintln!("crossing tick {t}: pos {:?}", sim.bond.pos); break; }
    }
    let (Some(f), Some(p)) = (at_face, past) else {
        panic!("walking north never crossed the slab face (z −118.0 [D probe]) in 240 ticks (face {at_face:?}, past {past:?}, pos {:?})", sim.bond.pos)
    };
    assert!(p - f <= 15,
        "the slab face holds the walk {} ticks — the invisible wall [M Player_Collision 0x194df0 scopes \
         the pass to obj+0x20's cel; Collide_Pick 0x1eb320 walks the straddled cels' own lists]", p - f);
    eprintln!("crossing max y {max_y:.3}");
    // The pre-fix flat pass not only stopped Bond at the face — it squeezed him UP it (y 43.75 →
    // 44.17) and over the top. The crossing must stay on the level floor.
    assert!(max_y < 43.9,
        "the crossing rides the level floor — no wall to climb (the pre-fix flat pass squeezed \
         Bond up the slab face) (max y {:.2})", max_y);
    for _ in 0..120 { sim.step_bond(forward, false, false, false); }
    eprintln!("north walk: pos {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
    assert!(sim.bond.pos[2] > -122.9,
        "the room's real north wall still blocks — the walk stayed inside Room_Guardhouse_F2 \
         (z to −122.33 [D cell bbox]) (got z {:.2})", sim.bond.pos[2]);
    assert!(sim.bond.pos[1] > 40.0, "the floor held across the band (got {:?})", sim.bond.pos);
}

/// Sister: the room's real west wall still blocks the same walk westward — the per-cel
/// attribution must not delete the room's own collision.
#[test]
fn the_missile_rooms_real_west_wall_still_blocks() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_08(&iso);
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::new([160.0, 43.12, -114.5], -std::f32::consts::FRAC_PI_2);
    for _ in 0..90 { sim.step_bond(Input::default(), false, false, false); }
    let west = Input { forward: 1.0, ..Input::default() };
    for _ in 0..210 { sim.step_bond(west, false, false, false); }
    eprintln!("west walk: pos {:?}", sim.bond.pos);
    assert!(sim.bond.pos[0] > 153.5 && sim.bond.pos[0] < 156.6,
        "the missile room's real west wall blocks (room x from 154.5 [D cell bbox]) (got x {:.2})", sim.bond.pos[0]);
    assert!(sim.bond.pos[1] > 40.0, "the floor held (got {:?})", sim.bond.pos);
}

/// Sister (the repro's own walk): eastward from the repro spot reaches the room's real east
/// wall — the fix must not open the building.
#[test]
fn the_missile_rooms_real_east_wall_still_blocks() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_08(&iso);
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::new([155.19, 43.12, -113.38], std::f32::consts::FRAC_PI_2);
    for _ in 0..90 { sim.step_bond(Input::default(), false, false, false); }
    let east = Input { forward: 1.0, ..Input::default() };
    for _ in 0..210 { sim.step_bond(east, false, false, false); }
    eprintln!("east walk: pos {:?}", sim.bond.pos);
    assert!(sim.bond.pos[0] > 158.0 && sim.bond.pos[0] < 164.0,
        "the walk crosses the room and stops at the real east wall (room x to 163.14 [D cell bbox]) \
         (got x {:.2})", sim.bond.pos[0]);
    assert!(sim.bond.pos[1] > 40.0, "the floor held (got {:?})", sim.bond.pos);
}

/// The mechanism pin: at the slab face the FLAT capsule push still reports the exterior slabs
/// (they exist in the shared world — the geometry fix-08-walls measured), while the SCOPED query
/// with Bond's own cel finds no wall — the per-cel attribution the game's pass runs on
/// [M Collide_Pick 0x1eb320: `Collide_StraddleCels` result → each cel's own +0x18 list].
#[test]
fn the_scoped_capsule_disagrees_with_the_flat_one_at_the_slab_face() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_08(&iso);
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::new([155.19, 43.12, -113.38], std::f32::consts::PI);
    for _ in 0..90 { sim.step_bond(Input::default(), false, false, false); }
    let cell = sim.bond.cel.expect("Bond's cel resolves inside the missile room");
    eprintln!("bond cel {cell} pos {:?}", sim.bond.pos);
    // Bond's own vertical cylinder spine (root + 0.275 .. root − 0.55, the walk's query shape),
    // x 157.5 (inside the room's x band, clear of the side walls), 0.25 past the slab's face
    // (z −118.0). The slab's faces there point −z, away from the room [D: the flat push below
    // is −z]. Under the game's one-sided cylinder push [M Intersect_CylGeom 0x109da0, flags 99
    // for a spine behind the plane] a room-side spine is never pushed by them at all, so the
    // mechanism pin reads past the face: the shared world still carries the slab (the flat push
    // shoves the spine on south), while Bond's own cel's lists carry none.
    let y = sim.bond.pos[1];
    let a = [157.5, y + 0.275, -118.25];
    let b = [157.5, y - 0.55, -118.25];
    let (front, _) = sim.world.capsule_push([157.5, y + 0.275, -117.75], [157.5, y - 0.55, -117.75], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
    assert!(front[2].abs() < 0.01, "the slab's −z faces never push a room-side spine (one-sided) (got {front:?})");
    let (flat, _) = sim.world.capsule_push(a, b, 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
    let (scoped, _) = sim.world.capsule_push_scoped(Some(cell), a, b, 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
    eprintln!("flat push {:?} scoped push {:?}", flat, scoped);
    assert!(flat[2] < -0.01,
        "the shared world still carries the exterior slabs (the flat push shoves the spine on south) [D probe: cell 15's tris z −125.0..−118.0 inside the room] (got {:?})", flat);
    assert!(scoped[2].abs() < 0.01,
        "Bond's own cel's lists carry no wall at the slab face [M Collide_Pick 0x1eb320 per-cel lists] (got {:?})", scoped);
    // Sister: the scoped query against a wall of the room's OWN cel still pushes (attribution,
    // not a blanket cull). A standing spine at x 154.3, z -114.5 meets the room's wall there
    // (push x about -0.18). The old horizontal spine ended ON a plane, the knife edge where
    // |v| -> 0 gives the zero push [M 0x109da0 `fVar30 == 0.0`]; it read 0.55 only through the
    // placement matrix's 1.0000002 scale (identity cels skip the matrix, `World::cel_ident`).
    let (wscop, _) = sim.world.capsule_push_scoped(Some(cell), [154.3, a[1], -114.5], [154.3, b[1], -114.5], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
    eprintln!("own-wall scoped push {:?}", wscop);
    assert!(wscop.iter().map(|v| v * v).sum::<f32>().sqrt() > 0.01,
        "a wall of Bond's own cel still collides through the scoped query (got {:?})", wscop);
}
