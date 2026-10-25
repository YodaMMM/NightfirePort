//! fix-guest-reactions — the party guests' reactions to an assault, and the death→restart flow
//! (07000007, driven through the REAL path: `nf_view::load` + `Sim::step_bond`).
//!
//! Symptoms on 07000007: punching a guest T-posed all the other guests; after a death the mission
//! replayed from the start in a loop; and punching a guard must be an automatic mission fail.
//!
//! The game's chains, ported here:
//! - `NDrone2_PunchImpact` 0x145f28:30 sets `Drone_AlertStatusSet(2)` on the victim; :45-56 on
//!   07000007 an UNARMED victim (`NDrone2_ObjectIsArmed` 0x178768) → `DroneFunc_SetMissionFailReason(drone, 5)`
//!   [M 0x13e560 → ch 99]; :55 health > 0 → `DroneFunc_SendHurtMessage` 0x145e28:19-57 — prop 0x32,
//!   the +8 once-per-life bit — **broadcasts msg 0x12** (`Drone_SM_BroadcastMsg`) with the victim as
//!   the alert source. The receivers' DSTATE case 0x12 → `DroneVision_EnemyAlerts` 0x177c18:41-51
//!   gates on prop 0x27 → `DroneVision_ShoutFromOtherDrone` → CivilianScared 0x12, whose ENTER
//!   (0x15d3a0) runs to CivilianHiding 0x13 with `CallAnim(0x19)` — the cower [M 0x15d520]. That
//!   cower is the animation the old build was missing (the T-pose).
//! - The death → the 0x40000042 fail/death menu → `P_ENDMISSION_Handler` 0x216618:19-21: Restart is
//!   a WARM reload of the CURRENT level on the ENTRY checkpoint (`Player_RamLoad(Bond, 1)`, the
//!   50.0 health floor [L 0x30cc34-3c]) — one restart, no replay loop (fix-restart-target's port).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;

use walker::{Walker, Wp};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Steer Bond (REAL inputs only) onto the guest's live front line at `dist`, and hold the
/// crosshair on its mid-body. The guests wobble their yaw in the stand-idle (the look-around),
/// so the front point is re-aimed from the guest's CURRENT yaw every tick — this is what makes
/// the sight pass's visibility cone and BondTalk's facing gate meet. Returns nothing; the caller
/// watches the guest each tick via the closure's walker.
fn hold_in_front_of(w: &mut Walker, guest: usize, dist: f32) {
    let sim = w.scene.sim.as_ref().unwrap();
    let g = sim.drones.list[guest].pos;
    let gyaw = sim.drones.list[guest].yaw;
    let front = [g[0] + gyaw.sin() * dist, g[1], g[2] + gyaw.cos() * dist];
    let b = sim.bond.pos;
    let eye = [b[0], b[1] + sim.bond.eye, b[2]];
    let desired = (g[0] - b[0]).atan2(g[2] - b[2]);
    let err = walker::wrap_pi(desired - sim.bond.yaw);
    let dy = (g[1] + 0.4) - eye[1]; // mid-body: the aimed-at cylinder spans feet-1.0..+1.8 [M 0x1a7810]
    let want_pitch = (dy / walker::dxz(g, b).max(0.5)).asin();
    let pitch_err = want_pitch - sim.bond.pitch;
    let to_front = (front[0] - b[0]).atan2(front[2] - b[2]);
    let front_err = walker::wrap_pi(to_front - sim.bond.yaw);
    let d = walker::dxz(front, b);
    let input = Input {
        turn: if d > 0.35 { front_err.clamp(-0.07, 0.07) } else { err.clamp(-0.05, 0.05) },
        look: pitch_err.clamp(-0.05, 0.05),
        forward: if d > 0.35 { 0.22 } else { 0.0 },
        ..Default::default()
    };
    w.tick(input, false);
}

