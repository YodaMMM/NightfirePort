//! 982-civilian-first-sighting-alert: CivilianMissionWait's case-3 first-sighting alert
//! (`NDrone2_DSTATE_CivilianMissionWait` 0x15dbb0 case 3, the part task 962 left out).
//!
//! A waiting civilian who SEES an armed Bond: `NDrone2_NearArmedPerson` gates [M 0x15dbb0:39-44],
//! `+0x4F8 |= 0x8000000` (the own-sighting bit) [M :45], then the prop-0x33 armed-intruder shout
//! ONCE per drone life: `+0x4FC |= 1`, `NDrone2_DroneAlertToObject(dc, 0x11, NDrone2_Player(),
//! 20.0, 20.0, 0.76)` — the float order is the disasm's [M 0x15dd3c call site: a0=dcv, a1=0x11,
//! a2=Player, f12/f13=0x41a00000, f14=0x3f428f5c] — and `+0x4F8 |= 0x200000` the once-only latch
//! [M :46-58]; then voice `+0xD8 == 0x10` → `+0x44 = 1` + Attack 0x56, else CivilianScared 0x12
//! [M :63-70]. The game delivers msg 0x11 (the alert type IS the broadcast message) with the alert
//! record whose position is the OBJECT's — the player's — +0x30 [M 0x14a538 callee: `sw $a2,
//! 0x290(s0)` + :18-21 body], and points the shared alert target at it [M 0x155d48:6-11]. The
//! receivers run `DroneVision_EnemyAlerts` case 0x11: the +0x5a0/+0x5b0/+0x5c0 bookkeeping then the
//! receiver's prop-0x28 gate → `DroneVision_ShoutFromOtherDrone` (returns 0 in this shipped build —
//! no state switch from the shout itself) [M 0x177c18:24-48]; the port's pre-dispatch SHOUT arm is
//! the reaction model (prop 0x28 → guard `enemy_alerts`, civilian → CivilianScared).
//!
//! Level 07000007: the party's 35 dtype-10 guests ALL carry props 0x33 (the shouter gate) and 0x3c
//! [D `nf-extract drone-props 07000007.bin`, 2026-10-23] — but they are voice-0xD, and the game's
//! `NearArmedPerson` returns 0 for voice 0xD on every level outside 0x7000009/0xa [M 0x1787c8:24-31]
//! — the party guests can NEVER raise this alert by the game's own design. The tests pin exactly
//! that: the voice-0xD hold with armed Bond in sight (an alert regression guard), the unarmed hold,
//! and — through the real `Drone::send` state-machine entry with only the per-tick vision inputs
//! set (the ai_live idiom; the resets at `Drones::tick` 0x14461 overwrite any cross-tick poke, so
//! `seen` is stamped between the tick and the direct send) — the alert/latch machinery the voice
//! gate un-reaches on this level. Opt-in via `NF_ISO`.
//!
//! State and msg ids are written raw (`0x16`, `0x17`, `0x22`, `0x12`, `0x56`, msg 3) so this file
//! compiles — and the assertions fail — against the pre-fix master.

use nf_view::scene::{Scene, Sim};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The walker: a dtype-10 guest with no switch channel (starts immediately) whose placement sits on
/// a kind-8 mission path node within the game's 0.3 acceptance [M AssignAIPath 0x154190].
fn find_walker(sim: &Sim) -> usize {
    let data = sim.data();
    let walker = sim.drones.list.iter().position(|dr| {
        dr.present && dr.dtype == 10 && dr.switch_channel == 0
            && data.nav.paths.iter().any(|p| p.kind & 8 != 0 && p.nodes.iter().any(|n| {
                (n.pos[1] - dr.feet()[1]).abs() <= 2.0
                    && (dr.pos[0] - n.pos[0]).powi(2) + (dr.pos[2] - n.pos[2]).powi(2) < 0.09
            }))
    }).expect("07000007 places a guest on a kind-8 mission path");
    drop(data);
    walker
}

/// Drive `msg` through the real state-machine entry (`Drone::send`, the path a data-driven sender
/// feeds) — the 962 idiom.
fn send_msg(scene: &mut Scene, walker: usize, msg: u8) {
    let sim = scene.sim.as_mut().unwrap();
    let difficulty = sim.drones.difficulty;
    // Disjoint field borrows only (no `sim.drones` in the Data) so `&mut sim.drones.rng` coexists.
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
        ai_points: &sim.ai_points,
    };
    let frame = sim.drones.frame;
    sim.drones.list[walker].send(&data, &mut sim.drones.rng, frame, msg);
}

