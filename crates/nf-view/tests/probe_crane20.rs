//! probe20: the deck→yard chain in one run. Test 1 maps the static world (the walkway strip,
//! the rim, the tip, the steps, the dock, the roof, the beam track, the beam shape). Test 2 is
//! the walk-off + chain attempt, parameterised by env so one build serves many runs:
//!   P20_SWINGWAIT: 0 = walk off at rest (no E #450); N>0 = E #450, wait N ticks (mid-swing),
//!                  9999 = E #450 + wait out the full 750-tick swing (end pose).
//!   P20_R / P20_OFF: the aim point on the strip frame (anchor (200.48,−262.37), the rest axis
//!                    unit ((188.65,−268.92)−anchor)/13.5, perp = (−uz,ux)).
//!   P20_TRACE: 1 = print every 4th tick through the walk-off.
//! Print-only; every walk-off run prints the landing spot in the strip frame.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

fn envf(name: &str, d: f32) -> f32 { std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(d) }

fn load_real() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    w
}

const ANCHOR: [f32; 2] = [200.48, -262.37];

/// All ray hits from `top` down to `bot`, printed as y values.
fn raycol(sim: &nf_view::scene::Sim, x: f32, z: f32, top: f32, bot: f32) -> String {
    let mut row = String::new();
    let mut t = top;
    for _ in 0..8 {
        match sim.world.segment_hit([x, t, z], [x, bot, z]) {
            Some((f, _)) => { let y = t - (t - bot) * f; row.push_str(&format!("{y:.2} ")); t = y - 0.05; }
            None => { row.push_str("-- "); break; }
        }
    }
    row
}

fn beam_pos(sim: &nf_view::scene::Sim) -> Option<[f32; 3]> {
    sim.drones.scripts.entity_matrix((0x0600_09ef, 1)).map(|m| [m[3][0], m[3][1], m[3][2]])
}

