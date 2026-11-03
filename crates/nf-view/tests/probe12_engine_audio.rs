//! level-12-copter-sound-distance probe 2: the port's live audio commands for the copter's
//! engine loop (id 0x10b [D script 060005bd stream 10]). Pokes the copter's gate channel 37
//! [D plc #970 p6], ticks, and dumps every Start/Position command the game-side pool emits —
//! is the engine started 3-D (pos) or 2-D (no pos), and does its position follow the copter?
//! Assertion-free diagnostic, run with --ignored --nocapture.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_07000012_engine_audio_commands() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000012.bin", false).unwrap());
    w.wait_enabled(3000, "the level start");
    for t in 0..120u32 {
        w.tick(Default::default(), false);
        let _ = t;
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(37, 1, sim.drones.frame);
    }
    for t in 0..600u32 {
        w.tick(Default::default(), false);
        // The App's sound_tick flow (nf_view/src/lib.rs sound_tick): the EE layer's per-frame
        // update with Bond as the listener, then the command drain.
        let sim = w.scene.sim.as_mut().unwrap();
        let (pos, f) = (sim.bond.head(), sim.bond.look());
        let l = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6);
        let norm = [f[2] / l, 0.0, -f[0] / l];
        sim.drones.audio.update(sim.sfx.as_ref(), (pos, norm, [0.0, 1.0, 0.0], f), &|_| false);
        let cmds = sim.drones.audio.take_commands();
        for c in cmds {
            let s = match &c {
                nf_game::sound::Cmd::Start { handle, id, pos, radii } => format!("START h={handle} id={id:#x} pos={pos:?} radii={radii:?}"),
                nf_game::sound::Cmd::Position { handle, pos } => format!("POS h={handle} pos={pos:?}"),
                nf_game::sound::Cmd::Listener { pos, .. } => format!("LISTENER pos={pos:?}"),
                other => format!("{other:?}"),
            };
            let keep = s.contains("0x10b") || s.contains("START") || s.contains("LISTENER") || s.contains("h=5 ");
            if keep && (t < 30 || !s.contains("LISTENER") || t % 120 == 0) {
                println!("t={t}: {s}");
            }
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let copter = sim.drones.copters.first().map(|c| c.pos());
    println!("copter pos after: {copter:?}");
    println!("listener: {:?}", sim.drones.audio.listener);

    // Driver-level measurement: the real level bank, the engine started 3-D exactly as the
    // game-side sent it, the listener placed at measured spots.
    let copter = copter.unwrap();
    let (slot, bank, sbf) = w.scene.sound_bank.clone().expect("the level bank");
    let mut drv = nf_audio::sfx::Driver::new();
    drv.load_bank(slot, bank, sbf);
    let mut out = Vec::new();
    drv.start(&mut out, 0x10b, Some(copter), 5, None);
    println!("start cmds: {out:?}");
    let fwd = [0.0f32, 0.0, 1.0]; // listener facing +z; left = up × fwd = (−1,0,0)... game: row0 = screen-left
    let left = [-1.0f32, 0.0, 0.0];
    for (name, lp) in [
        ("d20_front", [copter[0], copter[1], copter[2] - 20.0]),
        ("d50_front", [copter[0], copter[1], copter[2] - 50.0]),
        ("d50_behind", [copter[0], copter[1], copter[2] + 50.0]),
        ("d50_left", [copter[0] - 50.0, copter[1], copter[2]]),
        ("d120_front", [copter[0], copter[1], copter[2] - 120.0]),
    ] {
        drv.set_listener(lp, fwd, left);
        drv.set_position(5, None, copter);
        out.clear();
        drv.update(&mut out, &[]);
        let sets: Vec<_> = out.iter().filter_map(|c| match c {
            nf_audio::Cmd::Play { voice, left, right, .. } => Some((*voice, *left, *right)),
            nf_audio::Cmd::Set { voice, left, right, .. } => Some((*voice, *left, *right)),
            _ => None,
        }).collect();
        // The game's law [M 0x1060c], world units, inner 10 outer 100 [D bank]:
        let d = ((copter[0] - lp[0]).powi(2) + (copter[1] - lp[1]).powi(2) + (copter[2] - lp[2]).powi(2)).sqrt();
        let att = (100.0 - 100.0 * (d - 10.0) / 90.0).clamp(0.0, 100.0);
        let a2 = att * att / 100.0;
        let dir = [(copter[0] - lp[0]) / d, (copter[1] - lp[1]) / d, (copter[2] - lp[2]) / d];
        let dot = |a: [f32; 3], b: [f32; 3]| (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) * 1000.0;
        let rear = ((dot([-fwd[0], -fwd[1], -fwd[2]], dir) + 1000.0) * a2 / 2000.0).clamp(0.0, 100.0);
        let front = 100.0 - rear;
        let fl = ((((dot(left, dir) + 1000.0) * a2 / 2000.0).clamp(0.0, 100.0)) * front / 100.0).clamp(0.0, 100.0);
        let fr = ((((dot([-left[0], -left[1], -left[2]], dir) + 1000.0) * a2 / 2000.0).clamp(0.0, 100.0)) * front / 100.0).clamp(0.0, 100.0);
        let rb = (rear * 75.0 / 100.0).clamp(0.0, 100.0);
        println!("{name}: d={d:.1} measured (voice,left,right)={sets:?} | law fl={fl:.1} fr={fr:.1} rear={rb:.1} (att {att:.1} a2 {a2:.1})");
    }
}
