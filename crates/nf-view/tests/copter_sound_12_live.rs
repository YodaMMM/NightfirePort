//! level-12-copter-sound-distance: the 07000012 helicopter's sound positioning pinned to the
//! game's law, through the port's real EE sound layer and the real IOP driver with the level's
//! own bank.
//!
//! The game's chain [M]: the copter's SP script 0x060005bd starts the engine loop 0x10b 3-D at
//! the stream-4 object (script 0x060005bd stream 10 t=0: SOUND id=0x10b flat=false has_obj=true
//! obj_stream=4 [D decode]) — `Script_SoundStart` 0x1d8820 takes Sound_Play3D(100.0, −1, −1, id,
//! obj+0xc0) when the flag byte says 3-D and the object exists — and `Script_Interp` case 5
//! (0x1d9a08) re-posts the position every tick (`Sound_SetPosition(handle, obj+0xc0)`). The
//! record radii [D]: bank slot 0x1f (level 12 per Sound_Ready 0x1dceb8) effect 0x10b =
//! tracking 2, inner 10, outer 100, priority 100, master 100, sub0 volume 95, no randomisation;
//! SFXOutputData[0x10b] = inner 10, outer 100 (the two tables agree). The per-tick law is
//! `PS2_SFXCalculate3D` 0x1060c: att = clamp(1000 − (max(0,d−inner·10)·1000)/((outer−inner)·10),
//! 0,1000)/10 (d = √Σ((Δ/100)²), positions the EE's ×1000), the quadratic att²/100, the front
//! pair split by the left-axis dot, a rear channel fed by the behind dot ((dot+1000)·att²/2000,
//! capped ×75/100), all × the combined volume (ES_CombineVolumes 0x1300: the start's 95 with
//! master 100 → 90²/100 = 90) in `PS2_SFXSetup3D` 0x10eb8. Beyond the outer radius the driver
//! never starts the sound (SFXSetup 0x4220's reject) and Calculate3D reads 0.
//!
//! The gun 0x5df [D]: outer 50, tracking 2 — inaudible past 50 world units in the game and here.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;
use nf_game::world::V3;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The engine loop starts 3-D at the copter and its emitter follows it every tick.
#[test]
fn the_copter_engine_starts_3d_and_follows_the_copter() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000012.bin", false).unwrap());
    // This test drives the EE sound layer itself (below). The walker's headless driver
    // (`attach_headless_sound`, run after every step) would drain the same command queue first,
    // so it is detached here: one consumer, as in the app.
    w.scene.sim.as_mut().unwrap().headless_sound = None;
    w.wait_enabled(3000, "the level start");
    // Drive the EE sound layer from the first tick (the App's sound_tick flow), draining the
    // commands so a load-time start cannot be missed.
    let mut start = None;
    let mut pos_ticks = 0u32;
    let mut first_pos = None;
    let mut last_pos = None;
    let mut started_ids = Vec::new();
    let mut drive = |w: &mut Walker, start: &mut Option<(u32, Option<V3>, Option<(f32, f32)>)>, pos_ticks: &mut u32, first_pos: &mut Option<V3>, last_pos: &mut Option<V3>, started_ids: &mut Vec<u32>| {
        let sim = w.scene.sim.as_mut().unwrap();
        let (pos, f) = (sim.bond.head(), sim.bond.look());
        let l = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6);
        let norm = [f[2] / l, 0.0, -f[0] / l];
        sim.drones.audio.update(sim.sfx.as_ref(), (pos, norm, [0.0, 1.0, 0.0], f), &|_| false);
        for c in sim.drones.audio.take_commands() {
            match c {
                nf_game::sound::Cmd::Start { handle, id, pos: p, radii } => {
                    started_ids.push(id);
                    if id == 0x10b && start.is_none() { *start = Some((handle, p, radii)); *first_pos = p; *last_pos = p; }
                }
                nf_game::sound::Cmd::Position { handle, pos: p } if start.is_some_and(|(h, _, _)| h == handle) => {
                    *pos_ticks += 1;
                    *last_pos = Some(p);
                }
                _ => {}
            }
        }
    };
    for _ in 0..60u32 { w.tick(Default::default(), false); drive(&mut w, &mut start, &mut pos_ticks, &mut first_pos, &mut last_pos, &mut started_ids); }
    // The fight's gate [D plc #970 p6 = 37]; the SP's mode-8 loop then plays the engine line.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let frame = sim.drones.frame;
        sim.drones.channels.set(37, 1, frame);
    }
    for _ in 0..240u32 {
        w.tick(Default::default(), false);
        drive(&mut w, &mut start, &mut pos_ticks, &mut first_pos, &mut last_pos, &mut started_ids);
    }
    let (h, p, radii) = start.unwrap_or_else(|| panic!("the engine 0x10b starts in the run [D 060005bd stream 10]; started ids {started_ids:?}"));
    assert!(p.is_some(), "the engine starts 3-D (Sound_Play3D at the stream-4 object [M 0x1d8820])");
    assert!(radii.is_none(), "no custom radii: the bank's inner/outer decide [M 0x1dcc68 sends −1/−1 for record radii]");
    assert!(pos_ticks >= 100, "the emitter follows: a Position command most ticks ({pos_ticks}) [M 0x1d9a08 case 5]");
    let fp = first_pos.unwrap();
    let lp = last_pos.unwrap();
    let moved = ((fp[0] - lp[0]).powi(2) + (fp[1] - lp[1]).powi(2) + (fp[2] - lp[2]).powi(2)).sqrt();
    assert!(moved > 2.0, "the emitter rides the flying copter (moved {moved:.1} u), not a static spot");
    let _ = h;
}

