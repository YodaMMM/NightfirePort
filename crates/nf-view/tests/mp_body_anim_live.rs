//! mp-body-anim: the drawn MP bodies' full animation, end to end on 07000023 (2 humans + 2 bots,
//! slot 0's machine, the remote human driven by scripted session inputs):
//!
//! - standing still plays the stance set's idle (a clock-driven script from frame 1 — the old draw
//!   read the global clock and held every idle at its last frame);
//! - walking forward advances the walk distance, walking BACKWARDS runs it down (the walk plays in
//!   reverse — `ANIMOBJ+0xd4 = fwd × 0.5` is signed [M AnimSetUpdate 0x1c0938:96]);
//! - strafing appends the set's strafe word for the direction (dir 1 = positive = left) [M
//!   LAB_001c0dc8];
//! - a crouch press plays the class's stand→crouch transition, then the CROUCH set's idle, and the
//!   body draws lower; the next press plays crouch→stand [M Player_ChangeSubState 0x190630];
//! - looking up/down turns the upper body: the held gun points up/down with the view pitch [M
//!   AnimObjectAimAt 0x1bfe58];
//! - a killed slot stays drawn and plays the death script 0x6000151 [M 0x190630 case 0xd], then
//!   the respawn rebuilds the set.
//!
//! None of it may touch the lockstep hash (draw-only state): two sims ticking the same inputs, one
//! of them with every body's anim state scrambled, hash the same.
//!
//! Opt-in via `NF_ISO`.

use nf_game::mp::body_anim::LayerKind;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(path: &str) -> nf_view::scene::Scene {
    let mut settings = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    settings.human_count = 2;
    settings.bot_count = 2;
    settings.players[0].skin = 0;
    settings.players[1].skin = 9;
    let mut scene = nf_view::load_mp_carry(path, "07000023.bin", false, None, settings).expect("load 07000023");
    nf_view::net_slot0_setup(scene.sim.as_mut().unwrap());
    scene
}

fn row(f: impl Fn(&mut nf_net::Input)) -> [nf_net::Input; nf_net::MAX_AGENTS] {
    let mut r = [nf_net::input::NEUTRAL; nf_net::MAX_AGENTS];
    r[0].held = 2;
    r[1].held = 2;
    f(&mut r[1]);
    r
}

/// The slot-1 body's gun line: (the world direction of the muzzle datum's −X row — the P2K's hand
/// datum id + 3 or + 1 on the body skin, the line `AnimObjectAimAt` aims —, the drawn body origin y).
fn gun_and_origin(scene: &nf_view::scene::Scene, t: u32) -> ([f32; 3], f32) {
    let fs = scene.frame_state(None, t as f32, Default::default());
    let ci = scene.mp_bodies.iter().find(|(s, _)| *s == 1).unwrap().1;
    let (_, world, obj) = fs.posed[ci].as_ref().expect("slot 1 posed");
    let skin = &scene.characters[ci].skin;
    let id = scene.sim.as_ref().unwrap().weapons.third_person(2).unwrap().1 as u32;
    let d = skin.datums.iter().find(|d| d.id == id + 3 && d.bone >= 0).or_else(|| skin.datums.iter().find(|d| d.id == id + 1 && d.bone >= 0)).expect("a muzzle datum");
    let m = nf_formats::anim::mul(&nf_formats::anim::mul(&nf_formats::anim::quat_trans_to_mat(d.q, d.t), &world[d.bone as usize]), obj);
    let l = (m[0][0] * m[0][0] + m[0][1] * m[0][1] + m[0][2] * m[0][2]).sqrt();
    ([-m[0][0] / l, -m[0][1] / l, -m[0][2] / l], obj[3][1])
}

