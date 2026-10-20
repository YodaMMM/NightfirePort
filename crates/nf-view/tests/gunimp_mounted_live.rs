//! gunimp-mounted-pose: the mounted turret carries Bond's view through the real input path.
//!
//! The game's mounted camera is NOT a special turret camera: `Player_PositionCamera` 0x18f5d8
//! handles cam mode 0xe in the SAME case as mode 0 (lines 96-97) — Bond's own first-person eye.
//! What moves the view is `GunImp_Update` 0x1320d8:162-194, every mounted tick:
//! - the player object's matrix ← base·RotY(yaw_s) [M :175] — Bond's yaw follows the turret;
//! - the player's position ← base·RotY(yaw_s)·(0,0,seat) + base translation, written to +0xC0
//!   then copied to +0x30..+0x3C [M :176-192]; the seat offset's z comes from
//!   `*(float *)(celglist + 0x28) * 0.75` [M `GunImp_Create` 0x131f20:77-81] — the glist's
//!   bounds word at +0x28 = the object record's min.z (the bounds block copies at the +8 shift
//!   the verified centre/radius rows show [M glist.md]; [S] for that shift itself) — #225's
//!   min.z = −1.4362069 [D placements box] → seat = −1.0771552;
//! - the view pitch `BLData+0x8A8 = −pitch_s·0.5` [M :194];
//! - the barrel matrix = base·RotY(yaw_s)·RotX(pitch_s) [M :165-172] — exposed for the draw.
//! The aim: yaw_t −= stick-x·0.015/tick (unclamped), pitch_t −= stick-y·0.015/tick clamped
//! [−π/2, +π/4] [M :124-135]; both ease 0.15/tick into the smoothed mirrors [M :136-161].
//! The dismount (`GunImp_Deactivate` 0x132c18) restores control (cam mode 0, the saved
//! substate, `Player_Enable`, `Camera_CalcViewAngles`) and KEEPS the carried pose — the game
//! writes no object position in the deactivate [M 0x132c18:33-47].
//!
//! The mount: 0d #225 `PowerStationGunImpBase` (137.27, 9.33, −324.66, identity rotation
//! [D placements]). The catwalk deck floors at 7.98 around it [D coll-floor probes], so
//! standing Bond's position rides at ≈ 9.1 — the main agent's PCSX2 spot [L]. The test places
//! him on the deck south of the mount and walks the final approach through the real use pick.
//!
//! Run: `NF_ISO=… cargo test -p nf-view --test gunimp_mounted_live -- --test-threads=1`

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const MOUNT: [f32; 3] = [137.2719, 9.331467, -324.6627]; // [D placements #225]
const SEAT: f32 = -1.0771552; // [D min.z -1.4362069 × 0.75; M 0x131f20:81]

