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

/// A resting body is bit-identical over the sim: the type-0xFFFF barrels never integrate
/// (EulerEvolve's pos arm is type-0 only [M 0x1d796c]), zero omega skips apply_omega [M
/// 0x1d79ac gate], and no hit list ever arms (no blast) — so every slot keeps its create-time
/// bits. The RB tick draws from the shared RNG only under the heat gate [M 0x1d6e88]; the
/// port's per-object cel is absent, so the gate is closed every tick (the unit suite pins the
/// gate's draw counts; here the barrels' slots must not move at all).
#[test]
fn resting_barrels_stay_bit_identical_over_the_sim() {
    let Some(mut scene) = load() else { eprintln!("skip: NF_ISO not set"); return };
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(!sim.drones.rbodies.list.is_empty(), "07000006 places kind-0xD8 bodies [D docs/systems/levels.md RB rows]");
        for _ in 0..400 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    }
    let before = { let sim = scene.sim.as_ref().unwrap(); snap(sim) };
    for _ in 0..600 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    let after = { let sim = scene.sim.as_ref().unwrap(); snap(sim) };
    assert_eq!(before.len(), after.len());
    for (b, a) in before.iter().zip(after.iter()) {
        assert_eq!(b.0, a.0, "the placement list is stable");
        assert_eq!(b.1, a.1, "barrel at placement {} moved while resting — a resting body is bit-identical [M 0x1d6e38 type!=0: no linear arm; 0x1d79ac: zero omega skips apply_omega]", b.0);
    }
}

/// A blast next to a barrel changes its matrix the game's way: the explode sweep + the shared
/// falloff (Collide_SphereIntersect 0x17c518:67, the falloff :78-81) arm the class-5 hit; the
/// arm (impulse = (hit+0x8)·0.5·vel into +0x40, r×impulse into +0x50 [M 0x1d7238-0x1d72e4 —
/// the Rot twin at 0x1d7428 for the type-0xFFFF barrels]) lands in the accumulators and the
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
    nf_game::rigid_body::rb_explosion_hit_arm(&mut expect.st, vel, falloff, point);
    let obj_y = expect.st.trans[1];
    nf_game::rigid_body::rb_update(&mut expect.rb, &mut expect.st, None, obj_y, 0x6, |_| panic!("a resting apply draws nothing"));
    assert_eq!(post.st.matrix, expect.st.matrix, "the matrix must equal the arm's evolution [M 0x1d7238-0x1d72e4]");
    assert_eq!(post.st.omega, expect.st.omega, "the omega must equal the arm's [M 0x1d6ff8: the torque arm]");
    // The 600-tick sister: after the kick the body keeps evolving or rests — either way the
    // slots stay finite (no NaN poisoning from the impulse path).
    for _ in 0..600 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.rbodies.list[k].st.trans.iter().all(|v| v.is_finite()));
}
