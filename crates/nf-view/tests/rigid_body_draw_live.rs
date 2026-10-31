//! The rigid bodies (0xD8) draw at their body's pose. `RB_Create` 0x1d6bf0:53 points the body's
//! drawn matrix at obj+0x90 and `RB_EulerEvolve` 0x1d7810 rewrites it once the body is disturbed
//! [M], while the draw (`View_DrawObjectGlist` 0x1e6780) renders obj+0xD8 at obj+0x90. 07000004's
//! two hanging lamps (#39, #40 [D]) were baked into the static draw: a shot lamp's body swung but
//! the lamp on screen never moved.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

#[path = "common/walker.rs"]
#[allow(dead_code)]
mod walker;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn opts() -> nf_view::scene::ViewOpts { nf_view::scene::ViewOpts::default() }

#[test]
fn the_07000004_lamp_draws_at_its_body_pose_after_a_shot() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut w = Walker::new(nf_view::load(&iso(), "07000004.bin", false).expect("load 07000004"));
    assert_eq!(w.scene.rbody_placements, vec![39, 40], "07000004's 0xD8 rows are the two lamps [D #39/#40]");
    // `lock_slot`'s chain, then one slot per rotor, then one per rigid body.
    let base = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.doors.list.len() + sim.drones.sss.list.len() + sim.drones.breaks.list.len()
            + 2 * sim.drones.locks.len()
            + (1 + nf_game::gunimp::TURRET_ARTS.len()) * w.scene.gunimp_placements
            + 3 * sim.drones.sensors.list.len()
            + w.scene.dynobj_matrices.len()
            + w.scene.rotor_placements.len()
    };
    for bi in 0..2 {
        assert!(w.scene.char_verts.iter().any(|v| v.door == Some((base + bi) as u16)), "lamp {bi}: the mesh rides its slot");
    }
    // Sister: no static World draw sits at lamp #40 any more (the static copy would hide the swing).
    let at40 = |m: &nf_view::scene::Material| (m.centre[0] - 8.01).abs() < 0.5 && (m.centre[2] - 23.03).abs() < 0.5 && (m.centre[1] - 7.2).abs() < 1.0;
    assert!(!w.scene.materials.iter().any(|m| at40(m) && m.layer == nf_view::scene::Layer::World), "lamp #40 left the static draw");
    let (layout, _attrs) = w.scene.skin_layout();
    let pose = |scene: &nf_view::scene::Scene| {
        let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
        pal[layout.door_base as usize + base + 1]
    };
    let body = |scene: &nf_view::scene::Scene| {
        let st = &scene.sim.as_ref().unwrap().drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().st;
        let r = st.matrix;
        [[r[0][0], r[0][1], r[0][2], 0.0], [r[1][0], r[1][1], r[1][2], 0.0], [r[2][0], r[2][1], r[2][2], 0.0], [st.trans[0], st.trans[1], st.trans[2], 1.0]]
    };
    let close = |a: [[f32; 4]; 4], b: [[f32; 4]; 4]| (0..4).all(|r| (0..4).all(|c| (a[r][c] - b[r][c]).abs() < 1e-4));
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { break; }
    }
    let rest = pose(&w.scene);
    assert!(close(rest, body(&w.scene)), "at rest the lamp draws at its body (= the placement) [M 0x1d6bf0:53]");
    w.walk_nav_to([8.0, 4.4, 26.5], "under the whitelamp", 1.2);
    let at = body(&w.scene)[3];
    w.shoot_point([at[0], at[1], at[2]], "the whitelamp", |sim| sim.drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().st.omega != [0.0; 3]);
    for _ in 0..120 { w.tick(Default::default(), false); }
    let swung = pose(&w.scene);
    assert!(close(swung, body(&w.scene)), "after the shot the drawn lamp follows the body [M RB_EulerEvolve 0x1d7810]");
    assert!(!close(swung, rest), "the shot lamp's drawn pose moved off its rest pose");
}

/// The tilt of a body's drawn rotation off its rest rotation: acos(up.y) of the matrix's row 1
/// (`Mat_GetUp` 0x1e33b8 — the vector the up-righting tail reads).
fn tilt(m: &[[f32; 3]; 3]) -> f32 { m[1][1].clamp(-1.0, 1.0).acos() }

