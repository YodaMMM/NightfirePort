//! probe14-lid: E-opening vent-lid SP #266 at the drop-slot lip, then the drop through the open
//! slot onto the sleeper (the Jointy push handles the landing). Assertion-free.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{wrap_pi, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn lid_open_drop_watch() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    // crouch on (the crawl keeps Bond under the duct roof)
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.crouched = true;
    }
    for _ in 0..15 { w.tick(nf_game::player::Input::default(), false); }
    w.walk_to(Wp::new(41.0, 2.4, 63.9).r(0.4), "the crawl south to the lip");
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[lid] at the lip: {p:?}");
    }
    // E-press at the lid (SP #266's entity at (40.80, 2.37, 63.66) [D]): face it, one edge per
    // 6 ticks; watch the lid player's state.
    let lid_i = w.scene.sim.as_ref().unwrap().drones.scripts.players.iter()
        .position(|p| p.placement == 266).expect("lid SP #266 exists [D]");
    for k in 0..80u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (40.80f32 - sim.bond.pos[0]).atan2(63.66 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let st = sim.drones.scripts.players[lid_i].state;
        let pressing = st == 1 && k % 6 == 0;
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, pressing);
        if k % 10 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[lid] k{k}: state {} bond {:?}", sim.drones.scripts.players[lid_i].state, sim.bond.pos);
        }
        if st != 1 { eprintln!("[lid] ACTIVATED at k{k}"); break; }
    }
    // The 63-tick open, tracked; then the drop south through the slot.
    for k in 0..100 {
        w.tick(nf_game::player::Input::default(), false);
        if k % 15 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let m = sim.drones.scripts.objects.list.iter().flatten()
                .find_map(|o| match o { nf_game::nis::Obj::Entity { hash, matrix, .. } if *hash == 0x0200_0750 => Some(*matrix), _ => None });
            let slab = sim.world.dynamic_tris_near([40.3, 2.2, 62.5], [41.2, 2.6, 63.2]).len();
            eprintln!("[lid] open k{k}: first-lid-matrix {m:?} slab-tris {slab} bond {:?}", sim.bond.pos);
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.objects.list.iter().flatten()
            .find_map(|o| match o { nf_game::nis::Obj::Entity { hash, matrix, .. } if *hash == 0x0200_0750 => Some(*matrix), _ => None })
            .expect("the lid entity exists");
        eprintln!("[lid] lid entity matrix translation after open: [{:.3},{:.3},{:.3}]", m[3][0], m[3][1], m[3][2]);
    }
    // The drop: y-aware raw walk south until Bond is down on the slot floor (the walker's
    // arrival ignores y, which cut the fall short).
    let mut dropped = false;
    for k in 0..400u32 {
        let (y, z) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos[1], sim.bond.pos[2]) };
        if y < 0.5 && z < 63.0 { dropped = true; break; }
        let desired = if z > 62.9 { std::f32::consts::PI } else { 1.5707964 }; // south (−z), then east (+x)
        let sim = w.scene.sim.as_ref().unwrap();
        let err = wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { forward: 1.0, turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
        if k % 40 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[lid] drop k{k}: {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[lid] after the drop (ok={dropped}): {p:?}");
        let (a, b) = sim.bond.capsule_seg();
        let (push, walkable) = sim.world.capsule_push(a, b, 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        eprintln!("[lid] stall: push {push:?} walkable {walkable} segment {:?}", sim.world.segment(p, [p[0], p[1] - 3.0, p[2]]));
        let dtri = sim.world.dynamic_tris_near([40.4, 2.1, 62.2], [41.5, 3.1, 62.9]);
        eprintln!("[lid] stall dyn tris: {} first {:?}", dtri.len(), dtri.first());
        let sp = sim.drones.list.iter().position(|d| d.skin == 0x0500_0090).unwrap();
        eprintln!("[lid] sleeper now {:?}", sim.drones.list[sp].pos);
    }
    if std::env::var("LID_FULL").is_ok() {
        w.walk_to(Wp::new(41.0, -0.6, 62.8).r(1.0), "turn east on the corridor");
        let ok_east = w.try_walk_to(Wp::new(48.9, -0.6, 62.8).r(0.8), "the corridor east to the x≈49 corner");
        eprintln!("[lid] corridor east ok={ok_east} at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
        let ok_south = w.try_walk_to(Wp::new(49.05, -0.6, 43.6).r(0.9), "the corridor south to the vent mouths");
        {
            let p = w.scene.sim.as_ref().unwrap().bond.pos;
            eprintln!("[lid] corridor south ok={ok_south} at {p:?}");
        }
    }
}