/// The first ENABLED party guest (dtype 10), nearest to Bond's current spot. The idle state-1
/// guests sit disabled (`WaitSwitch` — `Drone_EnableAll(1)` skips them [M 0x1383e8:14]) and
/// bullets pass through disabled drones, so the punchable guests are the enabled 0x2f walkers.
fn pick_guest(sim: &nf_view::scene::Sim) -> (usize, [f32; 3]) {
    let bp = [sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]];
    let mut best: Option<(usize, f32)> = None;
    for (i, d) in sim.drones.list.iter().enumerate() {
        if !d.present || d.dtype != 10 || d.corpse || !d.enabled { continue; }
        let d2 = (d.pos[0]-bp[0]).powi(2) + (d.pos[2]-bp[2]).powi(2);
        if best.map_or(true, |(_, bd)| d2 < bd) { best = Some((i, d2)); }
    }
    let i = best.expect("an enabled party guest exists").0;
    let pos = sim.drones.list[i].pos;
    (i, pos)
}

/// Stand Bond beside `guest` facing its chest (the fire ray leaves the eye along the look).
fn aim_at(sim: &mut nf_view::scene::Sim, guest: usize) {
    let g = sim.drones.list[guest].pos;
    let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye, sim.bond.pos[2]];
    sim.bond.yaw = (g[0] - eye[0]).atan2(g[2] - eye[2]);
    let horiz = ((g[0] - eye[0]).powi(2) + (g[2] - eye[2]).powi(2)).sqrt();
    sim.bond.pitch = ((g[1] + 0.6 - eye[1]) / horiz).atan() / (std::f32::consts::FRAC_PI_2);
}

/// Walk Bond up to a guest and punch it once (the round spawns at the fire script's strike
/// frame +0x44, so the swing holds the trigger and waits, re-aiming every tick).
fn punch_once(sim: &mut nf_view::scene::Sim, guest: usize) {
    let g = sim.drones.list[guest].pos;
    sim.bond.pos = [g[0] - 0.9, g[1], g[2] + 0.9]; // ~1.27 away, inside the fists' 1.75 range
    sim.gun.weapon = 1; sim.gun.switch_request = 1; sim.gun.switch_state = 0;
    for _ in 0..40 {
        aim_at(sim, guest);
        sim.step_bond(Input::default(), true, false, false);
        for _ in 0..24 { aim_at(sim, guest); sim.step_bond(Input::default(), false, false, false); }
        let g = &sim.drones.list[guest];
        if g.health < 10.0 || g.knocked_out || g.corpse { return; }
    }
}

/// ── The punch = automatic mission fail (the protected actors) ───────────────────────────────────
/// `NDrone2_PunchImpact` 0x145f28:45-56: on 07000007 a punch on an UNARMED drone (the party
/// guests and bouncers all read unarmed — `ObjectIsArmed` 0x178768 on the class byte) raises
/// `DroneFunc_SetMissionFailReason(drone, 5)` [M 0x13e560 → ch 99] — punching a guard is an
/// automatic mission fail. Sister: the victim's health drops.
#[test]
fn punching_a_guest_fails_the_mission_at_once() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
    let (guest, _) = pick_guest(sim);
    punch_once(sim, guest);
    assert!(sim.drones.channels.on(99), "the punch on an unarmed party guest raises ch 99 [M 0x145f28:45-56]");
    assert_eq!(sim.drones.mission.status(), 2, "MissionState 2 (the fail) [M 0x182c60:45-57]");
    assert!(sim.drones.list[guest].health < 10.0, "the punch landed (the victim's health dropped)");
}

