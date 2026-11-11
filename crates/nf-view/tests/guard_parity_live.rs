//! Guard behaviour against the decompile, on each level's real drones and property tables:
//! the dead-guard alert bit, first-attack alert bits, the 09/0A sight rule, the 09/0A impact
//! reset, the sight gates, the hearing rows and the rescued hostage. Tags: [M] decompile.
//! docs/reports/agent-guards.md lists the evidence. Opt-in NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::{msg, state, Data, Drone, Rng};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }
fn scene(level: &str) -> nf_view::scene::Scene { nf_view::load(&iso(), level, false).expect("load") }

/// A clone of the first drone `pick` accepts, ready for direct messages.
fn drone(sim: &nf_view::scene::Sim, what: &str, pick: impl Fn(&Drone, &Data) -> bool) -> Drone {
    let d = sim.data();
    sim.drones.list.iter().find(|x| pick(x, &d)).unwrap_or_else(|| panic!("no drone: {what}")).clone()
}

fn went_to(g: &Drone, s: u8) -> bool { g.log.iter().any(|&(_, n)| n == s) }

/// Parks observer `o` 4 units from `s`, facing it.
fn face(sim: &mut nf_view::scene::Sim, o: usize, s: usize) {
    let sp = sim.drones.list[s].pos;
    let x = &mut sim.drones.list[o];
    x.pos = [sp[0] + 4.0, sp[1], sp[2]];
    x.yaw = (sp[0] - x.pos[0]).atan2(sp[2] - x.pos[2]);
    x.velocity = [0.0; 3];
}

/// A team-1 guard with prop 0x31 and a watching guard with prop 0x29, both alert-free.
fn victim_and_observer(sim: &nf_view::scene::Sim) -> (usize, usize) {
    let d = sim.data();
    let ok = |x: &Drone| x.present && x.enabled && !x.corpse && x.team == 1 && x.alert_bits == 0 && !x.alerted_by_drone;
    let v = sim.drones.list.iter().enumerate().position(|(i, x)| i != 0 && ok(x) && x.prop(&d, 0x31)).expect("a prop-0x31 guard");
    let o = sim.drones.list.iter().enumerate().position(|(i, x)| i != v && ok(x) && x.watching && x.prop(&d, 0x29)).expect("a watching prop-0x29 guard");
    (v, o)
}

fn corpse_run(headshot: bool) -> (u32, Option<usize>, usize) {
    let mut w = walker::Walker::new(scene("07000005.bin"));
    w.wait_enabled(6000, "level start");
    w.wait(120);
    let (v, o) = victim_and_observer(w.scene.sim.as_ref().unwrap());
    {
        let sim = w.scene.sim.as_mut().unwrap();
        face(sim, o, v);
        let x = &mut sim.drones.list[v];
        x.silent_kill = headshot; // `+0x3A`, written by the head hit [M 0x145558:78]
        x.health = 0.0;
        x.set_state(state::DEAD);
    }
    for _ in 0..200u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.list[o].assist_src == Some(v) { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    (sim.drones.list[v].alert_bits, sim.drones.list[o].assist_src, v)
}

/// `NDrone2_DSTATE_Dead` ENTER: prop 0x31 and no head kill → `+0x4FC |= 8` [M 0x163fe8:32-33]; the
/// corpse keeps +0x100 (SetAsDead clears 0x20/0x80 only [M 0x14a2f0]), so it stays a source and an
/// observer with prop 0x29 takes the alert [M 0x176250:119-193].
#[test]
fn a_guard_who_sees_a_body_shot_corpse_is_alerted() {
    let (bits, src, v) = corpse_run(false);
    assert_ne!(bits & 8, 0, "the dead guard raised bit 8 [M 0x163fe8:32]");
    assert_eq!(src, Some(v), "the observer took the alert from the corpse [M 0x176250:206]");
}

/// Sister: a head kill sets `+0x3A`, the bit stays down and nobody is alerted [M 0x163fe8:29-31].
#[test]
fn a_head_kill_leaves_no_alert_on_the_corpse() {
    let (bits, src, v) = corpse_run(true);
    assert_eq!(bits & 8, 0, "a head kill raises no bit [M 0x163fe8:29-31]");
    assert_ne!(src, Some(v), "nobody takes an alert from a head kill");
}

/// `DroneVision_FindAlertedDrones` returns at once on 07000004 and 0700001b [M 0x1767d0:26-31].
#[test]
fn guards_on_1b_do_not_alert_each_other_by_sight() {
    let mut w = walker::Walker::new(scene("0700001b.bin"));
    w.wait_enabled(6000, "level start");
    w.wait(120);
    let (s, o) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let live = |x: &Drone| x.present && x.enabled && !x.corpse && x.team == 1 && !x.alerted_by_drone;
        let o = sim.drones.list.iter().position(|x| live(x) && x.watching && x.alert_bits == 0).expect("a watching guard");
        let s = sim.drones.list.iter().enumerate().position(|(i, x)| i != o && !x.corpse && x.team == 1).expect("a second guard");
        (s, o)
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // Only one guard is out here, so the source is brought in by hand. Guards on this level have
        // sight range 0, leaving only the under-2-units rule [M 0x1760b0:54], hence 1.5 units on a
        // side with a clear line: this pins the level gate, not the geometry.
        let op = sim.drones.list[o].pos;
        let spot = {
            let d = sim.data();
            let from = sim.drones.list[o].head();
            [[1.5f32, 0.0], [-1.5, 0.0], [0.0, 1.5], [0.0, -1.5]].iter()
                .map(|f| [op[0] + f[0], op[1], op[2] + f[1]])
                .find(|p| d.world.line_of_sight(from, [p[0], p[1] + 0.6, p[2]], nf_game::world::LOS_GUARD_VISION))
                .expect("a clear side beside the observer")
        };
        let x = &mut sim.drones.list[s];
        x.pos = spot; x.velocity = [0.0; 3];
        x.present = true; x.enabled = true; x.state = state::IDLE; x.alert_bits = 0x10; x.attack_frame = 1;
        let y = &mut sim.drones.list[o];
        y.yaw = (spot[0] - op[0]).atan2(spot[2] - op[2]);
    }
    let watching_at_start = w.scene.sim.as_ref().unwrap().drones.list[o].watching;
    assert!(watching_at_start, "the observer can take an alert");
    for _ in 0..120u32 { w.tick(Input::default(), false); }
    assert_ne!(w.scene.sim.as_ref().unwrap().drones.list[o].assist_src, Some(s), "the pass never runs on 0700001b [M 0x1767d0:29-31]");
}

