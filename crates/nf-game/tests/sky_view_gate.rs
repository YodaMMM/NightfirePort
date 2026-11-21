//! G-019: `View_DrawSky` 0x1e5ad0 runs its slot loop only while `viewer+0x20` is set [M :91];
//! `Vision_Portal_Recurse` 0x1e86e8:33-41 sets that byte to "some VisCelList cel lacks flags bit
//! 0x10000000" (the no-sky room bit `parseentity_fixup_entity` 0x1d00c0 sets from room parameter 3).
//! With the gate clear the lightning timers, the thunder countdown, the per-call
//! `SuperHappyLightningFlag` clear and the 0x0700001B counter all hold.
//! Also the 1b sun rewrite [M :186-196] on the identity base (0x2d6230 = CONSTANT_MAT_IDENT [symtab]).
//!
//! Pure test: no NF_ISO needed.

use nf_game::celgraph::{chain_sees_sky, CelGraph, CelNode, CEL_NO_SKY};
use nf_game::drone::Rng;
use nf_game::sky::*;
use nf_formats::ps2float::mul;

fn storm_sky() -> Sky {
    let mut sky = Sky::default();
    // 0700000a's slot 3: a flash-only Thunderhead, interval 24 s, flash length 5 [D].
    sky.slots[3] = SkySlot { flash_glist: 1, interval: 1440, flash_len: 5, ..Default::default() };
    sky
}

#[test]
fn a_closed_sky_gate_holds_every_timer() {
    let mut open = storm_sky();
    let mut shut = storm_sky();
    let (mut r1, mut r2) = (Rng::new(0x51a7), Rng::new(0x51a7));
    // Open gate: the fresh 0 dark timer starts a flash on the first frame [M :112-145].
    let f = open.frame(0, false, true, &mut r1);
    assert_eq!(open.slots[3].flash, 5, "open gate: the flash starts on frame 1");
    assert_eq!(open.flash_mask, 1 << 3);
    assert!(f.thunder.is_empty());
    // Closed gate: nothing in the loop runs, the RNG is untouched, nothing draws.
    shut.globals.super_happy = true;
    for _ in 0..600 {
        let f = shut.frame(0, false, false, &mut r2);
        assert!(f.thunder.is_empty() && f.sun.is_none());
    }
    assert_eq!((shut.slots[3].flash, shut.slots[3].dark, shut.slots[3].thunder), (0, 0, 0), "timers hold");
    assert_eq!(shut.flash_mask, 0, "no slot draws");
    assert!(shut.globals.super_happy, "the SuperHappyLightningFlag clear sits inside the gate [M :93]");
    assert_eq!(r2.rand(1 << 15), Rng::new(0x51a7).rand(1 << 15), "no Rand_Rand draw while shut");
    // Reopening resumes exactly where the timers held: the same first frame the open sky had.
    shut.frame(0, false, true, &mut r2);
    assert_eq!(shut.slots[3].flash, 5);
}

#[test]
fn the_1b_counter_and_sun_hold_while_the_gate_is_shut() {
    let mut sky = Sky::default();
    // 0700001b: SKY_StarField slot 0, SKY_Earth slot 1, group 0, glist flags 0x222 [D].
    sky.slots[0] = SkySlot { glist: 1, flags: 0x222, ..Default::default() };
    sky.slots[1] = SkySlot { glist: 2, flags: 0x222, ..Default::default() };
    let mut rng = Rng::new(1);
    let f = sky.frame(0x0700_001b, false, true, &mut rng);
    assert_eq!(sky.rotating.count, 2, "two passes per open frame");
    // The sun the frame leaves is the second (Earth) pass's: count 2, identity base, zero slot rotation.
    let want = rotating_sky_sun([0.0; 3], mul(2.0, 0.00069444446));
    assert_eq!(f.sun.map(|v| v.map(f32::to_bits)), Some(want.map(f32::to_bits)));
    for _ in 0..100 { assert_eq!(sky.frame(0x0700_001b, false, false, &mut rng).sun, None); }
    assert_eq!(sky.rotating.count, 2, "no count while the gate is shut");
    // Paused (gate open): the block still rewrites the sun, the count holds [M :186-196, the
    // increment alone sits under GameState[0x50] == 0].
    let p = sky.frame(0x0700_001b, true, true, &mut rng);
    assert_eq!(sky.rotating.count, 2);
    assert_eq!(p.sun.map(|v| v.map(f32::to_bits)), Some(want.map(f32::to_bits)));
    // Sister: another level never writes the sun.
    assert_eq!(sky.frame(0x0700_000a, false, true, &mut rng).sun, None);
}

#[test]
fn the_1b_sun_is_camera_independent() {
    // The base the block copies is CONSTANT_MAT_IDENT (0x2d6230), so the vector is a pure function of
    // the count: spherical(1000, 0x409f2c8e, 0xbf29c920) rotated by RotMatrix([count/1440, 0, 0]).
    let spin = mul(1200.0, 0.00069444446);
    let v = rotating_sky_sun([0.0; 3], spin);
    // The rotated golden of sky_rotation.rs (scripts/sky_golden.py), same count.
    assert_eq!(v.map(f32::to_bits), [0xc43e_4a3f, 0xc40d_3c1b, 0xc39f_477d]);
    assert_eq!(MAT_IDENT, [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
}

fn node(flags: u32) -> CelNode { CelNode { min: [0.0; 3], max: [0.0; 3], flags, sphere: ([0.0; 3], 0.0) } }

#[test]
fn the_view_sees_sky_when_one_listed_cel_lacks_the_bit() {
    let g = CelGraph {
        cells: vec![node(0x804c047), node(0x1804c047), node(0x1804c023), node(0x8019)],
        portals: Vec::new(), room_to_cell: Vec::new(), dropped_portals: 0, chains: vec![Vec::new(); 4],
        own_cells_main: Vec::new(), portal_doors: Vec::new(),
    };
    assert_eq!(CEL_NO_SKY, 0x1000_0000);
    assert!(!chain_sees_sky(&g, &[1, 2]), "every listed room carries the bit: no sky");
    assert!(chain_sees_sky(&g, &[1, 0]), "one sky room is enough (the OR)");
    assert!(!chain_sees_sky(&g, &[]), "an empty list leaves the byte cleared");
}
