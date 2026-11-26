//! bond-hitboxes: the guards' rounds test Bond's 16 body boxes, not a cylinder.
//!
//! The game's rule [M]: a drone round's pick reaches the player through `Collide_Intersect`
//! 0x1ebaa8, which hands an object with `obj+0xFA & 8` (the player: 8 [L ee_start_05.bin]) to
//! `Collide_Jointy` 0x1ec180; its ray arm (0x201) walks `AnimGetCollData` 0x1c4ac8's boxes, the
//! `HumanCollisionBones` 0x2c6be8 table (16 rows + a -1 terminator [D ACTION.ELF]) on the player's
//! posed skeleton, and returns the FIRST box in list order the ray enters. The part id it stores
//! (HITDATA+0x52) only matters in multiplayer: `Player_HandlePain` 0x1902c8 applies Plr_DMod_Head /
//! UpperLimb / LowerLimb inside the `MPSettings+0x180 != 0` arm; the single-player arm scales by
//! difficulty alone [M 0x1902c8:28-49]. In single player Bond's anim set is one pose per stance:
//! 0x6000a00 standing, 0x60009ff crouched [D 0x2b91b0 / 0x2b91d8, M PlayerAnimSetInitNormal
//! 0x18fcd8 / Crouch 0x18fea0 force the 999 selector outside MP].
//!
//! The standing pin is the live palette: the player's AnimObject (+0xB8 bone matrices) in two
//! real-game EE RAM snapshots, boxes rebuilt with AnimGetCollData's own formula [L ee_start_05.bin,
//! ee_start_0700000a.bin], and the crouched pin is a live read of `HitList2` at `AnimGetCollData`'s
//! return for the player while the 07000012 start guards fired at a crouched Bond [L PCSX2
//! 2026-11-21, difficulty 1, Cross = crouch, obj+0xF6 = 4]. The port's `bond_hit_boxes`
//! reproduces all three to 2e-3.

use nf_game::damage::{first_box_hit, HitBox};
use nf_game::drone::{bond_hit_boxes, Player};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn boxes_at(scene: &nf_view::scene::Scene, pos: [f32; 3], yaw: f32, crouched: bool) -> Vec<HitBox> {
    let sim = scene.sim.as_ref().expect("sim");
    let p = Player { pos, yaw, crouched, ..Default::default() };
    bond_hit_boxes(&sim.data(), &p)
}

fn centre(b: &HitBox) -> [f32; 3] { [b.m[3][0], b.m[3][1], b.m[3][2]] }

/// The box's vertical reach from its centre (the half extents along its axes, projected on Y).
fn reach_y(b: &HitBox) -> f32 { (0..3).map(|k| b.m[k][1].abs() * b.half[k]).sum() }

fn check_live(level: &str, pos: [f32; 3], yaw: f32, crouched: bool, live: &[(i16, [f32; 3])]) {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let scene = nf_view::load(&iso, level, false).expect("load");
    let bx = boxes_at(&scene, pos, yaw, crouched);
    assert_eq!(bx.len(), 16, "16 HumanCollisionBones rows [D 0x2c6be8]");
    for (b, &(part, c)) in bx.iter().zip(live) {
        assert_eq!(b.part, part, "row order [D 0x2c6be8]");
        let got = centre(b);
        for k in 0..3 {
            assert!((got[k] - c[k]).abs() < 2e-3, "{level} box {part} centre {got:?} vs live {c:?} [L]");
        }
    }
}

#[test]
fn standing_boxes_match_the_live_palette_on_05() {
    // [L ee_start_05.bin]: obj+0x30 / obj+0x54 and the boxes AnimGetCollData builds from the
    // AnimObject +0xB8 palette (centre: bone 5 + (0, 5.y - 4.y, 0) for row 0, else the bone / other
    // midpoint).
    check_live("07000005.bin", [104.9540, 24.9633, 108.82527], -2.146_755, false, &[
        (5, [104.8813, 25.5373, 108.6303]), (4, [104.9244, 25.2390, 108.7433]), (53, [104.8801, 24.9058, 108.7998]),
        (54, [104.6737, 24.6985, 108.6895]), (55, [104.5490, 24.3017, 108.6604]), (56, [104.5057, 24.0181, 108.6848]),
        (49, [105.0119, 24.9115, 108.7676]), (50, [105.1539, 24.6729, 108.6351]), (51, [105.2793, 24.2688, 108.6835]),
        (52, [105.3527, 24.0184, 108.7534]), (20, [104.5938, 25.2805, 108.6733]), (21, [104.4488, 25.2838, 108.4754]),
        (23, [104.4283, 25.3599, 108.2796]), (32, [105.0783, 25.2701, 108.4876]), (35, [104.9543, 25.2616, 108.4395]),
        (39, [104.8059, 25.3218, 108.4822])]);
}

