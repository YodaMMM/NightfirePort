//! level-07 route probe: find the walkable real-input route to door #251's threshold that avoids
//! the story plate #412, and print the plate's bounds. Assertion-free, --nocapture.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dump_244(w: &Walker, tag: &str) {
    let sim = w.scene.sim.as_ref().unwrap();
    let i = sim.drones.doors.list.iter().position(|d| d.placement == 244);
    match i {
        Some(i) => {
            let d = &sim.drones.doors.list[i];
            let live = d.live_sph();
            println!("[d244 {tag}] pos {:?} kind {} state {:#04x} flags {:#04x} ch_open {} ch_lock {} glist {:#x}",
                d.pos, d.kind, d.state, d.flags, d.ch_open, d.ch_lock, d.glist);
            println!("[d244 {tag}] closed sph centre {:?} r {:.3} dir {} swing {} approach {} pos01 {:.2}",
                d.sph_centre, d.sph.1, d.dir, d.swing, d.approach, d.pos01);
            println!("[d244 {tag}] live sph centre {:?} r {:.3} matrix row2 {:?}",
                live.0, live.1, [d.matrix[0][2], d.matrix[1][2], d.matrix[2][2]]);
        }
        None => println!("[d244 {tag}] NO door #244 in the list ({})",
            sim.drones.doors.list.iter().map(|d| d.placement).collect::<Vec<_>>().into_iter().take(30).collect::<Vec<_>>().len()),
    }
}

#[test]
fn probe_the_route() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    // drone-bump-collision-4: the crowd answer — step around an armed bot the way a player
    // steers (aim beside it, keep walking; the push slides Bond along the capsule). Opt-in on
    // the walker; the arming itself is untouched [M the chain in the anchors' comment].
    w.crowd_aware = true;
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    dump_244(&w, "boot");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for t in sim.drones.triggers.iter().filter(|t| t.out == 0x48 || t.out == 0x47) {
            println!("[plate] out {:#x} lo {:?} hi {:?} centre {:?}", t.out, t.lo, t.hi, t.centre());
        }
    }
    for (name, at) in [
        // drone-bump-collision-4: the -3 round's north-lane/manual anchors (z ≈ −45.6..−46.6)
        // are WRONG — the nav web's hall nodes sit in the crowd band and the manual lane itself
        // wedges on the hall's west wall at x ≈ 80 (the -3 report's own "What's left" item 1).
        // The hall is crossed on walkreal_07000007_live's PROVEN south-band line — the west
        // mouth (76.6, −50.7) → the floor (83.1, −50.5) → the east (93.3, −50.4) — via
        // walk_nav_to, the exact anchors and leg style walkreal_07000007 walks green through
        // the armed crowd [drone-bump-collision-3: walkreal_07 2/2 on the kind-gated push].
        // The crowd is ARMED (the arming chain: no +0x4F8 collision bits in the party states
        // [M PartyGirlInit 0x1728c0 / PartyGirl 0x160140]; +0x154 on the bot's own control
        // tick [M PreDroneControl 0x148db0:35-48]; the near arm inside 2.0 [M 0x149ec8:30-33,
        // disasm]; kind +0x464 = 0x800 [M 0x140e90:95-101]; the Jointy push [M 0x1ec180:229-281;
        // drone_bump_live 2/2]) — this probe neither disarms nor dodges it; it IS the line a
        // player walks, on walkreal's measured timing.
        ("west aisle south", [35.7f32, 43.6, -70.6]),
        ("west aisle north", [42.6, 43.6, -70.5]),
        ("the west wall run", [42.6, 43.6, -45.0]),
        ("the north door", [49.4, 43.6, -44.7]),
        ("the north row west", [62.7, 43.6, -44.3]),
        ("the north row east", [76.5, 43.6, -44.9]),
        ("the hall west mouth", [76.6, 43.6, -50.7]),
        ("the hall floor", [83.1, 43.6, -50.5]),
        ("the hall east", [93.3, 43.6, -50.4]),
    ] {
        w.walk_nav_to(at, name, 1.5);
        println!("[route] {name} ok bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    }
    println!("[route] before E: ch48 {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) as u8);
    w.walk_to(Wp { at: [94.6, 43.6, -47.4], act: Act::Use { use_ticks: 240 }, arrive: 0.7 }, "E the hall door");
    dump_244(&w, "after E");
    println!("[route] after E: bond {:?} ch48 {}", w.scene.sim.as_ref().unwrap().bond.pos,
        w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) as u8);
    // Post-E legs — drone-bump-collision-10 re-anchor. The old MANUAL straight legs hugged the
    // hall's east furniture too tight and the capsule caught it [D the placement probe
    // probe_07_rimobj]: the ChestOfDrawsSmall_6a/6b pair (world AABB x 90.68..92.32, y
    // 42.89..44.00, z −46.39..−45.49 — the walkable rim at 43.215 the -9 capsule walk measured
    // is the pair's east-face collision edge; a player does not walk the 0.2 ledge of a chest,
    // she walks AROUND it) plus the OBJECT_BookCase (x 86.18..91.38, z −52.75..−48.26) sealing
    // the south-west. The old first leg's line also scraped the pair's east face (x 92.5 line
    // vs the face at 92.32 — 0.18 < the capsule 0.45). The nav web carries the lanes around the
    // furniture (the same web walkreal_07's green crossing walks) — walk_nav_to the same
    // anchors and let it route the gap between the chests and the plate box's west face.
    for (name, at) in [
        ("the hall's north-west corner", [92.6f32, 43.6, -47.5]),
        ("north past the plate's west edge", [92.5, 43.6, -45.3]),
        ("the middle band entry", [91.3, 43.6, -43.6]),
        ("the middle band west", [88.0, 43.6, -40.0]),
        ("#251's south side", [85.5, 43.6, -37.2]),
    ] {
        w.walk_nav_to(at, name, 1.2);
        println!("[route] {name} ok bond {:?} ch48 {} ch130 {}", w.scene.sim.as_ref().unwrap().bond.pos,
            w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) as u8,
            w.scene.sim.as_ref().unwrap().drones.channels.on(130) as u8);
    }
    w.walk_to(Wp::new(85.5, 43.6, -33.4), "through #251");
    println!("[route] through ch48 {} ch130 {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) as u8,
        w.scene.sim.as_ref().unwrap().drones.channels.on(130) as u8);
    println!("[route] through bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        println!("[route] ch0x48 {} ch130 {} door251 state {:#04x}",
            sim.drones.channels.on(0x48) as u8, sim.drones.channels.on(130) as u8,
            sim.drones.doors.list.iter().find(|d| d.placement == 251).map(|d| d.state).unwrap_or(0));
    }
    let _ = Input::default();
}
