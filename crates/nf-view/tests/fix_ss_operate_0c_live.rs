//! fix-0c-hatch-latches2 rebased (level-0c-complete): the `SS_Operate` 0x1a9df8 presence half —
//! the collision pass's contact product drives the mask-0x2 SSs (the 0700000c `obj_stor*_pulley`
//! winches #351/#364/#377 are mask 0x2, ch 0 [D]: bullet-driven, NEVER channel-driven — the old
//! port could not move them at all). A player-bullet flight segment overlapping an SS part's
//! registered dynamic box is the game's class-5 contact [M 0x1a9df8:12-45; the list is the
//! collision pass's product, the death inclusive].

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The mask-0x2 armed SSs (the disc's bullet-driven scripts [D census]) with their positions.
fn winches(w: &Walker) -> Vec<(usize, usize, [f32; 3])> {
    w.scene.sim.as_ref().unwrap().drones.sss.list.iter().enumerate()
        .filter(|(_, s)| s.mask & 0x2 != 0 && s.state == 0x40)
        .map(|(i, s)| (i, s.placement, s.pos())).collect()
}

#[test]
fn the_bullet_driven_winch_sweeps_on_a_shot_and_holds_without_one() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700000c.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 8.98).abs() < 2.0 && (p[1] - 15.07).abs() < 2.0 && (p[2] + 149.65).abs() < 2.0,
            "the walkway start #1 [D], got {p:?}");
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);

    // Sister FIRST: no contact — the armed winch holds its playhead (the old code's behaviour:
    // ch 0 can never rise, so a mask-0x2 SS rested forever [M 0x1a9df8:46-51 short-circuit]).
    let wins = winches(&w);
    assert!(!wins.is_empty(), "the disc's mask-0x2 winch SSs exist [D #351/#364/#377]");
    let (wi, wpl, wpos) = wins[0];
    eprintln!("[ssop] winch ss[{wi}] placement {wpl} at {wpos:?}");
    // The winch's disc record [D #351/#364/#377: obj_stor*_pully*, state 0x40, wrap 0 (re-arm),
    // mask 0x2 (bullet), channel 0 (no channel half), sound 0xffff (none), rest 0, spline 1,
    // rate 75] — the render contract the sweep plays out (docs/systems/ss-sweep.md).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[wi];
        assert_eq!((s.state, s.wrap_flag, s.mask, s.channel, s.sound), (0x40, 0, 0x2, 0, 0xffff),
            "the winch's disc params [D #351/#364/#377]");
        assert_eq!((s.rest, s.spline, s.rate), (0.0, true, 75), "rest 0, the spline interp, rate 75 [D]");
    }
    for _ in 0..300 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[wi];
        assert_eq!(s.t, 0.0, "the sister: 300 idle ticks never move a mask-0x2 SS (no contact, ch 0 [M 0x1a9df8])");
        assert_eq!(s.state, 0x40, "and it stays armed");
    }

    // The shot: the sniper from the patrol circle at the winch — the REAL fire path (aim, the
    // trigger, the bullet's flight segment through the part's dynamic box).
    // Reach stand-in: Bond to the circle (the walkreal's leg-6 spot; the pulley gap blocks the
    // real route — the documented stand-in).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [5.8, 14.0, -109.5];
        sim.bond.vel = [0.0; 3];
        sim.gun.switch_request = 0x1e; // the loadout's sniper [M 0x1a7ef8 case 0x700000c]
    }
    w.wait(40);
    let fired = shoot_sweeping(&mut w, [wpos[0], wpos[1], wpos[2]], 120.0, 2400, wi);
    assert!(fired, "no shot swept the winch ss[{wi}] (placement {wpl}) — the contact never registered");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[wi];
        assert_eq!(s.state, 1,
            "the winch ss[{wi}] latched state 1 on the operate tick [M 0x1a9948's 0x40 arm: *(ext+0x22) = 1]");
    }
    // The full sweep [M 0x1a9948's state 1]: one operate tick is enough — state 1 sweeps every
    // tick to the wrap (t past the last key → t = 0, state = wrap_flag == 0 ? 0x40 : 0), no
    // further contact needed. The old port re-required operate every tick, so the shot's single
    // contact tick stalled the render at t = one step (task 787's follow-up).
    let (keys_n, rate, wrap_flag) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[wi];
        (s.keys.len(), s.rate, s.wrap_flag)
    };
    let last = (keys_n - 1) as f32;
    let step = last / rate as f32;
    let mut max_t = 0.0f32;
    let start = w.total_ticks;
    let wrapped = loop {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[wi];
        max_t = max_t.max(s.t);
        if s.t == 0.0 && (s.state == 0x40 || s.state == 0) { break true; }
        if w.total_ticks - start > rate + 40 { break false; }
        w.tick(nf_game::player::Input::default(), false);
        if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
    };
    assert!(wrapped, "the winch ss[{wi}] never wrapped after the shot (t {max_t} state stalled) — the sweep did not run in state 1 [M 0x1a9948]");
    assert!(max_t >= last - step,
        "the winch ss[{wi}] peaked at t {max_t} but the last key is {last} (step {step}) — the sweep stopped short");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[wi];
        eprintln!("[ssop] after the shot: t {} state {:#x} pos {:?}", s.t, s.state, s.pos());
        assert_eq!(s.state, if wrap_flag == 0 { 0x40 } else { 0 },
            "the wrap re-arms (wrap_flag 0) or disarms [M 0x1a9948:119-128]");
    }
    // The visible outcome the render compares: the held matrix after the wrap is the LAST key's
    // transform (states 0/0x40 never re-pose [M 0x1a9948]; `interp` clamps past the last key).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[wi];
        let end = nf_game::door::interp(&s.keys, 0, last as u16, last, s.spline).expect("the end pose interpolates");
        assert_eq!(s.matrix, end,
            "the winch ss[{wi}] holds the last key's pose after the wrap (the render's end frame)");
        assert_ne!(s.matrix, nf_game::door::interp(&s.keys, 0, last as u16, 0.0, s.spline).unwrap(),
            "the end pose differs from key 0 — the winch visibly moved");
    }
    // The re-arm is operable: a second shot snaps the pose to key 0 (the wrap reset t) and replays
    // the sweep [M the 0x40 arm reads t as it stands; the wrap set t = 0, 0x1a9948:119-128]. The
    // pose lags the playhead by one step (sweep poses at t, then advances).
    let fired2 = shoot_sweeping(&mut w, [wpos[0], wpos[1], wpos[2]], 120.0, 2400, wi);
    assert!(fired2, "the re-armed winch ss[{wi}] ignored the second shot");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[wi];
        assert_eq!(s.state, 1, "the second shot re-latches state 1 [M 0x1a9948's 0x40 arm]");
        let n = (s.t / step).round();
        assert!(n >= 1.0 && (n * step - s.t).abs() < 1e-3 && n <= 3.0,
            "the replay just started in whole steps (t {}, step {step})", s.t);
        let posed_at = ((n - 1.0) * step).max(0.0);
        assert_eq!(s.matrix,
            nf_game::door::interp(&s.keys, 0, last as u16, posed_at, s.spline).unwrap(),
            "the replay's pose tracks the replayed playhead (posed one step behind t)");
        assert_ne!(s.matrix, end_pose_guard(&sim, wi, last as u16, s.spline),
            "the replay left the end pose — the winch visibly snapped back");
    }
    eprintln!("[ssop] done in {} ticks", w.total_ticks);
}

