//! g053 (GAPS G-053, MP scenarios): the carried objects' quick return after a kill-floor death.
//! `MP_PlayerKilled` 0x188f70 walks the dying body's hit list; a hurt volume (class 0x27) whose
//! `Hurt_GetType` is 5..7 (`4 < type < 8` [M :190-195]) sets the flag that preloads every carried
//! object's return counter to `30 * FRAME_RATE_INT` [M :281-283], so a flag dropped into a kill
//! floor comes home two frames later [M 0x1861a8:226-231]. Arena 07000025 carries type-6
//! damage-1000 kill floors [D: `Hurt::create` census, 6 volumes at y ≈ −4.6..−4.9] and every
//! scenario object. Offline (`--mp`, no session) the port passed `explosive = false` for Bond's
//! death always, so the flag lay in the pit for the full 30 s. NF_ISO-gated.

use nf_game::mp::{scenario, MpSettings};
use nf_view::scene::{Scene, Sim};

const ARENA: u32 = 0x0700_0025;

fn settings(sc: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(ARENA);
    s.scenario = sc;
    s.frag_limit = 10;
    s.time_limit = -1;
    s.human_count = 2;
    s.player_count = 2;
    s.total_players = 2;
    s.seed = 0x5ce7_a210;
    s.players[0].team = 0;
    s.players[1].team = 1;
    s.players[0].codename = "Alpha".into();
    s.players[1].codename = "Bravo".into();
    s
}

fn offline_load(sc: u32) -> Option<Scene> {
    let path = std::env::var("NF_ISO").ok()?;
    Some(nf_view::load_mp_carry(&path, &format!("{ARENA:08x}.bin"), false, None, settings(sc)).expect("loads"))
}

fn step(sim: &mut Sim) {
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
}

/// Bond (slot 0, team 0) holding team 1's flag. Returns the flag index.
fn take_enemy_flag(sim: &mut Sim) -> usize {
    let m = sim.mp.as_mut().unwrap();
    let p = sim.bond.pos;
    m.set_slot(0, p);
    m.touch_flag(0);
    let fi = (0..2).find(|&i| m.objects_rt.flags[i].as_ref().is_some_and(|f| f.carrier == Some(0))).expect("Bond carries a flag");
    assert_eq!(m.objects_rt.flags[fi].as_ref().unwrap().team, 1);
    fi
}

/// The centroid of the volume's first triangle - a point ON the volume (its box centre can
/// lie off an L-shaped floor).
fn tri_centre(h: &nf_game::hurt::Hurt) -> [f32; 3] {
    let t = h.tris[0];
    [(t[0][0] + t[1][0] + t[2][0]) / 3.0, (t[0][1] + t[1][1] + t[2][1]) / 3.0, (t[0][2] + t[1][2] + t[2][2]) / 3.0]
}

fn flag_state(sim: &Sim, fi: usize) -> i16 {
    sim.mp.as_ref().unwrap().objects_rt.flags[fi].as_ref().unwrap().state
}

/// Die in the kill floor while carrying the flag → the flag is home two frames after the drop.
#[test]
fn kill_floor_death_returns_the_flag_at_once() {
    let Some(mut scene) = offline_load(scenario::CAPTURE_THE_FLAG) else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_mut().unwrap();
    let h = sim.hurts.iter().find(|h| h.kind == 6 && h.damage >= 1000.0 && !h.disabled).expect("a type-6 kill floor").clone();
    let fi = take_enemy_flag(sim);
    let c = tri_centre(&h);
    let centre = [c[0], c[1] + 0.3, c[2]];
    sim.bond.pos = centre;
    let mut died_at = None;
    for t in 0..120 {
        step(sim);
        if sim.drones.death.active && died_at.is_none() {
            died_at = Some(t);
            assert_eq!(flag_state(sim, fi), 2, "the death drops the flag");
        }
        if let Some(d) = died_at {
            if t >= d + 3 { break; }
        }
    }
    let d = died_at.expect("the kill floor kills Bond");
    eprintln!("died at tick {d}, flag state {}", flag_state(sim, fi));
    assert_eq!(flag_state(sim, fi), 0, "the preloaded counter sends the flag home within 3 frames [M 0x188f70:281-283]");
}

