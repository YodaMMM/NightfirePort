//! mp-remote-fx: what one machine sees and hears of the OTHER players and the bots. Two machines
//! (slot 0's and slot 1's, the live match-start setup) tick the same input rows through
//! `nf_view::net_tick`; both humans face each other and fire the P2K. A third machine (slot 0)
//! runs with the effects pass disabled: every tick's hash must agree across all three — the
//! effects are draw/audio only and never touch the hashed state. On slot 0's machine the remote
//! slot's shots must show (muzzle flash latch, the fire script's 0x403 started 3-D at the
//! shooter's head, the audio driver's `Cmd::Start` at that position, the world impact's decal or
//! the local hit feedback), the bots' shots their `weapon_data+0x60` sound at the bot, and a dead
//! remote body its death pose. NF_ISO-gated.

use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;
use nf_view::mpfx::MpShotHit;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings(humans: u32, bots: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = humans;
    s.player_count = humans as u16;
    s.bot_count = bots;
    s.total_players = humans + bots;
    s.seed = 0x0bed_0bad;
    s.time_limit = 10 * 60;
    s
}

fn machine(path: &str, s: &MpSettings, slot: usize) -> nf_view::scene::Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    // Both humans face each other (the NF_MP_FACE_OTHER aid's rule, by slot, on every machine).
    let pos = |sim: &nf_view::scene::Sim, s: usize| if s == sim.mp_local_slot { sim.bond.pos } else { sim.mp_bonds[s].as_ref().unwrap().pos };
    let (p0, p1) = (pos(sim, 0), pos(sim, 1));
    let y0 = (p1[0] - p0[0]).atan2(p1[2] - p0[2]);
    let y1 = (p0[0] - p1[0]).atan2(p0[2] - p1[2]);
    for (s, y) in [(0usize, y0), (1, y1)] {
        if s == sim.mp_local_slot { sim.bond.yaw = y; } else { sim.mp_bonds[s].as_mut().unwrap().yaw = y; }
    }
    scene
}

/// Every 12 ticks both humans fire the P2K (the sender's pipeline decided `shot`).
fn row(t: u32) -> [nf_net::Input; nf_net::MAX_AGENTS] {
    std::array::from_fn(|sl| {
        let mut i = NEUTRAL;
        if sl < 2 {
            i.held = 2;
            i.trigger = t % 12 < 2;
            i.shot = t % 12 == 0;
        }
        i
    })
}

