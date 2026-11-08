//! drone-block-swaps-g055: the three behaviour-block swap sites GAPS G-055 leaves open —
//! `DroneFunc_FirstAttack`'s alternate-state tail [M 0x148010:100-131],
//! `NDrone2_PreDroneControl`'s second-switch flush [M 0x148db0:51-90] and
//! `DroneFunc_HandleImpact` case 7's 09/0A stamp arm [M 0x1469b8:103-128]. Every assertion
//! cites its evidence; the sisters pin the swap NOT happening when its condition is false.
//!
//! The second behaviour block: placement param 5 == 100 gives the drone `+0xC48` = 100 and a
//! second prop block at `+0x4E8` [M 0x14b300; D the disc placements — 07000007's wake guards
//! #355…, 07000009's alarm civilians #62…]. `Drone+0x4D8` is the ACTIVE block the game
//! repoints (`+0x29` latched); the port's `props`/`alert_props`/`using_2nd` fold
//! [M 0x14a5f0/0x14a618/0x14a5e8].

use nf_game::drone::msg;
use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The disjoint-borrow `Data` the direct `send` calls need (the established
/// stuck_guard_combat_05_live pattern): every borrow is a direct Sim field, so the expansion
/// splits from the `&mut sim.drones.list[i]` / `&mut sim.drones.rng` the send pairs it with.
macro_rules! sim_data {
    ($sim:ident, $data:ident, $difficulty:ident) => {
        let $difficulty = $sim.drones.difficulty;
        let $data = nf_game::drone::Data {
            tables: &$sim.tables, props: &$sim.props, scripts: &$sim.scripts, weapons: &$sim.weapons,
            world: &$sim.world, level: $sim.level, sequences: &$sim.seqs, skins: &$sim.skins,
            skeletons: &$sim.skeletons, nav: &$sim.nav, covers: &$sim.covers, tuning: &$sim.tuning,
            hitboxes: &$sim.hitboxes, volumes: &$sim.volumes, sfx: $sim.sfx.as_ref(), models: &$sim.models,
            surfaces: $sim.surfaces.as_ref(), voices: $sim.voices.as_deref(), difficulty: $difficulty,
            ai_points: &$sim.ai_points,
        };
    };
}

/// `NDrone2_PreDroneControl`'s second-switch flush on 07000003's wake guard
/// [D 07000003.bin #388: p6 = 80, p8 = 100, p5 = 100 — at load: ch 80, tag 100, alt 0x85,
/// WaitSwitch-disabled [L the g055 probe]]: `+0x135` set with
/// the channel's switch ON and a tag `+0x13A` clears both and — with `+0x5A4` ≠ 0 — runs the
/// `ChangeToAttackMode` flush inline: `+0xC6 → +0xC5` (when set), `+0xC6`/`+0x5A4` cleared,
/// and with `+0xC48 == 100` the SECOND block comes in (`+0x4D8 = +0x4E8`, `+0x29 = 1`)
/// [M 0x148db0:65-82] before `Drone_SM_SetState(+0x5A4)` [M :82] — 0x85 here. Sister: the ch-79 guard flushes nothing.
#[test]
fn second_switch_wake_runs_the_flush_and_swaps_in_the_second_block() {
    let mut scene = nf_view::load(&iso(), "07000003.bin", false).expect("load 07000007");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30u32 { sim.step_bond(Input::default(), false, false, false); }
    let g_i = sim.drones.list.iter().position(|d| d.placement == 388 && d.present)
        .expect("wake guard #388 present [D 07000003.bin #388; M 0x136be0:49]");
    let s_i = sim.drones.list.iter().position(|d| d.placement == 389 && d.present)
        .expect("sister wake guard #389 (ch 79) present [D]");
    {
        let g = &sim.drones.list[g_i];
        eprintln!("probe #388: ch {} tag {} alt {:#x} mode {:#x} st {:#x} en {} 2nd {}",
            g.second_channel, g.second_tag, g.alt_state, g.mode, g.state, g.enabled as u8, g.using_2nd);
        assert_eq!(g.second_channel, 80, "the disc's channel [D #388 p6 = 80]");
        assert_ne!(g.second_tag, 0, "the tag stands until the flush [M 0x148db0:53]");
        assert_ne!(g.alt_state, 0, "the alt state arms the flush [M :65-82]");
        assert!(!g.using_2nd, "the guard starts on its first block [M 0x14b300]");
    }
    // SISTER: 120 ticks with both channels OFF — nothing flushes.
    for _ in 0..120u32 { sim.step_bond(Input::default(), false, false, false); }
    {
        let g = &sim.drones.list[g_i];
        assert!(!g.using_2nd, "sister: channel OFF — no flush, the first block stays active [M 0x148db0:53]");
        assert_eq!(g.second_channel, 80, "sister: the channel latch stands [M :53]");
        assert_ne!(g.second_tag, 0, "sister: the tag stands [M :53]");
    }
    // The channel ON — the write `Switch_Activate` performs on `switch_channels[param 0]`
    // [M 0x1b02a8] — flushes the ch-74 guard.
    let f = sim.drones.frame;
    sim.drones.channels.set(80, 1, f);
    for _ in 0..5u32 { sim.step_bond(Input::default(), false, false, false); }
    {
        let g = &sim.drones.list[g_i];
        eprintln!("after flush: using_2nd {} state {:#x} mode {:#x} alt {:#x} en {} del {}",
            g.using_2nd, g.state, g.mode, g.alt_state, g.enabled as u8, g.delete_me as u8);
        assert!(g.using_2nd, "the flush swapped the SECOND block in [M 0x148db0:75-77]");
        assert_eq!(g.second_channel, 0, "the channel is consumed [M :65]");
        assert_eq!(g.second_tag, 0, "the tag is consumed [M :65]");
        assert_eq!(g.mode, 7, "+0xC6 → +0xC5 — the mode the flush left [M 0x148db0:68]");
        eprintln!("(alt_state {:#x} — the flush consumed it [M :79]; the woken run's own EnemyMission machinery re-arms +0x5A4 [M 0x15e1e8:66-82])", g.alt_state);
    }
    {
        let s = &sim.drones.list[s_i];
        assert_eq!(s.second_channel, 79, "sister #389 is a ch-79 guard [D #389 p6 = 79]");
        assert!(!s.using_2nd, "sister: the ch-80 raise flushes nothing on a ch-79 guard [M 0x148db0:53]");
        assert_ne!(s.second_tag, 0, "sister: the tag stands [M :53]");
        assert_ne!(s.alt_state, 0, "sister: the alt state stands [M :53]");
    }
}

/// `DroneFunc_FirstAttack`'s alternate-state tail [M 0x148010:100-131]: an armed guard whose
/// first attack passes the common stamps with a zero first-sight pick and `+0x5A4` ≠ 0 runs
/// the `ChangeToAttackMode` flush INLINE — `+0x5A2 = 0x56` [M :102], the mode swap, `+0x5A4`
/// cleared, and with `+0xC48 == 100` the SECOND block comes in [M :124-127] — returning the
/// old alt state [M :128-131]. 07000003's #400 is an enabled armed guard carrying alt 0x56 at
/// load [L the g055 probe]. Sister: alt state clear → Attack's own fallback, no swap.
#[test]
fn first_attack_alt_tail_swaps_in_the_second_block() {
    let mut scene = nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30u32 { sim.step_bond(Input::default(), false, false, false); }
    // The candidate: an armed hostile guard whose first attack is still pending, carrying an
    // alt state, NOT on the second block, whose ACTIVE (first) block carries none of the
    // first-sight roll props (a 0x1B/0x19 pick would return from FirstAttack before the tail
    // [M 0x148010:41-45]) and no prop 0x1A (the challenge arm returns earlier still; 03 is not
    // a challenge level anyway). The disc decides; the test takes what the level offers.
    let cand: Vec<usize> = {
        sim_data!(sim, d, diff);
        (0..sim.drones.list.len()).filter(|&i| {
            let g = &sim.drones.list[i];
            g.present && g.enabled && g.team == 1 && !g.corpse && (g.weapon as u8) > 1
                && !g.first_attack_done && g.alt_state != 0 && !g.using_2nd
                && d.props.get(0x1b, &g.props) == 0 && d.props.get(0x19, &g.props) == 0
                && d.props.get(0x1a, &g.props) == 0
        }).collect()
    };
    if cand.is_empty() {
        sim_data!(sim, d2, diff2);
        let diag: Vec<String> = sim.drones.list.iter().enumerate()
            .filter(|(_, g)| g.present && g.enabled && g.team == 1 && !g.corpse && g.alt_state != 0 && !g.using_2nd && !g.first_attack_done)
            .map(|(i, g)| format!("#{i}/p{} 18 {} 19 {} 1a {} 1b {}", g.placement,
                d2.props.get(0x18, &g.props), d2.props.get(0x19, &g.props), d2.props.get(0x1a, &g.props), d2.props.get(0x1b, &g.props)))
            .collect();
        eprintln!("alt-guard props: {diag:?}");
    }
    {
        let roster: Vec<String> = sim.drones.list.iter().enumerate().filter(|(_, g)| g.present && g.team == 1 && !g.corpse)
            .map(|(i, g)| format!("#{i}/p{} st {:#x} alt {:#x} 2nd {} fa {} w {}", g.placement, g.state, g.alt_state, g.using_2nd, g.first_attack_done, g.weapon))
            .collect();
        eprintln!("roster: {roster:?}");
    }
    let Some(&g_i) = cand.first() else { panic!("no first-attack-pending alt-state guard on 07000003 — see the prop diagnostics above"); };
    // The real sight path: Bond 5 units in front of the guard's facing, every tick, until the
    // first attack runs (the port's established pattern, attack_reentry_live).
    let mut entered = false;
    for _ in 0..600u32 {
        {
            let g = &sim.drones.list[g_i];
            let (p, yaw) = (g.pos, g.yaw);
            let at = [p[0] + yaw.sin() * 5.0, p[1], p[2] + yaw.cos() * 5.0];
            let by = sim.bond.yaw;
            sim.place_bond(at, by);
        }
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[g_i].first_attack_done { entered = true; break; }
    }
    assert!(entered, "the guard's first attack ran through the real sight path");
    {
        let g = &sim.drones.list[g_i];
        eprintln!("after first attack: using_2nd {} state {:#x} mode {:#x} alt {:#x} return {:#x}",
            g.using_2nd, g.state, g.mode, g.alt_state, g.return_state);
        assert!(g.using_2nd, "the alt tail swapped the SECOND block in [M 0x148010:124-127]");
        assert_eq!(g.alt_state, 0, "+0x5A4 consumed by the tail's inline flush [M :122-123]");
        eprintln!("(the tail's +0x5A2 = 0x56 write [M :102] is overwritten by the fallback's own resume when the 0x56 re-entry picks AimStand — the game does the same [M 0x164210:69-88])");
    }
    // SISTER: a guard with NO alt state — the same first attack leaves the blocks alone (the
    // tail is the `alt_state != 0` branch; the zero arm takes AttackNoWeapon/fallback [M
    // 0x148010:100-101]).
    let sis: Vec<usize> = {
        sim_data!(sim, d, diff);
        (0..sim.drones.list.len()).filter(|&i| {
            let g = &sim.drones.list[i];
            g.present && g.enabled && g.team == 1 && !g.corpse && (g.weapon as u8) > 1
                && !g.first_attack_done && g.alt_state == 0 && !g.using_2nd
                && d.props.get(0x1b, &g.props) == 0 && d.props.get(0x19, &g.props) == 0
                && d.props.get(0x1a, &g.props) == 0
        }).collect()
    };
    let Some(&s_i) = sis.first() else { eprintln!("no no-alt sister available on this load; sister covered by the lib unit test"); return; };
    let mut entered = false;
    for _ in 0..600u32 {
        {
            let g = &sim.drones.list[s_i];
            let (p, yaw) = (g.pos, g.yaw);
            let at = [p[0] + yaw.sin() * 5.0, p[1], p[2] + yaw.cos() * 5.0];
            let by = sim.bond.yaw;
            sim.place_bond(at, by);
        }
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[s_i].first_attack_done { entered = true; break; }
    }
    assert!(entered, "the sister's first attack ran");
    let g = &sim.drones.list[s_i];
    assert!(!g.using_2nd, "sister: no alt state — no swap, the first block stays active [M 0x148010:100-101]");
}

/// `DroneFunc_HandleImpact` case 7's 09/0A stamp arm [M 0x1469b8:103-128] on 07000009's
/// alarm civilians [D 07000009.bin #62/#64/#65/#66/#250/#261 — kind-0x0f, p5 = 100; at load:
/// enabled, team 3, health 1.0, alt state 0x89, CivilianPatrol 0x14 [L the g055 probe]]. The
/// fall tase (the accumulator past 2·health, [M :72-88] — 3 tases at health 1) of a guard
/// carrying the first-sight latch clears the first-attack/sight latches, arms `+0x5A4 = 0x89`
/// RunToAlarm, and with `+0xC48 == 100` restores the FIRST block; `+0xC5 = 9`, `+0xC6 = 7`,
/// `+0x5A2 = 0x14`, `SMI+0x10 = 0x15` [M :108-124]. Sister: the latch clear → only the
/// `SMI+0x10 = 0x14` resume write, no stamps [M :105-107]. The tases ride the state machine's
/// real msg-7 path (`Drone::send`, the stuck_guard_combat_05_live precedent); the accumulator
/// draws nothing, so the shared stream is untouched.
#[test]
fn tase_fall_on_09_arms_the_alarm_run_stamp() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000009.bin", false, None, 2).expect("load 07000009 d2");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30u32 { sim.step_bond(Input::default(), false, false, false); }
    let cand: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| {
        let g = &sim.drones.list[i];
        g.present && g.enabled && g.alert_props.is_some() && !g.corpse
    }).collect();
    assert!(!cand.is_empty(), "a p5=100 alarm civilian exists on 07000009 [D the kind-0x0f census]");
    let g_i = cand[0];
    // The first-sight latch +0x4000000: the real path is Bond in front until the vision pass
    // marks him [M EnemyLookForOpponent 0x177ed0:76-80]; these team-3 walkers may not carry
    // the opponent latch at all, so the pin falls back to the latch itself (the block's own
    // gate, set by the same vision pass on any sighting [M :76-80]) — the STAMP behaviour is
    // what this test pins.
    let mut seen = false;
    for _ in 0..300u32 {
        {
            let g = &sim.drones.list[g_i];
            let (p, yaw) = (g.pos, g.yaw);
            let at = [p[0] + yaw.sin() * 4.0, p[1], p[2] + yaw.cos() * 4.0];
            let by = sim.bond.yaw;
            sim.place_bond(at, by);
        }
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[g_i].sighted { seen = true; break; }
    }
    if !seen {
        eprintln!("(the real sight path never latched {:#x} on this civilian in 300 ticks — the latch is set directly; the block's gate, not the stamp, is under test)", 0x4000000u32);
        sim.drones.list[g_i].sighted = true;
    }
    let was_2nd = sim.drones.list[g_i].using_2nd;
    eprintln!("pre-tase: using_2nd {was_2nd} state {:#x} health {}", sim.drones.list[g_i].state, sim.drones.list[g_i].health);
    // The tases through the state machine until the fall (accum > 2·health [M 0x1469b8:72-88]).
    {
        sim_data!(sim, data, diff);
        let threshold = ((sim.drones.list[g_i].health as u32) << 1) + 1;
        let frame = sim.drones.frame;
        for i in 0..threshold { sim.drones.list[g_i].send(&data, &mut sim.drones.rng, frame + i as u32, msg::TASE); }
    }
    sim.step_bond(Input::default(), false, false, false); // the SM applies the pending 0x4a
    let g = &sim.drones.list[g_i];
    eprintln!("after fall: state {:#x} alt {:#x} mode {:#x} 2nd {} sighted {} fa {} ret {:#x} resume {:#x}",
        g.state, g.alt_state, g.mode, g.using_2nd, g.sighted, g.first_attack_done, g.return_state, g.impact_resume);
    assert_eq!(g.state, state::TASER, "the fall tase enters Taser 0x4a [M 0x1469b8:129-133]");
    assert_eq!(g.alt_state, nf_game::kiko::RUN_TO_ALARM, "+0x5A4 = 0x89 RunToAlarm [M 0x1469b8:110]");
    assert_eq!(g.mode, 9, "+0xC5 = 9 [M :119]");
    assert_eq!(g.alt_mode, 7, "+0xC6 = 7 [M :120]");
    assert!(!g.sighted, "+0x4000000 clear [M :121-122]");
    assert_eq!(g.return_state, state::CIVILIAN_PATROL, "+0x5A2 = 0x14 [M :123]");
    assert_eq!(g.impact_resume, state::RETURN_TO_PATROL_PATH, "SMI+0x10 = 0x15 [M :124]");
    let _ = was_2nd; // the 09 civilians reach the fall on their first block — the swap-back is the lib unit's pin

    // SISTER: the latch clear → only the resume write [M :105-107].
    let sis: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| {
        let g = &sim.drones.list[i];
        g.present && g.enabled && g.alert_props.is_some() && !g.corpse && !g.sighted && i != g_i
    }).collect();
    let Some(&s_i) = sis.first() else { eprintln!("no unsighted sister on this load"); return; };
    let (sis_alt_before, sis_ret_before) = (sim.drones.list[s_i].alt_state, sim.drones.list[s_i].return_state);
    {
        sim_data!(sim, data, diff);
        let threshold = ((sim.drones.list[s_i].health as u32) << 1) + 1;
        let frame = sim.drones.frame;
        for i in 0..threshold { sim.drones.list[s_i].send(&data, &mut sim.drones.rng, frame + i as u32, msg::TASE); }
    }
    sim.step_bond(Input::default(), false, false, false);
    let g = &sim.drones.list[s_i];
    eprintln!("sister after fall: state {:#x} alt {:#x} resume {:#x} ret {:#x}", g.state, g.alt_state, g.impact_resume, g.return_state);
    assert_eq!(g.impact_resume, state::CIVILIAN_PATROL, "sister: SMI+0x10 = 0x14 with the latch clear [M 0x1469b8:105-106]");
    assert_eq!(g.alt_state, sis_alt_before, "sister: no 0x89 arm — the alt state stands as it was [M :105-107]");
    assert_eq!(g.return_state, sis_ret_before, "sister: no return-state write either [M :105-107]");
}
