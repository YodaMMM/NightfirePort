//! agent-difficulty: the front end's level load keeps the picked difficulty with the dev tools on.
//!
//! The game has one level-load path: every create-time gate and tick reader sees the difficulty
//! global GameState+0x28 (0x2a3790) the difficulty page wrote [M C_SBNFDFCTY_Handler 0x216ca8:15;
//! rows 1/2/3 at 0x2df4c0+0xc/+0x24/+0x3c, D ACTION.ELF]. The port's front-end load used to take a
//! separate dev arm (`load_with_carry_dev`) that hard-coded difficulty 1 and the zero `bonus[0]`
//! word, so with the launcher's dev mode on, an Agent or 00 Agent mission built the Operative
//! roster (`Drone_Create` 0x136be0:49's placement param-2 gate, the `NDrone2_DefaultInit`
//! 0x14b300:504-535 captain criterion) and ran the Operative damage/accuracy rows. The app now
//! loads through `load_front_end` for both arms; this pins that the dev arm's roster and
//! difficulty equal the plain front-end load at every difficulty, on 07000001 (whose d1/d2/d3
//! rosters differ — difficulty3_07000001_live pins the three tables).

use nf_view::scene::Sim;

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

type Row = (usize, u32, u32, u8, f32, bool);

fn roster(s: &Sim) -> Vec<Row> {
    s.drones.list.iter().map(|d| (d.placement, d.skin, d.weapon, d.accuracy, d.health, d.captain)).collect()
}

#[test]
fn dev_front_end_load_keeps_the_picked_difficulty_07000001() {
    let iso = iso();
    let mut rosters = Vec::new();
    for difficulty in 1..=3u32 {
        let plain = nf_view::load_front_end(&iso, "07000001.bin", false, None, false, difficulty, 0)
            .expect("plain load").sim.expect("sim");
        let dev = nf_view::load_front_end(&iso, "07000001.bin", false, None, true, difficulty, 0)
            .expect("dev load").sim.expect("sim");
        assert_eq!(plain.drones.difficulty, difficulty);
        assert_eq!(dev.drones.difficulty, difficulty, "the dev arm runs at the picked difficulty");
        assert_eq!(dev.drones.progress.difficulty, difficulty, "the Progress mirror too");
        assert_eq!(roster(&dev), roster(&plain), "d{difficulty}: the dev arm builds the same guards");
        rosters.push(roster(&dev));
    }
    // The three rosters really differ on this level (so the equality above is not vacuous).
    assert_ne!(rosters[0], rosters[1], "d1 vs d2 roster");
    assert_ne!(rosters[1], rosters[2], "d2 vs d3 roster");
}
