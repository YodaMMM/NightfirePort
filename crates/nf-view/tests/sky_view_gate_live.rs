//! G-019: the sky's state only runs while the viewer sees sky. `parseentity_fixup_entity` 0x1d00c0
//! gives a room cel with parameter 3 set the flags word 0x1804c023 / 0x1804c047 (bit 0x10000000);
//! `Vision_Portal_Recurse` 0x1e86e8:33-41 sets `viewer+0x20` when some VisCelList cel lacks it, and
//! `View_DrawSky` 0x1e5ad0:91 skips its whole loop (lightning timers, thunder, the 1b count) without
//! it. On 0x0700001B the loop rewrites `SunVector` every frame [M :186-196] on the identity base.
//! Opt-in: set `NF_ISO`.
#[path = "common/walker.rs"]
mod walker;
use walker::Walker;
use nf_formats::ps2float::mul;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn no_sky_rooms_carry_the_fixup_bit() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    // 0700000a: 23 of its 30 room placements carry parameter 3; 0700000b 15 of 23; 0700001b none [D].
    for (lvl, n) in [("0700000a.bin", 23), ("0700000b.bin", 15), ("0700001b.bin", 0)] {
        let w = Walker::new(nf_view::load(&iso(), lvl, false).expect("load"));
        let s = w.scene.sim.as_ref().unwrap();
        let got = s.cel_graph.cells.iter().filter(|c| c.flags & 0x1000_0000 != 0).count();
        assert_eq!(got, n, "{lvl}: rooms flagged 0x1804c0xx");
        // Sister: the room bit 0x40000 stays on every flagged cel (the bit only adds).
        assert!(s.cel_graph.cells.iter().filter(|c| c.flags & 0x1000_0000 != 0).all(|c| c.flags & 0x0804_c000 == 0x0804_c000));
    }
}

#[test]
fn the_storm_timers_hold_while_bond_sees_no_sky() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso(), "0700000a.bin", false).expect("load"));
    w.tick(Default::default(), false);
    // The spawn view reaches a sky room: the dark timer counts down.
    let d0 = w.scene.sim.as_ref().unwrap().drones.sky.slots[3].dark;
    w.wait(60);
    let s = w.scene.sim.as_ref().unwrap();
    assert!(!s.drones.sky.no_sky_view);
    assert!(s.drones.sky.slots[3].dark < d0, "open sky: the dark interval runs");
    // Cel 8, the lower room at (-0.37, -14.81, -15.94): every listed cel is a no-sky room.
    {
        let s = w.scene.sim.as_mut().unwrap();
        s.bond.pos = [-0.3658471, -14.813901, -15.937269];
        s.bond.vel = [0.0; 3];
    }
    w.wait(4);
    let held = { let s = w.scene.sim.as_ref().unwrap(); assert!(s.drones.sky.no_sky_view, "cel 8 sees no sky"); s.drones.sky.slots[3] };
    w.wait(600);
    let s = w.scene.sim.as_ref().unwrap();
    let now = s.drones.sky.slots[3];
    assert_eq!((now.flash, now.dark, now.thunder), (held.flash, held.dark, held.thunder), "timers hold indoors");
    assert_eq!(s.drones.sky.flash_mask, 0, "nothing of the sky draws");
}

#[test]
fn the_1b_sun_follows_the_spin() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso(), "0700001b.bin", false).expect("load"));
    for _ in 0..300 { w.tick(Default::default(), false); }
    let s = w.scene.sim.as_ref().unwrap();
    let count = s.drones.sky.rotating.count;
    assert!(count >= 600);
    // The frame's last pass (SKY_Earth, slot 1, zero rotation [D]) leaves spin(count) applied to the
    // fixed spherical sun, on CONSTANT_MAT_IDENT (built here from the pre-existing pieces).
    let (r, th, ph) = nf_game::sky::ROTATING_SKY_SUN_ARGS;
    let mut v = nf_game::sky::spherical_2_cartesian(r, th, ph);
    let id = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let m = nf_game::sky::rotating_sky_matrix(&id, [0.0; 3], mul(count as f32, 0.00069444446));
    nf_game::sky::apply_matrix_lvi(&m, &mut v);
    assert_eq!(s.drones.sun_vector.map(f32::to_bits), v.map(f32::to_bits));
    // Sister: other levels keep their load-time sun (0700000a: none).
    let w2 = Walker::new(nf_view::load(&iso(), "0700000a.bin", false).expect("load"));
    let mut w2 = w2;
    for _ in 0..30 { w2.tick(Default::default(), false); }
    assert_eq!(w2.scene.sim.as_ref().unwrap().drones.sun_vector, [0.0; 3]);
}
