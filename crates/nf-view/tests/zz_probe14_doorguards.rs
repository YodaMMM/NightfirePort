//! Probe (uncommitted): 07000014 door guards after the 0x87/0x88 release, Bond held in the
//! generator hall out of sight. Prints the guards near the west door every 30 ticks.
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore]
fn probe14_door_guards_from_the_hall() {
    let mut scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    let door = scene.sim.as_ref().unwrap().drones.doors.list.iter().position(|d| d.ch_lock == 0x85).unwrap();
    scene.sim.as_mut().unwrap().drones.channels.set(0x87, 1, 1);
    scene.sim.as_mut().unwrap().drones.channels.set(0x88, 1, 1);
    let hall = [-76.8, -1.1 + 1.0347, 46.5];
    for t in 0..3000u32 {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond.pos = hall;
            sim.step_bond(nf_game::player::Input::default(), false, false, false);
        }
        if t % 30 == 0 {
            let sim = scene.sim.as_ref().unwrap();
            let mut line = format!("t{t} door st {:?} |", sim.drones.doors.list[door].state);
            for (i, d) in sim.drones.list.iter().enumerate() {
                let (dx, dz) = (d.pos[0] + 56.0, d.pos[2] - 40.0);
                if dx * dx + dz * dz < 22.0 * 22.0 {
                    line += &format!(" #{i} st {:#x} ({:.1},{:.1},{:.1}) al {:.2} los {} seen {} ev {}", d.state, d.pos[0], d.pos[1], d.pos[2], d.alert, d.los, d.seen, d.ever_seen);
                    if let Some(r) = d.route.as_ref() { line += &format!(" rt dest ({:.2},{:.2}) tgt ({:.2},{:.2}) rem {:.2} idx {}/{}", r.dest[0], r.dest[2], r.target[0], r.target[2], r.remaining, r.index, r.nodes.len()); }
                }
            }
            eprintln!("{line}");
        }
    }
}