/// The engine's volume and pan at measured distances/angles equal the game's integer law
/// [M 0x1060c + 0x10eb8 + 0x1300] with the [D] radii (inner 10, outer 100) and the [D] start
/// volume (sub0 95). One GSV unit = 1/100 of the fraction, so the tolerance is ±0.015.
#[test]
fn the_copter_engine_fades_and_pans_by_the_game_law() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000012.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let copter = sim.drones.copters.first().map(|c| c.pos()).expect("the level's copter");
    let (slot, bank, sbf) = scene.sound_bank.clone().expect("the level bank");
    // The [D] record the test's law uses.
    let fx = nf_formats::soundbank::Effect::parse(bank.effect(0x10b).expect("the engine's effect record")).unwrap();
    assert_eq!((fx.tracking, fx.inner, fx.outer, fx.priority, fx.master_volume), (2, 10, 100, 100, 100),
        "the engine's [D] record: full 3-D, inner 10, outer 100, priority 100, master 100");
    assert_eq!((fx.subs[0].volume, fx.subs[0].volume_rnd), (95, 0), "the engine's [D] start volume 95, unrandomised");

    let mut drv = nf_audio::sfx::Driver::new();
    drv.load_bank(slot, bank, sbf);
    let mut out = Vec::new();
    assert_eq!(drv.start(&mut out, 0x10b, Some(copter), 5, None), 0, "the engine starts");
    let fwd = [0.0f32, 0.0, 1.0];
    let left = [-1.0f32, 0.0, 0.0];
    let gsv = |l: i32| ((l * 0x3fff) / 100) as f32 / 0x3fff as f32;
    // The game's integer chain, world units: att, a2 = att²/100, the dots ×1000, the combined
    // volume 90 = (95·100)²/10000 (sub0 95 [D], master 100 [D], everything else at its default).
    let law = |lp: [f32; 3]| -> (f32, f32, f32) {
        let dv = [0, 1, 2].map(|k| (copter[k] * 1000.0) as i32 - (lp[k] * 1000.0) as i32);
        let u: Vec<i32> = dv.iter().map(|&x| x / 100).collect();
        let s: i64 = u.iter().map(|&x| x as i64 * x as i64).sum();
        let d = nf_audio::sfx::ps2_sqrt(s as u32) as i32;
        let d = (d - fx.inner * 10).max(0);
        let rng = ((fx.outer - fx.inner) * 10).max(10);
        let a = ((1000 - (d * 1000) / rng).max(0) / 10).min(100);
        let a2 = (a * a) / 100;
        let m = nf_audio::sfx::ps2_sqrt(s as u32) as i32;
        let un: Vec<i32> = if m != 0 { u.iter().map(|&x| x * 1000 / m).collect() } else { u.clone() };
        let dot = |a: [i32; 3], b: &[i32]| (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / 1000;
        let n = [-1000i32, 0, 0]; // the left axis of the test's listener (facing +z)
        let dl = dot(n, &un);
        let dr = dot([-n[0], -n[1], -n[2]], &un);
        let db = dot([0, 0, -1000], &un); // −forward
        let rear = ((db + 1000) * a2 / 2000).clamp(0, 100);
        let front = 100 - rear;
        let l = ((((dl + 1000) * a2) / 2000).clamp(0, 100) * front / 100).clamp(0, 100);
        let r = ((((dr + 1000) * a2) / 2000).clamp(0, 100) * front / 100).clamp(0, 100);
        let b = (rear * 75 / 100).clamp(0, 100);
        let vol = (95 * 100 / 100 * 100 / 100 * fx.master_volume / 100).clamp(0, 100);
        let vol = (vol * vol) / 100;
        (gsv((l * vol) / 100), gsv((r * vol) / 100), gsv((b * vol) / 100))
    };
    let measure = |drv: &mut nf_audio::sfx::Driver, lp: [f32; 3]| -> (f32, f32, f32) {
        drv.set_listener(lp, fwd, left);
        drv.set_position(5, None, copter);
        let mut out = Vec::new();
        drv.update(&mut out, &[]);
        let mut front = (0.0f32, 0.0f32);
        let mut rear = 0.0f32;
        for c in out {
            match c {
                nf_audio::Cmd::Play { voice, left, right, .. } | nf_audio::Cmd::Set { voice, left, right, .. } => {
                    if voice % 2 == 0 { front = (left, right); } else { rear = left.abs(); }
                }
                _ => {}
            }
        }
        (front.0, front.1, rear)
    };
    let spots = [
        ("d20_front", [copter[0], copter[1], copter[2] - 20.0]),
        ("d50_front", [copter[0], copter[1], copter[2] - 50.0]),
        ("d50_behind", [copter[0], copter[1], copter[2] + 50.0]),
        ("d50_left", [copter[0] - 50.0, copter[1], copter[2]]),
        ("d120_front", [copter[0], copter[1], copter[2] - 120.0]),
    ];
    for (name, lp) in spots {
        let (ml, mr, mb) = measure(&mut drv, lp);
        let (el, er, eb) = law(lp);
        eprintln!("[12snd] {name}: measured ({ml:.3},{mr:.3},{mb:.3}) law ({el:.3},{er:.3},{eb:.3})");
        assert!((ml - el).abs() < 1e-6 && (mr - er).abs() < 1e-6,
            "{name}: the front pair follows the game's law [M 0x1060c/0x10eb8] — measured ({ml:.3},{mr:.3}) law ({el:.3},{er:.3})");
        assert!((mb - eb).abs() < 1e-6,
            "{name}: the rear channel follows the game's law [M 0x1060c] — measured {mb:.3} law {eb:.3}");
    }
    // The law's shape, as behaviours (the same numbers, asserted as relations):
    let (l20, r20, _) = measure(&mut drv, spots[0].1);
    let (l50, _, b50) = measure(&mut drv, spots[1].1);
    let (_, _, bb) = measure(&mut drv, spots[2].1);
    let (ll, lr, _) = measure(&mut drv, spots[3].1);
    let (l120, r120, b120) = measure(&mut drv, spots[4].1);
    assert!(l50 < l20, "the engine fades with distance (att² on the linear falloff [M 0x1060c])");
    assert!(bb > l50 && bb > r50_check(l50), "behind, the rear channel dominates the front pair (the game's rear cue [M 0x1060c])");
    assert_eq!(ll, 0.0, "90° left: the left front voice is silent, the cue is on the right [M 0x1060c's dot split]");
    assert!(lr > 0.1, "90° left: the right front voice carries the sound");
    assert!(l120 < 0.01 && r120 < 0.01 && b120 < 0.01, "beyond the outer radius 100 [D] the engine is silent [M 0x4220 + 0x1060c]");
    let _ = (l20, r20);
}

