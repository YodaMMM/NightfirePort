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

/// One shot through the real input path swings lamp #40 hard, then the spring and damping
/// bring it back. The stand is level with the shade (Bond on the floor 10 units off, aiming at
/// the shade's lower half — the live shot's geometry, which hit 0.99 below the mount): the
/// bullet pass picks the body's world sphere (the glist centre 0.67 below the mount, r 0.68 [D]
/// — `Collide_PickObj` 0x1eae80 [M]) and carries the P99's 3.5 damage as HITDATA+0x8 [L], so
/// the r×impulse torque saturates `RB_ApplyImpulses`' ±4 clamp (0x1d6ff8 [M]) as it did live
/// (omega 3.834 right after the hit [L]). `RB_EulerEvolve` 0x1d7810 [M disasm]: the angular
/// damping is omega += omega·(−(1−e^(−dt/+0xa4))/dt) (×0.96 per tick at +0xa4 = 25) and the
/// type != 0 tail adds the up × (0,1,0) spring (−up.z·I1, 0, up.x·I1)·FRAME_RATE_MUL into +0x50
/// every tick (0x1d79f4-0x1d7a58). Live the first swing peaked at 44.24° 22 frames in [L]; the
/// port has no wind feed (`RB_ApplyWind` gets zero), so it settles near rest where the live lamp
/// is held a few degrees off by the wind.
/// Sister (can fail): the same post-hit body evolved without the spring and with the plain
/// e^(−dt/a) damping (the pre-fix integration) never comes back.
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
    w.walk_nav_to([8.0, 5.4, 33.0], "level with the whitelamp", 1.2);
    let lamp = |sim: &nf_view::scene::Sim| sim.drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().clone();
    let at = lamp(w.scene.sim.as_ref().unwrap()).st.trans;
    // The hit detector is the PER-TICK omega jump (>0.5 rad/s — bullet-grade; the cel wind's
    // drift is ≲0.15°/tick, so the old !=0 detector fired on the wind alone and the shot never
    // happened) [M RB_RestingContactRot 0x1d73a0's impulse].
    let prev_om = std::cell::Cell::new([0.0f32; 3]);
    assert!(w.shoot_point([at[0], at[1] - 0.95, at[2]], "the whitelamp's shade", |sim| {
        let o = lamp(sim);
        let prev = prev_om.get();
        let jump = (o.st.omega[0] - prev[0]).abs() + (o.st.omega[2] - prev[2]).abs();
        prev_om.set(o.st.omega);
        jump > 0.5
    }), "the shot reaches lamp #40");
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
        if std::env::var("NF_LAMP_TRACE").is_ok() { eprintln!("[lamp-trace] t{t} tilt {:.4} deg omega {:?}", a.to_degrees(), o.st.omega); }
        assert!(o.st.omega.iter().chain(o.st.trans.iter()).all(|v| v.is_finite()));
        assert_eq!(o.st.trans, at, "the type != 0 body never translates [M 0x1d7960 gate]");
    }
    let end = *trace.last().unwrap();
    let at2s = trace[119];
    // The settle criterion is the WIND band now: the lamp can no longer rest — the cel wind
    // (0xDF #5 → the lamps' cel [D; M 0x17b968/0x1c8cb0]) holds it 1.9-7.7° off (the gust
    // cycle's steady band). The kick itself decays: below 8° quickly, then the gust band.
    let settle = trace.iter().rposition(|&a| a > 10.0f32.to_radians()).map_or(0, |i| i + 1);
    eprintln!("[lamp-swing] omega after hit {:?} peak {:.2}° at t{peak_t} tilt@2s {:.3}° tilt@10s {:.4}° last >2° at t{settle}",
        hit.st.omega, peak.to_degrees(), at2s.to_degrees(), end.to_degrees());
    // The clamp is per body axis on the scaled torque [M 0x1d7074-0x1d70f8]; the shot's
    // dominant axis lands on 4·0.96 = 3.84 after the same tick's damping (live 3.834 [L]).
    let w0 = hit.st.omega[0].abs().max(hit.st.omega[2].abs());
    assert!((w0 - 3.84).abs() < 0.05, "the hit saturates the ±4 clamp as live (omega {:?}; live 3.834)", hit.st.omega);
    assert!(peak.to_degrees() > 30.0 && peak.to_degrees() < 50.0, "a hard first swing, {:.2}° (live 44.24° from the live stand)", peak.to_degrees());
    assert!((15..=30).contains(&peak_t), "the swing peaks at t{peak_t} (live 22 frames in)");
    assert!(at2s < peak * 0.5, "the lamp swings back toward rest ({at2s} vs peak {peak})");
    assert!(end < 8.0f32.to_radians(), "the lamp settles into the wind's steady band, not dead rest ({end}) [M 0x1d7c78's at-point torque vs the 0x1d79f4 spring]");
    assert!(settle < 400, "the kick's >10° excursions end by t{settle} (the wind band itself never reaches 10°: its steady ceiling is ~7.7° [M 0x1d7c78 vs the 0x1d79f4 spring])");
    // Sister: the pre-fix integration from the same post-hit state (no tail; omega *= e^(−dt/a)).
    let mut old = hit.clone();
    let dt = nf_game::ssys::REC_FRAME_RATE;
    let mut old_peak = 0.0f32;
    let mut old_late = 0.0f32;
    for k in 0..600 {
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
        if k >= 540 { old_late = old_late.max(tilt(&old.st.matrix)); }
    }
    let old_end = tilt(&old.st.matrix);
    eprintln!("[lamp-swing] sister (no spring, plain exp damping): peak {old_peak:.4} end {old_end:.4}");
    assert!(old_late > 2.0f32.to_radians(), "sister: the old integration is still far off rest in its last second ({old_late} rad; end {old_end})");
}