/// Park the guest in CivilianMissionWait through the real msg-0x22 entry [M 0x15d860:130-136].
fn park(scene: &mut Scene, walker: usize) {
    send_msg(scene, walker, 0x22);
    assert_eq!(scene.sim.as_ref().unwrap().drones.list[walker].state, 0x17, "parked in CivilianMissionWait");
}

/// A shout HEARER: a dtype-1 guard carrying prop 0x28 [D drone-props 07000007], active
/// (switch_channel 0), who is NOT the shouter — the receiver the consume pass delivers to.
fn pick_hearer(sim: &Sim, shouter: usize) -> usize {
    let data = sim.data();
    let hearer = sim.drones.list.iter().position(|dr| {
        dr.present && dr.enabled && dr.dtype == 1 && dr.switch_channel == 0
            && dr.prop(&data, 0x28)
    }).expect("07000007 has active dtype-1 guards carrying prop 0x28 [D drone-props]");
    drop(data);
    assert_ne!(hearer, shouter, "the hearer is a guard, not the shouting guest");
    hearer
}

/// The first sighting shouts the armed intruder ONCE (the latch).
///
/// Phase 1 — the parked guest (prop 0x33 [D], her `bond_armed` stamped by the warmup ticks' real
/// per-tick path, `seen` stamped for the direct UPDATE send): she stamps the own-sighting bit
/// [M 0x15dbb0:45], sets `+0x4FC` bit 0 [M :54] and the 0x200000 latch [M :58], queues the
/// DroneAlertToObject shout, and exits 0x17 per her voice [M :63-70]. One real tick consumes the
/// queue through the real broadcast pass and the prop-0x28 guard's EnemyAlerts bookkeeping runs
/// (her `resume_state` witness [M 0x177c18:24-48 through the port's pre-dispatch]).
/// Phase 2 — the latch: re-parked (the port models no scare→mission resume, so the harness
/// re-enters her through the real SetState(0x16) + msg-0x22 path) and sighting again, she re-stamps
/// the sighting but does NOT re-queue the shout [M :53-58].
/// Phase 3 — the control: with the port's latch field poked clear (the one non-real step, the
/// port's model of `+0x4F8 & 0x200000 == 0`), the same sighting re-queues the shout — the phase-2
/// silence is the latch's doing, not a dead queue path.
#[test]
fn the_first_sighting_shouts_the_armed_intruder_once() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let walker = find_walker(sim);
    sim.gun.weapon = 2; sim.gun.switch_request = 2; // the carried P2K — inside the armed set 2..0x47 [M DroneWeap_OpponentHasWeapon 0x17a4b8]
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    assert_eq!(sim.drones.list[walker].state, 0x16, "the guest runs CivilianMission [M 0x15d860]");
    assert!(sim.drones.list[walker].bond_armed, "the per-tick path stamped Bond armed");
    park(&mut scene, walker);
    // The voice substitution [S]: 07000007's own guests are voice 0xD and the game's
    // `NearArmedPerson` carve-out (0x1787c8:29-31) un-reaches the case-3 alert for them on this
    // level BY DESIGN (the carve-out hold is its own test below). The harness substitutes the
    // voice-0x6 civilian the disc places on 0700000c/0d (dtype-5 staff carrying 0x33+0x3c [D
    // `nf-extract drone-props`]) so the state machinery under test is reachable; everything else
    // is the real state machine.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.list[walker].voice_type = 0x6;
    }

    // Phase 1 — the sighting, through the real UPDATE entry with the sight stamped.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.list[walker].seen = true; // the acked sight +0x228 & 4 [M 0x1787c8:34-36]
    }
    send_msg(&mut scene, walker, 3); // UPDATE [M 0x15dbb0 case 3]
    let sim = scene.sim.as_mut().unwrap();
    {
        let dr = &sim.drones.list[walker];
        assert!(dr.own_sight, "the sighting stamps +0x4F8 0x8000000 (own) [M 0x15dbb0:45]");
        assert_eq!(dr.alert_bits & 1, 1, "+0x4FC bit 0 set with the shout [M 0x15dbb0:54]");
        assert!(dr.scare_shouted, "the once-latch +0x4F8 0x200000 set [M 0x15dbb0:58]");
        assert!(dr.pending_shout.is_some(), "the DroneAlertToObject(0x11, player, 20, 20, 0.76) shout queued [M 0x15dbb0:55-57]");
        let voice = dr.voice_type;
        if voice == 0x10 {
            assert_eq!(dr.state, 0x56, "voice 0x10: +0x44 = 1 and Attack [M 0x15dbb0:63-67]");
        } else {
            // The game calls SetState(0x12) [M 0x15dbb0:69-70]; the port's set_state runs the ENTER
            // synchronously and CivilianScared's ENTER chains on (scared-run → 0x64, else the
            // CivilianHiding cower) — the LOG is the witness, the same idiom the door_guard tests
            // use for Attack's submove chain.
            assert!(dr.state != 0x17, "she left the wait");
            assert!(dr.log.iter().any(|&(_, s)| s == 0x12), "SetState(CivilianScared 0x12) ran [M 0x15dbb0:69-70] — log {:?}",
                &dr.log[..dr.log.len().min(8)]);
        }
    }
    // One real tick: the broadcast pass consumes the queue [M Drone_SM_BroadcastMsg 0x175aa8 —
    // the port's consume pass] and the hearer guard processes the delivered msg 0x11. The shout's
    // position is the alert OBJECT's — the player's — position [M 0x14a538:18-21], captured at
    // queue time (the player never moved: the start pos), so the hearer parks 14 from BOND.
    let hearer = pick_hearer(sim, walker);
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.drones.list[hearer].resume_state, 0, "the hearer has not processed an alert yet");
    let shout_pos = sim.bond.pos;
    sim.drones.list[hearer].pos = [shout_pos[0] + 14.0, shout_pos[1], shout_pos[2]];
    sim.bond.pos = [sim.drones.list[walker].pos[0] + 40.0, sim.drones.list[walker].pos[1], sim.drones.list[walker].pos[2]];
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_mut().unwrap();
    assert_ne!(sim.drones.list[hearer].resume_state, 0,
        "the shout delivered: the prop-0x28 guard ran the EnemyAlerts bookkeeping [M 0x177c18:24-48]");

    // Phase 2 — the latch. Re-enter 0x17: the port models no scare→mission resume (the real
    // chain ended in the scared-run 0x13 with return_state CivilianHiding), so the harness re-arms
    // the subject with the raw state poke [S — the one non-real step; the 0x22 → 0x17 transition
    // after it is the real machine entry].
    {
        let dr = &mut sim.drones.list[walker];
        dr.state = 0x16; // [S] harness re-arm — see the comment above
    }
    park(&mut scene, walker);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.list[walker].seen = true;
    }
    send_msg(&mut scene, walker, 3);
    let sim = scene.sim.as_mut().unwrap();
    {
        let dr = &sim.drones.list[walker];
        assert!(dr.own_sight, "the second sighting re-stamps the own-sighting bit (unconditional [M 0x15dbb0:45])");
        assert!(dr.scare_shouted, "the latch stays set [M 0x15dbb0:53-58]");
        assert!(dr.pending_shout.is_none(),
            "the latch holds: a second sighting does NOT re-shout [M 0x15dbb0:53-58]");
    }

    // Phase 3 — the control: latch poked clear, the same sighting re-queues the shout. The same
    // [S] harness re-arm as phase 2.
    {
        let dr = &mut sim.drones.list[walker];
        dr.scare_shouted = false; // [S] the harness's model of `+0x4F8 & 0x200000 == 0`
        dr.state = 0x16; // [S] harness re-arm
    }
    park(&mut scene, walker);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.list[walker].seen = true;
    }
    send_msg(&mut scene, walker, 3);
    let sim = scene.sim.as_mut().unwrap();
    assert!(sim.drones.list[walker].pending_shout.is_some(),
        "the control: with the latch clear the same sighting re-queues the shout — phase 2's silence was the latch");
}

