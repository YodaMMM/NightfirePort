//! The mounted laser's beam rides the matrix the gun draws with (0700000d's mount #225).
//!
//! [M `GunImp_Update` 0x1320d8] the fire direction is `Mat_GetDir(dir, gun_cel+0x90)` (:325 —
//! `Mat_GetDir` 0x1e3398 reads the matrix's third row un-negated, the disassembly), and the
//! muzzle is `TurretGuns[+0x00]`'s 12-byte offset transformed by the SAME cel's matrix rows
//! with its +0xC0 translation added (:196-211). The cel the aim drives is
//! `base·RotY(yaw_s)·RotX(pitch_s)` (:165-172 — disassembly: `Concat(src, dst)` computes
//! `dst = src·dst`, and `Mat_CopyRot` copies only rows 0-2, keeping the destination's own
//! translation row). The beam quad is `PositionBeam(muzzle, ray hit / muzzle+dir·1000)`
//! (:372-399), so the beam's origin and axis must come from the same matrix the gun draws
//! with (`GunImp::barrel_matrix`) — an earlier port derived them from separate trig that
//! dropped the mount's base yaw (`barrel_dir` used `yaw_s` alone), which threw the beam off
//! the drawn barrel for any mount whose base yaw is not zero.
//!
//! [D placements #225] the 0d mount's rotation is identity (the test pins `yaw_base == 0`
//! through the sim), so the identity base also pins the general case's arithmetic below.
//! Opt-in via NF_ISO.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const MOUNT: [f32; 3] = [137.2719, 9.331467, -324.6627]; // [D placements #225]

/// Mount the deck gun (the aim test's walk), returning the scene.
fn mounted() -> Option<nf_view::scene::Scene> {
    let iso = iso()?;
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..2000 {
            sim.step_bond(Default::default(), false, false, false);
            if !sim.disabled { break; }
        }
        sim.bond.pos = [137.27, 9.33, -323.0];
        sim.bond.yaw = std::f32::consts::PI;
        sim.bond.pitch = 0.0;
        for _ in 0..6 {
            sim.step_bond(Default::default(), false, false, true);
            if sim.drones.props.gunimps.iter().any(|g| g.active) { break; }
        }
    }
    let ok = scene.sim.as_ref().unwrap().drones.props.gunimps.iter().any(|g| g.active);
    if !ok { return None; }
    Some(scene)
}

/// The expected fire muzzle: the TurretGuns offset through `barrel_matrix` — written out here
/// from the game's transform (offset against the matrix's rotation rows, plus the translation
/// row) rather than through `GunImp::muzzle`, so the test does not share the code under test.
fn expected_muzzle(g: &nf_game::gunimp::GunImp) -> nf_game::world::V3 {
    let m = g.barrel_matrix();
    let o = nf_game::gunimp::TURRET_GUNS[g.turret as usize].muzzle;
    [
        m[3][0] + m[0][0] * o[0] + m[1][0] * o[1] + m[2][0] * o[2],
        m[3][1] + m[0][1] * o[0] + m[1][1] * o[1] + m[2][1] * o[2],
        m[3][2] + m[0][2] * o[0] + m[1][2] * o[1] + m[2][2] * o[2],
    ]
}

/// The expected fire direction: `barrel_matrix`'s third row, normalised (Mat_GetDir 0x1e3398).
fn expected_dir(g: &nf_game::gunimp::GunImp) -> nf_game::world::V3 {
    let m = g.barrel_matrix();
    let f = [m[2][0], m[2][1], m[2][2]];
    let l = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
    [f[0] / l, f[1] / l, f[2] / l]
}

fn close(a: f32, b: f32) -> bool { (a - b).abs() < 1e-3 }

#[test]
fn the_beam_starts_on_the_drawn_muzzle_and_rides_the_barrel_axis_at_several_aim_angles() {
    let Some(mut scene) = mounted() else { eprintln!("NF_ISO not set or the mount is unreachable — skipping"); return };
    let weapons = scene_weapons();
    {
        let s = scene.sim.as_ref().unwrap();
        let g = &s.drones.props.gunimps[0];
        assert!(close(g.pos[0], MOUNT[0]) && close(g.pos[1], MOUNT[1]) && close(g.pos[2], MOUNT[2]),
            "the mount sits at #225's placement translation [D placements]");
        assert_eq!(g.yaw_base, 0.0, "#225's placement rotation is identity, so the base yaw is 0 [D placements]");
    }
    // Several aim angles across the mount's range (the pitch clamp is [−π/2, +π/4] [M :132-135]).
    let aims: [(f32, f32); 6] = [
        (0.0, 0.0),
        (std::f32::consts::FRAC_PI_2, 0.0),
        (std::f32::consts::PI, 0.3),
        (-2.0, -0.5),
        (0.7, 0.78),
        (-3.0, 0.0),
    ];
    let sim = scene.sim.as_mut().unwrap();
    for (yaw, pitch) in aims {
        // Set the raw aim and snap the smoothed mirrors (the draw and the fire read the mirrors).
        let g = &mut sim.drones.props.gunimps[0];
        g.yaw = yaw; g.pitch = pitch; g.yaw_s = yaw; g.pitch_s = pitch;
        g.cooldown = 0.0;
        let before = sim.drones.player_bullets.len();
        sim.drones.gunimp_tick(&weapons, 0.0, 0.0, true, false, &mut sim.bond);
        let g = &sim.drones.props.gunimps[0];
        let Some((a, b)) = g.beam else { panic!("a held shot leaves the beam armed (yaw {yaw}, pitch {pitch})") };
        let m0 = expected_muzzle(g);
        assert!(close(a[0], m0[0]) && close(a[1], m0[1]) && close(a[2], m0[2]),
            "the beam's origin is the drawn muzzle (barrel_matrix · TurretGuns offset [M 0x1320d8:196-211]) \
             at yaw {yaw} pitch {pitch}: beam {a:?} vs expected {m0:?}");
        let d = expected_dir(g);
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
        assert!(len > 900.0, "the beam reaches the 1000-unit ray tail [M :373-381] (got {len})");
        assert!(close((b[0] - a[0]) / len, d[0]) && close((b[1] - a[1]) / len, d[1]) && close((b[2] - a[2]) / len, d[2]),
            "the beam's direction is the drawn barrel's forward row [M :325 Mat_GetDir] \
             at yaw {yaw} pitch {pitch}: beam {:?} vs expected {d:?}", [b[0] - a[0], b[1] - a[1], b[2] - a[2]]);
        // The bullet flies from the same drawn muzzle [M :327-331 Bullet_init(muzzle, dir)].
        let bl = &sim.drones.player_bullets[before];
        assert!(close(bl.pos[0], m0[0]) && close(bl.pos[1], m0[1]) && close(bl.pos[2], m0[2]),
            "the round spawns on the drawn muzzle [M :327-331]: {:?} vs {m0:?}", bl.pos);
    }
}

/// The weapon table the sim needs for the tick (the same read the view's step uses).
fn scene_weapons() -> nf_formats::weapon::WeaponData {
    let iso = iso().expect("NF_ISO");
    let image = nf_formats::iso9660::IsoImage::open(&iso).expect("iso");
    let elf = image.read_file(image.find("/ACTION.ELF").expect("elf")).expect("elf");
    nf_formats::weapon::WeaponData::from_action_elf(&elf).expect("weapons")
}