/// One shot on lamp #40 sways it slightly and it settles back toward rest. `RB_EulerEvolve`
/// 0x1d7810 [M disasm]: the angular damping is omega += omega·(−(1−e^(−dt/+0xa4))/dt)
/// (0x1d78b0-0x1d78dc, ×0.96 per tick at +0xa4 = 25) and the type != 0 tail adds the
/// up × (0,1,0) spring (−up.z·I1, 0, up.x·I1)·FRAME_RATE_MUL into +0x50 every tick
/// (0x1d79f4-0x1d7a58), which `RB_ApplyImpulses` 0x1d6ff8 turns into omega the next tick.
/// Sister (can fail): the same post-hit body evolved without the spring and with the plain
/// e^(−dt/a) damping (the pre-fix integration) tilts monotonically past the bound.
#[test]
fn the_07000004_lamp_swings_back_toward_rest_after_one_shot() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut w = Walker::new(nf_view::load(&iso(), "07000004.bin", false).expect("load 07000004"));
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { break; }
    }
    w.walk_nav_to([8.0, 4.4, 26.5], "under the whitelamp", 1.2);
    let lamp = |sim: &nf_view::scene::Sim| sim.drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().clone();
    let at = lamp(w.scene.sim.as_ref().unwrap()).st.trans;
    assert!(w.shoot_point([at[0], at[1], at[2]], "the whitelamp", |sim| lamp(sim).st.omega != [0.0; 3]), "the shot reaches lamp #40");
    let hit = lamp(w.scene.sim.as_ref().unwrap());
    // 10 s of the real sim after the hit.
    let (mut peak, mut peak_t) = (0.0f32, 0u32);
    let mut trace = Vec::new();
    for t in 1..=600u32 {
        w.tick(Default::default(), false);
        let o = lamp(w.scene.sim.as_ref().unwrap());
        let a = tilt(&o.st.matrix);
        if a > peak { peak = a; peak_t = t; }
        trace.push(a);
        assert!(o.st.omega.iter().chain(o.st.trans.iter()).all(|v| v.is_finite()));
        assert_eq!(o.st.trans, at, "the type != 0 body never translates [M 0x1d7960 gate]");
    }
    let end = *trace.last().unwrap();
    let at2s = trace[119];
    eprintln!("[lamp-swing] omega after hit {:?} peak {peak:.5} rad at t{peak_t} tilt@2s {at2s:.6} tilt@10s {end:.7}", hit.st.omega);
    // The faithful sim's numbers (this shot): peak 0.00574 rad (0.33°) at t15, then the spring
    // pulls it back to 0.00384 rad by t36 where it holds — the residual is the game's dead band:
    // `RB_ApplyImpulses` skips a torque slot within Vec_IsEqual's 0x3951b717 (1.8e-4) of zero
    // [M 0x1d7014-0x1d702c] and EulerEvolve skips apply_omega for omega within it [M 0x1d79c8],
    // so sin(θ)·I1 below 1.8e-4 no longer turns the body.
    assert!((peak - 0.00574).abs() < 0.0005, "peak tilt {peak} rad (pinned 0.00574)");
    assert!((10..=25).contains(&peak_t), "the swing peaks at t{peak_t} (pinned t15)");
    assert!((end - 0.00384).abs() < 0.0004, "settled tilt {end} rad (pinned 0.00384)");
    assert!(at2s < peak * 0.75 && end < peak * 0.75, "the lamp swings back toward rest ({at2s}, {end} vs peak {peak})");
    assert!(trace[60..].iter().all(|&a| a == end), "settled from 1 s on (the dead band holds it still)");
    assert!(trace.iter().all(|&a| a < 0.05), "never more than a small sway");
    // Sister: the pre-fix integration from the same post-hit state (no tail; omega *= e^(−dt/a)).
    let mut old = hit.clone();
    let dt = nf_game::ssys::REC_FRAME_RATE;
    let mut old_peak = 0.0f32;
    for _ in 0..600 {
        nf_game::rigid_body::rb_apply_impulses(&mut old.rb, &mut old.st);
        old.st.lin_impulse = [0.0; 3];
        old.st.tor_impulse = [0.0; 3];
        let f = (-(dt / old.rb.sa4)).exp();
        for v in &mut old.st.omega { *v *= f; }
        let mut q = old.rb.quat;
        nf_game::rigid_body::rb_apply_omega(&mut q, old.st.omega, dt);
        old.rb.quat = q;
        old.st.matrix = nf_game::rigid_body::rb_set_state(&old.rb);
        old_peak = old_peak.max(tilt(&old.st.matrix));
    }
    let old_end = tilt(&old.st.matrix);
    eprintln!("[lamp-swing] sister (no spring, plain exp damping): peak {old_peak:.4} end {old_end:.4}");
    assert!(old_end >= old_peak * 0.99 && old_peak > peak * 2.0, "sister: the old integration tilts on monotonically");
}
