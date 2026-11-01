//! 07000016's launch pulses: how the player lives through them. The pack cinema 0x06000631 hands
//! Bond back on the pit floor (stream 8's single key [D pack_cinema_16_live]) and the launch
//! script 0x0600090a raises ch 0xf3 for ticks 1611..2170 and 6810..7320 [D stream 6/9], which arms
//! HurtPoly-ShuttleLaunch (placement 309: gate 0xf3, 1000 damage [D params]) — a 1000-damage
//! contact kills Bond in one hit [M Player_DealWithObjHit 0x194458:95-101 → Player_HandlePain
//! 0x1902c8]. The shelter is the pit's two BLAST ROOMS:
//! - the volume's six bands (y −24.08 … −21.32) are each the same pit-floor polygon, and that
//!   polygon leaves the two blast rooms out (west x ≈ −19..−13, east x ≈ 19..27, z ≈ −166..−175)
//!   [D the placement's collision tris; probe16_pitmap]; the contact is per tri — `Collide_Update`
//!   0x1ea398 runs `Collide_Intersect` on the volume's glist [M], the port's capsule-vs-tris;
//! - the rooms' door leaves are the launch script's entities 02000994..02000997 (streams 15..18):
//!   OPEN at 0..1000, CLOSED (the two leaves of a pair meet) over 1350..2230 and 6660..7360, open
//!   again from 2580 / 7710 [D keys] — each pulse lies inside a closed window. So the player has
//!   ~1000 ticks from the hand-back to get inside; the door shuts behind him, the pulse passes,
//!   the door reopens.
//! Opt-in via NF_ISO.

use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};
use nf_game::player::{Input, RamSave};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

/// The west room's interior and the open pit floor just outside its door (the walk's last
/// waypoint and the sister's stop).
const SHELTER: [f32; 2] = [-15.2, -170.4];
const OUTSIDE: [f32; 2] = [-9.0, -173.0];

/// [D] The door leaves and the pulse windows, straight from the disc's launch script.
#[test]
fn the_blast_doors_are_shut_through_both_pulses_and_open_at_the_hand_back() {
    let iso_path = iso();
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lb = LevelBin::parse(&data).unwrap();
    let e = lb.entries.iter().find(|e| e.kind == 7 && u32::from_str_radix(&e.name, 16) == Ok(0x0600090a)).unwrap();
    let n = Nis::parse(0x0600090a, lb.data(&data, e)).unwrap();
    // The pulse windows: stream 6 and stream 9's ch-243 writes.
    let writes = |si: usize| -> Vec<(u32, u32, u32)> {
        let mut t = 0u32;
        let mut out = Vec::new();
        for c in n.streams[si].decode() {
            match c {
                nf_formats::nis::Cmd::Wait(w) => t = w as u32,
                nf_formats::nis::Cmd::Event { kind: 0x0d, params } if params.len() >= 2 => out.push((t, params[0], params[1])),
                _ => {}
            }
        }
        out
    };
    let mut pulse: Vec<(u32, u32)> = writes(6).into_iter().chain(writes(9)).filter(|w| w.1 == 243).map(|w| (w.0, w.2)).collect();
    pulse.sort();
    assert_eq!(pulse, vec![(1611, 1), (2170, 0), (6810, 1), (7320, 0)], "the launch script's ch-243 pulses");
    // The leaf pairs: (stream 15, 16) the east door, (17, 18) the west door.
    let pose = |si: usize, at: f32| -> [f32; 3] {
        let s = &n.streams[si];
        let ks = &n.keys[s.first_key as usize..=s.last_key as usize];
        let k = ks.iter().rev().find(|k| k.time <= at + 0.5).unwrap_or(&ks[0]);
        [k.pos[0], k.pos[1], k.pos[2]]
    };
    let gap = |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    for (l, r, name) in [(15usize, 16usize, "east"), (17, 18, "west")] {
        assert_eq!(n.streams[l].last_key - n.streams[l].first_key, 9, "{name} leaf keys");
        // Open at the hand-back (the script starts one tick after it) and until 1000.
        for t in [0.0f32, 1000.0, 2580.0, 6310.0, 7710.0] {
            assert!(gap(pose(l, t), pose(r, t)) > 2.0, "the {name} door is open at {t} (gap {:.2})", gap(pose(l, t), pose(r, t)));
        }
        // Shut across each pulse: the leaves meet at 1350 and stay until 2230; 6660..7360.
        for t in [1350.0f32, 1611.0, 2170.0, 2230.0, 6660.0, 6810.0, 7320.0, 7360.0] {
            assert!(gap(pose(l, t), pose(r, t)) < 0.05, "the {name} door is shut at {t} (gap {:.2})", gap(pose(l, t), pose(r, t)));
        }
    }
    // Sister: the comparison can fail — at the open key the leaves are apart, so "shut" is false.
    assert!(gap(pose(17, 0.0), pose(18, 0.0)) >= 0.05);
}