#[test]
fn standing_boxes_match_the_live_palette_on_0a() {
    // [L ee_start_0700000a.bin], a second level and facing.
    check_live("0700000a.bin", [-3.931_852_6, 1.089_111, 10.397_76], 0.017_453_29, false, &[
        (5, [-4.0528, 1.6632, 10.5670]), (4, [-3.9833, 1.3649, 10.4681]), (53, [-3.9116, 1.0316, 10.4733]),
        (54, [-3.8877, 0.8243, 10.7061]), (55, [-3.8420, 0.4275, 10.8257]), (56, [-3.7976, 0.1439, 10.8480]),
        (49, [-4.0121, 1.0373, 10.3820]), (50, [-4.2013, 0.7987, 10.3384]), (51, [-4.2313, 0.3946, 10.2073]),
        (52, [-4.2144, 0.1442, 10.1074]), (20, [-3.8565, 1.4063, 10.7813]), (21, [-3.9394, 1.4096, 11.0122]),
        (23, [-4.0903, 1.4857, 11.1387]), (32, [-4.2813, 1.3960, 10.4835]), (35, [-4.2519, 1.3874, 10.6132]),
        (39, [-4.1335, 1.4476, 10.7123])]);
}

#[test]
fn crouched_boxes_match_the_live_hit_list_on_12() {
    // [L] HitList2 (0x319da0) at AnimGetCollData's return, a0 = the player object, crouched at the
    // 07000012 start: obj+0x30 (22.4889, 0.6077, 19.1626), obj+0x54 -pi/2.
    check_live("07000012.bin", [22.488_895, 0.607_688_3, 19.162_645], -1.570_796_3, true, &[
        (5, [22.1951, 1.0653, 18.9157]), (4, [22.3629, 0.8134, 19.0452]), (53, [22.3966, 0.5908, 19.1874]),
        (54, [22.1026, 0.6447, 19.1729]), (55, [21.9580, 0.4712, 19.1683]), (56, [21.9551, 0.1902, 19.2044]),
        (49, [22.4780, 0.5477, 19.0875]), (50, [22.3848, 0.3296, 18.8867]), (51, [22.4848, 0.2393, 18.9234]),
        (52, [22.6874, 0.2402, 19.0945]), (20, [22.1014, 0.7954, 19.2861]), (21, [22.0141, 0.7301, 19.2769]),
        (23, [21.9041, 0.7515, 19.1033]), (32, [22.5631, 0.8074, 18.8777]), (35, [22.6610, 0.6187, 18.8151]),
        (39, [22.6253, 0.4649, 18.6827])]);
}