/// A first attack from the guard's own sight with prop 0x33: FirstSightState takes bit 1 and the
/// 0x200000 latch [M 0x147c48:59,90], so FirstAttack's bit-0x10 block is skipped [M 0x148010:85-88].
#[test]
fn an_own_sight_first_attack_with_the_shout_prop_raises_bit_1_not_0x10() {
    let s = scene("07000005.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a prop-0x33 team-1 guard", |x, d| x.team == 1 && x.prop(d, 0x33) && !x.prop(d, 0x1b) && x.weapon > 1);
    g.state = state::ATTACK; g.own_sight = true; g.sighted = true; g.dist = 10.0;
    g.send(&d, &mut Rng::new(7), 500, msg::ENTER);
    assert_ne!(g.alert_bits & 1, 0, "bit 1 [M 0x147c48:59]");
    assert_eq!(g.alert_bits & 0x10, 0, "no bit 0x10 behind the 0x200000 latch [M 0x148010:85-88]");
    assert!(g.scare_shouted, "the 0x200000 latch [M 0x147c48:90]");
}

/// Sister: alerted by another guard (no own sight), FirstSightState is skipped and bit 0x10 stands alone.
#[test]
fn a_forced_first_attack_raises_0x10_and_marks_the_guard_sighted() {
    let s = scene("07000005.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a prop-0x33 team-1 guard", |x, d| x.team == 1 && x.prop(d, 0x33) && x.weapon > 1);
    g.state = state::ATTACK; g.own_sight = false; g.dist = 10.0;
    g.send(&d, &mut Rng::new(7), 500, msg::ENTER);
    assert_eq!(g.alert_bits & 0x11, 0x10, "bit 0x10 alone [M 0x148010:85-88]");
    assert!(g.sighted, "FirstAttack sets 0x4000000 on every path [M 0x148010:65]");
}

/// 09/0A skip FirstAttack's sight widening on both paths [M 0x148010:42-46, 69-73]; 05 widens.
#[test]
fn tower_guards_keep_their_view_cone_after_the_first_attack() {
    let s = scene("07000009.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    for own in [true, false] {
        let mut g = drone(sim, "a tower guard", |x, _| x.team == 3);
        let (range, fov) = (g.sight_range, g.half_fov);
        assert_ne!(fov, 0.0, "the tower guard starts with a cone");
        g.state = state::ATTACK; g.own_sight = own; g.dist = 10.0;
        g.send(&d, &mut Rng::new(7), 500, msg::ENTER);
        assert_eq!((g.sight_range, g.half_fov), (range, fov), "own sight {own}: no widening on 09/0A");
    }
    let s5 = scene("07000005.bin");
    let sim5 = s5.sim.as_ref().unwrap();
    let d5 = sim5.data();
    let mut g = drone(sim5, "a 05 guard", |x, _| x.team == 1 && x.weapon > 1);
    g.state = state::ATTACK; g.dist = 10.0;
    g.send(&d5, &mut Rng::new(7), 500, msg::ENTER);
    assert_eq!(g.half_fov, 0.0, "elsewhere the first attack opens the cone [M 0x148010:83]");
}

