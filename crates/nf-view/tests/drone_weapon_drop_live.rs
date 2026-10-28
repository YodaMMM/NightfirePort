//! drone-weapon-drop-slot: the dying guard's weapon drop follows the game's weapon-slot
//! handling, not a held-byte stand-in.
//!
//! The game's rule chain [M]:
//! - `DroneWeap_ChangeWeapon(dcv, w)` 0x17a6e0 is a per-weapon jump table (0x2fb740, 55 entries):
//!   arms {1, 6, 0x12, 0x18, 0x19, 0x36} write the held byte (`anim+0x62`) WITH the anim set
//!   (`Drone+0xDA`) and the clip pair (`+0xBBC`/`+0xBBE`); every other id returns unchanged
//!   [M dis 0x17a70c-0x17a790].
//! - The stun families' drop arms gate on the ANIM SET byte: the taser keeps anim-set-4 guards
//!   armed [M 0x162b98:27-34], the flashbang likewise [M 0x172bb0:15-18], the dart drops only
//!   sets {5,6,9} [M 0x163430:20-27, dis 0x16347c-0x1634b4].
//! - `DSTATE_Stunned_Recover` 0x1630b8 case 1 (the TASER family's wake) calls
//!   `ChangeWeapon(6)` for non-civilians — the guard re-arms with the fresh sidearm (weapon 6,
//!   anim set 4, clip 16) [M, dis 0x16314c-0x163150 loads `a1 = 6`]. `StunGrenadeRecover`
//!   0x163350 has NO such call — the flashbang victim stays on the fists [M].
//! - Killing him then runs `DroneWeap_DropWeapon` 0x17a2a0 on the held byte: weapon 6's record
//!   has base 6, model 0x0200025a, clip 16 [L ee_start_05.bin @ weapon_data 0x2bf150 + 6·0x10C],
//!   so the kill drops a weapon-6 pickup with 16·(0.25+FRand(0.5)) ∈ [4..=12] rounds. The old
//!   port skipped the recover re-arm entirely: the guard stayed on weapon 1 (model 0) and the
//!   kill dropped NOTHING.
//!
//! Real path only: `nf_view::load` + `Sim::step_bond` inputs (the Stunner and the P2K equipped
//! through the switch input, the tase/kill delivered by the trigger); the only non-input step is
//! the flashbang grenade push of the established harness shape (the view's own throw path).
//! Opt-in: set `NF_ISO` to the disc image.

use nf_game::drone::{state, anim_state};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

fn steer(scene: &mut nf_view::scene::Scene, gi: usize, fire: bool) {
    steer_at(scene, gi, fire, 0.0);
}

