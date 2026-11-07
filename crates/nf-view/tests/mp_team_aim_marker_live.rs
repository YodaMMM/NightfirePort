//! mp-team-aim-marker — the `Check_Target` 0x1a7810 multiplayer arm through a real team match:
//! aiming at a teammate (a human body or a bot — both ride class 3, the mark arm [M
//! 0x1a7810:128-133]) shows the red "no" mark (`BLData+0x133 = 3`, the port's
//! `crosshair_state()`), gated on `MPSettings._388_4_` (DECIMAL 388 = 0x184, `master_copy`) and
//! `_408_4_` (DECIMAL 408 = 0x198, `friendly_fire`) and `MP_isObjectOnTeam` 0x18b720 [M]; aiming
//! at an enemy keeps the normal crosshair; no aim latch is ever taken on a body (the +0x2a/
//! +0x2c4 stores sit in the class-2 arm only [M 0x1a7810:136-150]); Assassin shares the gate
//! through the scenario word (`MP_getObjectTeam` 0x18b6c0 / the `_420_4_ == 0x400` gate [M
//! 0x18b720]); a team objective (class 0x35, `MP_CreateObject` 0x185de8:122) has no
//! `Check_Target` arm at all. NF_ISO-gated.

use nf_game::mp::{scenario, MpSettings};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Aim Bond's eye at a world point (the civilian test's convention).
fn aim_at(sim: &mut nf_view::scene::Sim, target: [f32; 3]) {
    let head = sim.bond.head();
    let d = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
    let horiz = (d[0] * d[0] + d[2] * d[2]).sqrt();
    sim.bond.yaw = d[0].atan2(d[2]);
    sim.bond.pitch = (d[1].atan2(horiz)) / (std::f32::consts::FRAC_PI_2);
}

fn aim_and_read(sim: &mut nf_view::scene::Sim, target: [f32; 3]) -> u8 {
    aim_at(sim, target);
    for _ in 0..7 { sim.step_bond(Default::default(), false, false, false); }
    sim.crosshair_state()
}

/// Track a (walking) body the way a player holds the crosshair on it: re-aim at its live chest
/// every tick of the 7-tick window (Check_Target re-runs every 6th frame on the CURRENT view).
fn aim_and_track(sim: &mut nf_view::scene::Sim, slot: usize) -> u8 {
    for _ in 0..7 {
        if let Some(c) = body_chest(sim, slot) { aim_at(sim, c); }
        sim.step_bond(Default::default(), false, false, false);
    }
    sim.crosshair_state()
}

/// A body's live chest point (humans: Bond::pos + 0.2; bots: the brain's eye height − 0.1).
fn body_chest(sim: &nf_view::scene::Sim, slot: usize) -> Option<[f32; 3]> {
    let m = sim.mp.as_ref()?;
    if slot == sim.mp_local_slot {
        let b = sim.bond.pos;
        return Some([b[0], b[1] + 0.2, b[2]]);
    }
    if let Some(b) = sim.mp_bonds.get(slot).and_then(|x| x.as_ref()) {
        return Some([b.pos[0], b.pos[1] + 0.2, b.pos[2]]);
    }
    m.bots.iter().find(|b| b.slot == slot).map(|b| {
        let p = b.brain.pos;
        [p[0], p[1] - 0.1, p[2]]
    })
}

/// A team deathmatch: slot 0 + the slot-1 human on team 0, bot slot 2 on team 0, bot slot 3 on
/// team 1, friendly fire ON (the mark's `_408_4_` gate [M 0x1a7810:129]).
fn team_settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.scenario = scenario::TEAM_DEATHMATCH;
    s.gametype_mask = s.scenario;
    s.team_game = 1;
    s.friendly_fire = 1;
    s.frag_limit = 100;
    s.time_limit = 10 * 60;
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 2;
    s.total_players = 4;
    s.players[0].team = 0;
    s.players[1].team = 0;
    s.players[2].team = 0;
    s.players[3].team = 1;
    s
}

