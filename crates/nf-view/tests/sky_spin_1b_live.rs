//! The 0x0700001B rotating sky ticks in the real sim: `rotatosky.168` [0x30d878] advances once per
//! slot pass, two passes a frame [M View_DrawSky 0x1e5ad0:186-198]. PCSX2 on 0700001b reads
//! 148 -> 748 over in-game frames 26651 -> 26951 = 2.0 per frame [L 2026-11-18]. Other levels never
//! advance it. The sim used to tick the sky with level 0, so the 1b counter stayed at 0.
#[path = "common/walker.rs"]
mod walker;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn spin_after(level: &str, ticks: u32) -> u32 {
    let mut w = Walker::new(nf_view::load(&iso(), level, false).expect("load"));
    let c0 = w.scene.sim.as_ref().unwrap().drones.sky.rotating.count;
    for _ in 0..ticks { w.tick(Default::default(), false); }
    w.scene.sim.as_ref().unwrap().drones.sky.rotating.count - c0
}

#[test]
fn the_1b_sky_counter_advances_two_per_frame() {
    assert_eq!(spin_after("0700001b.bin", 300), 600);
}

#[test]
fn other_levels_never_spin_their_sky() {
    assert_eq!(spin_after("0700000a.bin", 300), 0);
}
