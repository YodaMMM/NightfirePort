//! rigid-body-wiring: the kind-0xD8 bodies are sim objects.
//!
//! `RB_Create` 0x1d6bf0 (via `parsemap_create_dynamic_objects` 0x1d04f0 case 0xD8) creates the
//! bodies at level load; `RB_Update` 0x1d6e38 ticks them in the per-kind dispatch (the
//! 0x26f9c8/0x26f9cc pair); blasts reach them through the class-5 hit arm of
//! `RB_RestingContact` 0x1d7168 / `RB_RestingContactRot` 0x1d73a0. A resting body is
//! bit-identical over the sim (the heat gate never opens — the port carries no per-object cel
//! yet, and the game's draw is gate-gated [M 0x1d6e38:0x1d6e88-0x1d6eb0]); a blast next to a
//! 07000006 barrel changes its obj+0x90 matrix by the game's impulse arm.
//!
//! Opt-in via `NF_ISO` (the slim ISO carries 07000006).

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load() -> Option<nf_view::scene::Scene> {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let iso = iso()?;
    Some(nf_view::load(&iso, "07000006.bin", false).expect("load 07000006"))
}

fn snap(sim: &nf_view::scene::Sim) -> Vec<(u16, [u8; 132])> {
    sim.drones.rbodies.list.iter().map(|o| {
        let mut b = [0u8; 132];
        let mut w = 0usize;
        for v in o.st.pos.iter().chain(o.st.omega.iter()).chain(o.st.vel.iter())
            .chain(o.st.ang_accel.iter()).chain(o.st.lin_impulse.iter())
            .chain(o.st.tor_impulse.iter()).chain(o.st.trans.iter()) {
            b[w..w + 4].copy_from_slice(&v.to_le_bytes()); w += 4;
        }
        for row in &o.st.matrix { for v in row { b[w..w + 4].copy_from_slice(&v.to_le_bytes()); w += 4; } }
        b[w..w + 4].copy_from_slice(&o.st.s9c.to_le_bytes()); w += 4;
        b[w..w + 4].copy_from_slice(&o.st.s13c.to_le_bytes()); w += 4;
        b[w] = o.flags as u8; b[w + 1] = (o.flags >> 8) as u8;
        (o.placement, b)
    }).collect()
}

/// A resting body under the cel wind is NOT bit-identical any more: `RB_ApplyWind` 0x1d7c78
/// lands the force AT A POINT for type != 0 → torque, the body's only per-frame torque when
/// nothing hits it. The 07000006 barrel (#65) shares room cel #29 with the SNOW_gen 0xDF #1
/// (p3 = 6 → gust target 0.6 [D the port's level parse; Env_Create 0x17b968 stores p3·0.1
/// into W+0x30/+0x34]), so the game's own barrel takes the same gust torque every tick. The
/// old "bit-identical rest" premise predates the wind port and contradicted that chain —
/// fixed (see the report; the TEST was wrong, the port model follows 0x1d6e38 → 0x1d7c78 →
/// 0x17c048). The buoyancy gate stays shut (the cel's water surface is the −500 default [M
/// build_alloc_cel 0x1c93f8]; the closure would panic on any draw), the Rot body still never
/// translates [M 0x1d7810's type-0-only position arm], and the drift is BOUNDED — the heavy
/// barrel's tilt stays under 0.5° over the window (its moments are ~20× the lamp's).
#[test]
fn resting_barrel_drifts_in_the_cel_wind_but_stays_bounded() {
    let Some(mut scene) = load() else { eprintln!("skip: NF_ISO not set"); return };
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(!sim.drones.rbodies.list.is_empty(), "07000006 places kind-0xD8 bodies [D docs/systems/levels.md RB rows]");
        assert!(!sim.drones.rbodies.winds.is_empty(), "the SNOW_gen 0xDF builds the cel wind [M Env_Create 0x17b968]");
        for _ in 0..400 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    }
    let before = { let sim = scene.sim.as_ref().unwrap(); snap(sim) };
    for _ in 0..600 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    let (after, tilt_max) = {
        let sim = scene.sim.as_ref().unwrap();
        let up = sim.drones.rbodies.list[0].st.matrix[1];
        let tilt = (up[0] * up[0] + up[2] * up[2]).sqrt().atan2(up[1]) * 57.29578;
        (snap(sim), tilt)
    };
    assert_eq!(before.len(), after.len());
    let mut any_moved = false;
    for (b, a) in before.iter().zip(after.iter()) {
        assert_eq!(b.0, a.0, "the placement list is stable");
        if b.1 != a.1 { any_moved = true; }
        // The Rot body never translates and the buoyancy gate never opens: the position slot
        // and the linear impulse stay at the create bits [M 0x1d7810 type-0-only; 0x1d6e88
        // gate: obj_y < −500 is false].
        let (bt, at) = (&b.1[72..84], &a.1[72..84]);
        assert_eq!(bt, at, "barrel at placement {} must not translate while resting", b.0);
        assert_eq!(&b.1[48..60], &a.1[48..60], "no buoyancy draw opens on this level (lin_impulse still) [M 0x1d6e88]");
    }
    assert!(any_moved, "the barrel's rotation state never changed — the cel wind did not reach the body (the 0xDF/cel chain broke)");
    assert!(tilt_max < 0.5, "the gust-driven tilt must stay bounded, got {tilt_max}°");
}

