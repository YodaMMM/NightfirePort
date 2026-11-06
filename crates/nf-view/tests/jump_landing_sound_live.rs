//! The jump's second sound belongs to the landing, not the head bump.
//!
//! [M `Player_HandleJump` 0x1916b0:68-71] a head hit (flag 0x20) sets state 2, zeroes the velocity
//! and returns silently. [M :191-201] the arm gated on flag 8 (feet on ground, set by
//! `Player_FeetOnPoint` 0x194188:185) plays 0x5FB when the contact byte is 0xD/0xE, else 0xA3, and
//! arms the two forced steps. The byte is the feet ray's first hit [M 0x194188:187-192].

use nf_game::player::{Bond, Input, LadderCtx};
use nf_game::world::CollisionWorld;
use nf_game::world::V3;

const ROOT: f32 = 1.1;

type Tri = ([V3; 3], u8, bool);

fn up_quad(tris: &mut Vec<Tri>, y: f32, mat: u8) {
    tris.push(([[-10.0, y, -10.0], [10.0, y, 10.0], [10.0, y, -10.0]], mat, false));
    tris.push(([[-10.0, y, -10.0], [-10.0, y, 10.0], [10.0, y, 10.0]], mat, false));
}

fn down_quad(tris: &mut Vec<Tri>, y: f32, mat: u8) {
    tris.push(([[-10.0, y, -10.0], [10.0, y, -10.0], [10.0, y, 10.0]], mat, false));
    tris.push(([[-10.0, y, -10.0], [10.0, y, 10.0], [-10.0, y, 10.0]], mat, false));
}

/// A dry floor at y 0, an optional up-facing surface over it, an optional ceiling.
fn world(surface: Option<(f32, u8)>, ceil: Option<(f32, u8)>) -> CollisionWorld {
    let mut tris = Vec::new();
    up_quad(&mut tris, 0.0, 0xA);
    if let Some((y, mat)) = surface { up_quad(&mut tris, y, mat); }
    if let Some((y, mat)) = ceil { down_quad(&mut tris, y, mat); }
    CollisionWorld::from_triangles_flags(tris)
}

/// Settle, press jump once, run to the landing. Returns the sounds, whether the head hit, and Bond.
fn jump(world: &CollisionWorld) -> (Vec<u32>, bool, Bond) {
    let mut b = Bond::new([0.0, ROOT, 0.0], 0.0);
    let ctx = LadderCtx::default();
    for _ in 0..12 { b.tick_ctx(world, &ctx, &Input::default()); b.sfx_out.clear(); }
    let (mut ids, mut bumped) = (Vec::new(), false);
    b.tick_ctx(world, &ctx, &Input { jump: true, ..Default::default() });
    ids.extend(b.sfx_out.drain(..).map(|(id, _)| id));
    assert_eq!(b.jump, 1, "the press launches");
    for _ in 0..180 {
        b.tick_ctx(world, &ctx, &Input::default());
        ids.extend(b.sfx_out.drain(..).map(|(id, _)| id));
        bumped |= b.head_bump;
        if b.jump == 0 { break; }
    }
    assert_eq!(b.jump, 0, "the jump lands");
    (ids, bumped, b)
}

#[test]
fn a_dry_landing_plays_a3_after_the_launch() {
    let (ids, _, _) = jump(&world(None, None));
    assert_eq!(ids, [0xa3, 0xa3], "launch, then landing");
}

#[test]
fn a_head_bump_is_silent() {
    let (ids, bumped, _) = jump(&world(None, Some((ROOT + 1.5, 0xA))));
    assert!(bumped, "the ceiling must stop the rise");
    assert_eq!(ids, [0xa3, 0xa3], "launch and landing only: the bump plays nothing [M 0x1916b0:68-71]");
}

#[test]
fn a_water_ceiling_does_not_change_the_landing() {
    let (ids, _, _) = jump(&world(None, Some((ROOT + 1.5, 0xE))));
    assert!(!ids.contains(&0x5fb), "the byte comes from the feet ray, not the ceiling: {ids:?}");
}

#[test]
fn landing_in_water_plays_5fb() {
    for mat in [0xD, 0xE] {
        let (ids, _, _) = jump(&world(Some((0.2, mat)), None));
        assert_eq!(ids, [0xa3, 0x5fb], "feet ray first hit {mat:#x}");
    }
}

#[test]
fn the_landing_arms_the_forced_steps() {
    let (_, _, b) = jump(&world(None, None));
    assert!(b.land_step, "BL+0x924 = 999.0 [M 0x1916b0:201]");
    assert_eq!(b.land_countdown, 8, "BL+0x966 = 8 [M 0x1916b0:200]");
}

/// The real water, not a hand-built plane: 07000002's shallows at (-79, 28). [L PCSX2: the landing
/// there reads byte 0xE, plays 0x5FB, then steps 0x28.]
#[test]
fn landing_in_the_02_shallows_splashes_and_wades() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..2000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    sim.bond.pos = [-79.0, 5.0, 28.0];
    for _ in 0..40 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.bond.grounded, "Bond stands in the shallows: {:?} substate {}", sim.bond.pos, sim.bond.substate);
    let mut heard = Vec::new();
    sim.step_bond(Input { jump: true, ..Default::default() }, false, false, false);
    for _ in 0..80 {
        sim.step_bond(Default::default(), false, false, false);
        heard.extend(sim.drones.audio.active_ids());
    }
    assert!(heard.contains(&0x5fb), "the landing splashes: {heard:x?}");
    assert!(heard.contains(&0x28), "the forced steps wade: {heard:x?}");
}

/// 07000007 refuses the jump until channel 0x48 [M Player_HandleJump 0x1916b0:106-113]. [L PCSX2:
/// the press leaves Bond at y 44.089 with channel 0x48 clear.]
#[test]
fn the_07_party_refuses_the_jump_until_channel_48() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..2000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.drones.channels.on(0x48), "the walk starts before channel 0x48");
    let y = sim.bond.pos[1];
    sim.step_bond(Input { jump: true, ..Default::default() }, false, false, false);
    assert!(sim.drones.audio.active_ids().contains(&0x1db), "the refused press plays 0x1db");
    assert_eq!(sim.bond.jump, 0, "no launch");
    for _ in 0..20 { sim.step_bond(Default::default(), false, false, false); }
    assert!((sim.bond.pos[1] - y).abs() < 0.05, "Bond stays down: {y} -> {}", sim.bond.pos[1]);
    let f = sim.drones.frame;
    sim.drones.channels.set(0x48, 1, f);
    sim.step_bond(Input { jump: true, ..Default::default() }, false, false, false);
    assert_eq!(sim.bond.jump, 1, "channel 0x48 frees the jump");
}