/// Runs the chain from the bridge trigger (the placed setup the timing sister uses — Bond put on
/// trigger #255, the touch fires ch 0x81), then from the hand-back steers Bond with Input only to
/// `target` through `via`. Returns (hp at the end of pulse 1 or at death, the volume's contact
/// count, Bond's position at the pulse's rise).
fn run_to(target: [f32; 2], via: &[[f32; 2]], delay: u32) -> (f32, u32, [f32; 3], bool) {
    let mut scene = nf_view::load_with_carry(&iso(), "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.pos = [2.4, 1.2, -175.5];
    }
    let mut back = false;
    let mut wp = 0usize;
    let mut rise_pos = None;
    let mut risen = false;
    let mut alive_inside_before_shut = false;
    let mut t_back = 0u32;
    for t in 0..4400u32 {
        let (p, yaw, disabled, on243) = {
            let s = scene.sim.as_ref().unwrap();
            (s.bond.pos, s.bond.yaw, s.disabled, s.drones.channels.on(243))
        };
        if !back && !disabled && p[1] < -20.0 { back = true; t_back = t; }
        if on243 && !risen { risen = true; rise_pos = Some(p); }
        if risen && !on243 { break; }
        if back && t == t_back + 1000 { alive_inside_before_shut = p[0] < -13.0; }
        let mut inp = Input::default();
        if back && t >= t_back + delay {
            let goal = if wp < via.len() { via[wp] } else { target };
            let (dx, dz) = (goal[0] - p[0], goal[1] - p[2]);
            if wp < via.len() && dx.hypot(dz) < 0.8 { wp += 1; }
            if dx.hypot(dz) > 0.4 {
                let mut d = dx.atan2(dz) - yaw;
                while d > std::f32::consts::PI { d -= 2.0 * std::f32::consts::PI; }
                while d < -std::f32::consts::PI { d += 2.0 * std::f32::consts::PI; }
                inp.turn = d.clamp(-0.08, 0.08);
                inp.forward = if d.abs() < 0.6 { 1.0 } else { 0.2 };
            }
        }
        scene.sim.as_mut().unwrap().step_bond(inp, false, false, false);
        if scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let hurt = sim.hurts.iter().find(|h| h.gate == 0xf3).expect("HurtPoly-ShuttleLaunch");
    (sim.drones.health.health, hurt.hits, rise_pos.expect("ch 243 rose"), alive_inside_before_shut)
}

/// The shelter, end to end on Input only: from the hand-back Bond runs west into the blast room
/// before its door shuts (script 1000..1350 [D]) and stands the whole pulse out untouched — the
/// volume never makes contact (its polygon excludes the room [D]). SISTERS: the same run stopping
/// on the open pit floor just outside the door dies in the pulse's first ticks, and a run that
/// starts too late finds the door shut.
#[test]
fn the_west_blast_room_shelters_bond_through_the_pulse_and_the_open_floor_does_not() {
    let via = [[-8.5f32, -173.5], [-11.0, -170.0]];
    let (hp, hits, at, inside) = run_to(SHELTER, &via, 0);
    eprintln!("[shelter] hp {hp} hits {hits} at the rise ({:.2},{:.2},{:.2}) inside-by-1000 {inside}", at[0], at[1], at[2]);
    assert!(inside, "Bond is inside the room before the door shuts (the hand-back + 1000)");
    assert!(at[0] < -13.0, "Bond stands inside the west room when ch 243 rises (x {:.2})", at[0]);
    assert_eq!(hits, 0, "the volume never touches Bond in the room — its polygon excludes it [D]");
    assert_eq!(hp, 100.0, "Bond lives through the pulse in the shelter");
    // SISTER: the open floor outside the door is inside the polygon — the same pulse kills him.
    let (hp2, hits2, at2, _) = run_to(OUTSIDE, &via[..1], 0);
    eprintln!("[shelter sister] hp {hp2} hits {hits2} at the rise ({:.2},{:.2},{:.2})", at2[0], at2[1], at2[2]);
    assert!(hits2 > 0 && hp2 <= 0.0, "the open pit floor outside the room is lethal in the pulse (hp {hp2}, hits {hits2})");
    // SISTER 2: the door really shuts. The same run started 1150 ticks after the hand-back
    // reaches the doorway after the leaves met (script 1350 [D]): the shut door (the leaf
    // entities' collision) keeps Bond out on the pit side of the door line (x ≈ −12.71 [D keys])
    // and the pulse kills him there.
    let (hp3, hits3, at3, _) = run_to(SHELTER, &via, 1150);
    eprintln!("[shelter late] hp {hp3} hits {hits3} at the rise ({:.2},{:.2},{:.2})", at3[0], at3[1], at3[2]);
    assert!(at3[0] > -12.8, "the shut door keeps a late Bond out (x {:.2})", at3[0]);
    assert!(hits3 > 0 && hp3 <= 0.0, "shut out, the pulse kills him (hp {hp3}, hits {hits3})");
}
