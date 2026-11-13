//! The released pad path without hardware: a synthesised pad state goes through the game's own
//! mapping (the disc's curve table [D ACTION.ELF gAnalogStickMappingFunction 0x2a3840], style 0's
//! actions, `PadDriver`) into the sim on a real level start — the same Input the `gamepad`
//! backend merges (crates/nf-view/src/gamepad.rs `Backend::merge`, which only adds gilrs' polling).

use nf_game::input::{PadConfig, PadDriver, PadState};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Holding the left stick forward walks Bond forward on 07000005's start; the stick released, he
/// stops [M Player_Move 0x191dd0:65-66 reads actions 2/1 as the forward/strafe inputs].
#[test]
fn the_pad_stick_walks_bond_on_a_level_start() {
    let iso_path = iso();
    let disc = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open the disc");
    let elf = disc.read_file(disc.find("/ACTION.ELF").expect("ACTION.ELF")).expect("read ACTION.ELF");
    let cfg = PadConfig::from_action_elf(&elf).expect("the pad curve table [D 0x2a3840]");
    let tuning = nf_formats::tuning::Tuning::from_action_elf(&elf).unwrap_or_default();
    let mut driver = PadDriver::new(cfg);

    let mut scene = nf_view::load(&iso_path, "07000005.bin", false).expect("load 07000005");
    for _ in 0..600 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    assert!(!scene.sim.as_ref().unwrap().disabled, "the level start hands Bond control");
    let (start, yaw) = { let b = &scene.sim.as_ref().unwrap().bond; (b.pos, b.yaw) };

    let mut step = |scene: &mut nf_view::scene::Scene, ly: u8, n: u32| {
        for _ in 0..n {
            let pad = PadState { buttons: 0, rx: 0x7f, ry: 0x7f, lx: 0x7f, ly, connected: true };
            let (input, _) = driver.tick(&pad, &tuning, 1.0, false, false);
            scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        }
    };
    step(&mut scene, 0x00, 60); // full stick up for one second
    let walked = scene.sim.as_ref().unwrap().bond.pos;
    let (dx, dz) = (walked[0] - start[0], walked[2] - start[2]);
    let along = dx * yaw.sin() + dz * yaw.cos();
    assert!(along > 0.5, "a second of full stick walks Bond forward along his facing: {along:.3} u (moved {dx:.3}, {dz:.3}, yaw {yaw:.3})");

    step(&mut scene, 0x7f, 60); // released
    let before = scene.sim.as_ref().unwrap().bond.pos;
    step(&mut scene, 0x7f, 30);
    let after = scene.sim.as_ref().unwrap().bond.pos;
    let drift = ((after[0] - before[0]).powi(2) + (after[2] - before[2]).powi(2)).sqrt();
    assert!(drift < 0.01, "the released stick stops him: {drift:.4} u in 30 ticks");
}