/// The end-pose matrix of SS `wi` (the last key's transform on its own keys/spline).
fn end_pose_guard(sim: &nf_view::scene::Sim, wi: usize, last: u16, spline: bool) -> nf_game::door::Mat4 {
    let s = &sim.drones.sss.list[wi];
    nf_game::door::interp(&s.keys, 0, last, last as f32, spline).expect("the end pose interpolates")
}

/// Aim at `target` and fire through the real trigger path until the SS index `wi` moves.
fn shoot_sweeping(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, wi: usize) -> bool {
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    loop {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let s = &sim.drones.sss.list[wi];
            if s.t > 0.0 || s.state != 0x40 { return true; }
        }
        if w.total_ticks - start >= max_ticks { return false; }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = target[0] - bp[0];
            let dz = target[2] - bp[2];
            let dy = target[1] - eye;
            if (dx * dx + dy * dy + dz * dz).sqrt() > range { return false; }
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
            let trigger = on_target;
            if trigger { last_tap = w.total_ticks; }
            let input = if trigger {
                nf_game::player::Input { aiming: true, ..Default::default() }
            } else {
                nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08),
                    look: perr.clamp(-0.08, 0.08),
                    aiming: true,
                    ..Default::default()
                }
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
        }
        w.total_ticks += 1;
        if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
    }
}