#[test]
#[ignore = "probe20: the static map; run with --ignored --nocapture"]
fn probe20_static_map() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let sim = w.scene.sim.as_ref().unwrap();
    // A. the rim + the strip along the rest axis.
    let (ax, az) = (ANCHOR[0], ANCHOR[1]);
    let (ux, uz) = ((188.65f32 - ax) / 13.5, (-268.92f32 - az) / 13.5);
    let (px, pz) = (-uz, ux);
    for r in [3.0f32, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5, 8.0, 8.5, 9.0, 10.0, 11.0, 12.0, 13.0, 13.5] {
        for off in [-1.5f32, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5] {
            let (x, z) = (ax + r * ux + off * px, az + r * uz + off * pz);
            let row = raycol(sim, x, z, 20.0, -1.0);
            eprintln!("[p20] axis r{r:>4.1} off{off:+.1} ({x:.1},{z:.1}): {row}");
        }
    }
    // B. the dock / building / roof grid.
    for z in [-264.0f32, -266.0, -268.0, -270.0, -272.0, -274.0, -276.0, -278.0, -280.0, -283.0, -286.0, -289.0, -292.0] {
        let mut row = String::new();
        for x in [184.0f32, 186.0, 188.0, 190.0, 192.0, 194.0, 196.0, 198.0, 200.0] {
            let hit = sim.world.segment_hit([x, 21.0, z], [x, -1.0, z]).map(|(t, _)| 21.0 - 22.0 * t);
            row.push_str(&format!("x{x:.0}:{hit:>5?} "));
        }
        eprintln!("[p20] dock z{z}: {row}");
    }
    // C. the beam track before the E: the entity's rest origin + a coarse footprint map.
    for e in 0..4u8 {
        let m = sim.drones.scripts.entity_matrix((0x0600_09ef, e));
        eprintln!("[p20] ship entity {e} rest {:?}", m.map(|m| [m[3][0], m[3][1], m[3][2]]));
    }
    for z in [-278.0f32, -280.0, -283.0, -286.0, -287.1, -288.0, -290.0, -292.0] {
        let mut row = String::new();
        for x in [135.0f32, 140.0, 145.0, 150.0, 155.0, 160.0, 165.0, 170.0, 175.0, 180.0] {
            let hit = sim.world.segment_hit([x, 17.0, z], [x, -1.0, z]).map(|(t, _)| 17.0 - 18.0 * t);
            row.push_str(&format!("x{x:.0}:{hit:>5?} "));
        }
        eprintln!("[p20] beamrest z{z}: {row}");
    }
    // D. the beam shapes' bounds.
    for h in [0x0200_0a80u32, 0x0200_0a81] {
        if let Some(t) = sim.world.shape(h) {
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for tri in t { for v in tri { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
            eprintln!("[p20] shape {h:08x} {} tris lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})", t.len(), lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        } else { eprintln!("[p20] shape {h:08x} absent"); }
        if let Some((lo, hi)) = sim.glist_bounds.get(&h) {
            eprintln!("[p20] glist {h:08x} lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        }
    }
    // E. the roof/catwalk over the beam's west half: x 137..185 at z −285..−291, top down from 21.
    for z in [-285.0f32, -287.1, -289.0, -291.0] {
        let mut row = String::new();
        for x in [137.0f32, 142.0, 147.0, 152.0, 158.0, 164.0, 170.0, 176.0, 182.0, 188.0, 193.0] {
            let hit = sim.world.segment_hit([x, 21.0, z], [x, 8.0, z]).map(|(t, _)| 21.0 - 13.0 * t);
            row.push_str(&format!("x{x:.0}:{hit:>5?} "));
        }
        eprintln!("[p20] roof z{z}: {row}");
    }
}

#[test]
#[ignore = "probe20: the fine maps (the catwalk gaps, the stairwell, the dock bay); run with --ignored --nocapture"]
fn probe20_fine_maps() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let sim = w.scene.sim.as_ref().unwrap();
    // A. The catwalk gaps: z −283..−292 × x 137..193, rays from y 17.4 (under the higher roof)
    //    down — a hole reads the yard (≈3.98/0.25) or the beam (14.32 at rest only x≤142).
    for z in [-283.0f32, -285.0, -286.5, -288.0, -289.5, -291.0] {
        let mut row = String::new();
        let mut x = 137.0f32;
        while x <= 193.5 {
            let hit = sim.world.segment_hit([x, 17.4, z], [x, 0.0, z]).map(|(t, _)| 17.4 - 17.4 * t);
            row.push_str(&format!("x{x:.0}:{hit:>4.1?} "));
            x += 2.0;
        }
        eprintln!("[p20] cat z{z}: {row}");
    }
    // B. The stairwell + the under-roof north strip: x 185..197 × z −264..−274 from y 18 down.
    for z in [-264.0f32, -265.5, -267.0, -268.5, -270.0, -271.5, -273.0, -274.5] {
        let mut row = String::new();
        let mut x = 185.0f32;
        while x <= 197.5 {
            let hit = sim.world.segment_hit([x, 18.0, z], [x, 0.0, z]).map(|(t, _)| 18.0 - 18.0 * t);
            row.push_str(&format!("x{x:.0}:{hit:>4.1?} "));
            x += 1.25;
        }
        eprintln!("[p20] north z{z}: {row}");
    }
    // C. The dock bay under the roof: x 184..197 × z −282..−292 from y 17.4 (the roof first hit
    //    excluded by starting under it) — the dock floor and any intermediate stair surfaces.
    for z in [-282.0f32, -284.0, -286.0, -288.0, -290.0, -292.0] {
        let mut row = String::new();
        let mut x = 184.0f32;
        while x <= 197.5 {
            let hit = sim.world.segment_hit([x, 17.4, z], [x, 0.0, z]).map(|(t, _)| 17.4 - 17.4 * t);
            row.push_str(&format!("x{x:.0}:{hit:>4.1?} "));
            x += 1.25;
        }
        eprintln!("[p20] bay z{z}: {row}");
    }
    // D. Horizontal rays through the stairwell volume at walk heights — where can a body at
    //    y 10-18 cross x 186..196 at z −267..−271? Ray from (185,z) east.
    for y in [10.0f32, 12.0, 13.4, 14.5, 16.0, 17.5] {
        for z in [-266.0f32, -268.0, -269.5, -271.0] {
            let hit = sim.world.segment([185.0, y, z], [197.0, y, z]);
            let hit2 = sim.world.segment([197.0, y, z], [185.0, y, z]);
            eprintln!("[p20] horiz y{y} z{z}: west→east {hit:?} east→west {hit2:?}");
        }
    }
    // E. The vertical passages: rays UP from y 9.8 at the dock/steps — where is the sky (the
    //    stairwell's open top) between the dock and the roof?
    for z in [-267.0f32, -268.5, -270.0, -271.5] {
        let mut row = String::new();
        let mut x = 185.0f32;
        while x <= 197.5 {
            let hit = sim.world.segment_hit([x, 9.9, z], [x, 19.0, z]).map(|(t, _)| 9.9 + 9.1 * t);
            row.push_str(&format!("x{x:.0}:{hit:>4.1?} "));
            x += 1.25;
        }
        eprintln!("[p20] up z{z}: {row}");
    }
}

#[test]
#[ignore = "probe20: the walkway/steps wall scan; run with --ignored --nocapture"]
fn probe20_walkway_scan() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let sim = w.scene.sim.as_ref().unwrap();
    // The walkway + wall region under the canopy: x 184..197 step 0.5, z −263..−270 step 0.5,
    // ray from y 14.6 (under the roof edge) — every surface below.
    let mut z = -263.0f32;
    while z >= -269.75 {
        let mut row = String::new();
        let mut x = 184.0f32;
        while x <= 197.0 {
            let mut col = String::new();
            let mut top = 14.6f32;
            for _ in 0..4 {
                match sim.world.segment_hit([x, top, z], [x, 0.0, z]) {
                    Some((f, _)) => { let y = top - top * f; col.push_str(&format!("{y:.2} ")); top = y - 0.05; }
                    None => { col.push_str("-- "); break; }
                }
            }
            row.push_str(&format!("x{x:.0}[{col}] "));
            x += 1.0;
        }
        eprintln!("[p20] wall z{z:.1}: {row}");
        z -= 0.75;
    }
    // Dense slot scan: x 184..196 step 0.25, z −266.0..−269.4 step 0.25 from y 14.6 — find the
    // bridge/plank across the void slot and the steps' top island's exact extent.
    let mut z2 = -266.0f32;
    while z2 >= -269.4 {
        let mut row = String::new();
        let mut x = 184.0f32;
        while x <= 196.0 {
            let hit = sim.world.segment_hit([x, 14.6, z2], [x, 0.0, z2]).map(|(t, _)| 14.6 - 14.6 * t);
            row.push_str(&format!("{hit:>4.1?} "));
            x += 0.5;
        }
        eprintln!("[p20] slot z{z2:.2}: {row}");
        z2 -= 0.25;
    }
    // Horizontal body-band rays north→south at the walkway heights: where is the south wall open?
    for y in [12.6f32, 13.25] {
        for x in [184.5f32, 186.0, 187.5, 188.5, 189.0, 189.5, 190.5, 192.0, 194.0, 195.5] {
            let hit = sim.world.segment([x, y, -264.8], [x, y, -269.5]);
            let hit2 = sim.world.segment([x, y, -269.5], [x, y, -264.8]);
            eprintln!("[p20] wall y{y} x{x}: n→s {hit:?} s→n {hit2:?}");
        }
    }
}

#[test]
#[ignore = "probe20: the beam watch (E #460 from the bay); run with --ignored --nocapture"]
fn probe20_beam_watch() {
    let mut w = load_real();
    // Park in the dock bay next to the control corner (the proven E approach).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [189.0, 10.6, -287.6];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let used = w.try_walk_to(Wp::at(189.0, 9.9, -288.7, Act::Use { use_ticks: 240 }), "E #460");
    eprintln!("[p20] E460 {used} ch0x2b {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b));
    // BOTH script objects: the beam 0x0200_0a80 and the trolley 0x0200_0a81 — the trolley's top
    // (11.33+2.95 = 14.28) matches the pad (14.3-14.7, x 186..192.5, z −269..−273); if it travels
    // east past the beam's 177.22 limit, it is the boarding car.
    for k in 0..100u32 {
        w.wait(30);
        let sim = w.scene.sim.as_ref().unwrap();
        let mut rows: Vec<(u32, [f32; 3])> = sim.drones.scripts.objects.list.iter().flatten()
            .filter_map(|o| if let nf_game::nis::Obj::Entity { hash, matrix, .. } = o {
                (*hash == 0x0200_0a80 || *hash == 0x0200_0a81)
                    .then_some((*hash, [matrix[3][0], matrix[3][1], matrix[3][2]]))
            } else { None })
            .collect();
        rows.sort_by(|a, b| (a.0, a.1[0].to_bits()).cmp(&(b.0, b.1[0].to_bits())));
        rows.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
        if k % 2 == 0 || rows.iter().any(|(h, p)| *h == 0x0200_0a81 && p[0] > 172.0) {
            for (h, p) in &rows {
                eprintln!("[p20] obj {h:08x} t{} ({:.2},{:.2},{:.2})", (k + 1) * 30, p[0], p[1], p[2]);
            }
        }
    }
}

#[test]
#[ignore = "probe20: leg2 (the walkway→bay drop + E #460) and the roof-stair scan; run with --ignored --nocapture"]
fn probe20_leg2_dock() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    assert!(w.try_walk_to(Wp { at: [198.93, 14.16, -264.15], act: Act::None, arrive: 0.8 }, "the cab"));
    assert!(w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 60 }), "E #450"));
    w.wait(900);
    // The walk-off (probe20_walk_off's proven parameters).
    let (ax, az) = (200.48f32, -262.37f32);
    let (ux, uz) = ((188.65f32 - ax) / 13.5, (-268.92f32 - az) / 13.5);
    let (px, pz) = (-uz, ux);
    let tx = ax + 6.0 * ux + 0.75 * px;
    let tz = az + 6.0 * uz + 0.75 * pz;
    for k in 0..160u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let desired = (tx - b[0]).atan2(tz - b[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            if err.abs() < 0.05 { break; }
            let mut sim_m = w.scene.sim.as_mut().unwrap();
            sim_m.step_bond(Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false, false, false);
        }
        w.total_ticks += 1;
    }
    let mut landed = false;
    for k in 0..400u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let desired = (tx - b[0]).atan2(tz - b[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let mut sim_m = w.scene.sim.as_mut().unwrap();
            sim_m.step_bond(Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false, false, false);
        }
        w.total_ticks += 1;
        let b = pos(&w);
        if w.scene.sim.as_ref().unwrap().bond.grounded && b[1] > 12.55 && b[1] < 13.65 && k > 40 {
            eprintln!("[p20] ON THE STRIP at {:?}", b);
            landed = true;
            break;
        }
    }
    assert!(landed, "the walk-off must land on the strip");
    // Leg 2: walk the strip west to its walkway (z −265.7), then off the walkway's SOUTH edge at
    // x ~185 — the bay's north-west corner (9.0) sits 3.1 m below [probe20 slot scan].
    for (at, rr, label) in [
        ([193.0f32, 13.2, -265.7], 1.2, "the walkway west"),
        ([184.0, 13.2, -265.7], 1.2, "the walkway mid"),
        ([172.0, 13.2, -265.7], 1.2, "the walkway far west"),
        ([165.5, 13.2, -265.7], 1.0, "the gap approach"),
        ([161.5, 9.6, -267.6], 1.8, "the drop through the floor gap to the bay"),
    ] {
        if !w.try_walk_to(Wp { at, act: Act::None, arrive: rr }, label) {
            eprintln!("[p20] leg2 stuck: {label} at {:?}", pos(&w));
        }
    }
    w.wait(80);
    eprintln!("[p20] after the drop: {:?} grounded {}", pos(&w), w.scene.sim.as_ref().unwrap().bond.grounded);
    // The bay walk to the control corner + E #460.
    for (at, rr, label) in [
        ([187.0f32, 10.0, -270.5], 1.5, "the bay north"),
        ([188.5, 10.0, -278.0], 2.0, "the bay mid"),
        ([189.3, 9.8, -287.2], 1.0, "the control corner"),
    ] {
        if !w.try_walk_to(Wp { at, act: Act::None, arrive: rr }, label) {
            eprintln!("[p20] bay leg stuck: {label} at {:?}", pos(&w));
        }
    }
    let used = w.try_walk_to(Wp::at(189.0, 9.9, -288.7, Act::Use { use_ticks: 240 }), "E #460");
    eprintln!("[p20] E460 {used} ch0x2b {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b));
    // Watch BOTH script objects (the beam 0x0200_0a80 and the trolley 0x0200_0a81) — the pad
    // (14.7, x 186..192.5, z −269..−273) matches the trolley's top (11.33+2.95); if the trolley
    // travels east past the beam's 177.22 limit, it is the boarding car.
    for k in 0..100u32 {
        w.wait(30);
        let sim = w.scene.sim.as_ref().unwrap();
        let objs: Vec<(u32, [f32; 3])> = sim.drones.scripts.objects.list.iter().flatten()
            .filter_map(|o| if let nf_game::nis::Obj::Entity { hash, matrix, .. } = o {
                (*hash == 0x0200_0a80 || *hash == 0x0200_0a81)
                    .then_some((*hash, [matrix[3][0], matrix[3][1], matrix[3][2]]))
            } else { None })
            .collect();
        let (bx, tx2) = (beam_pos(&sim).unwrap_or([-1.0; 3]), objs.clone());
        let _ = bx;
        if k % 2 == 0 || tx2.iter().any(|(h, p)| *h == 0x0200_0a81 && p[0] > 170.0) {
            for (h, p) in &tx2 {
                eprintln!("[p20] obj {h:08x} t{} ({:.2},{:.2},{:.2})", (k + 1) * 30, p[0], p[1], p[2]);
            }
        }
    }
}