/// Sister: without property 0x33 there is no alert. The sighting itself still stamps (the
/// own-sighting bit is unconditional [M 0x15dbb0:45]) and she still exits per her voice, but
/// `+0x4FC` bit 0 stays clear [M :54 — inside the property gate], the latch stays clear and no
/// shout is queued. The level's own data has every guest carrying 0x33 [D drone-props 07000007],
/// so the sister clears it the way `behaviour_util_getProperty(0x33, +0x4d8)` == 0 reads [M
/// 0x15dbb0:46-47].
#[test]
fn without_property_33_the_sighting_never_shouts() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let walker = find_walker(sim);
    sim.gun.weapon = 2; sim.gun.switch_request = 2; // the P2K [D loadout]
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    {
        let dr = &mut sim.drones.list[walker];
        sim.props.set(0x33, &mut dr.props, 0);
        if let Some(ref mut ap) = dr.alert_props { sim.props.set(0x33, ap, 0); }
    }
    park(&mut scene, walker);
    // The same voice substitution as the first test [S — see its comment].
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.list[walker].voice_type = 0x6;
        sim.drones.list[walker].seen = true;
    }
    send_msg(&mut scene, walker, 3);
    let sim = scene.sim.as_mut().unwrap();
    {
        let dr = &sim.drones.list[walker];
        assert!(dr.own_sight, "+0x4F8 0x8000000 is unconditional [M 0x15dbb0:45]");
        assert_eq!(dr.alert_bits & 1, 0, "no +0x4FC bit 0 without prop 0x33 [M 0x15dbb0:46-54]");
        assert!(!dr.scare_shouted, "the latch never set — no shout happened");
        assert!(dr.pending_shout.is_none(), "no DroneAlertToObject shout without the property");
    }
}

