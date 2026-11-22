//! The `+0x4F8 & 0x100` (enabled) gate of `DroneVision_ProcessDroneSight` 0x176a28 (all three
//! loops, :81 / :165 / :188) and `Drone_GetOpponentInfo` 0x138f68:16-18, counted on every level.
//! Opt-in NF_ISO. The census prints, per level, every drone whose state trace over 600 ticks
//! (Bond standing at the spawn, no input) and the end random-stream position; diff two runs to see
//! which drones a change moves.
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
#[ignore = "assertion-free census; run with --ignored"]
fn census() {
    let Some(iso) = iso() else { return };
    for &id in nf_view::dev::LEVELS {
        let name = format!("{id:08x}.bin");
        let Ok(mut s) = nf_view::load(&iso, &name, false) else { println!("{:02x} no load", id & 0xff); continue };
        let Some(sim) = s.sim.as_mut() else { println!("{:02x} no sim", id & 0xff); continue };
        let n = sim.drones.list.len();
        let mut trace: Vec<Vec<(u8, bool, bool)>> = vec![Vec::new(); n];
        let mut off_vis = vec![0u32; n];
        let mut off_play = vec![0u32; n];
        let mut play_at = None;
        let mut t = 0u32;
        let mut played = 0u32;
        // 600 ticks from the load, then on to 600 ticks of play (the intro hold ends) when the
        // load opens on a cut scene.
        while t < 600 || (played < 600 && t < 4600) {
            t += 1;
            sim.step_bond(Input::default(), false, false, false);
            if play_at.is_none() && !sim.disabled { play_at = Some(t); }
            if !sim.disabled { played += 1; }
            for (i, x) in sim.drones.list.iter().enumerate() {
                if i >= n { break; }
                let t = (x.state, x.seen, x.enabled);
                if trace[i].last() != Some(&t) { trace[i].push(t); }
                if x.present && !x.corpse && x.vision_on && !x.enabled { off_vis[i] += 1; if !sim.disabled { off_play[i] += 1; } }
            }
        }
        let r = &sim.drones.rng;
        println!("{:02x} n{n} rng {:08x}{:08x} play {play_at:?}", id & 0xff, r.x, r.y);
        for i in 0..n {
            if off_vis[i] > 0 || trace[i].len() > 1 {
                let h = trace[i].iter().fold(0xcbf29ce484222325u64, |h, &(a, b, c)| {
                    (h ^ (a as u64 | (b as u64) << 8 | (c as u64) << 9)).wrapping_mul(0x100000001b3)
                });
                println!("  d{i} offvis {} inplay {} steps {} hash {h:016x} end {:?}", off_vis[i], off_play[i], trace[i].len(), trace[i].last());
            }
        }
    }
}