#[test]
#[ignore = "probe20: the roof-access scan (the apron face + the bay ceiling edges); run with --ignored --nocapture"]
fn probe20_roof_scan() {
    let mut w = load_real();
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    // A. Rays UP from the apron/bay floors: any stair surfaces 9.5..18.5 on the building's east
    //    face (x 194..202, z −272..−284) and west face (x 178..186, z −268..−284)?
    for (x0, x1, z0, z1, tag) in [(194.0f32, 202.0, -272.0f32, -284.0f32, "east"),
                                 (178.0, 186.0, -268.0, -284.0, "west")] {
        let mut z = z0;
        while z >= z1 {
            let mut row = String::new();
            let mut x = x0;
            while x <= x1 {
                let hit = sim.world.segment_hit([x, 9.8, z], [x, 18.9, z]).map(|(t, _)| 9.8 + 9.1 * t);
                row.push_str(&format!("x{x:.0}:{hit:>4.1?} "));
                x += 1.0;
            }
            eprintln!("[p20] roofup {tag} z{z}: {row}");
            z -= 1.5;
        }
    }
    // B. Horizontal rays INTO the building at 10..18: the passages through its east/west faces.
    for y in [10.5f32, 12.5, 14.7, 16.5, 18.0] {
        for z in [-270.0f32, -274.0, -278.0, -282.0] {
            let e = sim.world.segment([203.0, y, z], [190.0, y, z]);
            let wst = sim.world.segment([176.0, y, z], [190.0, y, z]);
            eprintln!("[p20] hface y{y} z{z}: east→west {e:?} west→east {wst:?}");
        }
    }
    // C. The 14.7 pad's extent: rays up from 9.9 over x 178..197, z −269..−273.
    for z in [-269.0f32, -270.0, -271.0, -272.0, -273.0] {
        let mut row = String::new();
        let mut x = 178.0f32;
        while x <= 197.0 {
            let hit = sim.world.segment_hit([x, 9.9, z], [x, 18.9, z]).map(|(t, _)| 9.8 + 9.2 * t);
            row.push_str(&format!("x{x:.0}:{hit:>4.1?} "));
            x += 1.0;
        }
        eprintln!("[p20] pad z{z}: {row}");
    }
}