/// Bond placed on the catwalk south of the mount, facing it (+z → the mount is at LOWER z,
/// so the facing is −z: yaw π), walked the final approach until the use pick reaches.
fn approach_and_mount(scene: &mut nf_view::scene::Scene) {
    {
        let sim = scene.sim.as_mut().unwrap();
        // On the 7.98 deck south of the mount (standing ≈ 9.1 [L]; D coll-floor 7.9847).
        sim.bond.pos = [MOUNT[0], 9.2, MOUNT[2] - 2.6];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = 0.0; // facing +z, at the mount
        sim.bond.pitch = -0.19; // down at the turret head (the eye ≈ 9.9, the head 9.33)
    }
    for _ in 0..20 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    // Walk in: hold forward until the turret is under the head ray (the pick's 2.2 reach).
    for _ in 0..80 {
        let d = {
            let sim = scene.sim.as_ref().unwrap();
            let head = sim.bond.head();
            let to = [MOUNT[0] - head[0], MOUNT[1] - head[1], MOUNT[2] - head[2]];
            (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt()
        };
        if d < 1.7 { break; }
        let mut input = Input::default();
        input.forward = 1.0;
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, true); // E mounts
    // The mount lands in the use phase (after that tick's imp tick) — one more tick carries.
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
}

#[test]
fn mounted_stick_right_turns_the_view_and_dmount_restores() {
    let mut scene = nf_view::load(&iso(), "0700000d.bin", false).expect("load 0700000d");
    // The intro must not own the player when the mount test starts.
    for _ in 0..3100 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    approach_and_mount(&mut scene);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.props.gunimps[0].active, "the use pick mounted the turret [M GunImp_Activate 0x132f00]");
        // The carried pose at yaw_s = pitch_s = 0: the seat straight behind the base [M 0x1320d8:175-194].
        let p = sim.bond.pos;
        assert!((p[0] - MOUNT[0]).abs() < 0.02, "seat x = the base x at yaw 0, got {p:?}");
        assert!((p[1] - MOUNT[1]).abs() < 0.02, "seat y = the base y, got {p:?}");
        assert!((p[2] - (MOUNT[2] + SEAT)).abs() < 0.02, "seat z = base + the seat offset, got {p:?}");
        assert!((sim.bond.yaw - 0.0).abs() < 1e-4, "Bond's yaw = the turret's (base yaw + yaw_s = 0) [M :175]");
        assert!((sim.bond.pitch - 0.0).abs() < 1e-4, "the view pitch = −pitch_s·0.5 = 0 [M :194]");
    }

    // Stick-right (turn +1) for 60 ticks: the turret's yaw target runs to −0.9 and Bond's yaw
    // follows it exactly (the carried yaw IS yaw_base + yaw_s [M :175]).
    for _ in 0..60 {
        let mut input = Input::default();
        input.turn = 1.0;
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        let g = &sim.drones.props.gunimps[0];
        // The recurrence iterated: the target moves −0.015/tick [M :125] and the smoothed mirror
        // eases 0.15/tick toward it [M :136-161] — a moving target lags, so pin the ITERATED
        // value, not a settled closed form.
        let mut want = 0.0f32;
        {
            let mut tgt = 0.0f32;
            for _ in 0..60 { tgt -= 0.015; want += (tgt - want) * 0.15; }
        }
        assert!((g.yaw - (-0.9)).abs() < 1e-4, "the yaw target ran to −0.9 [M :125]");
        assert!((g.yaw_s - want).abs() < 1e-4, "the smoothed yaw eased to {want:.4}, got {}", g.yaw_s);
        assert!((sim.bond.yaw - (g.yaw_s)).abs() < 1e-4,
            "Bond's yaw rides the turret's smoothed yaw [M :175], got {} vs {}", sim.bond.yaw, g.yaw_s);
        assert!(sim.bond.yaw < -0.6, "stick-right turned the view with the turret");
        // The seat orbits: the carried position matches the pose getter exactly [M :176-192].
        let (pos, yaw, pitch) = g.mounted_pose();
        assert!((sim.bond.pos[0] - pos[0]).abs() < 1e-5 && (sim.bond.pos[2] - pos[2]).abs() < 1e-5,
            "the carried position = base·RotY·seat + base translation [M :176-192]");
        let _ = (yaw, pitch);
        assert!((sim.bond.pos[2] - (MOUNT[2] + SEAT * g.yaw_s.cos())).abs() < 1e-4,
            "the seat z = base.z + seat·cos(yaw_s) [M :176-192]");
    }

    // Pitch: look +1 for 60 ticks → the pitch target −0.9 (inside [−π/2, +π/4]) and the view
    // pitch = −pitch_s·0.5 EXACTLY [M :194].
    for _ in 0..60 {
        let mut input = Input::default();
        input.look = 1.0;
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        let g = &sim.drones.props.gunimps[0];
        assert!((g.pitch - (-0.9)).abs() < 1e-4, "the pitch target −0.9 inside the clamp [M :130-137]");
        let mut want = 0.0f32;
        {
            let mut tgt = 0.0f32;
            for _ in 0..60 { tgt -= 0.015; want += (tgt - want) * 0.15; }
        }
        assert!((g.pitch_s - want).abs() < 1e-4, "the smoothed pitch eased to {want:.4}, got {}", g.pitch_s);
        assert!((sim.bond.pitch - (-g.pitch_s * 0.5)).abs() < 1e-6,
            "the view pitch = −pitch_s·0.5 [M :194], got {} vs {}", sim.bond.pitch, -g.pitch_s * 0.5);
    }

    // The barrel matrix getter: the forward row IS barrel_dir; the translation the base [M :165-172].
    {
        let sim = scene.sim.as_ref().unwrap();
        let g = &sim.drones.props.gunimps[0];
        let m = g.barrel_matrix();
        let d = g.barrel_dir();
        assert!((m[2][0] - d[0]).abs() < 1e-6 && (m[2][1] - d[1]).abs() < 1e-6 && (m[2][2] - d[2]).abs() < 1e-6,
            "the barrel matrix's forward row = barrel_dir [M :165-172]");
        assert!((m[3][0] - MOUNT[0]).abs() < 1e-5 && (m[3][1] - MOUNT[1]).abs() < 1e-5 && (m[3][2] - MOUNT[2]).abs() < 1e-5,
            "the barrel matrix sits at the base translation [D #225]");
    }

    // Dismount (E): control returns and the pose is KEPT [M 0x132c18:33-47].
    let kept = scene.sim.as_ref().unwrap().bond.pos;
    let kept_yaw = scene.sim.as_ref().unwrap().bond.yaw;
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, true);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(!sim.drones.props.gunimps[0].active, "the dismount deactivated the imp [M 0x132c18]");
        let p = sim.bond.pos;
        // The deactivate writes no position [M 0x132c18:33-47]; the resumed walk's first tick
        // settles him (the seat rides 1.35 above the deck — the fall + push ≈ 0.06/tick).
        assert!((p[0] - kept[0]).abs() < 0.2 && (p[2] - kept[2]).abs() < 0.2,
            "the carried pose is kept on dismount (the settle aside) [M 0x132c18:33-47], got {p:?} vs {kept:?}");
        assert!((sim.bond.yaw - kept_yaw).abs() < 0.01, "the yaw kept [M 0x132c18]");
    }
    // The walk resumed: forward input moves Bond again (control restored [M 0x132c18:36-39]).
    let before = scene.sim.as_ref().unwrap().bond.pos;
    for _ in 0..30 {
        let mut input = Input::default();
        input.forward = 1.0;
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        let moved = (sim.bond.pos[0] - before[0]).powi(2) + (sim.bond.pos[2] - before[2]).powi(2);
        assert!(moved > 0.25, "Bond walks after the dismount (control restored [M 0x132c18:36-39]), moved {moved:.3}");
    }
}