/// The 09/0A block of `DroneFunc_HandleImpact` runs on the dart, the smoke and the flash as on the
/// tase [M 0x1469b8:220-246, 283-305, 331-357]: a sighted guard forgets and arms the alarm run.
#[test]
fn a_sighted_tower_guard_hit_by_a_dart_smoke_or_flash_resets_and_arms_the_alarm_run() {
    let s = scene("07000009.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    for m in [msg::STUN_DART, msg::SMOKE, msg::FLASH] {
        let mut g = drone(sim, "a tower guard", |x, _| x.team == 3);
        for p in [0x1c, 0x34, 0x36] { d.props.set(p, &mut g.props, 0); }
        g.state = state::CIVILIAN_PATROL;
        g.sighted = true; g.first_attack_done = true; g.alert_bits = 0x10;
        g.send(&d, &mut Rng::new(7), 500, m);
        assert!(!g.sighted, "msg {m:#x}: the sighting clears [M 0x1469b8:121]");
        assert!(!g.first_attack_done, "msg {m:#x}: the first-attack latch clears [M 0x1469b8:109]");
        assert_eq!(g.alert_bits, 0, "msg {m:#x}: +0x4FC = 0 [M 0x1469b8:112]");
        assert_eq!(g.alt_state, 0x89, "msg {m:#x}: +0x5A4 = RunToAlarm [M 0x1469b8:110]");
        assert_eq!(g.impact_resume, state::RETURN_TO_PATROL_PATH, "msg {m:#x}: resume 0x15 [M 0x1469b8:127]");
    }
}

/// `NDrone2_IsCivilian` guards rise to `+0x118`, others to Attack [M 0x163350:18-23].
#[test]
fn a_flashed_tower_guard_gets_up_to_his_resume_state() {
    let s = scene("07000009.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a tower guard", |x, _| x.team == 3);
    g.state = state::STUN_GRENADE_RECOVER;
    g.impact_resume = state::CIVILIAN_PATROL;
    g.send(&d, &mut Rng::new(7), 500, msg::ENTER);
    assert_eq!(g.anim.call.map(|c| c.1), Some(state::CIVILIAN_PATROL), "the get-up's follow-on is +0x118 [M 0x163350:21]");
}

/// The tase recovery has the same split, and voice 0x10 turns team 1 first [M 0x1630b8:16-27].
#[test]
fn a_tased_tower_guard_gets_up_to_his_resume_state_and_a_voice_0x10_drone_turns_hostile() {
    let s = scene("07000009.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a tower guard", |x, _| x.team == 3);
    g.state = state::STUNNED_RECOVER;
    g.impact_resume = state::CIVILIAN_PATROL;
    g.send(&d, &mut Rng::new(7), 500, msg::ENTER);
    assert_eq!(g.anim.call.map(|c| c.1), Some(state::CIVILIAN_PATROL), "the follow-on is +0x118 [M 0x1630b8:25]");
    let mut g = drone(sim, "a tower guard", |x, _| x.team == 3);
    g.voice_type = 0x10;
    g.state = state::STUNNED_RECOVER;
    g.impact_resume = state::CIVILIAN_PATROL;
    g.send(&d, &mut Rng::new(7), 500, msg::ENTER);
    assert_eq!(g.team, 1, "voice 0x10 joins team 1 [M 0x1630b8:16-18]");
    assert_eq!(g.anim.call.map(|c| c.1), Some(state::ATTACK), "and rises to Attack [M 0x1630b8:21-22]");
}

/// `EnemyLookForOpponent`: team 3 sees on 09/0A only [M 0x177ed0:29-38].
#[test]
fn a_tower_guard_outside_his_civilian_states_can_attack_on_sight() {
    let s = scene("07000009.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a tower guard", |x, d| x.team == 3 && x.prop(d, 0x3c));
    g.state = state::ALERT; g.seen = true; g.alert = 0.9; g.sighted = false; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::UPDATE);
    assert!(g.own_sight, "the sighting landed [M 0x177ed0:76-80]; log {:?}", g.log);
}

