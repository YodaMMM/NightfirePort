//! bath-line repro: run the APP sound path (the real mixer at real time — `NF_SILENT=1
//! NF_NULL_MIXER=1` makes `Output::open` the null mixer) through the 07000002 bath scene
//! (SP #399, script 0x0600007f) reached by the real walked route — no teleports after the
//! spawn — and read the scene's lines' start/stop against their disc length from the
//! `NF_SOUND_LOG` output. Opt-in via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp, wrap_pi};
use nf_game::player::Input;

fn ch_on(w: &Walker, c: u8) -> bool { w.scene.sim.as_ref().unwrap().drones.channels.on(c as u32) }
fn geisha(w: &Walker, ch: u8) -> Option<usize> {
    w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| d.second_channel == ch)
}

/// Kill every armed guard near `around` through the real fire path (the walkreal idiom).
fn clear_captors(w: &mut Walker, around: [f32; 3], label: &str) {
    for round in 0..8 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.team == 1 && d.health > 0.0 && !d.corpse
                && d.pos.iter().zip(around).map(|(a, b)| (a - b).powi(2)).sum::<f32>() < 64.0)
        };
        let Some(di) = target else { return };
        let dest = if round == 0 { around } else { w.scene.sim.as_ref().unwrap().drones.list[di].pos };
        w.walk_nav_to(dest, &format!("{label} captor {round} approach"), 3.5);
        if !w.shoot(di, &format!("{label} captor {round}")) {
            eprintln!("[02] {label}: captor {round} (drone {di}) survived the clip");
        }
    }
}

fn playing(w: &Walker, hash: u32) -> bool {
    w.scene.sim.as_ref().unwrap().drones.scripts.players.iter().any(|p| p.scripts.contains(&hash) && p.state == 2 && p.info[p.cur].as_ref().is_some_and(|i| i.playing()))
}

