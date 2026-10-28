//! drone-bump-collision: the actor-actor capsule collide, end to end through the real input path.
//!
//! The two proven halves:
//! - The player side: Bond is pushed off every armed class-2 capsule with the FULL overlap, in
//!   every stance [M Collide_Jointy 0x1ec180:229-266 — SP fVar12 = 1.0; no stance gate in
//!   Player_Collision 0x194df0; the class-3-other return 0x1ec180:231-233 keeps the push
//!   Bond-side only]. Walking into a standing guard is therefore BLOCKED at the capsule
//!   distance instead of passing through.
//! - The drone side: the class-2 collide consumer's player arm sends msg 0x22 on the contact
//!   [M NDrone2_DealWithObjHit 0x13fff0:108-125 — Drone_Message(obj, 0x22, 0, 0) per contact,
//!   the contact list rebuilt each pass]; the capsule itself is NDrone2_Collision 0x140e90's
//!   two-end shape (ends pos+up·r and pos+up·(r−hip), r 0.3 down / 0.55 DSTATE 0xC / 0.4).
//!
//! Level: 07000014 (the drop-slot estate — the small boot, the sleeper_push_live pattern).

#[path = "common/walker_b2.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn boot_and_find_guard() -> (nf_view::scene::Scene, usize) {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..3000 {
            sim.step_bond(nf_game::player::Input::default(), false, false, false);
            if !sim.disabled { break; }
        }
        for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    }
    let gi = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.list.iter().position(|d| {
            d.started && d.enabled && !d.corpse && d.health > 0.0
        }).expect("07000014 boots a live guard (the sleeper slot patroller [D])")
    };
    (scene, gi)
}

/// Bond walking a line into a standing guard is BLOCKED: the Jointy push cancels his forward
/// advance at the capsule distance and he never passes through the guard's spine column.
#[test]
fn bond_walking_into_a_standing_guard_is_blocked_at_the_capsule_distance() {
    let (mut scene, gi) = boot_and_find_guard();
    // Park Bond 1.6 ahead of the guard (along Bond's +x), facing him, then walk forward.
    let gpos = scene.sim.as_ref().unwrap().drones.list[gi].pos;
    let (ta, tb, tr) = scene.sim.as_ref().unwrap().drones.list[gi].bump_capsule();
    let _ = (ta, tb);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [gpos[0] - 1.6, gpos[1], gpos[2]];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::FRAC_PI_2; // face +x, toward the guard (the walker's east convention, cf. sleeper_push_live)
    }
    let mut min_d = f32::MAX;
    let mut passed = false;
    for _ in 0..240 {
        scene.sim.as_mut().unwrap().step_bond(
            nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let gp = sim.drones.list[gi].pos;
        let d = ((p[0] - gp[0]).powi(2) + (p[2] - gp[2]).powi(2)).sqrt();
        min_d = min_d.min(d);
        if p[0] > gp[0] + 0.2 { passed = true; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    let gp = sim.drones.list[gi].pos;
    let d = ((p[0] - gp[0]).powi(2) + (p[2] - gp[2]).powi(2)).sqrt();
    // The contact distance: the radius sum (guard capsule r + Bond 0.55) minus one tick of
    // integration slack — the push resolves to the spines' distance each tick [M 0x1ec180:251-252].
    eprintln!("[bump] guard {gp:?} bond {p:?} final d {d:.3} min_d {min_d:.3} tr {tr} passed(x) {passed}");
    // The proven mechanism [M 0x1ec180:229-266]: the push resolves the spines to the radius sum
    // each tick (min_d settles at r_sum 0.95 minus one tick of integration slack) and never lets
    // the pair interpenetrate; a tangent delta turns the push LATERAL, so walking into the guard
    // ends deflected around him (the brief's "stops or deflects") — passing through the spine
    // column is what never happens, and that is min_d's floor here.
    assert!(min_d > 0.5,
        "Bond must not interpenetrate the guard's capsule: min_d {min_d:.3}");
    assert!(min_d < 1.1,
        "the push must engage AT the capsule distance (r_sum 0.95 + slack): min_d {min_d:.3}");
}

/// The capsule arms census [M NDrone2_Collision 0x140e90:59-124]: every live boot drone's
/// bump_capsule matches its own state fields — top = pos + (0, r, 0), bottom = pos + (0, r−1, 0),
/// r 0.3 when the down flag +0x2F stands, else 0.55 in DSTATE 0xC, else 0.4.
#[test]
fn the_bump_capsule_arms_follow_the_drone_state_fields() {
    let (scene, _gi) = boot_and_find_guard();
    let sim = scene.sim.as_ref().unwrap();
    let mut checked = 0;
    for d in sim.drones.list.iter() {
        if !d.present || !d.enabled || d.corpse || d.health <= 0.0 { continue; }
        let (a, b, r) = d.bump_capsule();
        let want_r = if d.down { 0.3 } else if d.state == nf_game::drone::state::HOSTAGE_SAVED { 0.55 } else { 0.4 };
        assert_eq!(r, want_r, "radius arm for drone state {} down {}", d.state, d.down);
        assert!((a[1] - (d.pos[1] + r)).abs() < 1e-4, "top end = pos + up·r");
        assert!((b[1] - (d.pos[1] + r - 1.0)).abs() < 1e-4, "bottom end = pos + up·(r − hip 1.0)");
        assert_eq!(a[0], d.pos[0], "upright: x unchanged");
        assert_eq!(b[2], d.pos[2], "upright: z unchanged");
        checked += 1;
    }
    assert!(checked > 0, "the census needs at least one live boot drone");
}