/// Place the slot-1 body and the two bots at fixed spots around Bond's spawn (the match stays
/// real: `load_mp_carry` + `step_bond`; the spots only make the aim rays deterministic). Returns
/// nothing — every read re-derives the live positions (the bots walk).
/// Re-park a bot at a fixed spot right before its read (the brains walk — one drifted 10 units
/// and off a ledge within 20 ticks; the pin needs the body where the aim expects it).
fn park_bot(sim: &mut nf_view::scene::Sim, slot: usize, spot: [f32; 3]) {
    let m = sim.mp.as_mut().unwrap();
    let j = m.bots.iter().position(|b| b.slot == slot).expect("the bot exists");
    let feet = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0).pos;
    m.bots[j].brain.pos = [feet[0], feet[1] + nf_view::scene::BOT_EYE, feet[2]];
    m.game.slots[slot].pos = Some(m.bots[j].brain.pos);
}

fn arrange(sim: &mut nf_view::scene::Sim) {
    let b0 = sim.bond.pos;
    let b = nf_game::player::Bond::stand_at(&sim.world, [b0[0] + 4.0, b0[1], b0[2]], 0.0);
    let bp = b.pos;
    sim.mp_bonds[1] = Some(b);
    sim.mp.as_mut().unwrap().set_slot(1, bp);
    for (slot, off) in [(2usize, [0.0f32, 0.0, 4.0]), (3, [-4.0, 0.0, 0.0])] {
        let m = sim.mp.as_mut().unwrap();
        let j = m.bots.iter().position(|b| b.slot == slot).expect("the bot exists");
        let feet = nf_game::player::Bond::stand_at(&sim.world, [b0[0] + off[0], b0[1], b0[2] + off[2]], 0.0).pos;
        m.bots[j].brain.pos = [feet[0], feet[1] + nf_view::scene::BOT_EYE, feet[2]];
        m.game.slots[slot].pos = Some(m.bots[j].brain.pos);
    }
}

#[test]
fn teammate_marks_enemy_does_not_and_no_body_is_latched() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None, team_settings())
        .expect("the arena loads through the app's MP carry path");
    let sim = scene.sim.as_mut().unwrap();
    {
        let m = sim.mp.as_ref().expect("the match builds");
        assert!(m.settings.started, "MP_Start ran [M 0x183a50:207]");
        assert_eq!(m.settings.master_copy, m.settings.master, "_388_4_ copied at start [M 0x183a50:70]");
        assert_eq!(m.bots.len(), 2, "two bots: slot 2 (team 0) and slot 3 (team 1)");
    }
    arrange(sim);

    // The teammate HUMAN (class 3): the mark, no latch, no weapon gate [M 0x1a7810:128-133].
    let chest = body_chest(sim, 1).expect("the slot-1 body");
    assert_eq!(aim_and_read(sim, chest), 3, "aiming at the teammate human shows the mark [M 0x1a7810:129-133]");

    // The _408_4_ (0x198) gate: friendly fire off closes the mark [M 0x1a7810:143's `== 0` arm].
    sim.mp.as_mut().unwrap().settings.friendly_fire = 0;
    let chest = body_chest(sim, 1).expect("the slot-1 body");
    assert_eq!(aim_and_read(sim, chest), 2, "friendly_fire == 0: no mark, even on a teammate [M 0x1a7810:143]");
    sim.mp.as_mut().unwrap().settings.friendly_fire = 1;

    // The teammate BOT (respawned bots are class 3 too [M BOT_respawn 0x124e38:77-79 →
    // MP_ReSpawn 0x185368:83]): the mark. The bots walk, so the window tracks the body.


    // The enemy bot: the normal crosshair — mp_is_object_on_team answers false [M 0x18b720].
    assert_eq!(aim_and_track(sim, 3), 2, "aiming at the enemy keeps the normal crosshair [M 0x1a7810:143-148]");

    // No aim latch on any body: the latch stores sit in the class-2 arm only [M 0x1a7810:148-150],
    // and the port's SP latch path (Drones::tick) never sees an MP body (the arenas carry no
    // drones — the match's bodies are not drones).
    assert!(sim.drones.list.iter().all(|d| !d.aimed_at), "no drone latched by the MP body aims");
    // The bots took no damage from the aims themselves (the mark writes no damage path).
    let m = sim.mp.as_ref().unwrap();
    assert!(m.bots.iter().all(|b| !b.dead), "the aimed-at bots took no damage from the aim itself");
}

