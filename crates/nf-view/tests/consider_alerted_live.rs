//! consider-alerted-exact (915): `DroneVision_ConsiderAlerted` 0x176250 on the real path —
//! the 07 doorway interrogation (856's interogate_07_live setup) latching the party, the
//! guest copy + forced attack, the 07 forced-attack refusal keeping the latch/copy, and the
//! 05 one-second gate. Evidence: docs/systems/drone-alerts-consider.md ([M] decompile +
//! disassembly 0x176250).

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// 856's doorway guard (interogate_07_live): skin 0x13 at (110.2, 44.1, -60.8) [D placement].
fn guard07(sim: &nf_view::scene::Sim) -> usize {
    sim.drones.list.iter().position(|d| d.skin == 0x0500_0013 && (d.pos[0] - 110.16).abs() < 1.0 && (d.pos[2] + 60.78).abs() < 1.0)
        .expect("the prop-0x1A guard (skin 0x13 at (110.2, 44.1, -60.8))")
}

/// An enabled, watching, bits-clear drone (a party guest or guard) other than `skip`.
fn watcher(sim: &nf_view::scene::Sim, skip: usize) -> usize {
    sim.drones.list.iter().enumerate().position(|(i, d)|
        i != skip && d.present && d.enabled && !d.corpse && d.watching && d.alert_bits == 0 && !d.alerted_by_drone)
        .expect("an enabled watching drone with clear alert bits")
}

/// The 07 doorway flow: Bond walks through the touch, the woken guard's first sight takes
/// Interogate 0x3c [M 0x147c48:92-100], `BeginInterogate` latches the party [M 0x1782a8], the
/// beats run 60 → (anim 0x0A + line 0x51) → 180 → Attack [M 0x161a40 case 0xc] — the ATTACK
/// entry stamps the attack bit 0x10 [M 0x148010:88] and +0x524 [M 0x148010:41]. Returns with the
/// guard out of Interogate and attacking.
fn interrogate_to_attack() -> (walker::Walker, usize) {
    let mut w = walker::Walker::new(nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007"));
    w.wait_enabled(6000, "level start");
    w.wait(60);
    let g = guard07(w.scene.sim.as_ref().unwrap());
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert!(!sim.drones.list[g].enabled, "sister: the guard sleeps until ch 0x93 [D touch #455]");
        sim.bond = nf_game::player::Bond::new([105.3, 44.2, -55.0], std::f32::consts::PI);
    }
    let mut entered = false;
    for t in 0..800u32 {
        let forward = if !entered && t < 400 { 0.6 } else { 0.0 };
        w.tick(Input { forward, ..Default::default() }, false);
        let st = w.scene.sim.as_ref().unwrap().drones.list[g].state;
        if !entered && st == state::INTEROGATE { entered = true; }
        if entered && st != state::INTEROGATE { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[g];
        assert!(entered, "the woken guard's first sight takes Interogate 0x3c [M 0x147c48:92-100]");
        assert!(sim.drones.party_loud, "BeginInterogate latched the party (DAT_0029aa86) [M 0x1782a8]");
        assert_ne!(d.alert_bits & 0x10, 0, "the attack commit stamped the attack bit [M 0x148010:88]");
        assert_ne!(d.attack_frame, 0, "+0x524 stamped [M 0x148010:41]");
    }
    (w, g)
}

/// The headline: a party guest parked beside the interrogating-turned-attacking guard is alerted
/// through the widened cone (party_loud: +0xE8 = 20.0, +0xE4 = π [M 0x1764d0-0x17650c]) — the
/// copy (`+0x4FC |= 0x10 << 16` [M 0x176250:202]), the assist pointer (+0x528 [M :206]) and the
/// forced attack — REFUSED: on 07000007 with the party latched, DoForcedAttack's gate lets only modes 0x13/0x16 through [M 0x148c60:13-15] (daily audit 2026-10-22: the old pin had the latch test inverted).
#[test]
fn the_07_latched_party_guest_copies_the_guard_but_the_gate_refuses_the_attack() {
    let (mut w, g) = interrogate_to_attack();
    let o = watcher(w.scene.sim.as_ref().unwrap(), g);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let gp = sim.drones.list[g].pos;
        let o2 = &mut sim.drones.list[o];
        o2.pos = [gp[0] + 3.0, gp[1], gp[2] + 0.5];
        o2.yaw = (gp[0] - o2.pos[0]).atan2(gp[2] - o2.pos[2]);
        o2.velocity = [0.0; 3];
    }
    let mut hit = None;
    for _ in 0..60u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[o];
        if d.alerted_by_drone && d.assist_src == Some(g) { hit = Some(d.alert_bits >> 16); break; }
    }
    let high = hit.expect("the guest took the drone alert once the guard attacked");
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[o];
    assert_eq!(high & 0x10, 0x10, "the copied half holds 0x10 << 16 [M 0x176250:202]");
    // The forced attack is REFUSED: the level is 07000007, the latch DAT_0029aa86 is set and the
    // guest's mode is neither 0x13 nor 0x16 [M 0x148c60:13-15] — she holds the party.
    assert_eq!(d.state, state::PARTY_GIRL, "the latched 07 gate refuses the guest's forced attack [M 0x148c60:13-15], got {:#x}", d.state);
}

