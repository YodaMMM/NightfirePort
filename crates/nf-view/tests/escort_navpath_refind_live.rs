//! The 07000001 escort contact's nav-path re-resolve inside `NDrone2_ReFindMissionPath` 0x153a30.
//! The rule [M 0x153a30:134-137]: in the unreachable-but-closest arm (no mission-route node within
//! 20 that MoveTest reaches, so the anchor is the closest node), a drone whose voice set +0xD8 is 9
//! (skins 0x0500001b / 0x0500004f — the contact) rewrites its nav path +0x954 with
//! `AINetwork_NavPathForPosition(obj+0x30, obj+0x20, 1)` [M 0x158488]: the nearest same-cel node
//! that `NDrone2_MoveTest` (flags 0) reaches, else the nearest node of any path. The nav path is
//! what `NDrone2_ClosestAIPoint` 0x151b18 matches the hide points against, so it picks where the
//! contact runs to hide next. The old port left +0x954 untouched — the first assert fails on it.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::{msg, state};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_contact_re_resolves_its_nav_path_in_the_unreachable_arm() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001"));
    for _ in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { break; }
    }
    w.wait_enabled(6000, "the intro tail");
    let sim = w.scene.sim.as_mut().unwrap();
    let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    assert_eq!(sim.drones.list[ci].voice_type, 9, "skin 0x4f is voice set 9 [M 0x14b300]");
    let route = sim.drones.list[ci].route.clone().expect("the contact carries the mission route (+0x970)");
    let nav = &sim.nav;
    let route_nodes: Vec<[f32; 3]> = route.nodes.iter().map(|&n| nav.paths[route.path].nodes[n as usize].pos).collect();
    // A nav node of a kind-0 path from which no mission-route node is reachable: each route node is
    // outside the < 400 squared gate or fails the drone's MoveTest from the feet — so the arm is
    // the unreachable one [M 0x153a30:213-273].
    let world = &sim.world;
    let far = nav.paths.iter().filter(|p| p.kind & 0xd == 0).flat_map(|p| p.nodes.iter().map(|n| n.pos))
        .find(|np| {
            let feet = [np[0], np[1] + 0.4, np[2]];
            route_nodes.iter().all(|rp| walker::dxz(*rp, *np) >= 20.0 || nav.move_test(world, feet, *rp, false, false).0 != 1)
        })
        .expect("a nav node with no reachable mission-route node");
    let pos = [far[0], far[1] + 1.0, far[2]];
    // The expected path, computed here from the decompile's loop shape: the nearest node within the
    // cel band whose MoveTest passes, else the nearest node anywhere.
    let mut best: Option<(f32, usize)> = None;
    for (pi, p) in nav.paths.iter().enumerate().filter(|(_, p)| p.kind & 0xd == 0) {
        for n in p.nodes.iter().filter(|n| (n.pos[1] - pos[1]).abs() < 4.0) {
            let dd = walker::dxz(n.pos, pos);
            if best.is_none_or(|b| dd < b.0) && nav.node_scan_ok(pos, n.pos) { best = Some((dd, pi)); }
        }
    }
    let want = best.map(|b| b.1).expect("a reachable nav node at the far spot");
    {
        let d = &mut sim.drones.list[ci];
        d.pos = pos;
        d.nav_path = Some(None); // a sentinel the old arm leaves standing
        d.state = state::RETURN_TO_PATROL_PATH;
    }
    let data = sim.data();
    let frame = sim.drones.frame;
    let mut rng = sim.drones.rng.clone();
    let mut dr = sim.drones.list[ci].clone();
    dr.send(&data, &mut rng, frame, msg::ENTER);
    assert_eq!(dr.nav_path, Some(Some(want)),
        "voice set 9 re-resolves +0x954 in ReFindMissionPath's unreachable arm [M 0x153a30:134-137]");

    // Any other voice set leaves +0x954 alone in the same arm.
    let mut other = sim.drones.list[ci].clone();
    other.voice_type = 1;
    let mut rng = sim.drones.rng.clone();
    other.send(&data, &mut rng, frame, msg::ENTER);
    assert_eq!(other.nav_path, Some(None), "the write is voice-set-9 only [M 0x153a30:134]");
}
