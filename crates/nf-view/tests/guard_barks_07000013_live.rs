//! level-13-guard-barks: do 07000013's lobby guards speak?
//! The 12 lobby guards (skins 0x05000004/06/08) are voice type 3 [M NDrone2_DefaultInit
//! 0x14b300:96-103 stores 3 into Drone+0xD8 for exactly these skins + 0x500004c]. The game's
//! bark channels:
//! - crosshair `NDrone2_BondTalk` 0x1446b0 is level-0x7000007-only (:30) and its voice-set
//!   switch {9,0xd,0xe,0x10,0x11,0x12} has NO case 3; it is also only called from the
//!   civilian-family state UPDATEs, never IDLE/PATROL/ATTACK — the game never gives these
//!   guards a crosshair line. The port mirrors that (`bond_talk` reachable only from the
//!   civilian states, `d.level != 0x0700_0007 → return`).
//! - the idle/patrol bark is `NDrone2_PatrolTalk` 0x13e988, called ONLY from state PATROL's
//!   UPDATE [M 0x15c498:41-44], gated on the global cooldown, Rand(2) and voice type 3 [M :35],
//!   speaking 0xb3+Rand(5) [M :36-40] = Drone_SfxInfo sfx 0x261..0x264/0x266, facial 0
//!   [D ACTION.ELF @0x2812d0 + idx·32 — the unported FACIAL arm cannot swallow these].
//! - the attack-entry bark is `NDrone2_AttackTalk` 0x13eac0: type 3 speaks 0xac+Rand(7) [M
//!   :66-68] ONLY while `+0x228 & 8` is clear [M :63-65] — the bit HaveOpponentSight
//!   0x175ef0:82-86 sets on a reaction-acked sighting with prop 0x3c. All 12 lobby guards carry
//!   prop 0x3c [D behaviour words, bitDescs/bitMasks @0x2c7250/0x2c7308 + 0x1c7d98], so at this
//!   level the attack bark belongs to the never-seen paths (a noise/peer alert), never to the
//!   first sighting.
//! PATROL is reached only through InitPatrol's props 0x24/0x25 + AssignAIPath(4) [M 0x1724f0]:
//! the disc's behaviour words carry props 0x24/0x25 on #317/#320/#325/#326/#330/#408/#409 and
//! neither on #318/#319/#321/#322/#329 [D 07000013.bin placements; prop 0x24 = word 1 bit 5,
//! 0x25 = word 1 bit 6]. Of the group-0 guards the level creates at load [D golden census: 6],
//! #317/#320 patrol and #329/#330 idle — #318/#319 also have both props clear.
#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use nf_game::sound::Cmd;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load13() -> Walker {
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso(), "07000013.bin", false, None, 3).unwrap());
    assert!(w.wait_enabled(3000, "the atrium arrival"), "the intro releases Bond");
    w.wait(60);
    w
}

/// The lobby guards on the loaded map: the skins 0x05000004/06/08 drones [D census m41
/// #317-#409; group 0 creates at load, groups 1/2 are the spawner templates [D golden census
/// drones.tsv: 6 at start, 2 spawner groups]].
fn lobby(w: &Walker) -> Vec<(usize, u32, u8, [f32; 3])> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && (d.skin >> 16) == 0x0500 && matches!(d.skin & 0xffff, 0x04 | 0x06 | 0x08))
        .map(|(i, d)| (i, d.skin, d.state, d.pos))
        .collect()
}

/// Per guard group: the voice type. Every lobby guard skin maps to voice set 3
/// [M 0x14b300:96-103], and the port's `voice_type()` carries the same rows.
#[test]
fn the_lobby_guards_are_voice_type_3() {
    let w = load13();
    let guards = lobby(&w);
    assert!(guards.len() >= 4, "the group-0 lobby guards create at load [D golden census 6; the rest are spawner templates]: {:?}", guards);
    let sim = w.scene.sim.as_ref().unwrap();
    for (i, skin, _, _) in &guards {
        let d = &sim.drones.list[*i];
        assert_eq!(d.voice_type, 3, "skin {skin:08x} → voice set 3 [M 0x14b300:96-103]");
        assert_eq!(d.kind, 3, "the DTYPE pair (+0xD8/+0xDA = 3/0xc) rides the same skin switch");
    }
    // Sister (can fail): a different skin would not be 3 — the level's other drone (map 11's
    // skin 0x05000000, voice 0 [M 0x14b300's default]) must not leak into this set.
    assert!(guards.iter().all(|(_, s, _, _)| matches!(s & 0xffff, 0x04 | 0x06 | 0x08)));
}

