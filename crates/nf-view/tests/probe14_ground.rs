//! level-14-interactables-2 probe: the crouch grounded band. The port computes the contact as
//! `t·ray` with `ray = cc + 0.6`; the game's `Player_FeetOnPoint` 0x194188 reads `HITDATA+0xC`,
//! which `Intersect_RayGeom` 0x10a8c8 fills with the ABSOLUTE distance `|from − hit|` (hit =
//! `from + t·dir`, :176-179/:252-256, the sqrt-of-squares store :312-325), and builds its ray as
//! `to = HITTEST+0x30 + up·(−(cc+0.6))` (:89-98) where `HITTEST+0x30` = `BLData+0x770` is the
//! crouch capsule's B point `pos + up·(0.45000002 − cc)` [M `Player_Collision` 0x194df0 case 3 +
//! tail store :405-408]. This probe prints both chains bit-exactly at the live crouch rest.
//! Assertion-free.

use nf_game::player::Input;
use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn wrap_pi(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; }
    a
}

fn hb(f: f32) -> String { format!("{:08x}", f.to_bits()) }

#[test]
fn probe_crouch_ground_chain() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();

    // The crawl north from the start (probe14_vent's walk) — the flicker zone before the grate.
    for k in 0..480 {
        let p = sim.bond.pos;
        let desired = (40.7 - p[0]).atan2(39.0 - p[2]);
        let err = wrap_pi(desired - sim.bond.yaw);
        sim.step_bond(Input { forward: 1.0, turn: err.clamp(-0.08, 0.08), crouch: true, ..Default::default() }, false, false, false);
        let b = &sim.bond;
        if !b.crouched { continue; }
        let p = b.pos;
        let cc = b.cc;

        // The port's current chain.
        let ray = cc + 0.6;
        let to_port = [p[0], p[1] - ray, p[2]];

        // The game's chain: B = pos + up·(0.45000002 − cc) [M 0x194df0 case 3, 0x3EC66667];
        // to = B + up·(−(cc+0.6)) [M 0x194188:89-98, up = (0,1,0) so the muls are exact].
        let b_off = 0.450_000_02f32 - cc;
        let b_y = p[1] + b_off;
        let scalar = -(cc + 0.6);
        let to_game_y = b_y + scalar;
        let to_game = [p[0], to_game_y, p[2]];

        let hit_port = sim.world.segment_hit_scoped(b.cel, p, to_port);
        let hit_game = sim.world.segment_hit_scoped(b.cel, p, to_game);

        let port_contact = hit_port.map(|(t, _)| t * ray);
        let game_contact = hit_game.map(|(t, _)| {
            let d_y = to_game[1] - p[1]; // [M 0x1ebaa8 0x201 branch: dir = to − from, f32]
            let prod = t * d_y;          // [M 0x10a8c8:176 dir·t]
            let hit_y = p[1] + prod;     // [M 0x10a8c8:177 from + dir·t]
            let dy = p[1] - hit_y;       // [M 0x10a8c8:313-314 from − hit]
            (dy * dy).sqrt()             // [M 0x10a8c8:318-325 sqrt-of-squares]
        });
        let band = |v: &Option<f32>| match v {
            Some(v) => format!("{}", v - 0.1 <= cc),
            None => "nohit".into(),
        };

        if k < 40 || k % 30 == 0 {
            eprintln!(
                "t{k}: y {} grounded {} cc {} ray {} b_off {} to_game_y {} d_y {} t_port {:?} t_game {:?} | port {port_contact:?} band{} | game {game_contact:?} band{}",
                hb(p[1]), b.grounded, hb(cc), hb(ray), hb(b_off), hb(to_game_y),
                hit_game.as_ref().map(|_| hb(to_game[1] - p[1])).unwrap_or_default(),
                hit_port.map(|(t, _)| hb(t)), hit_game.map(|(t, _)| hb(t)),
                band(&port_contact), band(&game_contact),
            );
        }
        if sim.bond.pos[2] < 41.0 { eprintln!("REACHED the #301 band at t{k}"); break; }
    }
}