#[test]
fn the_body_spans_and_the_rays_the_old_cylinder_got_wrong() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "0700000c.bin", false).expect("load");
    let pos = [0.0, 10.0, 0.0];
    let stand = boxes_at(&scene, pos, 0.0, false);
    let crouch = boxes_at(&scene, pos, 0.0, true);
    assert_eq!((stand.len(), crouch.len()), (16, 16));
    let span = |bx: &[HitBox]| bx.iter().fold((f32::MAX, f32::MIN), |(lo, hi), b| {
        let (c, r) = (b.m[3][1] - pos[1], reach_y(b));
        (lo.min(c - r), hi.max(c + r))
    });
    // Standing: feet box bottoms 1.02 under obj+0x30, head box top 0.73 over it [L, the 05/0a rows].
    let (lo, hi) = span(&stand);
    assert!((lo + 1.021).abs() < 2e-3 && (hi - 0.734).abs() < 2e-3, "standing span {lo} .. {hi}");
    // Crouched (0x60009ff): 0.55 under to 0.61 over obj+0x30 (which itself sits ~0.48 lower) [L 12 crouched].
    let (lo, hi) = span(&crouch);
    assert!((lo + 0.554).abs() < 2e-3 && (hi - 0.608).abs() < 2e-3, "crouched span {lo} .. {hi}");

    // Rays across Bond (yaw 0: he faces +z), as a guard's round would cross him side-on.
    let across = |bx: &[HitBox], y: f32, z: f32| first_box_hit([pos[0] - 5.0, pos[1] + y, z], [10.0, 0.0, 0.0], bx);
    let cyl = |v: &nf_game::bullet::PlayerVolume, y: f32, z: f32|
        nf_game::bullet::segment_cylinder([pos[0] - 5.0, pos[1] + y, z], [pos[0] + 5.0, pos[1] + y, z], v);
    // The old rule: a cylinder r 0.25 from 1.03 under obj+0x30 to the eye + 0.17 (eye +0.70
    // standing, +0.25 crouched with the 0.45 crouch drop).
    let old_stand = nf_game::bullet::PlayerVolume { feet: [pos[0], pos[1] - 1.03, pos[2]], height: 0.70 + 0.17 + 1.03, radius: 0.25 };
    let old_crouch = nf_game::bullet::PlayerVolume { feet: [pos[0], pos[1] - 1.03, pos[2]], height: 0.25 + 0.17 + 1.03, radius: 0.25 };
    // 1. Over the standing head (+0.80): the boxes stop at +0.734, the old top was +0.87.
    assert!(across(&stand, 0.80, 0.167).is_none(), "a round 0.07 over the head box misses");
    assert!(cyl(&old_stand, 0.80, 0.167).is_some(), "the old rule counted it as a hit");
    // 2. The gun hand 0.74 in front of his chest is boxed: a round there hits (outside the old
    //    0.25 radius).
    let arm = across(&stand, 0.397, 0.738).map(|h| h.0);
    assert!(matches!(arm, Some(21 | 23)), "the gun arm boxes (forearm 0x15 / hand 0x17), got {arm:?}");
    assert!(cyl(&old_stand, 0.397, 0.738).is_none());
    // 3. Crouched, through the head box (+0.50 over obj+0x30): a hit; the old cylinder topped out
    //    at +0.42.
    assert_eq!(across(&crouch, 0.50, 0.294).map(|h| h.0), Some(5), "the crouched head box");
    assert!(cyl(&old_crouch, 0.50, 0.294).is_none());
    // 4. Crouched, 0.8 under obj+0x30 is below his boots; the old cylinder reached 1.03 down.
    assert!(across(&crouch, -0.80, 0.0).is_none());
    assert!(cyl(&old_crouch, -0.80, 0.0).is_some());
}

/// The top of the boxes over `pos` (the highest reach of any box).
fn top(bx: &[HitBox]) -> f32 { bx.iter().map(|b| b.m[3][1] + reach_y(b)).fold(f32::MIN, f32::max) }

fn sim_boxes(sim: &nf_view::scene::Sim) -> (Vec<HitBox>, Vec<HitBox>) {
    let p = Player { pos: sim.bond.pos, yaw: sim.bond.yaw, crouched: sim.bond.crouched, ..Default::default() };
    let d = sim.data();
    // What the round pass tests (`Drones::tick`'s `bond_hit_boxes_posed` on `Drones::bond_pose`),
    // and the stance script's frame 1 (the pre-fix rule).
    (nf_game::drone::bond_hit_boxes_posed(&d, &p, sim.drones.bond_pose.as_ref()), bond_hit_boxes(&d, &p))
}

/// The boxes of `script` at `time` for Bond where he stands now.
fn script_boxes(sim: &nf_view::scene::Sim, script: u32, time: f32) -> Vec<HitBox> {
    let skin = &sim.skins[&nf_game::player::player_skin_hash(sim.level)];
    let skel = &sim.skeletons[&skin.skeleton];
    let pose = nf_game::anim::script_pose_drone(&sim.scripts, &sim.seqs, skel, skin, script, time).expect("script on the disc");
    let p = Player { pos: sim.bond.pos, yaw: sim.bond.yaw, crouched: sim.bond.crouched, ..Default::default() };
    nf_game::drone::bond_hit_boxes_posed(&sim.data(), &p, Some(&pose))
}

fn same(a: &[HitBox], b: &[HitBox]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (0..4).all(|r| (0..3).all(|k| (x.m[r][k] - y.m[r][k]).abs() < 1e-4)))
}