/// The same read refuses a mode 7/0xE guard without prop 0x57 in state 99 [M 0x177ed0:56-69].
#[test]
fn a_run_to_point_guard_without_prop_0x57_keeps_going_past_bond() {
    let s = scene("07000005.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a mode-7 guard", |x, d| x.mode == 7 && !x.prop(d, 0x57) && x.prop(d, 0x3c));
    g.state = state::GO_TO_GOAL; g.seen = true; g.alert = 0.9; g.sighted = false; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::UPDATE);
    assert!(!went_to(&g, state::ATTACK) && !g.sighted, "state 99 refuses the sighting [M 0x177ed0:60-69]; log {:?}", g.log);
}

/// ... and any guard while `+0x4FC & 0x20` stands [M 0x177ed0:55].
#[test]
fn a_guard_holding_the_interrogate_bit_does_not_attack_on_sight() {
    let s = scene("07000005.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a team-1 guard", |x, d| x.team == 1 && x.prop(d, 0x3c) && x.prop(d, 0x51));
    g.state = state::ALERT; g.seen = true; g.alert = 0.9; g.sighted = false; g.alert_bits = 0x20; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::UPDATE);
    assert!(!went_to(&g, state::ATTACK) && !g.sighted, "bit 0x20 refuses [M 0x177ed0:55]; log {:?}", g.log);
}

/// ... and a guard that owns a cover [M 0x177ed0:52-54; the bit is `cover_claimed`, 0x155a38].
#[test]
fn a_guard_holding_a_cover_does_not_attack_on_sight() {
    let s = scene("07000005.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a team-1 guard", |x, d| x.team == 1 && x.prop(d, 0x3c) && x.prop(d, 0x51));
    g.state = state::ALERT; g.seen = true; g.alert = 0.9; g.sighted = false; g.cover_claimed = true; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::UPDATE);
    assert!(!went_to(&g, state::ATTACK) && !g.sighted, "bit 0x800 refuses [M 0x177ed0:52-54]; log {:?}", g.log);
}

/// `NDrone2_DefaultInit`: mode 0xD gains props 0x3C/0x36/0x35 [M 0x14b300:745-748], and 0x36 is the
/// smoke proofing `DroneFunc_HandleImpact` reads [M 0x1469b8:198-201]. Modes 0xC and 0x11 lose the
/// explosive-shy props [M 0x14b300:782-797]. Every level that has these modes.
#[test]
fn per_mode_props_match_default_init_on_every_level_that_has_them() {
    let mut smoke_proof = 0;
    for level in ["07000004.bin", "07000016.bin", "07000001.bin", "07000002.bin", "07000008.bin"] {
        let s = scene(level);
        let sim = s.sim.as_ref().unwrap();
        let d = sim.data();
        for x in &sim.drones.list {
            match x.mode {
                0xd => {
                    for p in [0x3c, 0x36, 0x35] { assert!(x.prop(&d, p), "{level} drone {} prop {p:#x}", x.index); }
                    smoke_proof += 1;
                }
                0xc => for p in [0x34, 0x36, 0x3b, 0x3a] { assert!(!x.prop(&d, p), "{level} drone {} prop {p:#x}", x.index); },
                0x11 => for p in [0x3b, 0x3a] { assert!(!x.prop(&d, p), "{level} drone {} prop {p:#x}", x.index); },
                _ => {}
            }
        }
    }
    assert!(smoke_proof >= 3, "the mode-0xD drones of 04 and 16 were checked, got {smoke_proof}");
}

/// `DroneVision_AlertSound`'s rows: modes 9/0x12/0x13/0x16 never investigate; above 0.7 they attack
/// [M 0x177800:60-79].
#[test]
fn a_mode_9_guard_attacks_a_loud_noise_above_0_7_and_never_investigates() {
    let s = scene("07000009.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let base = drone(sim, "a tower guard", |x, d| x.mode == 9 && x.prop(d, 0x1f));
    let mut g = base.clone();
    g.state = state::CIVILIAN_PATROL; g.alert = 0.5; g.sound_share = 0.5; g.dist = 10.0; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::SOUND);
    assert!(!went_to(&g, state::HEARD_NOISE), "mode 9 has no HeardNoise row [M 0x177800:68-72]; log {:?}", g.log);
    let mut g = base.clone();
    g.state = state::CIVILIAN_PATROL; g.alert = 0.9; g.sound_share = 0.5; g.dist = 10.0; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::SOUND);
    assert!(went_to(&g, state::ATTACK), "above 0.7 mode 9 attacks [M 0x177800:70-78]; log {:?}", g.log);
    assert!(!went_to(&g, state::HEARD_NOISE), "and does not investigate; log {:?}", g.log);
}