/// The 07000007 party guests are voice 0xD, and the game's `NearArmedPerson` returns 0 for voice
/// 0xD outside 0x7000009/0xa [M 0x1787c8:24-31] — an armed Bond in sight NEVER trips the case-3
/// alert arm for them [M 0x15dbb0:39-44 → BondTalk + stay]: the wait holds (the TIMER_C re-arm
/// keeps her parked while Bond stands ahead), nothing stamps, nothing delivers. This is the
/// regression guard for the carve-out: the new alert code must not fire through it.
#[test]
fn the_voice_0xd_guests_never_raise_the_alert() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let walker = find_walker(sim);
    sim.gun.weapon = 2; sim.gun.switch_request = 2; // the P2K [D loadout]
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.list[walker].bond_armed, "Bond armed through the per-tick path");
    park(&mut scene, walker);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.list[walker].seen = true; // she IS seeing armed Bond
    }
    send_msg(&mut scene, walker, 3);
    let sim = scene.sim.as_mut().unwrap();
    {
        let dr = &sim.drones.list[walker];
        assert_eq!(dr.voice_type, 0xd, "the 07000007 guests are voice 0xD [D skins; M 0x1787c8:29-31]");
        assert_eq!(dr.state, 0x17, "the voice-0xD carve-out keeps her in the wait [M 0x15dbb0:39-44]");
        assert!(!dr.own_sight, "no sighting stamp — case 3 never passed the NearArmedPerson gate");
        assert_eq!(dr.alert_bits & 1, 0, "no +0x4FC bit 0");
        assert!(!dr.scare_shouted, "no latch");
        assert!(dr.pending_shout.is_none(), "no shout");
    }
    // The hold is durable: with armed Bond standing ahead, the wait outlives the 2 s clock (the
    // TIMER_C re-arm — Bond inside 3.0 in the cone [M 0x15dbb0:87-106]).
    {
        let pos = sim.drones.list[walker].pos;
        let yaw = sim.drones.list[walker].yaw;
        sim.bond.pos = [pos[0] + yaw.sin() * 1.5, pos[1], pos[2] + yaw.cos() * 1.5];
    }
    for _ in 0..240 {
        sim.step_bond(Default::default(), false, false, false);
        assert_eq!(sim.drones.list[walker].state, 0x17, "she holds the wait with armed Bond ahead [M 0x15dbb0:39-44]");
    }
    {
        let dr = &sim.drones.list[walker];
        assert!(!dr.own_sight && !dr.scare_shouted && dr.alert_bits & 1 == 0 && dr.pending_shout.is_none(),
            "still nothing stamped or queued after 4 s");
    }
}

/// Sister: an UNARMED Bond (fists, weapon 1 — outside the armed set 2..0x47 the port folds from
/// `DroneWeap_OpponentHasWeapon` 0x17a4b8) is not an armed person even for a voice that could
/// qualify: `NDrone2_NearArmedPerson` is 0 [M 0x15dbb0:40-44], so she stays in the wait (the
/// BondTalk arm — the level voice sets, unported like the whole civilian talk family), nothing
/// stamps, nothing delivers.
#[test]
fn an_unarmed_bond_gets_bondtalk_and_no_alert() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let walker = find_walker(sim);
    sim.gun.weapon = 1; sim.gun.switch_request = 0x47; // the fists — outside the armed set [M 0x17a4b8 via the port's fold]
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.drones.list[walker].bond_armed, "the per-tick path stamped Bond unarmed");
    park(&mut scene, walker);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.list[walker].seen = true;
    }
    send_msg(&mut scene, walker, 3);
    let sim = scene.sim.as_mut().unwrap();
    {
        let dr = &sim.drones.list[walker];
        assert_eq!(dr.state, 0x17, "unarmed Bond is no armed person: she holds the wait [M 0x15dbb0:39-44]");
        assert!(!dr.own_sight, "no sighting stamp [M 0x15dbb0:45 never ran]");
        assert_eq!(dr.alert_bits & 1, 0, "no +0x4FC bit 0");
        assert!(!dr.scare_shouted, "no latch");
        assert!(dr.pending_shout.is_none(), "no shout");
    }
}
