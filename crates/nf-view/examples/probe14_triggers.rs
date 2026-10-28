//! level-14-panel-sequence probe: dump the 07000014 trigger placements near the sabotage chain —
//! the touch #725 (out 0xA0) volume bounds, the escape touch (out 0x0C), and the sound triggers'
//! live state after forcing channels — so the test can teleport into the real volumes. Prints only.
use nf_game::player::Input;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_mut().unwrap();
    for t in &sim.drones.triggers {
        if matches!(t.out, 0xa0 | 0x0c | 0x0f) || t.kind == 3 || t.kind == 4 {
            println!("trigger placement {} kind {} out {:#04x} mask {:#x}", t.placement, t.kind, t.out, t.mask);
        }
    }
    println!("sound triggers of interest:");
    for t in &sim.drones.sound_triggers.list {
        if matches!(t.on, 0x398 | 0x5e3 | 0xf1 | 0xdb | 0x85 | 0x87 | 0x88) {
            println!("  on {:#04x} off {:#04x} id {:#06x} is2d {} one_shot {}", t.on, t.off, t.id, t.is2d, t.one_shot);
        }
    }
    // Force 0xdb on and tick: does the sound trigger start?
    sim.drones.channels.set(0xdb, 1, 1);
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    println!("after forcing 0xdb: playing(0x398) = {}", sim.drones.sound_triggers.playing(0x398));
}