#[test]
#[ignore = "probe20: the walk-off; run with --ignored --nocapture"]
fn probe20_walk_off() {
    let swing_wait = envf("P20_SWINGWAIT", 9999.0) as u32;
    let aim_r = envf("P20_R", 7.25);
    let aim_off = envf("P20_OFF", -0.375);
    let trace = envf("P20_TRACE", 0.0) > 0.5;
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    assert!(w.try_walk_to(Wp { at: [198.93, 14.16, -264.15], act: Act::None, arrive: 0.8 }, "the cab"));
    let swung = swing_wait > 0;
    if swung {
        assert!(w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 60 }), "E #450"),
            "E #450 must fire");
        if swing_wait >= 9999 {
            w.wait(900);
        } else {
            w.wait(swing_wait);
        }
        let rot = w.scene.sim.as_ref().unwrap().drones.scripts.entity_matrix((0x0600_0a08, 0))
            .map(|m| m[0][2].atan2(m[0][0]));
        eprintln!("[p20] walk-off at swing t{swing_wait} rot {rot:?} bond {:?}", pos(&w));
    } else {
        eprintln!("[p20] walk-off at REST bond {:?}", pos(&w));
    }
    // The walk-off: aim at the strip point, forward held, turn-converged. Trace the fall.
    let (ax, az) = (ANCHOR[0], ANCHOR[1]);
    let (ux, uz) = ((188.65f32 - ax) / 13.5, (-268.92f32 - az) / 13.5);
    let (px, pz) = (-uz, ux);
    let tx = ax + aim_r * ux + aim_off * px;
    let tz = az + aim_r * uz + aim_off * pz;
    eprintln!("[p20] aim ({tx:.2},{tz:.2}) r{aim_r} off{aim_off}");
    // Phase 1: turn in place until the aim error is small (the post-swing carry can leave Bond
    // facing anywhere; walking while turning wanders him into the mast).
    for k in 0..160u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let desired = (tx - b[0]).atan2(tz - b[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            if err.abs() < 0.05 { break; }
            let mut sim_m = w.scene.sim.as_mut().unwrap();
            sim_m.step_bond(Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false, false, false);
        }
        w.total_ticks += 1;
    }
    eprintln!("[p20] pre-turned at {:?}", pos(&w));
    let mut landed_strip = false;
    let mut stalled = 0u32;
    let mut last = pos(&w);
    for k in 0..700u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let dx = b[0] - ax;
            let dz = b[2] - az;
            let r_now = dx * ux + dz * uz;
            let desired = (tx - b[0]).atan2(tz - b[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let grounded = sim.bond.grounded;
            let moved = walker::dxz(b, last);
            last = b;
            stalled = if grounded && moved < 0.012 && r_now < 5.5 { stalled + 1 } else { 0 };
            if stalled == 30 { eprintln!("[p20] STALL at ({:.2},{:.2},{:.2}) k{k}", b[0], b[1], b[2]); }
            if stalled > 120 { eprintln!("[p20] ABORT: stalled out at {b:?}"); break; }
            let mut sim_m = w.scene.sim.as_mut().unwrap();
            sim_m.step_bond(Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false, false, false);
        }
        w.total_ticks += 1;
        let b = pos(&w);
        let g = w.scene.sim.as_ref().unwrap().bond.grounded;
        if trace && k % 4 == 0 {
            eprintln!("[p20] t{k} ({:.2},{:.2},{:.2}) g{g}", b[0], b[1], b[2]);
        }
        if g && k > 30 {
            let dx = b[0] - ax;
            let dz = b[2] - az;
            let r = dx * ux + dz * uz;
            let off = dx * px + dz * pz;
            if r > 5.5 && b[1] > 12.55 && b[1] < 13.65 {
                eprintln!("[p20] LANDED HIGH k{k} ({:.2},{:.2},{:.2}) r{r:.2} off{off:.2}", b[0], b[1], b[2]);
                if off > -1.6 && off < 1.4 { landed_strip = true; }
                break;
            }
            if b[1] < 11.0 {
                eprintln!("[p20] LANDED LOW k{k} ({:.2},{:.2},{:.2}) r{r:.2} off{off:.2} — missed the strip", b[0], b[1], b[2]);
                break;
            }
        }
    }
    eprintln!("[p20] landed_strip {landed_strip} at {:?}", pos(&w));
    if !landed_strip { return; }
    // Out along the axis to the strip's end, down to the dock.
    for k in 0..260u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let desired = (188.65f32 - b[0]).atan2(-268.92f32 - b[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let mut sim_m = w.scene.sim.as_mut().unwrap();
            sim_m.step_bond(Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false, false, false);
        }
        w.total_ticks += 1;
        if k % 20 == 0 { eprintln!("[p20] out k{k} ({:.2},{:.2},{:.2})", pos(&w)[0], pos(&w)[1], pos(&w)[2]); }
        if pos(&w)[1] < 10.4 { break; }
    }
    w.wait(80);
    let dock = pos(&w);
    eprintln!("[p20] DOCK {dock:?} grounded {}", w.scene.sim.as_ref().unwrap().bond.grounded);
    if dock[1] < 8.5 || dock[1] > 11.2 { eprintln!("[p20] ABORT: not on the dock"); return; }
    // The dock walk to the control corner + E #460.
    for (at, rr, label) in [
        ([196.0f32, 9.7, -277.0], 2.0, "the dock strip"),
        ([192.0, 9.7, -283.0], 1.5, "the dock south"),
        ([189.3, 9.8, -287.2], 1.0, "the control corner"),
    ] {
        if !w.try_walk_to(Wp { at, act: Act::None, arrive: rr }, label) {
            eprintln!("[p20] dock leg stuck: {label} at {:?}", pos(&w));
        }
    }
    let used = w.try_walk_to(Wp::at(189.0, 9.9, -288.7, Act::Use { use_ticks: 240 }), "E #460");
    eprintln!("[p20] E460 {used} ch0x2b {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b));
    // The beam's shuttle: the entity's origin every 60 ticks for 3000 ticks.
    for k in 0..50 {
        w.wait(60);
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = beam_pos(&sim).unwrap_or([-1.0; 3]);
        let b = pos(&w);
        eprintln!("[p20] beam t{} ({:.2},{:.2},{:.2}) bond ({:.2},{:.2},{:.2})", (k + 1) * 60, bp[0], bp[1], bp[2], b[0], b[1], b[2]);
        if k == 24 {
            // Mid-shuttle: ray the beam's walk surface at its current x.
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some([bx, by, bz]) = beam_pos(&sim) {
                for dz in [-1.5f32, -0.75, 0.0, 0.75, 1.5] {
                    let hit = sim.world.segment_hit([bx, by + 4.0, bz + dz], [bx, by - 4.0, bz + dz])
                        .map(|(t, _)| by + 4.0 - 8.0 * t);
                    eprintln!("[p20] beamtop t1500 x{bx:.2} z{:.2} dz{dz:+.1} top {hit:?}", bz + dz);
                }
            }
        }
    }
    for f in &w.report.falls { eprintln!("[p20] FALL {f}"); }
}

