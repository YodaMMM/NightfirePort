//! stairwell-bullets-11: the rounds aimed at 07000011's abseiling guards stopped on something
//! invisible in the well — the port's player-bullet flight trace stopped on the first triangle it
//! met, including the pass-flagged ones the game's flight walks past. The blocker is map43#126,
//! whose own disc name says it: "stairwell_blocking_ploys, this may look od buts its a fudge!!!!
//! do not change or thre game will break!" — glist flags 0x10, its triangles carry the surface
//! byte 0x40 [D 07000011.bin], and the census probe (crates/nf-game/src/world.rs
//! stairwell_bullet_census_07000011) shows it crossing every Bond→guard ray at t 0.05-0.10.
//!
//! The game's rule [M]: a flying bullet ARMS its HITTEST pick every tick (+0x94 = 0x101/0x201,
//! +0x9a = the surface mask 0x20 for guns without weapon_data+0x6c & 0x800 [M Bullet_update
//! 0x12e7b0 disasm 0x12f030-0x12f058, 0x12f134/0x12f154]) and the sorted contact list is walked
//! past every pass-flagged hit to the first solid one [M Collide_Update 0x1ea398:216-229 — pass =
//! HITDATA+0x51 bit 1 (node 0x40 / surface 0x40, Collide_Intersect disasm 0x1ebf78-0x1ebf80) |
//! +0x50 bit 0x40 | the 0x80 random roll | (mask 0x20 → glass 0x10 passes)]. The fudge's 0x40
//! surfaces pass; the round reaches the rope guard. The port's flight now uses the same
//! drop (`segment_bullet`).
//!
//! Evidence tags: [M] decompile address, [D] disc bytes, [L] live measurement.

use nf_view::scene::Scene;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_d3() -> Scene {
    nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load 07000011 d3")
}

fn by_placement(scene: &Scene, p: usize) -> usize {
    scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.placement == p && d.present)
        .unwrap_or_else(|| panic!("stairwell guard placement {p} present at load [D]"))
}

/// The reported repro spot [D the reported spot]: Bond root (15.33, 26.69, 1.33). The abseiler p258's
/// rope anchor is (18.895, 46.516, 1.434) [D]; the hang parks him ~3 above the height Bond held
/// when the wake plate fired [M NDrone2_DSTATE_AbseilSlide 0x169730:55-58] — here the plate spot's
/// 29.37, so he hangs at ~32.25, and the spot→guard ray crosses the fudge's 0x40 plane.
#[test]
fn the_round_from_the_reported_spot_reaches_and_damages_the_rope_guard() {
    let mut scene = load_d3();
    {
        let sim = scene.sim.as_mut().unwrap();
        // Wake p258 through the real plate (TouchOnce #253 → ch 0x13 [D], the guards test's spot).
        sim.bond.pos = [22.34, 29.37, 3.52];
        sim.bond.vel = [0.0; 3];
    }
    let mut woke = false;
    for _ in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(0x13) { woke = true; break; }
    }
    assert!(woke, "the 0x13 plate fires from the reported plate spot [D]");
    let pi = by_placement(&scene, 258);
    let mut hung = None;
    for _ in 0..400 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let dr = &scene.sim.as_ref().unwrap().drones.list[pi];
        if dr.state == nf_game::drone::state::ABSEIL_HANG { hung = Some(dr.pos); break; }
    }
    let guard_at = hung.expect("p258 hangs [M 0x169730:55-58]");
    // Move Bond to the reported spot; the hanging guard keeps the hang height.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [15.33, 26.69, 1.33];
        sim.bond.vel = [0.0; 3];
    }
    // Aim at the hanging guard's chest through the real aim fields.
    {
        let sim = scene.sim.as_mut().unwrap();
        let target = [guard_at[0], guard_at[1] + 0.3, guard_at[2]];
        let d = [target[0] - sim.bond.pos[0], target[1] - (sim.bond.pos[1] + 0.70), target[2] - sim.bond.pos[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let dir = [d[0] / l, d[1] / l, d[2] / l];
        sim.bond.yaw = dir[0].atan2(dir[2]);
        sim.bond.pitch = dir[1].clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI;
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 12;
        sim.gun.pending = 0;
    }
    let health_before = scene.sim.as_ref().unwrap().drones.list[pi].health;
    let mut input = nf_game::player::Input::default();
    input.fire = true;
    let mut damaged = false;
    for t in 0..60u32 {
        scene.sim.as_mut().unwrap().step_bond(if t == 0 { input } else { Default::default() }, t == 0, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[pi];
        if dr.health < health_before || dr.hit_by_player {
            damaged = true;
            eprintln!("[bullets] the round reached the guard at tick {t}: health {:.2} -> {:.2}",
                health_before, dr.health);
            break;
        }
    }
    assert!(damaged, "the round passes the fudge's 0x40 plane [M 0x1ea398:216-229] and damages \
        the rope guard (health {health_before} -> {})",
        scene.sim.as_ref().unwrap().drones.list[pi].health);
}

/// Sister: a round into a real wall still stops there — from the same spot, aimed DOWN at the
/// stairwell's own steps (ROOM_StairWell_2 geometry, unflagged mat 0x0a [D map43#127]), the round
/// records a world impact below Bond and the guard takes nothing.
#[test]
fn a_round_into_the_real_wall_still_stops_there() {
    let mut scene = load_d3();
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [22.34, 29.37, 3.52];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(0x13) { break; }
    }
    let pi = by_placement(&scene, 258);
    for _ in 0..400 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.list[pi].state == nf_game::drone::state::ABSEIL_HANG { break; }
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [15.33, 26.69, 1.33];
        sim.bond.vel = [0.0; 3];
        // Straight down at the steps: the floor ray finds solid geometry below the spot.
        sim.bond.yaw = 0.0;
        sim.bond.pitch = -1.0;
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 12;
        sim.gun.pending = 0;
    }
    let health_before = scene.sim.as_ref().unwrap().drones.list[pi].health;
    let mut input = nf_game::player::Input::default();
    input.fire = true;
    let mut impact = None;
    for t in 0..60u32 {
        scene.sim.as_mut().unwrap().step_bond(if t == 0 { input } else { Default::default() }, t == 0, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for im in &sim.drones.player_impacts {
            if let nf_game::drone::PlayerImpact::World(p) = im { impact = Some(*p); }
        }
        if impact.is_some() { eprintln!("[bullets] the wall round stops at tick {t}"); break; }
    }
    let p = impact.expect("the downward round stops on the stairwell's own geometry [D mat 0x0a]");
    assert!(p[1] < 26.0, "the wall impact sits below the spot's floor line ({p:?})");
    assert_eq!(scene.sim.as_ref().unwrap().drones.list[pi].health, health_before,
        "the guard takes nothing from the wall shot");
}
