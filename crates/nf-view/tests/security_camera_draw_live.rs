//! The security cameras draw (0x28 sensors, 07000014's security centre).
//!
//! `Sensor_Create` 0x1af010 keeps the placement's glist on the sensor object (obj+0xD8) at the
//! swept/tracked pose obj+0x90, and creates the mount proxy (param 7) that `Sensor_SetBeam`
//! 0x1af398:50-54 re-aligns upright to the pose's direction every tick [M]. The port draws both
//! from posed slots after the gun emplacements.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn slot_verts(scene: &nf_view::scene::Scene, slot: usize) -> (usize, [f32; 3], f32) {
    let posed = scene.posed_vertices(None, 0.0, Default::default());
    let (mut n, mut c, mut top) = (0, [0.0f32; 3], f32::MIN);
    for (v, p) in scene.char_verts.iter().zip(posed.iter()) {
        if v.door.map(usize::from) == Some(slot) && p.pos[1] > -5000.0 {
            n += 1;
            for k in 0..3 { c[k] += p.pos[k]; }
            top = top.max(p.pos[1]);
        }
    }
    if n > 0 { for k in 0..3 { c[k] /= n as f32; } }
    (n, c, top)
}

#[test]
fn the_security_cameras_draw_their_body_and_mount() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let (cam_base, pos) = {
        let s = scene.sim.as_ref().unwrap();
        assert_eq!(s.drones.sensors.list.len(), 14, "07000014's security centre: 14 cameras [D census]");
        let base = s.drones.doors.list.len() + s.drones.sss.list.len() + s.drones.breaks.list.len()
            + 2 * s.drones.locks.len() + (1 + nf_game::gunimp::TURRET_ARTS.len()) * scene.gunimp_placements;
        (base, s.drones.sensors.list[0].pos)
    };
    let (n_body, c_body, _) = slot_verts(&scene, cam_base);
    let (n_mount, _, mount_top) = slot_verts(&scene, cam_base + 1);
    assert!(n_body > 0, "camera 0's body draws (it never did: kind 0x28 was outside the misc sweep)");
    assert!(n_mount > 0, "camera 0's mount (param 7, 0x02000652) draws");
    let d = ((c_body[0] - pos[0]).powi(2) + (c_body[1] - pos[1]).powi(2) + (c_body[2] - pos[2]).powi(2)).sqrt();
    assert!(d < 1.0, "the body sits at the camera ({c_body:?} vs {pos:?})");
    // The body follows the sweep pose; the mount stays upright (its top does not move when the
    // camera pitches down).
    {
        let s = scene.sim.as_mut().unwrap();
        let m = &mut s.drones.sensors.list[0].matrix;
        let (sp, cp) = (0.5f32).sin_cos();
        let (up, fw) = (m[1], m[2]);
        for k in 0..3 { m[1][k] = up[k] * cp - fw[k] * sp; m[2][k] = fw[k] * cp + up[k] * sp; }
    }
    let (_, c_body2, _) = slot_verts(&scene, cam_base);
    let (_, _, mount_top2) = slot_verts(&scene, cam_base + 1);
    let moved = ((c_body2[0] - c_body[0]).powi(2) + (c_body2[1] - c_body[1]).powi(2) + (c_body2[2] - c_body[2]).powi(2)).sqrt();
    assert!(moved > 1e-3, "the body follows the pose (moved {moved})");
    assert!((mount_top2 - mount_top).abs() < 1e-3, "the mount stays upright under a pitch ({mount_top} -> {mount_top2})");
}
