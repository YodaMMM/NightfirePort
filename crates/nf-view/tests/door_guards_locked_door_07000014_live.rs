//! 07000014: the guards the fusebox release (ch 0x87 + ch 0x88 -> 0x8C) sends toward the locked
//! west door must not path through it. The game's A* rejects a link between two door-type nodes
//! while the node's bound door is locked [M AINetwork_DoAStarPath 0x1571b0:262-271 →
//! Door_IsLocked 0x136810], so their combat route to Bond fails and they hold
//! COMBAT_NO_ROUTE 0x5f at the door — no OpenDoor 0x81 [L 2026-10-30 PCSX2 fresh boot, Bond
//! held in the generator hall: CUR 0x5f / PREV 0x5a at the door, never 0x81].
//! The route ends: mission routes arrive at 0.5 [M NDrone2_InitAIPath 0x153720:113; L the live
//! route+0x8C = 0.500].

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const OPEN_DOOR: u8 = 0x81;

#[test]
fn released_guards_do_not_work_the_locked_west_door() {
    let mut scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    let door = scene.sim.as_ref().unwrap().drones.doors.list.iter().position(|d| d.ch_lock == 0x85)
        .expect("the west door with lock ch 0x85 [D placements 151/152]");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x87, 1, 1);
        sim.drones.channels.set(0x88, 1, 1);
    }
    let hall = [-76.8, -1.1 + 1.0347, 46.5];
    let mut at_door_max = 0usize;
    let mut came = false;
    for t in 0..3000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = hall;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.doors.is_locked(door, &sim.drones.channels), "the door stays locked (no cascade fired)");
        let near: Vec<&nf_game::drone::Drone> = sim.drones.list.iter()
            .filter(|d| d.health > 0.0 && (d.pos[0] + 51.5).powi(2) + (d.pos[2] - 40.0).powi(2) < 9.0).collect();
        at_door_max = at_door_max.max(near.len());
        came |= !near.is_empty();
        for d in &near {
            assert_ne!(d.state, OPEN_DOOR, "t{t}: a released guard works the LOCKED door at ({:.1},{:.1}) — the A* door gate [M 0x1571b0:262-271]", d.pos[0], d.pos[2]);
        }
    }
    assert!(came, "the released guards reach the door side [L: two guards stop at (-51.6,41.2) / (-51.2,38.8)]");
    eprintln!("[014] guards near the door (max at once): {at_door_max}");
}

/// Sister: the gate is the LOCK, not the door. With the lock channel on (the cascade's ch 0x85),
/// the same door is no route block (the A* expands through it).
#[test]
fn the_unlocked_door_is_no_route_block() {
    let scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_ref().unwrap();
    let door = sim.drones.doors.list.iter().position(|d| d.ch_lock == 0x85).unwrap();
    let mut ch = sim.drones.channels.clone();
    assert!(sim.drones.doors.is_locked(door, &ch), "locked at load");
    ch.set(0x85, 1, 1);
    assert!(!sim.drones.doors.is_locked(door, &ch), "ch 0x85 unlocks it [M Door_IsLocked 0x136810:10-16]");
}