#[test]
fn repro_bath_scene_line_with_the_app_mixer() {
    // The env vars must be set before the first `Output::open` / `sound_log` read.
    std::env::set_var("NF_SILENT", "1");
    std::env::set_var("NF_NULL_MIXER", "1");
    std::env::set_var("NF_SOUND_LOG", "1");
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso, "07000002.bin", false).expect("load 07000002"));
    w.lenient = true;

    // 0. The level-start cut-scene plays out (real, not skipped).
    w.skip_cutscenes = false;
    let mut intro_done = false;
    for _ in 0..4200 {
        if !w.scene.sim.as_ref().unwrap().disabled { intro_done = true; break; }
        w.tick(Default::default(), false);
    }
    eprintln!("[02] intro done {intro_done} at sim frame {}", w.scene.sim.as_ref().unwrap().drones.frame);
    w.skip_cutscenes = true;

    // 1. The spawn room → ladder #114 → the garden.
    for (name, at) in [
        ("bunker room south", [-27.2f32, 1.05, 31.5]),
        ("bunker corridor", [-27.2, 1.05, 36.0]),
        ("corridor south", [-27.2, 1.05, 39.5]),
        ("alcove mouth", [-27.2, 1.05, 41.0]),
    ] {
        w.walk_to(Wp { at, act: Act::None, arrive: 1.0 }, name);
    }
    w.walk_to(Wp { at: [-26.87, 2.54, 42.23], act: Act::Ladder { top_y: 6.0 }, arrive: 0.9 }, "ladder #114");
    w.walk_to(Wp { at: [-26.0, 5.9, 43.4], act: Act::None, arrive: 1.4 }, "the garden");
    w.walk_to(Wp { at: [-24.6, 5.9, 42.8], act: Act::None, arrive: 1.4 }, "the garden east");

    // 2. The west wing to the ch-0x17 plate.
    for (name, at) in [
        ("garden path", [-22.2f32, 5.9, 34.1]),
        ("south hall", [-23.3, 5.9, 27.6]),
        ("south hall west", [-31.2, 5.9, 22.1]),
        ("west wing south", [-37.4, 5.9, 8.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    w.lenient = true;
    w.walk_to(Wp { at: [-35.2, 6.5, 5.05], act: Act::Wait(10), arrive: 1.2 }, "the ch-0x17 plate");
    w.lenient = false;
    if !ch_on(&w, 0x17) {
        eprintln!("[02] the ch-0x17 plate raise driven (the walker can't cross its quad)");
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x17, 1, sim.drones.frame);
        w.wait(30);
    }
    assert!(ch_on(&w, 0x17), "the geisha group is enabled [D tr[003] out 0x17]");

    // 3. The captors die (real fire) → the rescue 0x49 → the death channels → 0x3B.
    let g1 = geisha(&w, 0x49).expect("geisha 1");
    let at = w.scene.sim.as_ref().unwrap().drones.list[g1].pos;
    clear_captors(&mut w, at, "geisha 1");
    for _ in 0..240 { if ch_on(&w, 0x49) { break; } w.tick(Default::default(), false); }
    assert!(ch_on(&w, 0x49), "geisha 1 saved: ch 0x49");
    for _ in 0..100 { if ch_on(&w, 0x29) && ch_on(&w, 0x2a) && ch_on(&w, 0x88) { break; } w.tick(Default::default(), false); }
    assert!(ch_on(&w, 0x29) && ch_on(&w, 0x2a) && ch_on(&w, 0x88), "the three captor deaths");
    assert!(ch_on(&w, 0x3b), "AND #397 → 0x3B");
    eprintln!("[02] 0x3B on at sim frame {}", w.scene.sim.as_ref().unwrap().drones.frame);

    // 4. Swap in the app's sound path: the headless driver's banks/streams stay, the null
    //    mixer runs it at real time (the app's `build_sound` output under these env vars).
    let mut sound = {
        let sim = w.scene.sim.as_mut().unwrap();
        let mut s = sim.headless_sound.take().expect("the walker's headless sound").0;
        s.output = nf_audio::Output::open();
        assert!(s.output.is_some(), "the null mixer must open under NF_SILENT+NF_NULL_MIXER");
        s
    };
    eprintln!("[02] null mixer attached at sim frame {}", w.scene.sim.as_ref().unwrap().drones.frame);

    // 5. The paced walk into the touch zone (−45.4, 5.74, 11.7) — one tick per 1/60 s, the
    //    app's per-frame order (step, then sound_tick).
    let target = [-45.4f32, 5.74, 11.7];
    let mut next = std::time::Instant::now() + std::time::Duration::from_millis(17);
    let mut paced = 0u32;
    let mut since_scene_end = None;
    let mut scene_started = false;
    let frame0 = w.scene.sim.as_ref().unwrap().drones.frame;
    loop {
        let (pos, yaw) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos, sim.bond.yaw) };
        let dist = ((target[0] - pos[0]).powi(2) + (target[2] - pos[2]).powi(2)).sqrt();
        let input = if dist > 0.8 {
            let desired = (target[0] - pos[0]).atan2(target[2] - pos[2]);
            let err = wrap_pi(desired - yaw);
            Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 0.2 { 0.3 } else { 1.0 }, ..Default::default() }
        } else { Input::default() };
        w.tick(input, false);
        { let sim = w.scene.sim.as_mut().unwrap(); nf_view::sound_tick(&mut sound, sim); }
        paced += 1;
        let sim = w.scene.sim.as_ref().unwrap();
        if ch_on(&w, 0x3c) && !scene_started {
            scene_started = true;
            eprintln!("[02] 0x3C on at sim frame {} (+{paced} paced)", sim.drones.frame);
        }
        if ch_on(&w, 0x35) && playing(&w, 0x0600_007f) {
            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| eprintln!("[02] the bath scene (SP #399, 0600007f) started at sim frame {}", sim.drones.frame));
        }
        if scene_started && !playing(&w, 0x0600_007f) {
            let n = since_scene_end.unwrap_or(0);
            since_scene_end = Some(n + 1);
            if n == 0 { eprintln!("[02] the bath scene ended at sim frame {}", sim.drones.frame); }
        }
        if since_scene_end.is_some_and(|n| n > 240) { break; }
        // A generous cap: the scene is 740 ticks; the walk-in a few hundred.
        if paced > 3600 { eprintln!("[02] the paced cap hit (scene_started {scene_started})"); break; }
        let now = std::time::Instant::now();
        if next > now { std::thread::sleep(next - now); }
        next += std::time::Duration::from_millis(17);
    }
    eprintln!("[02] done: paced {paced} ticks, frames {}..{}", frame0, w.scene.sim.as_ref().unwrap().drones.frame);
    assert!(scene_started, "the bath scene must start through the real route (0x3B ∧ 0x3C → 0x35)");
}