#[test]
#[ignore = "probe20: the building interior scan; run with --ignored --nocapture"]
fn probe20_interior_scan() {
    let mut w = load_real();
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    // The interior under the roof: rays up from y 1.3 over x 184..202, z −272..−293 — the
    // interior floor, the ceilings, and any interior stair surfaces (10.3/10.6 readings seen
    // from above at x 192-194/195).
    for z in [-272.0f32, -275.0, -278.0, -281.0, -284.0, -287.0, -290.0, -293.0] {
        let mut row = String::new();
        let mut x = 184.0f32;
        while x <= 202.0 {
            let mut col = String::new();
            let mut bot = 1.3f32;
            for _ in 0..5 {
                match sim.world.segment_hit([x, bot, z], [x, 19.0, z]) {
                    Some((f, _)) => { let y = bot + (19.0 - bot) * f; col.push_str(&format!("{y:.2} ")); bot = y + 0.05; }
                    None => { col.push_str("-- "); break; }
                }
            }
            row.push_str(&format!("x{x:.0}[{col}] "));
            x += 1.5;
        }
        eprintln!("[p20] int z{z}: {row}");
    }
    // The interior's ground-level entrances: horizontal rays at y 2.0 and 4.5 across x 176..204.
    for y in [2.0f32, 4.5] {
        for z in [-274.0f32, -278.0, -282.0, -286.0, -290.0] {
            let e = sim.world.segment([204.0, y, z], [184.0, y, z]);
            let wst = sim.world.segment([176.0, y, z], [196.0, y, z]);
            eprintln!("[p20] int h y{y} z{z}: east→west {e:?} west→east {wst:?}");
        }
    }
}

