//! level-0a-rain-indoors-4 probe (a dump, not a test): where does the bathhouse-floor shout
//! chain break on the rain-fixed stream? The mixer test
//! `bathhouse_shout_mixer_live::the_bathhouse_floor_guards_shout_reaches_the_mixer_too` needs a
//! floor guard's shout voice (sfx ids 111..115 [D Drone_SfxInfo 0x33..0x36, repeat 120]) live in
//! the driver WITH its stream Play (pitch 1881) in the same tick batch, within 4 cycles of
//! 300 ticks after the plate (#341, ch 0x15) cycles Bond away/back. The sister pool test
//! (`bathhouse_voices_live`) only polls `drones.audio.live_ids()` and PASSED on the same tree,
//! so the break is somewhere between the AttackTalk speak roll [M 0x13eac0] and the mixer Play.
//! This probe replays the exact choreography and dumps, per tick: the EE pool's live ids, the
//! the reported live voice ids, every stream Play; per guard: `voice`/`last_voice` transitions and
//! the state-log entries added each cycle (the talk sites are the ATTACK / AIM_STAND_FIRE 0x68 /
//! COMBAT_NO_MOVE 0x59 ENTERs).
//!
//! Run with:
//!   NF_ISO=<iso> cargo run -p nf-view --release --example probe_02_shout_window

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn bare_sound(scene: &mut nf_view::scene::Scene) -> nf_audio::sfx::Sound {
    let mut snd = nf_audio::sfx::Sound {
        driver: Default::default(),
        output: None,
        captured: None,
        music_files: None,
        music: None,
        music_pending: Vec::new(),
    };
    if let Some((slot, bank, sbf)) = scene.sound_bank.take() { snd.driver.load_bank(slot, bank, sbf); }
    if let Some((lut, bin)) = scene.streams.take() { snd.driver.load_streams(lut, std::sync::Arc::new(bin)); }
    snd
}

/// The voices test's tick, verbatim (reaps ALL voices each tick, no capture).
fn tick_voices(scene: &mut nf_view::scene::Scene, snd: &mut nf_audio::sfx::Sound) {
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    nf_view::sound_tick(snd, sim);
    let all: Vec<u32> = (0..nf_audio::sfx::HW_VOICES as u32).collect();
    let mut out = Vec::new();
    snd.driver.update(&mut out, &all);
}

/// One real-path tick with the stand-in's end report (the mixer test's tick, verbatim).
fn tick(scene: &mut nf_view::scene::Scene, snd: &mut nf_audio::sfx::Sound) -> (Vec<u32>, Vec<nf_audio::Cmd>) {
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    nf_view::sound_tick(snd, sim);
    let ids = snd.driver.ids();
    let keep = snd.driver.looping_voice_ids();
    let ended: Vec<u32> = (0..nf_audio::sfx::HW_VOICES as u32).filter(|v| !keep.contains(v)).collect();
    let mut out = Vec::new();
    snd.driver.update(&mut out, &ended);
    let cmds = match snd.captured.as_ref() {
        Some(c) => { let mut g = c.lock().unwrap(); std::mem::take(&mut *g) }
        None => Vec::new(),
    };
    (ids, cmds)
}

fn stream_plays(cmds: &[nf_audio::Cmd]) -> Vec<(f32, f32)> {
    cmds.iter().filter_map(|c| match c {
        nf_audio::Cmd::Play { pitch: 1881, left, right, .. } => Some((*left, *right)), _ => None })
        .collect()
}

fn shout_pool_ids(sim: &nf_view::scene::Sim) -> Vec<u32> {
    sim.drones.audio.live_ids().into_iter().filter(|id| (111..=115).contains(id)).collect()
}