#[test]
fn remote_body_idles_walks_backwards_strafes_crouches_aims_and_dies() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load(&path);
    let mut t = 0u32;
    let mut tick = |scene: &mut nf_view::scene::Scene, r: &[nf_net::Input; nf_net::MAX_AGENTS], t: &mut u32| {
        nf_view::net_slot0_tick(scene.sim.as_mut().unwrap(), r);
        *t += 1;
    };
    let anim = |scene: &nf_view::scene::Scene| scene.sim.as_ref().unwrap().mp_anim[1].clone();
    let idle_1 = scene.sim.as_ref().unwrap().mp_anim_sets.as_ref().unwrap().set(1, false);
    let crouch_1 = scene.sim.as_ref().unwrap().mp_anim_sets.as_ref().unwrap().set(1, true);

    // 1. Idle: the class-1 idle, its clock running from the set init.
    for _ in 0..40 { tick(&mut scene, &row(|_| {}), &mut t); }
    let a = anim(&scene);
    let idle = a.layers.last().unwrap();
    assert_eq!(idle.kind, LayerKind::Idle(idle_1.idle), "the P2K body idles on the class-1 set's idle");
    assert!(idle.time > 20.0 && idle.time < 40.0, "the idle runs from its own start, not the match clock: {}", idle.time);

    // 2. Forward: the walk, the distance up.
    for _ in 0..20 { tick(&mut scene, &row(|i| i.forward = 1.0), &mut t); }
    let p0 = anim(&scene).phase;
    for _ in 0..40 { tick(&mut scene, &row(|i| i.forward = 1.0), &mut t); }
    let a = anim(&scene);
    assert!(matches!(a.layers.last().unwrap().kind, LayerKind::Walk { .. }), "walking: {:?}", a.layers);
    assert!(a.phase > p0 + 1.0, "forward walks the distance up: {p0} → {}", a.phase);

    // 3. Backwards: the same walk, the distance down (the walk plays in reverse).
    for _ in 0..20 { tick(&mut scene, &row(|i| i.forward = -1.0), &mut t); }
    let p1 = anim(&scene).phase;
    for _ in 0..40 { tick(&mut scene, &row(|i| i.forward = -1.0), &mut t); }
    let a = anim(&scene);
    assert!(matches!(a.layers.last().unwrap().kind, LayerKind::Walk { .. }));
    assert!(a.phase < p1 - 1.0, "backwards walks the distance down: {p1} → {}", a.phase);

    // 4. Strafe left (positive strafe = dir 1): the set's dir-1 word as an overlay.
    for _ in 0..30 { tick(&mut scene, &row(|i| i.strafe = 1.0), &mut t); }
    let a = anim(&scene);
    assert_eq!(a.strafe_dir(), 1);
    let st = a.layers.iter().find(|l| matches!(l.kind, LayerKind::Strafe(_))).expect("a strafe overlay");
    assert_eq!(st.kind, LayerKind::Strafe(idle_1.strafe_left));
    assert!(st.weight() > 0.9, "pure sideways: full overlay weight {}", st.weight());
    for _ in 0..30 { tick(&mut scene, &row(|i| i.strafe = -1.0), &mut t); }
    let a = anim(&scene);
    assert_eq!(a.layers.iter().find(|l| matches!(l.kind, LayerKind::Strafe(_))).unwrap().kind, LayerKind::Strafe(idle_1.strafe_right));

    // 5. Crouch: one press → the stand→crouch transition, then the crouch set's idle, drawn lower.
    for _ in 0..40 { tick(&mut scene, &row(|_| {}), &mut t); }
    let (_, stand_y) = gun_and_origin(&scene, t);
    tick(&mut scene, &row(|i| i.crouch = true), &mut t);
    let a = anim(&scene);
    assert!(a.crouch);
    assert_eq!(a.layers.last().unwrap().kind, LayerKind::OneShot(nf_formats::mpanim::stand2crouch_script(1)), "{:?}", a.layers);
    for _ in 0..40 { tick(&mut scene, &row(|_| {}), &mut t); }
    let a = anim(&scene);
    assert_eq!(a.layers.last().unwrap().kind, LayerKind::Idle(crouch_1.idle), "the crouch set after the transition");
    let (_, crouch_y) = gun_and_origin(&scene, t);
    assert!(crouch_y < stand_y - 0.2, "crouched draws lower: {stand_y:.3} → {crouch_y:.3}");
    // Crouch-walk: the crouch set's move word.
    for _ in 0..30 { tick(&mut scene, &row(|i| i.forward = 1.0), &mut t); }
    let a = anim(&scene);
    assert!(matches!(a.layers.last().unwrap().kind, LayerKind::Walk { walk, .. } if walk == crouch_1.walk), "crouch-walk: {:?}", a.layers);
    for _ in 0..30 { tick(&mut scene, &row(|_| {}), &mut t); }
    tick(&mut scene, &row(|i| i.crouch = true), &mut t);
    assert_eq!(anim(&scene).layers.last().unwrap().kind, LayerKind::OneShot(nf_formats::mpanim::crouch2stand_script(1)));
    for _ in 0..70 { tick(&mut scene, &row(|_| {}), &mut t); }
    assert!(!anim(&scene).crouch);

    // 6. Aim: look up, then down — the gun follows the pitch.
    for _ in 0..40 { tick(&mut scene, &row(|i| i.look = 1.0), &mut t); }
    let pitch_up = scene.sim.as_ref().unwrap().mp_bonds[1].as_ref().unwrap().pitch;
    let (g_up, _) = gun_and_origin(&scene, t);
    for _ in 0..80 { tick(&mut scene, &row(|i| i.look = -1.0), &mut t); }
    let pitch_down = scene.sim.as_ref().unwrap().mp_bonds[1].as_ref().unwrap().pitch;
    let (g_down, _) = gun_and_origin(&scene, t);
    eprintln!("pitch up {pitch_up:.3} gun {g_up:?}; pitch down {pitch_down:.3} gun {g_down:?}");
    assert!(pitch_up > 0.2 && pitch_down < -0.2, "the look input pitched the remote view ({pitch_up}, {pitch_down})");
    assert!(g_up[1] > 0.3, "looking up raises the gun: {g_up:?}");
    assert!(g_down[1] < -0.3, "looking down lowers the gun: {g_down:?}");
    // Level again.
    for _ in 0..40 { tick(&mut scene, &row(|i| i.look = 1.0), &mut t); if scene.sim.as_ref().unwrap().mp_bonds[1].as_ref().unwrap().pitch >= 0.0 { break; } }

    // 7. Death: the slot dies → drawn, the death script; the respawn rebuilds the set.
    {
        let s = scene.sim.as_mut().unwrap();
        // The combat path's kill (`step_net_combat`): health 0, the death timer, the match's kill.
        s.mp_hp[1] = 0.0;
        s.mp_dead_ticks[1] = 1;
        if let Some(m) = s.mp.as_mut() { m.player_killed(1, Some(0), false, false, &mut s.drones.rng); }
    }
    tick(&mut scene, &row(|_| {}), &mut t);
    let a = anim(&scene);
    assert!(a.dead);
    assert_eq!(a.layers.len(), 1);
    assert_eq!(a.layers[0].kind, LayerKind::OneShot(nf_formats::mpanim::DEATH_SCRIPT));
    let ci = scene.mp_bodies.iter().find(|(s, _)| *s == 1).unwrap().1;
    assert!(!scene.char_hidden(ci, Default::default()), "the dead body stays drawn");
    for _ in 0..120 { tick(&mut scene, &row(|_| {}), &mut t); }
    let fs = scene.frame_state(None, t as f32, Default::default());
    let (_, world, _) = fs.posed[ci].as_ref().expect("the dead body is posed");
    let head_y = world[5][3][1];
    eprintln!("dead body head bone y (object space) {head_y:.3}");
    assert!(head_y < 0.6, "the death script lays the body down (head bone {head_y:.3})");
    // The respawn (this fixture's match is not the running master state, so `mp_respawn`'s gate
    // refuses — the slot's life comes back by hand, as `step_net_combat`'s respawn writes it).
    {
        let s = scene.sim.as_mut().unwrap();
        s.mp_dead_ticks[1] = 0;
        s.mp_hp[1] = 100.0;
        if let Some(m) = s.mp.as_mut() { m.do_respawn(1, 0); }
    }
    for _ in 0..10 { tick(&mut scene, &row(|_| {}), &mut t); }
    let a = anim(&scene);
    assert!(!a.dead, "respawned");
    assert!(matches!(a.layers.last().unwrap().kind, LayerKind::Idle(_)), "the rebuilt set idles: {:?}", a.layers);
}