/// ── The guests' panic and its cower (the T-pose) ────────────────────────────────────────────────
/// The punch victim screams (`SendHurtMessage` 0x145e28 — prop 0x32) and the OTHER guests (prop
/// 0x27) receive msg 0x12 → CivilianScared → CivilianHiding with the cower 0x19 [M 0x15d3a0 /
/// 0x15d520]. The victim itself is scared at once (`Drone_AlertStatusSet(2)` 0x145f28:30) and
/// flees (state 0x56, the run the port's flee anims serve). Sister: the victim's alert ≥ 0.66.
#[test]
fn punching_a_guest_panics_the_other_guests_into_the_cower() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
    let (guest, _) = pick_guest(sim);
    let victim_alert_before = sim.drones.list[guest].alert;
    punch_once(sim, guest);
    assert!(sim.drones.list[guest].alert >= 0.66,
        "the victim is scared (Drone_AlertStatusSet(2) [M 0x145f28:30]), was {:.2} from {:.2}",
        sim.drones.list[guest].alert, victim_alert_before);
    // The others: within ~20 units of the scream someone must reach the civilian panic states,
    // and a hiding guest must be playing the cower (anim state 0x19).
    let mut scared = 0; let mut cowering = 0;
    for _ in 0..200 {
        sim.step_bond(Input::default(), false, false, false);
        scared = 0; cowering = 0;
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present || d.dtype != 10 || d.corpse || i == guest || !d.enabled { continue; }
            let dist = ((d.pos[0] - sim.drones.list[guest].pos[0]).powi(2) + (d.pos[2] - sim.drones.list[guest].pos[2]).powi(2)).sqrt();
            if dist > 25.0 { continue; }
            if matches!(d.state, 0x12 | 0x13) { scared += 1; }
            if d.state == 0x13 && d.anim.state == 0x19 { cowering += 1; }
        }
        if scared > 0 { break; }
    }
    assert!(scared > 0, "punching a guest panics the guests near the scream into CivilianScared/Hiding (not the T-posed crowd)");
    assert!(cowering > 0, "a hiding guest plays the cower anim 0x19 (the animation the old build was missing — the T-pose)");
}

