//! The gun emplacement's gun child draws and follows the mount (0700000d's laser mount #225).
//!
//! `GunImp_Create` 0x131f20:85-89 creates the gun child from `TurretGuns[ext+0x9d].art` (SP: entry
//! 0xb, art 0x02000b50) and `GunImp_Update` 0x1320d8:162-173 re-poses every tick: the base object at
//! `base · RotY(yaw_s)`, the child at `base · RotY(yaw_s) · RotX(pitch_s)` [M]. The port draws both
//! from posed slots (`gunimp::GunImp::base_matrix` / `barrel_matrix`).

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const MOUNT: [f32; 3] = [137.2719, 9.331467, -324.6627]; // [D placements #225]

/// (visible vertex count, their centroid) of the posed slot `slot`.
fn slot_verts(scene: &nf_view::scene::Scene, slot: usize) -> (usize, [f32; 3]) {
    let posed = scene.posed_vertices(None, 0.0, Default::default());
    let mut n = 0;
    let mut c = [0.0f32; 3];
    for (v, p) in scene.char_verts.iter().zip(posed.iter()) {
        if v.door.map(usize::from) == Some(slot) && p.pos[1] > -5000.0 {
            n += 1;
            for k in 0..3 { c[k] += p.pos[k]; }
        }
    }
    if n > 0 { for k in 0..3 { c[k] /= n as f32; } }
    (n, c)
}

#[test]
fn the_laser_mount_draws_its_gun_and_the_gun_turns_with_the_mount() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    let (base, art) = {
        let s = scene.sim.as_ref().unwrap();
        assert_eq!(s.drones.props.gunimps.len(), 1, "0700000d places one gun emplacement [D #225]");
        let locks = s.drones.locks.len();
        let base = s.drones.doors.list.len() + s.drones.sss.list.len() + s.drones.breaks.list.len() + 2 * locks;
        let art = s.drones.props.gunimps[0].art_slot().expect("the SP turret's art is in TURRET_ARTS");
        (base, art)
    };
    let gun = base + 1 + art;
    let (n_base, _) = slot_verts(&scene, base);
    let (n_gun, c0) = slot_verts(&scene, gun);
    assert!(n_base > 0, "the mount's base draws");
    assert!(n_gun > 0, "the gun child (TurretGuns[0xb].art 0x02000b50) draws");
    let d = ((c0[0] - MOUNT[0]).powi(2) + (c0[2] - MOUNT[2]).powi(2)).sqrt();
    assert!(d < 2.0, "the gun sits on the mount ({c0:?} vs {MOUNT:?})");
    // Sister: the other arts' slots stay hidden (only the mount's own art shows).
    let other = base + 1 + (art + 1) % nf_game::gunimp::TURRET_ARTS.len();
    assert_eq!(slot_verts(&scene, other).0, 0, "a gun art that is not this mount's stays hidden");
    // The gun turns with the mount: a quarter turn of yaw moves the gun's centroid around the base.
    scene.sim.as_mut().unwrap().drones.props.gunimps[0].yaw_s = std::f32::consts::FRAC_PI_2;
    let (_, c1) = slot_verts(&scene, gun);
    let moved = ((c1[0] - c0[0]).powi(2) + (c1[2] - c0[2]).powi(2)).sqrt();
    assert!(moved > 0.05, "the gun child follows the mount's yaw (moved {moved})");
}