fn main() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let voices_mode = std::env::var("NF_SHOUT_TICK").as_deref() == Ok("voices");
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let mut snd = bare_sound(&mut scene);
    let no_cap = std::env::var("NF_SHOUT_NOCAP").is_ok();
    if !no_cap {
        let cap: std::sync::Arc<std::sync::Mutex<Vec<nf_audio::Cmd>>> = Default::default();
        snd.captured = Some(cap.clone());
    }
    // Free Bond (the intro skip, the mixer test's prologue).
    let c = {
        let sim = scene.sim.as_ref().unwrap();
        let c = sim.drones.triggers.iter().find(|t| t.out == 0x15).expect("the floor plate exists").centre();
        [c[0], c[1] - 1.0, c[2]]
    };
    for f in 0..3000 {
        { let sim = scene.sim.as_ref().unwrap(); for id in shout_pool_ids(sim) { println!("INTRO tick {f}: pool id {id}"); } }
        if voices_mode { tick_voices(&mut scene, &mut snd) } else { let _ = tick(&mut scene, &mut snd); } if !scene.sim.as_ref().unwrap().disabled { println!("intro freed at tick {f}"); break; }
    }
    { scene.sim.as_mut().unwrap().drones.scripts.skip_request = true; }
    for f in 0..300 {
        { let sim = scene.sim.as_ref().unwrap(); for id in shout_pool_ids(sim) { println!("SETTLE tick {f}: pool id {id}"); } }
        if voices_mode { tick_voices(&mut scene, &mut snd) } else { let _ = tick(&mut scene, &mut snd); }
    }
    { scene.sim.as_mut().unwrap().bond.pos = c; }
    println!("ch 0x15 on after the teleport ticks: {}", scene.sim.as_ref().unwrap().drones.channels.on(0x15));
    {
        let sim = scene.sim.as_ref().unwrap();
        println!("plate centre (bond's c) = ({:.2},{:.2},{:.2})", c[0], c[1], c[2]);
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.team != 1 { continue; }
            println!("  guard #{i} sw_ch {:#04x} enabled {} present {} state {:#x} pos ({:.1},{:.1},{:.1})",
                d.switch_channel, d.enabled, d.present, d.state, d.pos[0], d.pos[1], d.pos[2]);
        }
    }
    let mut floor: Vec<usize> = Vec::new(); // filled as guards enable

    let (mut ch15_seen, mut ch81_seen) = (false, false);
    let mut voice_events: Vec<String> = Vec::new();
    let mut pool_hits: Vec<(u32, usize, usize)> = Vec::new(); // (id, cycle, tick-in-cycle)
    let mut play_hits: Vec<(f32, f32, usize, usize)> = Vec::new();
    let mut ids_hits: Vec<(u32, usize, usize)> = Vec::new();
    let mut log_marks: Vec<usize> = floor.iter().map(|&i| scene.sim.as_ref().unwrap().drones.list[i].log.len()).collect();

    'outer: for k in 0..4 {
        { scene.sim.as_mut().unwrap().bond.pos = if k % 2 == 1 { [-47.0, 6.0, -23.3] } else { c }; }
        for t in 0..300 {
            {
                let sim = scene.sim.as_ref().unwrap();
                for id in shout_pool_ids(sim) { pool_hits.push((id, k, t)); voice_events.push(format!("cycle {k} tick {t} BEFORE tick: pool id {id}")); }
                if sim.drones.channels.on(0x15) && !ch15_seen { ch15_seen = true; println!(">>> ch 0x15 ROSE at cycle {k} tick {t}"); }
                if sim.drones.channels.on(0x81) && !ch81_seen { ch81_seen = true; println!(">>> ch 0x81 ROSE at cycle {k} tick {t}"); }
            }
            let (ids, cmds) = if voices_mode { tick_voices(&mut scene, &mut snd); (Vec::new(), Vec::new()) } else { tick(&mut scene, &mut snd) };
            let sim = scene.sim.as_ref().unwrap();
            for id in shout_pool_ids(sim) { pool_hits.push((id, k, t)); voice_events.push(format!("cycle {k} tick {t} AFTER tick: pool id {id}")); }
            for id in ids.iter().filter(|id| (111..=115).contains(*id)) { ids_hits.push((*id, k, t)); }
            for (l, r) in stream_plays(&cmds) { play_hits.push((l, r, k, t)); }
            for (i, d) in sim.drones.list.iter().enumerate() {
                if d.enabled && d.present && d.team == 1 && !floor.contains(&i) {
                    floor.push(i);
                    voice_events.push(format!("cycle {k} tick {t}: guard #{i} ENABLED (state {:#x})", d.state));
                }
                if floor.contains(&i) {
                    if let Some((_, sfx)) = d.voice {
                        if (111..=115).contains(&sfx) {
                            voice_events.push(format!("cycle {k} tick {t}: guard #{i} voice sfx {sfx} (state {:#x})", d.state));
                        }
                    }
                }
            }
            if !pool_hits.is_empty() && !play_hits.is_empty() { break 'outer; }
        }
        // Per-cycle state-log tail per guard.
        let sim = scene.sim.as_ref().unwrap();
        println!("\n--- cycle {k} end: ch 0x15 {} pool hits {:?}, driver-id hits {:?}, stream plays {} ---",
            sim.drones.channels.on(0x15), pool_hits.iter().map(|h| h.0).collect::<Vec<_>>(), ids_hits.iter().map(|h| h.0).collect::<Vec<_>>(), play_hits.len());
        for (gi, &i) in floor.iter().enumerate() {
            let d = &sim.drones.list[i];
            let mark = *log_marks.get(gi).unwrap_or(&0);
            let new: Vec<String> = d.log[mark.min(d.log.len())..].iter().map(|(f, s)| format!("{}:{:#x}", f, s)).collect();
            if gi < log_marks.len() { log_marks[gi] = d.log.len(); } else { log_marks.push(d.log.len()); }
            let vs = match d.voice { Some((_, sfx)) => format!("voice sfx {sfx}"), None => "no voice".into() };
            let dist = ((d.pos[0] - sim.bond.pos[0]).powi(2) + (d.pos[2] - sim.bond.pos[2]).powi(2)).sqrt();
            println!("  guard #{i} state {:#x} {} last_voice {} dist {dist:.1} pos ({:.1},{:.1},{:.1}) log +[{}]",
                d.state, vs, d.last_voice, d.pos[0], d.pos[1], d.pos[2], new.join(" "));
        }
    }
    println!("\n=== voice events (speech played a shout line) ===");
    for v in &voice_events { println!("  {v}"); }
    println!("=== pool hits (id, cycle, tick): {pool_hits:?}");
    println!("=== driver-id hits: {ids_hits:?}");
    println!("=== stream plays (l, r, cycle, tick): {play_hits:?}");

    // ---- the voices floor test, verbatim (fresh scene) ----
    println!("\n########## VOICES REPLICA ##########");
    let mut scene2 = nf_view::load(&iso, "07000002.bin", false).expect("reload 07000002");
    let mut snd2 = bare_sound(&mut scene2);
    if voices_mode { } else { snd2.captured = None; }
    let mut scratch2: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for _ in 0..3000 { tick_voices(&mut scene2, &mut snd2); if !scene2.sim.as_ref().unwrap().disabled { break } }
    { scene2.sim.as_mut().unwrap().drones.scripts.skip_request = true; }
    for _ in 0..300 { tick_voices(&mut scene2, &mut snd2); }
    assert!(!scene2.sim.as_ref().unwrap().disabled, "Bond is free after the intro");
    let c2 = {
        let sim = scene2.sim.as_ref().unwrap();
        let c = sim.drones.triggers.iter().find(|t| t.out == 0x15).expect("the floor plate exists").centre();
        [c[0], c[1] - 1.0, c[2]]
    };
    { scene2.sim.as_mut().unwrap().bond.pos = c2; }
    let mut pool_hits2: Vec<u32> = Vec::new();
    for _ in 0..30 {
        {
            let sim = scene2.sim.as_ref().unwrap();
            for id in sim.drones.audio.live_ids() { if (111..=115).contains(&id) && !pool_hits2.contains(&id) { pool_hits2.push(id); } }
        }
        tick_voices(&mut scene2, &mut snd2);
    }
    println!("replica: ch 0x15 on = {}", scene2.sim.as_ref().unwrap().drones.channels.on(0x15));
    for k in 0..4 {
        {
            let sim = scene2.sim.as_mut().unwrap();
            sim.bond.pos = if k % 2 == 1 { [-47.0, 6.0, -23.3] } else { c2 };
        }
        for t in 0..300 {
            {
                let sim = scene2.sim.as_ref().unwrap();
                for id in sim.drones.audio.live_ids() { if (111..=115).contains(&id) && !pool_hits2.contains(&id) { println!("replica hit: id {id} cycle {k} tick {t}"); pool_hits2.push(id); } }
            }
            tick_voices(&mut scene2, &mut snd2);
        }
    }
    println!("replica pool hits: {pool_hits2:?}");
}
