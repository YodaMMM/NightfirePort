//! Temporary (not committed): the kill rate of the bot matches, built against both the old and the
//! new code (only APIs both have).

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn run(humans: usize, bots: u32, ticks: u32, moving: bool) {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = humans as u32;
    s.player_count = humans as u16;
    s.bot_count = bots;
    s.total_players = humans as u32 + bots;
    s.seed = 0x0bed_0bad;
    s.frag_limit = 120;
    s.time_limit = 10 * 60;
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().unwrap();
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b));
    sim.swap_local_slot(0);
    for t in 0..ticks {
        let row: [nf_net::Input; nf_net::MAX_AGENTS] = std::array::from_fn(|sl| {
            let mut i = NEUTRAL;
            if moving && sl < humans {
                let ph = t + sl as u32 * 131;
                i.forward = if ph % 300 < 220 { 0.7 } else { 0.0 };
                i.turn = if ph % 200 < 50 { 0.025 } else { 0.0 };
            }
            i
        });
        nf_view::net_tick(sim, &row, 0);
    }
    let m = sim.mp.as_ref().unwrap();
    let kills: i32 = m.game.slots.iter().map(|s| s.kills).sum();
    let deaths: i32 = m.game.slots.iter().map(|s| s.deaths).sum();
    let hdeaths: i32 = m.game.slots[..humans].iter().map(|s| s.deaths).sum();
    let bdeaths = deaths - hdeaths;
    let shots: u32 = m.bots.iter().map(|b| b.shots).sum();
    let mins = ticks as f32 / 3600.0;
    eprintln!("KILLRATE {humans}h+{bots}b moving {moving}: {ticks} ticks, kills {kills} deaths {deaths} ({:.2} kills/min; human deaths {hdeaths} = {:.2}/min, bot deaths {bdeaths} = {:.2}/min), bot shots {shots}", kills as f32 / mins, hdeaths as f32 / mins, bdeaths as f32 / mins);
}

#[test]
fn killrate() {
    run(1, 3, 5 * 3600, false);
    run(2, 4, 5 * 3600, true);
    run(1, 8, 5 * 3600, false);
}