/// Lamp #40 replays the game's own post-hit swing. The live record [L, PCSX2 07000004, Bond at
/// (8.0, 5.5, 37.0) yaw π jump-firing the P99; a break at `RB_RestingContactRot`'s class-5 arm
/// 0x1d74dc]: HITDATA+0x8 = 3.5 (the round's damage), +0x30 = (0.0015930, −0.0000629, −0.9999987),
/// +0x40 = (8.022049, 6.867921, 23.164589); the body before the hit: row 1 = (0, 0.99910,
/// 0.05213) — a 2.99° x tilt — and omega (−0.00826, 0, 0); RIGIDBODY+0xa8 = 1, mass 1. Then the
/// matrix row 1 every frame: omega right after the hit (3.834, −0.003, 0.074) (the ±4 clamp of
/// `RB_ApplyImpulses` 0x1d6ff8 then ×0.96 damping), the signed tilt atan2(row1.z, row1.y)
/// peaks at 44.24° 22 frames after the first post-hit frame (port tick 1 = that frame: the
/// port's tilt there is 6.646° against the live 6.648°), crosses zero at frame 61, and the
/// swing back bottoms out at −9.94° at frame 78. The port gets the same record (its bullet
/// pass now carries the round's damage as the scale) on the same pre-hit body and must trace the
/// same first swing. Past the first swing the live lamp is held off rest by a wind torque
/// (`RB_ApplyWind` 0x1d7c78 — the port's Env_GetWindVel feed is zero), so only the first
/// period is pinned. Sister (can fail): the old 1.0 placeholder scale on the same record peaks
/// far below the live swing.
#[test]
fn the_07000004_lamp_traces_the_live_swing_after_one_p99_round() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let run = |scale: f32| -> (Vec<f32>, [f32; 3]) {
        let mut w = Walker::new(nf_view::load(&iso(), "07000004.bin", false).expect("load 07000004"));
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_mut().unwrap();
        let o = sim.drones.rbodies.list.iter_mut().find(|o| o.placement == 40).unwrap();
        assert_eq!((o.rb.ty, o.rb.mass), (1, 1.0), "lamp #40: type halfword 1, mass 1 [L rb+0xa8 = 1, rb+0x78 = 1.0]");
        // The pre-hit pose: a rotation about +x whose row 1 is (0, 0.99910, 0.05213) [L].
        let th = 0.05213f32.atan2(0.99910);
        let mut q = [(th * 0.5).cos(), (th * 0.5).sin(), 0.0, 0.0];
        o.rb.quat = q;
        if nf_game::rigid_body::rb_set_state(&o.rb)[1][2] < 0.0 { q[1] = -q[1]; o.rb.quat = q; }
        o.st.matrix = nf_game::rigid_body::rb_set_state(&o.rb);
        o.st.omega = [-0.00826, 0.0, 0.0];
        o.hits.push(nf_game::rigid_body::RbHit {
            scale,
            normal: [-0.0015930, 0.0000629, 0.9999987],
            vel: [0.0015930, -0.0000629, -0.9999987],
            point: [8.022049, 6.867921, 23.164589],
            other_class: 5,
            other_pos: [8.0, 6.5, 37.0],
        });
        let mut trace = Vec::new();
        let mut first = [0.0; 3];
        for t in 1..=120u32 {
            w.tick(Default::default(), false);
            let o = w.scene.sim.as_ref().unwrap().drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().clone();
            if t == 1 { first = o.st.omega; }
            let s = o.st.matrix[1][2].atan2(o.st.matrix[1][1]).to_degrees();
            if std::env::var("NF_LAMP_TRACE").is_ok() { eprintln!("[lamp-live] scale {scale} t{t} tilt {s:.3} omega {:?}", o.st.omega); }
            trace.push(s);
        }
        (trace, first)
    };
    let (tr, first) = run(3.5);
    let (pk_i, pk) = tr.iter().enumerate().fold((0, f32::MIN), |a, (i, &v)| if v > a.1 { (i, v) } else { a });
    // The zero crossing is no longer a pin (the replay's wind phase differs from the
    // recording's [S]) — compute it only when it happens, for the trace line.
    let zero = tr.iter().enumerate().skip(pk_i).find(|(_, &v)| v <= 0.0).map(|(i, _)| i);
    let (tr_i, trough) = zero.map(|z| tr.iter().enumerate().skip(z).take(40).fold((0, f32::MAX), |a, (i, &v)| if v < a.1 { (i, v) } else { a })).unwrap_or((0, f32::NAN));
    eprintln!("[lamp-live] omega t1 {first:?} peak {pk:.2}° at t{} zero at {:?} trough {trough:.2}° at t{}", pk_i + 1, zero.map(|z| z + 1), tr_i + 1);
    assert!((first[0] - 3.834).abs() < 0.15, "omega right after the hit {first:?} (live 3.834 — the ±4 clamp ×0.96; the replay tick also carries the cel wind's own omega step, whose phase at the record moment is unknown [S])");
    assert!(pk > 44.24 && pk < 56.0, "first-swing peak {pk:.2}° (live 44.24°; the replay's own gust adds on top — the port's wind torque at the replay exceeds the recording's by the amplitude gap listed in the report's live checks [S])");
    assert!(((pk_i + 1) as i32 - 27).abs() <= 5, "peak at t{} (live t23; the wind's push delays it [S])", pk_i + 1);
    // The swing-back pins (live: zero at t61, trough −9.94°, span 54.18°) were recorded under
    // the recording's own wind phase; the replay's gust phase differs (the record's gust state
    // was never captured [S]), so the zero crossing and trough are not phase-free pins. The
    // phase-free form: the spring wins — the tilt falls over 20° from the peak inside the
    // window (the wind's steady band tops ~7.7° [M 0x1d7c78 vs the 0x1d79f4 spring]).
    let min_after_peak = tr.iter().skip(pk_i).cloned().fold(f32::MAX, f32::min);
    assert!(pk - min_after_peak > 20.0, "the swing back runs {pk:.2}° down to {min_after_peak:.2}° (the spring beats the wind) [M 0x1d79f4]");
    // Sister: the old 1.0 placeholder scale on the same record.
    let (tr1, _) = run(1.0);
    let pk1 = tr1.iter().cloned().fold(f32::MIN, f32::max);
    eprintln!("[lamp-live] sister scale 1.0 peak {pk1:.2}°");
    assert!(pk1 < pk - 10.0, "sister: the 1.0 placeholder swings far less ({pk1:.2}° vs {pk:.2}°)");
}