/// The stance change: `Player_ChangeSubState` 0x190630 builds the new stance's set, then
/// `PlayerAnimStand2Crouch` 0x190198 / `PlayerAnimCrouch2Stand` 0x190068 replace every script with
/// the single-player transition (class 3: 0x60001dc down, 0x60001dd up [M]) at speed 4
/// (`+0x98 = 4.0` [M]), and `AnimGetCollData` 0x1c4ac8 builds the boxes from that frame
/// (`psiBuildMatrixPalette` on the anim object [M :28]). Both scripts are 50 frames on the disc
/// [D], so for 12 ticks the body is part way between the stances: a round over the crouched head
/// still hits the body that is going down, and a round under the standing head top misses the
/// body that is still coming up. The old rule took the new stance's pose at once.
#[test]
fn stance_changes_take_the_transition_frames() {
    use nf_game::player::Input;
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000012.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    assert!(!sim.bond.crouched);
    // Standing still: the set's idle (0x6000a00, 2 frames, held) = the stance pose.
    let (now, old) = sim_boxes(sim);
    assert_eq!(now.len(), 16);
    assert!(same(&now, &old), "standing still the set pose is the stand script's");
    let stand_top = top(&old) - sim.bond.pos[1];

    // Down.
    sim.step_bond(Input { crouch: true, ..Default::default() }, false, false, false);
    assert!(sim.bond.crouched, "one press crouches");
    for k in 1..=12u32 {
        let (now, old) = sim_boxes(sim);
        let want = script_boxes(sim, 0x0600_01dc, (1.0 + 4.0 * k as f32).min(50.0));
        assert!(same(&now, &want), "tick {k}: the boxes ride 0x60001dc at frame {}", 1 + 4 * k);
        if k == 1 {
            // A round 0.06 over the crouched head's top: the old rule missed, the body that is
            // still going down takes it.
            let y = top(&old) + 0.06;
            // Across him (side-on) through the head box's own spot.
            let yaw = sim.bond.yaw;
            let ray = [yaw.cos() * 10.0, 0.0, -yaw.sin() * 10.0];
            let (cx, cz) = (now[0].m[3][0], now[0].m[3][2]);
            assert!(first_box_hit([cx - ray[0] * 0.5, y, cz - ray[2] * 0.5], ray, &old).is_none(), "over the crouched boxes");
            assert!(first_box_hit([cx - ray[0] * 0.5, y, cz - ray[2] * 0.5], ray, &now).is_some(), "the body going down is hit");
        }
        sim.step_bond(Input::default(), false, false, false);
    }
    // The hold ends on the stopped script and the crouch set takes over.
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    let (now, old) = sim_boxes(sim);
    assert!(sim.bond.crouched && same(&now, &old), "after the transition the crouch set's pose");

    // Up.
    sim.step_bond(Input { crouch: true, ..Default::default() }, false, false, false);
    assert!(!sim.bond.crouched, "a second press stands");
    let (now, old) = sim_boxes(sim);
    assert!(same(&now, &script_boxes(sim, 0x0600_01dd, 5.0)), "the boxes ride 0x60001dd from frame 5");
    // A round just under the standing top: the old rule hit the full-height body, the body that
    // is still coming up is under it.
    let y = sim.bond.pos[1] + stand_top - 0.03;
    let h = &old[0];
    let (cx, cz) = (h.m[3][0], h.m[3][2]);
    let yaw = sim.bond.yaw;
    let ray = [yaw.cos() * 10.0, 0.0, -yaw.sin() * 10.0];
    assert!(first_box_hit([cx - ray[0] * 0.5, y, cz - ray[2] * 0.5], ray, &old).is_some(), "the old full-height head");
    assert!(first_box_hit([cx - ray[0] * 0.5, y, cz - ray[2] * 0.5], ray, &now).is_none(), "the body coming up is under it");
}

/// Moving: every word of the single-player sets is the stance script (0x2b91b0 all 0x6000a00,
/// 0x2b91d8 all 0x60009ff [D]), so the walk/run layers `AnimSetUpdate` 0x1c0938 appends hold the
/// stance pose; the boxes while walking equal the standing ones.
#[test]
fn walking_keeps_the_stance_boxes() {
    use nf_game::player::Input;
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    for k in 0..60 {
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        if k < 10 { continue; }
        assert!(sim.bond_anim.layers.iter().any(|l| matches!(l.kind, nf_game::mp::body_anim::LayerKind::Walk { .. })), "the walk layer plays");
        let (now, old) = sim_boxes(sim);
        assert_eq!(now.len(), 16);
        for (a, b) in now.iter().zip(&old) { for c in 0..3 { assert!((a.m[3][c] - b.m[3][c]).abs() < 1e-3); } }
    }
}