/// Sister: a death with no hurt volume in the hit list (the health run out on open floor) leaves
/// the flag down for the full 30 s count.
#[test]
fn an_ordinary_death_leaves_the_flag_down() {
    let Some(mut scene) = offline_load(scenario::CAPTURE_THE_FLAG) else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..5 { step(sim); }
    let fi = take_enemy_flag(sim);
    sim.drones.health.health = 0.0;
    let mut died = false;
    for _ in 0..10 {
        step(sim);
        died |= sim.drones.death.active;
    }
    assert!(died);
    assert_eq!(flag_state(sim, fi), 2, "an ordinary death keeps the 30 s count");
}

fn bot_settings(sc: u32) -> MpSettings {
    let mut s = settings(sc);
    s.human_count = 1;
    s.player_count = 1;
    s.bot_count = 1;
    s.total_players = 2;
    s.players[1].team = 1;
    s
}

fn bot_load(sc: u32) -> Option<Scene> {
    let path = std::env::var("NF_ISO").ok()?;
    Some(nf_view::load_mp_carry(&path, &format!("{ARENA:08x}.bin"), false, None, bot_settings(sc)).expect("loads"))
}

/// A bot carrying the flag into the kill floor dies there (`NDrone2_DealWithObjHit` 0x13fff0's
/// class-0x27 MP arm [M :320-335]): one death, no credit to anyone, and the flag is home two
/// frames later (the type-6 volume in the hit list [M 0x188f70:190-195,281-283]). Before, the
/// bots took no hurt-volume damage at all. Sister: the same bot standing on open floor lives.
#[test]
fn a_bot_dies_in_the_kill_floor_and_the_flag_comes_home() {
    let Some(mut scene) = bot_load(scenario::CAPTURE_THE_FLAG) else { eprintln!("skip: NF_ISO not set"); return };
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..10 { step(sim); }
    let h = sim.hurts.iter().find(|h| h.kind == 6 && h.damage >= 1000.0 && !h.disabled).expect("a type-6 kill floor").clone();
    let (bslot, bpos) = {
        let m = sim.mp.as_ref().unwrap();
        assert_eq!(m.bots.len(), 1);
        (m.bots[0].slot, m.bots[0].brain.pos)
    };
    // sister first: ten ticks on its spawn floor and the bot is alive and unhurt by any volume
    assert!(!sim.mp.as_ref().unwrap().bots[0].dead, "alive on its spawn floor (at {bpos:?})");
    let fi = {
        let m = sim.mp.as_mut().unwrap();
        m.set_slot(bslot, [bpos[0], bpos[1] - nf_game::player::EYE, bpos[2]]);
        m.touch_flag(bslot);
        (0..2).find(|&i| m.objects_rt.flags[i].as_ref().is_some_and(|f| f.carrier == Some(bslot))).expect("the bot carries a flag")
    };
    let c = tri_centre(&h);
    // the eye 1.8 above the floor (the root BOT_EYE below it, the floor 1.0 below the root)
    let eye = [c[0], c[1] + 1.0 + nf_game::player::EYE, c[2]];
    {
        let bot = &mut sim.mp.as_mut().unwrap().bots[0];
        bot.brain.pos = eye;
        bot.brain.body.pos = eye;
        bot.brain.body.vel = [0.0; 3];
        bot.brain.route = Default::default();
    }
    let mut died_at = None;
    for t in 0..30 {
        step(sim);
        let m = sim.mp.as_ref().unwrap();
        if m.bots[0].dead && died_at.is_none() { died_at = Some(t); }
        if died_at.is_some_and(|d| t >= d + 3) { break; }
    }
    let d = died_at.expect("the kill floor kills the bot");
    let m = sim.mp.as_ref().unwrap();
    eprintln!("bot died at tick {d}");
    assert_eq!(m.game.slots[bslot].deaths, 1, "one death");
    assert_eq!(m.game.slots[0].kills, 0, "nobody is credited");
    assert_eq!(m.objects_rt.flags[fi].as_ref().unwrap().state, 0, "the flag is home within 3 frames");
}
