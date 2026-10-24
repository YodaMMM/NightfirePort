//! level-0a-office-hatch-8 probe: the bunker trio's shout pan at the mixer, per-tick.
//! Runs the bathhouse_shout_mixer_live route, traces the shared RNG state through the boot,
//! then prints every stream Play with the bond/listener state around the shout.
//! Scratch tool — not a test. NF_MPFX_LOG=1 adds the Listener/Start command lines.

use nf_game::player::Input;

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

fn tick(scene: &mut nf_view::scene::Scene, snd: &mut nf_audio::sfx::Sound) -> (Vec<u32>, Vec<nf_audio::Cmd>) {
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input::default(), false, false, false);
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

fn rng_line(scene: &nf_view::scene::Scene, tag: &str) {
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[rng {tag}] f{} draws {} x {:08x} y {:08x}", sim.drones.frame, sim.drones.rng.draws, sim.drones.rng.x, sim.drones.rng.y);
}

fn main() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002");
    let mut snd = bare_sound(&mut scene);
    let cap: std::sync::Arc<std::sync::Mutex<Vec<nf_audio::Cmd>>> = Default::default();
    snd.captured = Some(cap.clone());
    let mut waited = 0;
    while scene.sim.as_ref().unwrap().disabled && waited < 3000 {
        if waited % 50 == 0 { rng_line(&scene, "intro"); }
        let _ = tick(&mut scene, &mut snd);
        waited += 1;
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        eprintln!("[intro] waited {waited} disabled {} bond.pos {:?} yaw {} look {:?}", sim.disabled, sim.bond.pos, sim.bond.yaw, sim.bond.look());
        sim.drones.scripts.skip_request = true;
    }
    for k in 0..300 {
        if k % 50 == 0 { rng_line(&scene, "postskip"); }
        let _ = tick(&mut scene, &mut snd);
    }
    rng_line(&scene, "pre-tp");
    {
        let sim = scene.sim.as_ref().unwrap();
        eprintln!("[pre-tp] bond.pos {:?} head {:?} look {:?} yaw {}", sim.bond.pos, sim.bond.head(), sim.bond.look(), sim.bond.yaw);
        for (i, d) in sim.drones.list.iter().enumerate() {
            let dx = d.pos[0] + 33.0; let dy = d.pos[1] - 6.0; let dz = d.pos[2] + 20.0;
            if dx * dx + dy * dy + dz * dz < 400.0 {
                eprintln!("[guard {i}] pos {:?} state {} los {} seen {} ever_seen {} alert {:.4} yaw {:.4}",
                    d.pos, d.state, d.los, d.seen, d.ever_seen, d.alert, d.yaw);
            }
        }
        let c = sim.drones.triggers.iter().find(|t| t.out == 0x81).unwrap().centre();
        eprintln!("[tp] plate centre {c:?} -> bond pos {:?}", [c[0], c[1] - 1.0, c[2]]);
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [c[0], c[1] - 1.0, c[2]];
    }
    for f in 0..1200 {
        let (ids, cmds) = tick(&mut scene, &mut snd);
        let shout = ids.iter().find(|id| (111..=115).contains(*id)).copied();
        let plays: Vec<(u32, f32, f32)> = cmds.iter().filter_map(|c| match c {
            nf_audio::Cmd::Play { voice, pitch: 1881, left, right, .. } => Some((*voice, *left, *right)),
            _ => None }).collect();
        if shout.is_some() || !plays.is_empty() || f < 2 {
            let sim = scene.sim.as_ref().unwrap();
            eprintln!("[f{f}] bond.pos {:?} head {:?} look {:?} ids-shout {shout:?} plays1881 {plays:?}",
                sim.bond.pos, sim.bond.head(), sim.bond.look());
        }
        if let Some(id) = shout {
            if !plays.is_empty() {
                eprintln!("[SHOUT] id {id} at f{f} plays {plays:?}");
                break;
            }
        }
    }
}