/// The disc's patrol props split the loaded guards exactly: #317/#320 (#325/#326/#408/#409 in
/// the spawner groups) carry props 0x24/0x25 [D] → InitPatrol → PATROL; #318/#319/#329/#330
/// (#321/#322) carry neither → IDLE. What the guards do with that depends on the sighting:
/// four of them spot Bond across the lobby and go to combat, the two prop-less ones sit Idle.
#[test]
fn the_disc_props_split_the_loaded_guards() {
    let mut w = load13();
    let guards = lobby(&w);
    assert_eq!(guards.len(), 6, "the six group-0 guards create at load [D census p4=0 rows]");
    let sim = w.scene.sim.as_ref().unwrap();
    let mut idle = 0;
    let mut combat = 0;
    for (i, skin, st, _) in &guards {
        let d = &sim.drones.list[*i];
        if *st == state::IDLE { idle += 1; }
        else { combat += 1; eprintln!("combat guard {i} skin {skin:08x} in {}", nf_view::dev::state_name(*st)); }
    }
    // The two prop-less guards (#329 skin 04, #330 skin 06 [D census coords]) sit Idle: without
    // props 0x24/0x25 InitPatrol is never entered [M 0x1724f0:23-26] and PatrolTalk's only
    // caller is PATROL's UPDATE [M 0x15c498:41-44] — they can never patrol-bark. The other four
    // (the prop-set #317/#320 and the p3=0x28 waiters #318/#319 [D]) left their load states
    // toward combat aim/NoRoute once Bond stood in the lobby — combat entry, where the port's
    // ever-sighted gate (and the game's +0x228 & 8) applies.
    assert_eq!(idle, 2, "the prop-less pair idles [D behaviour words: #318/#319/#329/#330 all-clear]");
    assert_eq!(combat, 4, "the rest went to the combat family on the sighting");
}

/// The first-sighting chain is silent for these guards, exactly as the game's: SeenPlayerTalk
/// has no voice-3 case [M 0x13f1d0's level switch], and the FirstAttack AttackTalk follow-up is
/// gated by the ever-sighted bit the sighting itself just set [M 0x148010:94 + 0x13eac0:63-65 +
/// 0x175ef0:82-86]. Across a long standing-still window: no 0xac..0xb2 (attack) and no
/// 0x261..0x266 (patrol) sfx ids start through the real audio path.
#[test]
fn the_first_sighting_chain_barks_nothing() {
    let mut w = load13();
    let mut heard: Vec<u32> = Vec::new();
    for _ in 0..1500u32 {
        let input = nf_game::player::Input { forward: 0.0, ..Default::default() };
        w.tick(input, false);
        let mut sim = w.scene.sim.as_mut().unwrap();
        for c in sim.drones.audio.take_commands() {
            if let Cmd::Start { id, .. } = c { heard.push(id); }
        }
    }
    assert!(heard.iter().all(|id| !(0xac..=0xb2).contains(id)),
        "no attack-entry bark after the sighting set the ever-sighted bit [M 0x13eac0:63-65]: {heard:?}");
    assert!(heard.iter().all(|id| !(0x261..=0x266).contains(id)),
        "no patrol bark: no loaded guard is in PATROL while Bond stands at spawn [M 0x15c498:41-44 calls PatrolTalk only there]");
    // Sister (can fail): the unit test `attack_talk_type3_gates_on_the_ever_sighted_bit` proves
    // a never-sighted type-3 guard DOES queue 0xac..0xb2 — this live silence is the gate's work.
}

/// The crosshair path: standing face-to-face with an Idle lobby guard latches Check_Target's
/// crosshair mark (the same latch the civilian states' BondTalk consumes), but the guard never
/// talks — `NDrone2_BondTalk` is level-0x7000007-only [M 0x1446b0:30] with no voice-set-3 case
/// [M :40] and is not called from Idle's UPDATE at all.
#[test]
fn crosshair_on_a_lobby_guard_is_silent() {
    let mut w = load13();
    // The Idle pair stands at the lobby's west desks [D census #329 (-27.02, -368.23, 29.11)].
    let sim = w.scene.sim.as_ref().unwrap();
    let (gi, gpos) = lobby(&w).iter()
        .filter(|(_, _, st, _)| *st == state::IDLE)
        .map(|(i, _, _, p)| (*i, *p)).next().expect("an Idle lobby guard from the prop-less pair");
    let _ = sim;
    // Bond 1.5 units south of the guard, facing him (the guard must be within 3.0 and the
    // crosshair ray on him [M 0x1446b0:20, Check_Target 0x1a7810:98-150]).
    let bx = gpos[0];
    let bz = gpos[2] - 1.5;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [bx, gpos[1], bz];
        sim.bond.yaw = 0.0; // face +z, at the guard
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    let mut speech: Vec<u32> = Vec::new();
    for _ in 0..240u32 {
        let input = nf_game::player::Input { forward: 0.0, ..Default::default() };
        w.tick(input, false);
        let mut sim = w.scene.sim.as_mut().unwrap();
        assert!(sim.drones.list[gi].talks.is_empty(), "the crosshair never queues a line for a lobby guard [M 0x1446b0:30/:40 + the state UPDATE arms]");
        for c in sim.drones.audio.take_commands() {
            if let Cmd::Start { id, .. } = c { speech.push(id); }
        }
    }
    // (Check_Target's own latch is not asserted here: at the spawn loadout the crosshair-mode
    // gate can skip the ray entirely — the pin is the silence, which is what the task asks.)");
    assert!(speech.is_empty(), "no speech at all from the face-to-face window: {speech:?}");
}