/// The bots' bodies animate from their own walking bodies: a moving bot's drawn yaw faces its
/// walking body's heading frame (`Bond` yaw), its clocks pick the walk, and its aim follows its
/// opponent. And nothing here enters the lockstep hash.
#[test]
fn bot_bodies_animate_and_the_anim_state_stays_out_of_the_hash() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut a = load(&path);
    let mut b = load(&path);
    let r = row(|_| {});
    let mut walked = 0;
    for t in 0..600u32 {
        let ha = nf_view::net_slot0_tick(a.sim.as_mut().unwrap(), &r);
        // Scramble the draw-only state on b every tick: the hash must not see it.
        if let Some(s) = b.sim.as_mut() {
            for x in s.mp_anim.iter_mut() { x.phase += 7.0; x.aim = [1.0, -1.0]; }
        }
        let hb = nf_view::net_slot0_tick(b.sim.as_mut().unwrap(), &r);
        assert_eq!(ha, hb, "tick {t}: the body anim state is not hashed");
        let s = a.sim.as_ref().unwrap();
        for slot in [2usize, 3] {
            let Some(inp) = s.mp_body_input(slot) else { continue };
            if inp.fwd.abs() > 0.05 && matches!(s.mp_anim[slot].layers.last().map(|l| l.kind), Some(LayerKind::Walk { .. })) { walked += 1; }
        }
    }
    assert!(walked > 30, "the bots' bodies walk on their walking bodies' speed ({walked} ticks)");
}
