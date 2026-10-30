//! Countdown Part 1, the door guards (07000014): the spawner clones the cascade releases hunt
//! west and the route's door node parks them in `DSTATE_OpenDoor` (0x81) at the locked
//! Door_Slide_11A/B pair — the game's uninterruptible locked-door wait [M 0x1681f8: the
//! switch handles ONLY 0/1/2/3/6/7/8/9/0x17/0x18/0x19/0xC/0x20/0x21, so noise/sight/shout
//! re-route to case 0 and drop; case 3 leaves only on `NDrone2_DoorIsOpen`; case 0xC re-arms
//! the 2-tick `NDrone2_OpenDoor` retry with the bark on the door anim's event 6 (case 0x20,
//! `Drone_SfxInfo[0x27]` = sound 1477)]. The port must hold the same wait. Opt-in via NF_ISO.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_door_guards_wait_at_the_locked_door_in_opendoor() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    // The spawners start on 0x8C [D spawners #720/#1333 p_start 0x8C]: seed the sabotage AND's
    // two words through the REAL AND mux (the real laser cut is the walk's leg 3; the panel
    // tests pin the cut itself) [D AND plc (0x87,0x88)→0x8C].
    scene.sim.as_mut().unwrap().drones.channels.set(0x87, 1, 1);
    scene.sim.as_mut().unwrap().drones.channels.set(0x88, 1, 1);
    scene.sim.as_mut().unwrap().bond.pos = [-87.5, -1.7, 33.4]; // the sabotage room, west end
    // The real flow gives the guards about a minute before the E press: the clones hunt west
    // and park at the locked door [M NDrone2_ReachedDestNode 0x153358:144-163 → OpenDoor].
    let mut waiters = Vec::new();
    let mut opendoor_ticks = 0u32;
    for k in 0..2400u32 {
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        // The group-13 spawner's clones (the west-door garrison; the 0x85 pair is door 45/46
        // [D placements 151/152]).
        let clones: Vec<usize> = (0..sim.drones.list.len())
            .filter(|&i| sim.drones.list[i].present && (sim.drones.list[i].group == 9 || sim.drones.list[i].group == 13))
            .collect();
        for &c in &clones {
            if sim.drones.list[c].state == 0x81 {
                opendoor_ticks += 1;
                // At the locked door: east of the door plane (x ≈ −52.4 west face), inside a
                // few units of it — the route's door node, not a wander.
                let p = sim.drones.list[c].pos;
                if p[0] > -54.5 && p[0] < -49.0 && (p[2] - 39.5).abs() < 4.0 && !waiters.contains(&c) {
                    waiters.push(c);
                }
            }
        }
        if k % 600 == 0 {
            let sts: Vec<String> = clones.iter().map(|&c| {
                let x = &sim.drones.list[c];
                format!("#{c} ({:.1},{:.1},{:.1}) st{}", x.pos[0], x.pos[1], x.pos[2], x.state)
            }).collect();
            eprintln!("[014] k{k}: {}", sts.join(" | "));
        }
    }
    assert!(!waiters.is_empty(), "the released clones park at the locked west door in OpenDoor [M 0x153358:144-163]");
    assert!(opendoor_ticks > 1200, "the wait is sustained ({} OpenDoor ticks across the window) [M 0x1681f8 case 3's DoorIsOpen gate]", opendoor_ticks);
    // The door is still locked and shut the whole window [M Door_Update 0x1363c8 — no 0x85 edge].
    let sim = scene.sim.as_ref().unwrap();
    let door = sim.drones.doors.list.iter().position(|d| d.ch_lock == 0x85)
        .expect("Door_Slide_11A/B with lock ch 0x85 exists [D placements 151/152]");
    assert!(sim.drones.doors.is_locked(door, &sim.drones.channels), "the door never unlocked in the wait window");
    assert!(sim.drones.doors.list[door].pos01 < 0.05, "the door never moved in the wait window");
    eprintln!("[014] waiters {:?} ({} OpenDoor ticks)", waiters, opendoor_ticks);
}