/// A blast next to a barrel changes its matrix the game's way: the explode sweep + the shared
/// falloff (Collide_SphereIntersect 0x17c518:67, the falloff :78-81) arm the class-5 hit; the
/// arm (impulse = (hit+0x8)·0.5·vel into +0x40, r×impulse into +0x50 [M 0x1d7238-0x1d72e4 —
/// the Rot twin at 0x1d7428 for the type-1 barrel, its cross's y lane masked]) lands in the accumulators and the
/// NEXT tick's RB_Update applies them (EulerEvolve) — the quat steps and the drawn obj+0x90
/// rows leave identity. The hit record's velocity/point construction is the port's stand-in
/// [S — the game's explosion-object contact record is unported]; this test pins the routing:
/// the port's post-blast state must equal the arm applied to the same record.
#[test]
fn blast_next_to_a_barrel_changes_its_matrix_the_games_way() {
    let Some(mut scene) = load() else { eprintln!("skip: NF_ISO not set"); return };
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..400 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    }
    let (k, pre, centre) = {
        let sim = scene.sim.as_ref().unwrap();
        let k = sim.drones.rbodies.list.iter().position(|o| o.st.trans[0].is_finite())
            .expect("a body to blast");
        (k, sim.drones.rbodies.list[k].clone(), sim.drones.rbodies.list[k].st.trans)
    };
    // The blast: off-centre (+x+y of the body) — a dead-centre hit's r = −vel·s13c is PARALLEL
    // to the impulse and the cross (the torque) is zero [M 0x1d7238: the r×impulse arm], which
    // for a type != 0 body (no linear arm, [M 0x1d6ff8]) means no motion at all. The props-path
    // shape (radius 3, damage 200 — the Destroy_Smash blast the fusebox path pushes [M 0x12c424]).
    let blast_pos = [centre[0] + 2.0, centre[1] + 1.5, centre[2] + 0.5];
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.pending_blasts.push((blast_pos, 3.0, 200.0, usize::MAX, 0, 0x0600_0027, 3.0, 0));
        sim.step_bond(Input::default(), false, false, false); // spawn_blast arms the hit
    }
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); // the update applies
    let sim = scene.sim.as_ref().unwrap();
    let post = &sim.drones.rbodies.list[k];
    let moved = post.st.matrix != pre.st.matrix || post.st.trans != pre.st.trans
        || post.st.omega != pre.st.omega || post.st.pos != pre.st.pos;
    assert!(moved, "the barrel's pose never moved — the blast did not reach the body");
    // The routing pin: replay the arm on the pre-state with the same record the sweep built.
    let falloff = nf_game::mp::combat::blast_falloff(blast_pos, 3.0, 200.0, centre, pre.st.s13c);
    assert!(falloff >= 0.0002, "the sweep must admit a body 2 units out");
    let dir = [centre[0] - blast_pos[0], centre[1] - blast_pos[1], centre[2] - blast_pos[2]];
    let mag = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let vel = [dir[0] / mag, dir[1] / mag, dir[2] / mag];
    // The same stand-in point the arming built [S]: surface point toward the blast + the
    // fixed tangent (cross(vel, +y) normalised, half the body radius).
    let tangent = {
        let c = [vel[1] * 0.0 - vel[2] * 1.0, 0.0, vel[0] * 1.0 - vel[1] * 0.0];
        let m = (c[0] * c[0] + c[2] * c[2]).sqrt();
        [c[0] / m * pre.st.s13c * 0.5, 0.0, c[2] / m * pre.st.s13c * 0.5]
    };
    let point = [
        centre[0] + vel[0] * pre.st.s13c + tangent[0],
        centre[1] + vel[1] * pre.st.s13c + tangent[1],
        centre[2] + vel[2] * pre.st.s13c + tangent[2],
    ];
    let mut expect = pre.clone();
    // The barrel is type 1 (param 2 [D]; level_tag+0x34 [M 0x1d1b68:74]), so the hit walks
    // `RB_RestingContactRot`'s class-5 arm with the type-1 lane mask (the cross's y zeroed
    // [M disasm 0x1d753c-0x1d757c]).
    assert_eq!(expect.rb.ty, 1, "the 07000006 barrel's type halfword is param 2 = 1");
    let rec = nf_game::rigid_body::RbHit { scale: falloff, normal: [-vel[0], -vel[1], -vel[2]], vel, point, other_class: 5, other_pos: blast_pos };
    nf_game::rigid_body::rb_resting_contact_rot(&expect.rb, &mut expect.st, &[rec], &mut Vec::new());
    let obj_y = expect.st.trans[1];
    // The replay now carries the cel wind the sim's tick used: the barrel's wind cell read at
    // the post point (one gust tick apart — the gust's per-tick drift is ~0.3%, absorbed by
    // the 1e-3 tolerances below; the blast impulse dominates by 100×) [M 0x1d7c78's
    // Env_GetWindVel call].
    let wind = {
        let sim = scene.sim.as_ref().unwrap();
        nf_game::rigid_body::env_get_wind_vel(&sim.drones.rbodies.winds[0], pre.st.trans)
    };
    nf_game::rigid_body::rb_update(&mut expect.rb, &mut expect.st, None, obj_y, 0x6, wind, pre.st.trans, |_| panic!("a resting apply draws nothing"));
    for (c, e) in post.st.matrix.iter().flatten().zip(expect.st.matrix.iter().flatten()) {
        assert!((c - e).abs() < 1.0e-3, "the matrix must equal the arm's evolution [M 0x1d7238-0x1d72e4]: {c} vs {e}");
    }
    for (c, e) in post.st.omega.iter().zip(expect.st.omega.iter()) {
        assert!((c - e).abs() < 1.0e-3, "the omega must equal the arm's [M 0x1d6ff8: the torque arm]: {c} vs {e}");
    }
    // The 600-tick sister: after the kick the body keeps evolving or rests — either way the
    // slots stay finite (no NaN poisoning from the impulse path).
    for _ in 0..600 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.rbodies.list[k].st.trans.iter().all(|v| v.is_finite()));
}