/// Sister (the armed scare): the guest's own scare gate — `NDrone2_NearArmedPerson` 0x1787c8:
/// prop 0x3c + the reaction-acked sight (`+0x228 & 4`) + Bond armed; kind-0xd guests (voice 13,
/// most of the party floor) are EXEMPT by the game's own `if (kind == 0xd) return 0`
/// [M 0x1787c8:26-28], so the pick is the nearest non-0xd guest. The state the guest sits in
/// decides the rest: Civilian 0x11's UPDATE (0x15d118 case 3) and PartyGirl's (0x160140 case 3)
/// both run the gate every tick and go to CivilianScared 0x12; Civilian's scare arm SHOUTS the
/// armed intruder with prop 0x33 (`DroneAlertToObject(20, 20, 0.7625, msg 0x11, BOND)` [M
/// 0x15d118:50-66]) — PartyGirl additionally raises the prop-0x52 fail reason [M 0x160140:74-76],
/// Civilian's arm does not, so no ch-99 pin here. Driven with the REAL input path only (the
/// walker: walk + turn + the look axis; no teleports, no drone pokes — level-07-guest-talk).
#[test]
fn an_armed_bond_scares_a_guest_who_shouts_the_intruder() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load"));
    w.wait_enabled(9000, "the party intro");
    w.wait(300);
    // The scare-capable pick: the nearest enabled guest whose voice is NOT the exempt 0xd.
    let (guest, gpos) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let mut best: Option<(usize, f32)> = None;
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present || d.corpse || !d.enabled || d.dtype != 10 || d.voice_type == 0xd { continue; }
            let d2 = (d.pos[0]-bp[0]).powi(2) + (d.pos[2]-bp[2]).powi(2);
            if best.map_or(true, |(_, b)| d2 < b) { best = Some((i, d2)); }
        }
        let (i, _) = best.expect("a non-0xd party guest exists near Bond's spawn");
        (i, sim.drones.list[i].pos)
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.list[guest].prop(&sim.data(), 0x3c),
            "the picked guest carries prop 0x3c (the NearArmedPerson gate) [D drone-props 07000007]");
    }
    // Walk to 2.2 in FRONT of the guest (on its facing side — the sight pass's visibility needs
    // the guest's view cone; the front approach makes the first round-robin turn count).
    let y = w.scene.sim.as_ref().unwrap().drones.list[guest].yaw;
    w.walk_to(Wp::new(gpos[0] + y.sin() * 2.2, gpos[1], gpos[2] + y.cos() * 2.2), "in front of the guest");
    // Draw the P2K with the real weapon-switch input (the armed gate).
    for _ in 0..40 { w.tick(Input { switch_to: Some(2), ..Default::default() }, false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 2, "the P2K equips (the real switch path)");
    // Hold on the guest's live front line, the crosshair on its mid-body, and wait for the
    // scare: seen (the reaction-acked sight) → the state's NearArmedPerson → CivilianScared.
    let mut scared = false; let mut shouted = false;
    for t in 0..4000u32 {
        hold_in_front_of(&mut w, guest, 2.2);
        let sim = w.scene.sim.as_ref().unwrap();
        let g = &sim.drones.list[guest];
        scared |= matches!(g.state, 0x12 | 0x13);
        shouted |= g.scare_shouted;
        if t % 500 == 0 {
            eprintln!("[scare +{t}] g st {:#x} los {} seen {} dist {:.1} shouted {} ch99 {}",
                g.state, g.los as u8, g.seen as u8, walker::dxz(g.pos, sim.bond.pos), g.scare_shouted as u8, sim.drones.channels.on(99) as u8);
        }
        if scared && shouted { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(scared, "an armed Bond in the guest's view scares it into CivilianScared/Hiding \
        ([M 0x15d118 case 3 / 0x160140 case 3] — NearArmedPerson 0x1787c8 with the acked sight)");
    assert!(shouted, "prop 0x33's once-bit shouts the armed intruder to everyone (msg 0x11) [M 0x15d118:60-66]");
}

/// Walking up to a guest with the REAL input path (the walk + the turn/look axes) and holding the
/// crosshair on it: BondTalk's own gates all ack — the +0x2A crosshair latch (`Check_Target`
/// 0x1a7810), Bond within 3.0 (+0x1a0), the guest's facing within ±π/2 (+0x1d4) — and an UNARMED
/// Bond must NOT scare the guest: the ObjectIsArmed gate [M 0x1446b0:16] keeps the guest in its
/// talk branch (0x11/0x2f, no shout, no ch 99). The line itself: every 07000007 BondTalk table
/// entry is FACIAL-PRESENT on this level [D the voice-table dump: 0x58..0x63, 0x84..0x8f, 0x8e,
/// 0x95..0x9a all carry facial script hashes present in the archive] — the game plays them through
/// the facial pipeline (`AnimFacialAppend` +0x53c, the script queued at +0x4c, its sound via
/// `AnimScriptPeekSfx`+`SFXPreCache` [M 0x141b00 facial arm; BondTalk 0x1446b0 advances the
/// cooldown once +0x53c stands]) — the port's FACIAL path is unported and its silence is PINNED
/// by `speech_facial_fallback_live` (present scripts stay on the silent path), so no line id can
/// be pinned as sound here; see the report's live checks for the game-side proof request.
#[test]
fn walking_up_to_a_guest_acks_the_talk_gates_and_does_not_scare() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load"));
    w.wait_enabled(9000, "the party intro");
    w.wait(300);
    // The nearest enabled guest (the 0xd voice is the game's biggest chatter set).
    let (guest, gpos) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let mut best: Option<(usize, f32)> = None;
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present || d.corpse || !d.enabled || d.dtype != 10 { continue; }
            let d2 = (d.pos[0]-bp[0]).powi(2) + (d.pos[2]-bp[2]).powi(2);
            if best.map_or(true, |(_, b)| d2 < b) { best = Some((i, d2)); }
        }
        let (i, _) = best.expect("an enabled party guest near Bond's spawn");
        (i, sim.drones.list[i].pos)
    };
    // The fists (the talk gate refuses an armed Bond [M 0x1446b0:16]) — the real switch input.
    for _ in 0..40 { w.tick(Input { switch_to: Some(1), ..Default::default() }, false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 1, "the fists equip (the real switch path)");
    // Walk up and hold on the guest's live front line with the crosshair on its mid-body.
    let y = w.scene.sim.as_ref().unwrap().drones.list[guest].yaw;
    w.walk_to(Wp::new(gpos[0] + y.sin() * 2.0, gpos[1], gpos[2] + y.cos() * 2.0), "in front of the guest");
    let mut aimed = false; let mut seen = false; let mut scared = false; let mut shouted = false;
    for _ in 0..2000u32 {
        hold_in_front_of(&mut w, guest, 2.0);
        let sim = w.scene.sim.as_ref().unwrap();
        let g = &sim.drones.list[guest];
        aimed |= g.aimed_at;
        seen |= g.seen;
        scared |= matches!(g.state, 0x12 | 0x13);
        shouted |= g.scare_shouted;
        if aimed && seen { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let g = &sim.drones.list[guest];
        assert!(aimed && seen, "the real walk-up acks the crosshair latch (+0x2A [M 0x1a7810]) and the             reaction-acked sight (+0x228 & 4 [M 0x175ef0]) on the guest");
        let dist = walker::dxz(g.pos, sim.bond.pos);
        assert!(dist < 3.0, "Bond stands inside the BondTalk radius (3.0 [M 0x1446b0:12]), got {dist:.1}");
        assert!(!scared && !shouted, "an UNARMED Bond up close never scares the guest (the ObjectIsArmed             gate keeps the talk branch [M 0x1446b0:16]); state {:#x} shouted {}", g.state, shouted);
        assert!(matches!(g.state, 0x11 | 0x2f | 0x14), "the guest sits in a talk-capable state             (Civilian 0x11 / PartyGirl 0x2f / CivilianPatrol 0x14), got {:#x}", g.state);
    }
}

/// ── Die once → exactly one restart ──────────────────────────────────────────────────────────────
/// The death sequence ends in the 0x40000042 menu; its Restart (`P_ENDMISSION_Handler`
/// 0x216618:19-21) reloads the CURRENT level warm on the ENTRY checkpoint — exactly once. The
/// port's replay loop (the mission restarting over and over)
/// re-entered the death sequence after the reload; the entry-checkpoint save predates the fail
/// (fix-restart-target), so the reload cannot re-die. Sister: a second `fail_choose_restart` is a
/// no-op (the menu is consumed by the first answer).
#[test]
fn one_death_restarts_exactly_once() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
    let (h, r) = (&mut sim.drones.health, &mut sim.drones.rng); h.pain_generic(r, 1000.0, 1, 0x0700_0007, 0);
    // The death sequence runs to the menu (the red fade); auto_restart answers it instantly in the
    // view, so the test drives the page itself (the 180-fix-menus flow).
    sim.drones.auto_restart = false;
    let mut menus = 0;
    for _ in 0..1200 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.fail_menu_open() { menus += 1; break; }
    }
    assert!(menus == 1, "the death opens the 0x40000042 menu exactly once [M 0x182c60:77-81]");
    let level = sim.drones.level_current;
    sim.drones.fail_choose_restart();
    assert!(sim.drones.level_exit.is_some(), "the Restart requests the level load [M 0x216618:20]");
    assert_eq!(sim.drones.level_exit, Some(level), "the Restart targets the CURRENT level [M 0x216618:20 GameState._12_4_]");
    assert!(!sim.drones.level_exit_cold, "the Restart is the WARM arm (the entry checkpoint) [M 0x216618:20]");
    assert!(sim.drones.level_exit_restart, "the Restart re-serves the entry RamSave [M 0x216618:21 Player_RamLoad(Bond, 1)]");
    // The menu is consumed by the first answer: a second `fail_choose_restart` is a no-op, and the
    // death sequence does not re-enter (the reload's entry save predates the fail).
    sim.drones.fail_choose_restart();
    assert!(!sim.drones.fail_menu_open() && sim.drones.level_exit == Some(level),
        "the menu is consumed and the restart target is unchanged [M 0x216618]");
    for _ in 0..300 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.drones.level_exit, Some(level), "no further restart is queued — exactly one (no replay loop)");
    assert!(!sim.drones.fail_menu_open() && !sim.drones.mission.menu_open,
        "the death sequence does not re-enter the menu after the answer (the reload's entry save predates the fail)");
}