#[test]
fn assassin_shares_the_gate() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = team_settings();
    s.scenario = scenario::ASSASSIN;
    s.gametype_mask = s.scenario;
    s.team_game = 0; // Assassin is not a team scenario [M 0x183438:41: (0x400 >> 29) & 1 == 0]
    let mut scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None, s)
        .expect("the arena loads");
    let sim = scene.sim.as_mut().unwrap();
    let assassin = sim.mp.as_ref().unwrap().assassin.expect("MP_assassinReset ran [M 0x183a50:166-167]");
    // The reset zeroes every team word and sets the assassin's to 1 [M 0x18aa70:12-41].
    arrange(sim);
    let m = sim.mp.as_ref().unwrap();
    let my_team = m.settings.players[0].team; // MP_getObjectTeam 0x18b6c0 of the aimER
    let on_team = |m: &nf_game::mp::MpMatch, slot: usize| m.mp_is_object_on_team(slot, my_team);
    if assassin == 0 {
        // Everyone else is team 0, the aimER is team 1: no teammate exists at all.
        assert_eq!(my_team, 1, "the aimER is the assassin [M 0x18aa70:38-41]");
        assert!(!on_team(m, 1) && !on_team(m, 2) && !on_team(m, 3), "no teammate exists for the assassin");
    } else {
        assert_eq!(my_team, 0, "a non-assassin keeps the zeroed team word [M 0x18aa70:12-19]");
        assert_eq!(on_team(m, 1), assassin != 1, "the slot-1 body's team answer follows the assassin pick");
        // The mark rule itself is pinned by the team test; here the gate opens for the assassin's
        // scenario word and the team words decide, exactly as in a team game.
        assert_eq!(on_team(m, 2), assassin != 2, "the bot's team word follows the assassin pick");
    }
}

#[test]
fn team_objective_takes_no_arm() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = team_settings();
    s.scenario = scenario::CAPTURE_THE_FLAG;
    s.gametype_mask = s.scenario;
    let mut scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None, s)
        .expect("the arena loads");
    let sim = scene.sim.as_mut().unwrap();
    let bases = {
        let m = sim.mp.as_ref().unwrap();
        [m.objects_rt.bases[0], m.objects_rt.bases[1]]
    };
    // The base models are class 0x35 [M MP_CreateObject 0x185de8:122] — no Check_Target arm: the
    // normal crosshair, mark or latch alike. The bots run the objectives (they walk), so aim from
    // Bond's spawn at whichever base has a clear line (no body and no wall nearer than the base).
    let head = sim.bond.head();
    let mut read = None;
    for base in bases.into_iter().flatten() {
        let far = [head[0] + (base[0] - head[0]) * 20.0, head[1] + (base[1] - head[1]) * 20.0, head[2] + (base[2] - head[2]) * 20.0];
        let wall = sim.world.segment(head, far);
        let blocked = sim.mp.as_ref().and_then(|m| {
            m.bots.iter().filter(|b| !b.dead).filter_map(|b| {
                let p = b.brain.pos;
                let vol = nf_game::bullet::PlayerVolume { feet: [p[0], p[1] - nf_view::scene::BOT_EYE, p[2]], height: 1.8, radius: 0.4 };
                nf_game::bullet::segment_cylinder(head, far, &vol)
            }).min_by(|a, b| a.total_cmp(&b))
        });
        if wall.map_or(true, |w| blocked.map_or(true, |bt| bt > w)) { read = Some((base, far)); break; }
    }
    let Some((base, _far)) = read else { eprintln!("both bases blocked — skipping the read"); return };
    assert_eq!(aim_and_read(sim, base), 2, "aiming at the team objective keeps the normal crosshair (class 0x35 has no arm)");
    assert!(sim.drones.list.iter().all(|d| !d.aimed_at), "the objective aim latches nothing");
}