/// Sister: with the party NOT latched, the 07 gate lets the forced attack through [M 0x148c60:13-15] — and the latch,
/// the copy and the assist pointer are already in place (ConsiderAlerted's effect runs before
/// its message [M 0x176250:199-208], and DoForcedAttack's refusal touches none of them [M
/// 0x148c60:14-47]).
#[test]
fn the_07_calm_party_lets_the_forced_attack_through_with_the_copy_and_assist() {
    let mut w = walker::Walker::new(nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007"));
    w.wait_enabled(6000, "level start");
    w.wait(120); // the level's first second is gone (the +0x524 gate on a 0 stamp [M 0x176250:91])
    let (s, o) = {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.party_loud, "no interrogation ran");
        // Every 07 drone is unarmed (the party [D placement/census]) — the ATTACK-entry stamp of
        // bit 0x10 [M 0x148010:88] is unreachable for them, so the sister pokes the source's
        // +0x4FC = 0x10 directly ([S] scenario setup — the pass itself and the 07 gate it feeds
        // are the pinned, [M]-evidenced behaviour).
        let s = sim.drones.list.iter().enumerate().position(|(i, d)|
            i != 0 && d.present && d.enabled && d.team == 1 && !d.corpse).expect("an enabled team-1 drone");
        (s, watcher(sim, s))
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert_ne!(sim.drones.list[o].mode, 0x13, "sister setup: the observer must not be a bouncer");
        assert_ne!(sim.drones.list[o].mode, 0x16, "sister setup: the observer must not be a CivilianMission walker");
        // The source carries the attack bits (see above); the observer parks 4 units away, aimed
        // at the source (inside her natural π/2 / 24 cone).
        sim.drones.list[s].alert_bits = 0x10; // [S] setup — see above
        let sp = sim.drones.list[s].pos;
        let o2 = &mut sim.drones.list[o];
        o2.pos = [sp[0] + 4.0, sp[1], sp[2]];
        o2.yaw = (sp[0] - o2.pos[0]).atan2(sp[2] - o2.pos[2]);
        o2.velocity = [0.0; 3];
    }
    w.tick(Input::default(), false); // the alert pass considers the pair
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[o];
    assert!(d.alerted_by_drone, "the observer latched (+0x4F8 & 0x20000) [M 0x176250:201]");
    assert_eq!(d.assist_src, Some(s), "+0x528 = the source [M 0x176250:206]");
    assert_ne!(d.alert_bits & 0x10_0000, 0, "the copy arrived [M 0x176250:202]");
    // The calm party (latch 0) passes the gate [M 0x148c60:13-15]: DoForcedAttack's SetState(0x56) on an
    // unarmed guest falls through `DroneFunc_AttackNoWeapon` 0x14af80 to CivilianScared 0x12 (→ hiding
    // 0x13) [M 0x164210:24-40] — she leaves the party.
    assert!(matches!(d.state, state::CIVILIAN_SCARED | state::CIVILIAN_HIDING | state::DRAW_WEAPON | state::ATTACK),
        "the calm 07 gate lets the forced attack through, got {:#x}", d.state);
}

