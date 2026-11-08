//! The station zip line on 07000008, the game's one type-2 wire [D #19 p1 = 2], ridden through the
//! real input path from its icon: `Player_HandleJump` 0x1916b0:118-147 grabs the nearest wire under
//! context 3 (icon #105), `Player_CollWire` 0x197130 faces the rider down the slope, and
//! `Player_Zipline` 0x197f50 adds 0.15 to the forward lane of a quad `Player_Update` zeroes every
//! live tick [M disasm 0x1937e0 `lh v1,244(s3)` → 0x193830 `Vec_Zero(BL+0x10)`], so the ride runs at
//! a constant 0.15 per tick along the facing.

use nf_game::player::{Input, SS_WALK, SS_ZIPLINE};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn the_zip_line_rides_down_the_slope_at_the_games_speed_through_the_three_plates() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let w = sim.wires.first().expect("07000008 has the zip line").clone();
    assert_eq!(w.wire_type, 2, "#19 is a type-2 wire [D p1]");
    let (a, b) = w.zipline_segment();
    let (low, high) = if a[1] < b[1] { (a, b) } else { (b, a) };

    // The icon #105's spot on the walkway [D].
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [133.4, 49.5, -101.2], 1.57);
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.bond.icon_ctx, 3, "the icon arms the wire context [M 0x196e80]");
    sim.step_bond(Input { jump: true, ..Default::default() }, false, false, false);
    assert_eq!(sim.bond.substate, SS_ZIPLINE, "the jump grabs the zip line");

    // Facing down the slope [M 0x197130 `if (fStack_12c < fStack_11c)`].
    let (s, c) = sim.bond.yaw.sin_cos();
    let down = [low[0] - high[0], low[2] - high[2]];
    assert!(s * down[0] + c * down[1] > 0.0, "the rider faces the low end: yaw {}", sim.bond.yaw);

    // Constant speed from the first tick: 60 ticks cover 60 · 0.15 · cos(pitch) plus the grab anim's
    // small root motion.
    let start = sim.bond.pos;
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    let run = ((sim.bond.pos[0] - start[0]).powi(2) + (sim.bond.pos[2] - start[2]).powi(2)).sqrt();
    assert!((8.0..9.6).contains(&run), "60 ticks at 0.15 per tick along the facing, ran {run}");
    assert_eq!(sim.bond.move_local, [0.0, 0.15], "the forward lane is rebuilt from zero each tick");

    // The rest of the ride: the end anim starts within the time the line takes at that speed.
    let mut ticks = 60;
    while sim.bond.mode != 9 && ticks < 400 {
        sim.step_bond(Input::default(), false, false, false);
        ticks += 1;
    }
    assert!((130..=160).contains(&ticks), "the line takes about 2.5 seconds, took {ticks} ticks");

    // The three plates under the line all fire and mux #102 raises Bond moment ch 0x68 [D].
    for ch in [0x5a, 0x5b, 0x5c, 0x68] {
        assert!(sim.drones.channels.on(ch), "ch {ch:#x} after the ride");
    }

    // The drop: back on foot on the landing, with the regrab hold on.
    for _ in 0..200 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.bond.substate, SS_WALK, "the end anim hands back to the walk");
    assert!(sim.bond.grounded, "Bond lands: {:?}", sim.bond.pos);
    assert!((sim.bond.pos[0] - low[0]).abs() < 1.5, "he drops at the low end: {:?}", sim.bond.pos);
}