/// Sister: an unmounted Bond is unaffected by the turret — the same stick-right runs his NORMAL
/// look rate (not the turret's 0.015/tick), no imp activates, and the pose never rides the seat.
#[test]
fn unmounted_bond_is_unaffected() {
    let mut scene = nf_view::load(&iso(), "0700000d.bin", false).expect("load 0700000d");
    for _ in 0..3100 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [MOUNT[0], 9.2, MOUNT[2] - 2.6];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = 0.0;
    }
    for _ in 0..20 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let yaw0 = scene.sim.as_ref().unwrap().bond.yaw;
    for _ in 0..60 {
        let mut input = Input::default();
        input.turn = 1.0;
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.props.gunimps[0].active, "no mount without the use key");
    let d = (sim.bond.yaw - yaw0).abs();
    assert!(d > 0.5, "the normal look turned the view (yaw moved {d:.3})");
    // The turret's own aim state never moved (the imp is idle [M 0x1320d8:113-118 — the update
    // returns unless state 1]).
    assert!((sim.drones.props.gunimps[0].yaw_s).abs() < 1e-6, "the turret's smoothed yaw never moved");
    // The seat never carried him: his position is off the seat line (the walk's own physics).
    let seat_z = MOUNT[2] + SEAT;
    assert!((sim.bond.pos[2] - seat_z).abs() > 0.05, "Bond is not parked on the seat");
}
