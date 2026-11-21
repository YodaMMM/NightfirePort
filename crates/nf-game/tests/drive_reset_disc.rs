//! The player car's recovery on the real land tracks (opt-in: `NF_ISO=<image>`): 1200 ticks of
//! full throttle and no steering from each track's EStartMission spawn, with the engine's
//! righting reset (`driving::reset::check`, FUN_00188200 → FUN_00187b00 [M]) run after every
//! step as the view runs it. The car leaves the road, rolls, and must end the run on the
//! track's ground, never below the collision's lowest point.

use nf_formats::crp::Crp;
use nf_formats::iso9660::IsoImage;
use nf_game::driving::reset::{self, ResetState};
use nf_game::driving::{Car, CarInput, CarParams, CollisionGround, Rigid, DT};

fn unpack(raw: Vec<u8>) -> Vec<u8> {
    if raw.len() >= 2 && (raw[0] & 0x3e) == 0x10 && raw[1] == 0xfb {
        nf_formats::refpack::decompress(&raw).expect("refpack")
    } else {
        raw
    }
}

/// The player's pvehicle file per land track: MIS4 ships vanquishalps.atr, MIS3 supersnow.atr,
/// RACE cobra_player.atr, MIS01 vanquish.atr [D: the archives' pvehicle lists]. Which file the
/// game picks for MIS3 is not traced (the view takes supersnow as well).
fn player_atr(viv: &str) -> &'static str {
    match viv {
        "MIS3" => "supersnow",
        "MIS4" => "vanquishalps",
        "RACE" => "cobra_player",
        _ => "vanquish",
    }
}

struct Run {
    end: [f32; 3],
    ground: Option<f32>,
    lowest: f32,
    min_y: f32,
    resets: u32,
    up_y: f32,
}

fn run(viv: &str, with_reset: bool) -> Option<Run> {
    let iso = IsoImage::open(&std::env::var("NF_ISO").ok()?).ok()?;
    let entry = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let bigf = nf_formats::disc::read_bigf(&iso, entry).ok()?;
    let base = entry.lba as u64 * 2048;
    let read = |pred: &dyn Fn(&str) -> bool| -> Option<Vec<u8>> {
        let e = bigf.entries.iter().find(|x| pred(&x.name.to_ascii_lowercase()))?;
        Some(unpack(iso.read_bytes(base + e.offset as u64, e.size as u64).ok()?))
    };
    let crp_bytes = read(&|n| n.contains("data\\track\\") && n.ends_with(".crp"))?;
    let want = format!("data\\sim\\attrib\\pvehicle\\{}.atr", player_atr(viv));
    let atr = read(&|n| n == want).expect("the player's .atr is in the archive");
    let crp = Crp::parse(&crp_bytes).ok()?;
    let tris = crp.extract_collision_tris().ok()?;
    let min_y = tris.iter().flat_map(|t| t.v.iter().map(|v| v[1])).fold(f32::MAX, f32::min);
    let spawn = nf_formats::trackstart::track_spawn(&crp, &tris).ok()?;
    let g = CollisionGround::new(tris);
    let rigid = Rigid::game_defaults();
    let mut car = Car::new(CarParams::parse_atr(&String::from_utf8_lossy(&atr)), spawn.pos, spawn.yaw);
    let mut st = ResetState::new(&car);
    let full = CarInput { throttle: 1.0, brake: 0.0, steer: 0.0, handbrake: false };
    let (mut resets, mut lowest) = (0, f32::MAX);
    for _ in 0..1200 {
        car.step(&full, &g, &rigid, DT);
        if with_reset {
            if let Some(r) = reset::check(&car, g.tris(), &mut st, None) {
                resets += 1;
                r.apply(&mut car);
            }
        }
        lowest = lowest.min(car.pos[1]);
    }
    let ground = nf_formats::trackstart::ground_height(g.tris(), car.pos[0], car.pos[1], car.pos[2]);
    Some(Run { end: car.pos, ground, lowest, min_y, resets, up_y: car.rot[1][1] })
}

fn check_track(viv: &str) {
    let Some(r) = run(viv, true) else { eprintln!("skip: NF_ISO unset or no {viv} track"); return };
    eprintln!("{viv}: end {:?} ground {:?} up.y {:.3} lowest {:.2} (collision min {:.2}) resets {}", r.end, r.ground, r.up_y, r.lowest, r.min_y, r.resets);
    let g = r.ground.unwrap_or_else(|| panic!("{viv}: no ground under the car's end point {:?}", r.end));
    let above = r.end[1] - g;
    assert!((0.0..3.0).contains(&above), "{viv}: the car ends {above} m above the ground under it (end {:?})", r.end);
    assert!(r.lowest > r.min_y, "{viv}: the car went below the collision's lowest point ({} < {})", r.lowest, r.min_y);
    assert!(r.resets > 0, "{viv}: the straight run tips the car over at least once and the reset rights it");
}

#[test]
fn full_throttle_no_steer_1200_ticks_ends_on_the_mis01_ground() { check_track("MIS01"); }

#[test]
fn full_throttle_no_steer_1200_ticks_ends_on_the_mis3_ground() { check_track("MIS3"); }

#[test]
fn full_throttle_no_steer_1200_ticks_ends_on_the_mis4_ground() { check_track("MIS4"); }

#[test]
fn full_throttle_no_steer_1200_ticks_ends_on_the_race_ground() { check_track("RACE"); }

/// The sister run: the same MIS3 drive with the reset left out ends with the car on its back
/// (up.y < 0.3) on the ground; with it, the car finishes on its wheels.
#[test]
fn mis3_the_reset_is_what_puts_the_car_back_on_its_wheels() {
    let (Some(without), Some(with)) = (run("MIS3", false), run("MIS3", true)) else { eprintln!("skip: NF_ISO unset"); return };
    eprintln!("MIS3 without the reset: up.y {:.3} at {:?}; with: up.y {:.3}", without.up_y, without.end, with.up_y);
    assert!(without.up_y < reset::TIP_UP_Y, "without the reset the car is left tipped (up.y {})", without.up_y);
    assert!(with.up_y > 0.9, "with it the car ends upright (up.y {})", with.up_y);
}