/// The 05 sister: the one-second gate on the source's +0x524 ([M 0x176250:91 —
/// `frame < stamp + FRAME_RATE_DIV·60` refuses]) holds for the first 60 ticks after the attack
/// began and opens after it; the forced attack then proceeds for every mode (no 07 gate here).
#[test]
fn the_05_one_second_gate_holds_then_opens() {
    let mut w = walker::Walker::new(nf_view::load(&iso(), "07000005.bin", false).expect("load 07000005"));
    w.wait_enabled(6000, "level start");
    w.wait(120);
    let (s, o, o_state) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = sim.drones.list.iter().enumerate().position(|(i, d)|
            i != 0 && d.present && d.enabled && d.team == 1 && !d.corpse).expect("an enabled team-1 guard");
        let o = watcher(sim, s);
        (s, o, sim.drones.list[o].state)
    };
    let attack_frame = {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.list[s].set_state(state::ATTACK);
        let sp = sim.drones.list[s].pos;
        let o2 = &mut sim.drones.list[o];
        o2.pos = [sp[0] + 4.0, sp[1], sp[2]];
        o2.yaw = (sp[0] - o2.pos[0]).atan2(sp[2] - o2.pos[2]);
        o2.velocity = [0.0; 3];
        w.tick(Input::default(), false);
        let f = w.scene.sim.as_ref().unwrap().drones.list[s].attack_frame;
        assert_ne!(f, 0, "the ATTACK entry stamped +0x524 [M 0x148010:41]");
        f
    };
    // Inside the second: the gate refuses — no alert yet.
    for _ in 0..4 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.frame < attack_frame + 60, "still inside the gate window");
        assert!(!sim.drones.list[o].alerted_by_drone, "the 1-second gate refuses [M 0x176250:91]");
        assert_ne!(sim.drones.list[s].alert_bits & 0x10, 0, "the source's bits are up — only the gate holds");
    }
    // Past it: the alert lands and the forced attack proceeds (07000005 has no party gate).
    // The game's write order inside the pass frame decides WHAT survives it [M 0x176250:199-208 —
    // the effect (latch :201, copy :202/:207, assist :206) ends with Drone_Message(0x1f) :208;
    // 0x175a38 sends delay-0 → Drone_SM_RouteMsgDCV 0x171df8 calls the DSTATE func inline; Idle
    // 0x15bf18:129-130 case 0x1f → DoForcedAttack 0x148c60:24-28 — the latch is CONSUMED
    // (`& 0xfffdffff` :27), 0x24000000 stamped and SetState(0x56)]: the frame leaves the copy +
    // the assist pointer and a CLEARED latch. The cleared latch is the proof the forced attack
    // RAN — a refused one keeps the latch (the_07_latched_party_guest pins that half). (The old
    // pin `alerted_by_drone == true` here contradicted 0x148c60:27: it held on master only
    // because the port then ignored the consumption; fixed with the addresses, 2026-10-23.)
    let mut hit = false;
    for _ in 0..120u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[o];
        if !d.alerted_by_drone && d.assist_src == Some(s) && d.alert_bits & 0x10_0000 != 0 { hit = true; break; }
    }
    assert!(hit, "the gate opened and the alert landed (the latch consumed by the same-frame forced attack [M 0x148c60:27])");
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[o];
    // The forced attack ran on 05 for every mode [M 0x148c60:9 — the 07 arm is level-gated]:
    // the ATTACK entry stamped the attack bit [M 0x148010:88] and the state left the unaware
    // family for the attack chain (0x56 → DrawWeapon/combat — it chains within the tick).
    assert_ne!(d.alert_bits & 0x10, 0, "the forced attack's ATTACK entry stamped bit 0x10 [M 0x148010:88]");
    assert_ne!(d.state, o_state, "the observer left {} for the attack chain, got {:#x}", format!("{o_state:#x}"), d.state);
}