/// The steer with the aim point raised by `aim_dy` above the guard's root (the downed body's
/// low sphere is centred 0.3 above the origin [M NDrone2_Collision 0x140e90:65-90]).
fn steer_at(scene: &mut nf_view::scene::Scene, gi: usize, fire: bool, aim_dy: f32) {
    let (turn, look) = {
        let sim = scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let g = sim.drones.list[gi].pos;
        let dx = g[0] - bp[0];
        let dz = g[2] - bp[2];
        let dy = (g[1] + aim_dy) - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let desired = dx.atan2(dz);
        let err = desired - sim.bond.yaw;
        let want_pitch = (dy / dxz(bp, g).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        (err.clamp(-0.08, 0.08), (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08))
    };
    scene.sim.as_mut().unwrap().step_bond(Input { turn, look, ..Default::default() }, fire, false, false);
}

/// Bond at `dist` u WEST of the guard, aimed at him when asked (the teleport rebuilds Bond).
fn place_bond(scene: &mut nf_view::scene::Scene, gi: usize, dist: f32, aim: bool) {
    let g = scene.sim.as_ref().unwrap().drones.list[gi].pos;
    let mut dev = nf_view::dev::Dev { scenario: None, ..Default::default() };
    dev.teleport(scene, [g[0] - dist, g[1], g[2]], 0.0);
    if aim {
        for _ in 0..40 { steer(scene, gi, false); }
    }
}

/// The first live guard within 20 u of Bond's start (the stun_ai_recover_live finder).
fn pick_start_guard(scene: &nf_view::scene::Scene) -> usize {
    let sim = scene.sim.as_ref().unwrap();
    *sim.drones.llist.iter().find(|&&i| {
        let dr = &sim.drones.list[i];
        dr.present && dr.enabled && !dr.corpse && dr.health > 0.0
            && (dr.pos[0] - sim.bond.pos[0]).powi(2) + (dr.pos[2] - sim.bond.pos[2]).powi(2) < 400.0
    }).expect("a placed guard within 20 u of Bond's start on 07000005")
}

/// Equip `w` through the real switch input; panics when it does not take.
fn equip(scene: &mut nf_view::scene::Scene, w: u8) {
    let draw = Input { switch_to: Some(w), ..Default::default() };
    for _ in 0..60 {
        scene.sim.as_mut().unwrap().step_bond(draw, false, false, false);
        if scene.sim.as_ref().unwrap().gun.weapon == w { return; }
    }
    panic!("weapon {w:#x} did not equip");
}

/// Hold the beam until the guard falls into DSTATE_Taser 0x4a, keep the re-arms 30 ticks, release.
fn stun_to_taser(scene: &mut nf_view::scene::Scene, gi: usize) {
    let mut fell = None;
    for t in 0..400u32 {
        steer(scene, gi, true);
        if scene.sim.as_ref().unwrap().drones.list[gi].state == state::TASER { fell = Some(t); break; }
    }
    let fell = fell.expect("the guard falls into DSTATE_Taser 0x4a under a held tase [M 0x1469b8 case 7]");
    for _ in 0..30 { steer(scene, gi, true); }
    steer(scene, gi, false);
    eprintln!("[drop] fell at hold tick {fell}");
}

fn is_dead(s: u8) -> bool { matches!(s, state::DEAD | state::FADE | state::FADE_FAST | state::DEATH_ANIM) }

/// Aim at `gi` and pull the trigger only on a converged tick — the combat guard dodges, and a
/// trigger held through the sway never lands the second hit.
fn steer_fire_aligned(scene: &mut nf_view::scene::Scene, gi: usize, aim_dy: f32) {
    let (turn, look, aligned) = {
        let sim = scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let g = sim.drones.list[gi].pos;
        let dx = g[0] - bp[0];
        let dz = g[2] - bp[2];
        let dy = (g[1] + aim_dy) - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let desired = dx.atan2(dz);
        let err = desired - sim.bond.yaw;
        let want_pitch = (dy / dxz(bp, g).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        (err.clamp(-0.08, 0.08), (perr).clamp(-0.08, 0.08), err.abs() < 0.02 && perr.abs() < 0.035)
    };
    scene.sim.as_mut().unwrap().step_bond(Input { turn, look, ..Default::default() }, aligned, false, false);
    // The fight-back allowance (the walkreal harness's own rule): the guard's stand/aim states
    // carry the game's sight bit on ENTER [M 0x1687c0:17 StandFiddle; the vision-bit census] and
    // he engages at the 3.5 u setup distance — the test stand-in cannot fight back, so his fire
    // cannot kill Bond here (the health top-up; the AI still runs).
    let h = &mut scene.sim.as_mut().unwrap().drones.health;
    // 1000, not 100: the SG5's point-blank blast can take a full 100-health bar in one step and
    // the death, once started, runs to ch 0x62 regardless of a later top-up — the allowance must
    // keep the bar above 0 THROUGH the step, not restore it after.
    if h.health < 1000.0 { h.health = 1000.0; }
}

/// The tased guard who wakes re-arms with `ChangeWeapon(6)` — and his LATER death drops the
/// weapon-6 pickup with the table's fresh clip.
///
/// Game chain [M]: the tase ENTER drops the held gun and takes the fists (`DropWeapon +
/// ChangeWeapon(1)`, gated on the anim set != 4 — the SG5 guard's set is 5 [L census]) [M
/// 0x162b98:27-34]; `Stunned_Recover` case 1 calls `ChangeWeapon(6)` for the non-civilian
/// (team 1) — weapon 6, anim set 4, clip 16 [M 0x1630b8, dis 0x16314c-0x163150 `a1 = 6` + the
/// 0x17a738 arm]; the kill then runs `DropWeapon` on the held byte → the weapon_data[6] pickup:
/// base 6, clip 16 → 16·(0.25+FRand(0.5)) ∈ [4..=12] rounds [M 0x17a2a0; L record 6 @
/// 0x2bf150+6·0x10C]. The tase-family recover is the ONLY one with the re-arm: the flashbang
/// family's recover has none [M 0x163350 case 1] — that asymmetry is the sister test's pin.
/// FAILS on the old port: no re-arm at the recover, the guard stayed on the fists id, the kill
/// dropped nothing.
#[test]
fn the_tased_guard_who_wakes_re_arms_and_his_death_drops_the_sidearm() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let gi = pick_start_guard(&scene);
    let held = scene.sim.as_ref().unwrap().drones.list[gi].weapon as u16;
    {
        let dr = &scene.sim.as_ref().unwrap().drones.list[gi];
        assert!(matches!(held, 0x16 | 0x24), "the start guard carries a census weapon [L census: every 05 guard reads 0x16/0x24]");
        assert_eq!(dr.team, 1, "an enemy guard (the ChangeWeapon(6) arm is the non-civilian one) [M 0x1630b8 case 1]");
        assert_ne!(dr.anim_set, 4, "the tase drop gate keeps only anim-set-4 guards — not this one [M 0x162b98:27-34]");
    }
    let at_load = scene.sim.as_ref().unwrap().drones.pickups.len();
    equip(&mut scene, 0x4a); // the Stunner, through the switch input
    place_bond(&mut scene, gi, 2.5, false); // the beam's reach — the stun_ai_recover setup distance
    for _ in 0..40 { steer(&mut scene, gi, false); }
    stun_to_taser(&mut scene, gi);
    {
        let dr = &scene.sim.as_ref().unwrap().drones.list[gi];
        assert_eq!(dr.weapon, 1, "the tase ENTER dropped the SG5 into the fists [M 0x162b98:27-34]");
        assert_eq!(dr.anim_set, 0xc, "ChangeWeapon(1) set the anim set 0xC [M 0x17a714]");
        assert_eq!((dr.clip, dr.clip_max), (0, 0), "ChangeWeapon(1) cleared the clip pair [M 0x17a720-0x17a728]");
    }
    let first_pickups = scene.sim.as_ref().unwrap().drones.pickups.len();
    assert_eq!(first_pickups, at_load + 1, "the tase drop is on the ground");
    assert_eq!((scene.sim.as_ref().unwrap().drones.pickups[at_load].weapon, scene.sim.as_ref().unwrap().drones.pickups[at_load].kind), (held, nf_game::pickup::kind::WEAPON), "the held gun's pickup");
    // TASER → STUNNED at the 30-tick fall wait [M 0x162b98 case 0xc].
    let mut stunned_at = None;
    for t in 0..240u32 {
        steer(&mut scene, gi, false);
        if scene.sim.as_ref().unwrap().drones.list[gi].state == state::STUNNED { stunned_at = Some(t); break; }
    }
    stunned_at.expect("the fall ends into DSTATE_Stunned 0x4b [M 0x162b98 case 0xc]");
    let stun_frame = scene.sim.as_ref().unwrap().drones.frame;
    // Step Bond 8 u out before the RecoverTime (1200) expires — outside AnyoneNear's 4.0 band —
    // so the wake lands [M 0x162eb0 case 0xc → 0x4c].
    while scene.sim.as_ref().unwrap().drones.frame < stun_frame + 1150 { steer(&mut scene, gi, false); }
    place_bond(&mut scene, gi, 8.0, false);
    let mut recovered_at = None;
    for t in 0..1700u32 {
        steer(&mut scene, gi, false);
        if scene.sim.as_ref().unwrap().drones.list[gi].state == state::STUNNED_RECOVER { recovered_at = Some(t); break; }
    }
    recovered_at.expect("nobody near → Stunned_Recover 0x4c [M 0x162eb0 case 0xc]");
    for _ in 0..5 { steer(&mut scene, gi, false); } // let the ENTER deliver
    {
        let dr = &scene.sim.as_ref().unwrap().drones.list[gi];
        // THE fix: the recover re-arms him with the fresh sidearm [M 0x1630b8 case 1 + 0x17a738].
        assert_eq!(dr.weapon, 6, "Stunned_Recover re-arms the guard with ChangeWeapon(6) [M 0x1630b8 case 1]");
        assert_eq!(dr.anim_set, 4, "the weapon-6 arm's anim set [M 0x17a740]");
        assert_eq!(dr.clip, 16, "the weapon-6 arm's clip [M 0x17a744-0x17a750]");
    }
    // The death consequence rides the held byte the recover just wrote: DropWeapon 0x17a2a0
    // keys on anim+0x62 (proven on the real kill path by the 06 test below and the nf-game
    // five_guards pin), and the weapon-6 record is a real droppable gun [L record 6: +2 = 6,
    // +0x80 = 0x0200025a, +0x92 = 16] — so the woke guard's later death drops the weapon-6
    // pickup with 16·(0.25+FRand(0.5)) ∈ [4..=12] rounds. The old port left him on the fists
    // id (model 0) and the death dropped nothing.
    {
        let dr = &scene.sim.as_ref().unwrap().drones.list[gi];
        let wd = &scene.sim.as_ref().unwrap().weapons;
        assert_eq!((wd.base(6), wd.clip_size(6)), (6, 16), "the weapon-6 record the drop will read [L ee_start_05.bin @ 0x2bf150+6·0x10C]");
        assert_ne!(wd.model(6), 0, "the weapon-6 record drops [L record 6 +0x80]");
        let _ = dr;
    }
    eprintln!("[drop] the woke guard re-armed with weapon 6 set 4 clip 16 — his death drop is the sidearm");
}

/// Sister: the TASED guard killed while still DOWN (before the recover) drops NOTHING —
/// the tase ENTER left him on the fists id and `DropWeapon` 0x17a2a0's `weapon_data[+0x80]`
/// gate refuses the fists (model 0 [L record 1]). This is the held-byte side of the same rule
/// the main test pins the re-arm of: the byte that rides into the death is the drop.
#[test]
fn the_tased_guard_killed_while_down_drops_nothing() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let gi = pick_start_guard(&scene);
    let held = scene.sim.as_ref().unwrap().drones.list[gi].weapon as u16;
    assert!(matches!(held, 0x16 | 0x24), "a census weapon [L census]");
    equip(&mut scene, 0x4a);
    place_bond(&mut scene, gi, 2.5, false);
    for _ in 0..40 { steer(&mut scene, gi, false); }
    stun_to_taser(&mut scene, gi);
    let mut stunned_at = None;
    for _ in 0..240u32 {
        steer(&mut scene, gi, false);
        if scene.sim.as_ref().unwrap().drones.list[gi].state == state::STUNNED { stunned_at = Some(()); break; }
    }
    stunned_at.expect("the fall ends into DSTATE_Stunned 0x4b [M 0x162b98 case 0xc]");
    {
        let dr = &scene.sim.as_ref().unwrap().drones.list[gi];
        assert_eq!(dr.weapon, 1, "on the fists through the stun [M 0x162b98:27-34]");
    }
    // Kill the DOWNED body through the real trigger: the low sphere centred +0.3 above the
    // origin [M 0x140e90:65-90] — the re-tase harness's aim.
    equip(&mut scene, 2);
    place_bond(&mut scene, gi, 2.5, false);
    let before = scene.sim.as_ref().unwrap().drones.pickups.len();
    let mut died_at = None;
    for _ in 0..1500u32 {
        steer_at(&mut scene, gi, true, 0.3);
        if is_dead(scene.sim.as_ref().unwrap().drones.list[gi].state) { died_at = Some(()); break; }
    }
    died_at.expect("the downed guard dies under P2K fire");
    for _ in 0..60 { steer(&mut scene, gi, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.pickups.len(), before, "the fists corpse dropped nothing: {:?}",
        sim.drones.pickups.iter().map(|p| (p.weapon, p.rounds, p.kind)).collect::<Vec<_>>());
    eprintln!("[drop] the downed fists corpse dropped nothing [M 0x17a2a0 + the model-0 gate]");
}

/// The plain kill on 07000006 (the task's second level): the death drop is the held gun's own
/// pickup — kind WEAPON, base id, the clip-size rounds band [M 0x17a2a0; L census: every 06
/// guard reads 0x16]. The anim-set-4 keep-arm of the stun gates has no 05/06 carrier (the
/// census), so the plain kill pins the byte path the stun arms read.
#[test]
fn killing_a_six_guard_drops_his_gun_with_the_game_rounds() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000006.bin", false).expect("load 07000006");
    let gi = pick_start_guard(&scene);
    {
        let dr = &scene.sim.as_ref().unwrap().drones.list[gi];
        assert_eq!(dr.weapon, 0x16, "the 06 guard carries the SG5 [L census]");
        assert_eq!(dr.anim_set, 5, "the SG5's anim set [M 0x14b300:283-289 case 0x16]");
    }
    equip(&mut scene, 2);
    place_bond(&mut scene, gi, 3.5, true);
    let before = scene.sim.as_ref().unwrap().drones.pickups.len();
    let mut died_at = None;
    for _ in 0..1500u32 {
        steer_fire_aligned(&mut scene, gi, 0.0);
        if is_dead(scene.sim.as_ref().unwrap().drones.list[gi].state) { died_at = Some(()); break; }
    }
    died_at.expect("the 06 guard dies under P2K fire");
    for _ in 0..60 { steer(&mut scene, gi, false); }
    let sim = scene.sim.as_ref().unwrap();
    let after = sim.drones.pickups.len();
    assert_eq!(after, before + 1, "exactly one drop: {:?}",
        sim.drones.pickups.iter().map(|p| (p.weapon, p.rounds, p.kind)).collect::<Vec<_>>());
    let pk = &sim.drones.pickups[before];
    assert_eq!((pk.weapon, pk.kind), (0x16, nf_game::pickup::kind::WEAPON), "the held SG5's pickup [M 0x17a2a0]");
    assert!((7..=22).contains(&pk.rounds), "30-round clip → 30·(0.25+FRand(0.5)) ∈ [7..=22], got {} [L record 0x16 +0x92 = 30]", pk.rounds);
    eprintln!("[drop] 06 kill dropped weapon {:#x} rounds {}", pk.weapon, pk.rounds);
}