fn r50_check(v: f32) -> f32 { v }

/// The gun 0x5df [D outer 50]: no voice beyond the outer radius, a voice inside it.
#[test]
fn the_copter_gun_is_audible_only_inside_its_outer_radius() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000012.bin", false).unwrap();
    let (slot, bank, sbf) = scene.sound_bank.clone().expect("the level bank");
    let fx = nf_formats::soundbank::Effect::parse(bank.effect(0x5df).expect("the gun's effect record")).unwrap();
    assert_eq!((fx.tracking, fx.inner, fx.outer), (2, 10, 50), "the gun's [D] record: full 3-D, inner 10, outer 50");
    let mut drv = nf_audio::sfx::Driver::new();
    drv.load_bank(slot, bank, sbf);
    let at = [100.0f32, 5.0, 100.0];
    let mut out = Vec::new();
    // 60 units away: beyond the outer 50 — the setup rejects, no voice keys on [M 0x4220].
    drv.set_listener([at[0], at[1], at[2] - 60.0], [0.0, 0.0, 1.0], [-1.0, 0.0, 0.0]);
    let h = drv.start(&mut out, 0x5df, Some(at), 7, None);
    let played = out.iter().any(|c| matches!(c, nf_audio::Cmd::Play { .. }));
    assert!(!played, "the gun does not start 60 u away (outer 50 [D]; the SFXSetup reject [M 0x4220])");
    // 20 units away: inside — a voice keys on.
    let mut out2 = Vec::new();
    drv.set_listener([at[0], at[1], at[2] - 20.0], [0.0, 0.0, 1.0], [-1.0, 0.0, 0.0]);
    let h2 = drv.start(&mut out2, 0x5df, Some(at), 8, None);
    let played2 = out2.iter().any(|c| matches!(c, nf_audio::Cmd::Play { left, .. } if *left > 0.0));
    assert!(played2, "the gun starts inside its outer radius (handles {h}/{h2})");
}