/// fix-915: on 0700000c/0d an observer whose voice set (+0xD8) is 5 or 6 takes a source's alert
/// with NO accept chain and NO sight test — the speech arm and its talking skip both branch to the
/// effect block [M-dis 0x1764a8 / 0x1764c8 → 0x176684-0x1766c0]. Setup [S]: the observer is parked
/// 60 units from the source (2500 < sq-dist — `DroneCanSeeDrone` 0x1760b0:45 would refuse); the
/// pass must still latch it. Sister: a non-5/6 observer at the same spot is NOT alerted (it goes
/// through the chain + sight and the distance refuses).
#[test]
fn the_0c_voice_5_6_observer_takes_the_alert_without_sight() {
    let mut w = walker::Walker::new(nf_view::load(&iso(), "0700000c.bin", false).expect("load 0700000c"));
    w.wait_enabled(6000, "level start");
    w.wait(120);
    let (s, o, other) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let ok = |d: &nf_game::drone::Drone| d.present && d.enabled && !d.corpse && d.watching && d.alert_bits == 0 && !d.alerted_by_drone && d.team != 2 && d.mode != 5;
        let s = sim.drones.list.iter().position(|d| d.present && d.enabled && d.team == 1 && !d.corpse).expect("an enabled team-1 guard");
        let o = sim.drones.list.iter().enumerate().position(|(i, d)| i != s && ok(d) && d.voice_type.wrapping_sub(5) < 2)
            .expect("a watching voice-set 5/6 drone on 0700000c");
        let other = sim.drones.list.iter().enumerate().position(|(i, d)| i != s && i != o && ok(d) && d.voice_type.wrapping_sub(5) >= 2);
        (s, o, other)
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.list[s].set_state(state::ATTACK);
        let sp = sim.drones.list[s].pos;
        for (k, i) in [Some(o), other].into_iter().flatten().enumerate() {
            let d = &mut sim.drones.list[i];
            d.pos = [sp[0] + 60.0, sp[1], sp[2] + 3.0 * k as f32];
            d.velocity = [0.0; 3];
        }
    }
    // The speech arm's effect block ends with Drone_Message(0x1f) [M-dis 0x1764a8/0x1764c8 →
    // 0x176684-0x1766c0] and DoForcedAttack consumes the latch in the same frame [M 0x148c60:27
    // `& 0xfffdffff`; the dispatch is inline — 0x175a38 delay-0 → 0x171df8 → the DSTATE case-0x1f
    // 0x15bf18:129-130]: what survives is the assist pointer and the << 16 copy, with the latch
    // cleared. (The old pin `alerted_by_drone == true` contradicted 0x148c60:27: it held on
    // master only because the port then ignored the consumption; fixed with the addresses,
    // 2026-10-23.)
    let mut hit = false;
    for _ in 0..200u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[o];
        if !d.alerted_by_drone && d.assist_src == Some(s) && d.alert_bits & 0x10_0000 != 0 { hit = true; break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[o];
    assert!(hit, "the voice-5/6 observer 60 units away took the alert without sight [M-dis 0x1764a8/0x1764c8 → 0x176684] (the latch consumed by the same-frame forced attack [M 0x148c60:27])");
    assert_eq!(d.assist_src, Some(s), "+0x528 = the source [M 0x1766c0 block]");
    assert_ne!(d.alert_bits & 0x10_0000, 0, "the << 16 copy of the source's attack bit");
    if let Some(x) = other {
        let x = &sim.drones.list[x];
        let far = { let (a, b) = (x.pos, sim.drones.list[s].pos); (a[0]-b[0]).powi(2) + (a[1]-b[1]).powi(2) + (a[2]-b[2]).powi(2) > 2500.0 };
        if far { assert!(!x.alerted_by_drone || x.assist_src != Some(s), "sister: a non-5/6 observer beyond 50 units is refused by DroneCanSeeDrone [M 0x1760b0:45]"); }
    }
}