/// 0C/0D voice sets 5/6: any heard sound is an attack, with no 50-unit gate [M 0x177800:32-46].
#[test]
fn a_voice_6_guard_on_0c_attacks_any_sound_at_any_distance() {
    let s = scene("0700000c.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a voice-6 guard", |x, d| x.voice_type == 6 && x.prop(d, 0x1f) && x.team == 1);
    g.state = state::IDLE; g.alert = 0.0; g.sound_share = 0.05; g.dist = 80.0; g.sighted = false; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::SOUND);
    assert!(went_to(&g, state::ATTACK), "voice 5/6 on 0C/0D attacks [M 0x177800:75-78]; log {:?}", g.log);
}

/// 07000014 state 0x1B ignores sound [M 0x177800:34-36].
#[test]
fn an_enemy_mission_guard_on_14_ignores_sound() {
    let s = scene("07000014.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a hearing guard", |x, d| x.team == 1 && x.prop(d, 0x1f));
    g.state = state::ENEMY_MISSION; g.alert = 0.9; g.sound_share = 0.5; g.dist = 10.0; g.sighted = false; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::SOUND);
    assert!(!went_to(&g, state::HEARD_NOISE), "state 0x1B on 14 is deaf [M 0x177800:34-36]; log {:?}", g.log);
}

/// The alarm: modes 9 and 0x12 ignore it, every other mode attacks, with no hearing gates
/// [M 0x177c18:107-118].
#[test]
fn the_alarm_rows_follow_the_mode() {
    let s = scene("07000009.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a tower guard", |x, _| x.mode == 9);
    g.state = state::CIVILIAN_PATROL; g.alert = 0.9; g.dist = 10.0; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::ALARM);
    assert!(g.log.is_empty(), "mode 9 drops the alarm [M 0x177c18:110]; log {:?}", g.log);
    let s7 = scene("07000007.bin");
    let sim7 = s7.sim.as_ref().unwrap();
    let d7 = sim7.data();
    let mut g = drone(sim7, "a mode-0x16 walker", |x, _| x.mode == 0x16);
    g.state = state::CIVILIAN_MISSION; g.alert = 0.0; g.dist = 80.0; g.log.clear();
    g.send(&d7, &mut Rng::new(7), 500, msg::ALARM);
    assert!(went_to(&g, state::ATTACK) || g.first_attack_done, "other modes attack, far or not [M 0x177c18:116-118]; log {:?}", g.log);
}

/// HostageHide gives way to HostageIdle with no armed drone within 20 [M 0x15cfe0:23-29].
#[test]
fn a_rescued_hostage_stops_cowering_when_no_armed_guard_is_near() {
    let mut w = walker::Walker::new(scene("07000003.bin"));
    w.wait_enabled(6000, "level start");
    w.wait(60);
    let h = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|x| x.present && x.mode == 5).expect("a hostage");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for (i, x) in sim.drones.list.iter_mut().enumerate() {
            if i != h && x.team == 1 { x.pos[1] += 500.0; x.velocity = [0.0; 3]; }
        }
        let x = &mut sim.drones.list[h];
        x.enabled = true;
        x.set_state(state::HOSTAGE_HIDE);
    }
    let mut calm = false;
    for _ in 0..30u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.list[h].state == state::HOSTAGE_IDLE { calm = true; break; }
    }
    assert!(calm, "nobody armed within 20: Hide gives way to Idle [M 0x15cfe0:23-29]");
}

/// HostageSaved: an alternate state and no second channel stops him where he stands, Idle with
/// nobody armed near [M 0x15ccf0:23-34] — he does not run to a point.
#[test]
fn a_saved_hostage_with_an_alternate_state_stays_put() {
    let s = scene("07000003.bin");
    let sim = s.sim.as_ref().unwrap();
    let d = sim.data();
    let mut g = drone(sim, "a hostage", |x, _| x.present && x.mode == 5);
    g.state = state::HOSTAGE_SAVED; g.alt_state = 0x12; g.second_channel = 0; g.dist = 50.0; g.log.clear();
    g.send(&d, &mut Rng::new(7), 500, msg::ENTER);
    assert!(went_to(&g, state::HOSTAGE_IDLE), "the alternate-state arm [M 0x15ccf0:23-34]; log {:?}", g.log);
}