#[test]
#[ignore = "probe20: the walkway west-of-184 scan; run with --ignored --nocapture"]
fn probe20_west_scan() {
    let mut w = load_real();
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    // The walkway's west continuation: x 148..185, z −263..−270, from y 14.6 down.
    for z in [-263.0f32, -264.0, -265.0, -265.7, -266.3, -267.0, -268.0, -269.0] {
        let mut row = String::new();
        let mut x = 148.0f32;
        while x <= 185.0 {
            let hit = sim.world.segment_hit([x, 14.6, z], [x, 0.0, z]).map(|(t, _)| 14.6 - 14.6 * t);
            row.push_str(&format!("x{x:.0}:{hit:>4.1?} "));
            x += 1.0;
        }
        eprintln!("[p20] west z{z}: {row}");
    }
    // Horizontal at walk heights from the walkway west: can a body cross x 196→170 at y 12.6/13.25?
    for y in [12.6f32, 13.25] {
        for z in [-265.0f32, -265.7, -266.2] {
            let hit = sim.world.segment([196.0, y, z], [148.0, y, z]);
            eprintln!("[p20] westh y{y} z{z}: 196→148 {hit:?}");
        }
        for x in [150.0f32, 156.0, 162.0, 168.0, 174.0, 180.0] {
            let hit = sim.world.segment([x, y, -264.8], [x, y, -269.5]);
            eprintln!("[p20] southwall y{y} x{x}: n→s {hit:?}");
        }
    }
}