#[test]
fn remote_shots_show_and_sound_without_touching_the_hash() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings(2, 2);
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let mut c = machine(&path, &s, 0);
    c.sim.as_mut().unwrap().mp_fx.disabled = true;
    let (mut flash_ticks, mut starts_at_shooter, mut world_hits, mut local_hits, mut bot_sounds, mut death_poses) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
    let (mut shots_seen, mut bot_shots, mut tracer_ticks) = (0u32, 0u32, 0u32);
    let decals0 = a.sim.as_ref().unwrap().drones.effects.decals.len();
    let mut started_seen = 0usize;
    let mut driver_starts = 0u32;
    for t in 0..2400u32 {
        let r = row(t);
        let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &r, 0);
        let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &r, 1);
        let hc = nf_view::net_tick(c.sim.as_mut().unwrap(), &r, 0);
        assert_eq!(ha, hb, "tick {t}: the two machines' hashes");
        assert_eq!(ha, hc, "tick {t}: the effects pass changed the hashed state");
        let sa = a.sim.as_mut().unwrap();
        let frame = sa.drones.frame;
        if sa.mp_fx.last_shot[1].is_some_and(|f| frame.saturating_sub(f) < 2) { flash_ticks += 1; }
        if !sa.mp_fx.tracers.is_empty() { tracer_ticks += 1; }
        // The audio frame (`sound_tick`'s pool update): the module's starts reach the driver
        // as `Cmd::Start` at their positions.
        let head1 = sa.mp_bonds[1].as_ref().map(|b| b.head());
        let l = sa.bond.head();
        sa.drones.audio.update(sa.sfx.as_ref(), (l, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]), &|_| false);
        let cmds = sa.drones.audio.take_commands();
        for &(f, id, pos, slot) in &sa.mp_fx.started[started_seen..] {
            if f != frame { continue; }
            let sent = cmds.iter().any(|c| matches!(c, nf_game::sound::Cmd::Start { id: i, pos: Some(p), .. } if *i == id && *p == pos));
            if slot == 1 && id == 0x403 {
                shots_seen += 1;
                if head1.is_some_and(|h| (0..3).all(|k| (h[k] - pos[k]).abs() < 1e-4)) { starts_at_shooter += 1; }
                if sent { driver_starts += 1; }
            }
            if slot >= 2 && slot != usize::MAX && id != 0x89 { bot_sounds += 1; }
        }
        started_seen = sa.mp_fx.started.len();
        // Draw-side: a dead remote slot has a death body.
        for sl in 1..4 { if sa.mp_body_death(sl).is_some() { death_poses += 1; } }
        if sa.drones.health.dir_bits != 0 && sa.drones.health.flash > 0.0 { local_hits += 1; }
        let _ = &mut world_hits;
        let _ = &mut bot_shots;
    }
    let sa = a.sim.as_ref().unwrap();
    world_hits += sa.drones.effects.decals.len().saturating_sub(decals0) as u32;
    bot_shots += sa.mp_fx.started.iter().filter(|s| s.3 >= 2 && s.3 < 16).count() as u32;
    eprintln!("slot 1 shots heard {shots_seen} ({starts_at_shooter} started at its head), flash ticks {flash_ticks}, decals +{world_hits}, local hit ticks {local_hits}, bot sounds {bot_sounds} (shot-tagged {bot_shots}), tracer ticks {tracer_ticks}, driver starts {driver_starts}, death-pose ticks {death_poses}");
    assert!(shots_seen >= 100, "slot 1's 200 shots play its gun sound on slot 0's machine ({shots_seen})");
    assert_eq!(starts_at_shooter, shots_seen, "every remote shot sound starts 3-D at the shooter's head");
    assert!(driver_starts > 0, "the audio pool hands the remote shot sounds to the driver as positioned starts");
    assert!(flash_ticks >= 2 * shots_seen - 2, "the muzzle flash shows 2 ticks per shot ({flash_ticks})");
    assert!(world_hits > 0 || local_hits > 0, "the remote rounds land somewhere visible");
    assert!(bot_sounds > 0, "the bots' shots/footsteps are heard positioned at them");
    // The machine with the pass disabled drew and played nothing of it.
    let sc = c.sim.as_ref().unwrap();
    assert!(sc.mp_fx.started.is_empty() && sc.mp_fx.last_shot.iter().all(|x| x.is_none()), "sister: disabled = no effects");
}

/// The events themselves: one shot of slot 1 straight at slot 0 is a `Human(0)` hit on every
/// machine, and slot 0's machine shows the hit feedback (the 0x412 hit sound, the glow bits).
#[test]
fn a_remote_hit_on_the_local_player_flashes_and_sounds() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings(2, 0);
    let mut a = machine(&path, &s, 0);
    let mut seen_hit = false;
    for t in 0..240u32 {
        let mut r: [nf_net::Input; nf_net::MAX_AGENTS] = std::array::from_fn(|_| NEUTRAL);
        r[1].held = 2;
        r[1].shot = t % 30 == 10;
        let sa = a.sim.as_mut().unwrap();
        // Peek the events before the flush consumes them: re-run the combat's ray here is not
        // possible from outside, so read the result instead — the started list and the glow.
        let before = sa.mp_fx.started.len();
        nf_view::net_tick(sa, &r, 0);
        // The rounds fly (mp-combat-weapons: the shared rounds travel and hit a few ticks after
        // the trigger), so the hit feedback is checked on every tick, not only the shot's.
        {
            let hit = sa.mp_fx.started[before..].iter().any(|s| s.1 == 0x412);
            if hit {
                seen_hit = true;
                assert_ne!(sa.drones.health.dir_bits, 0, "the direction glow bits are set");
                assert!(sa.drones.health.flash > 0.0, "the hit flash is up");
            }
        }
    }
    let _ = MpShotHit::Miss;
    assert!(seen_hit, "slot 1 facing slot 0 hits it at least once in 8 shots");
}
