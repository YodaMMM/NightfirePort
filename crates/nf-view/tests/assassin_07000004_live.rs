//! 07000004's assassin and score, read against the game on the real level.
//!
//! - the score: `UpdateMFX_02MayhewD` 0x112bf8 walks the level track through markers 2, 3 and 4 as
//!   the story scene plays [M :41-65];
//! - the fighter: `NDrone2_DefaultInit` 0x14b300:744-753 gives mode 0xd its own accuracy byte and
//!   properties after the mode table has been read [M];
//! - the fight: the flip and sword states take half damage [M 0x16cdd0:39,49 and siblings], the
//!   close band backs off first [M 0x16bb50:48-63], the run-in keeps running on a direct route
//!   [M 0x16cb70:51-61], the flips hold their heading [M 0x16cdd0 case 3, no SetAngleToObj].

#[path = "common/walker.rs"]
mod walker;

use nf_game::kiko;
use nf_game::player::{Bond, Input};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The real load, through the intro and the story scene, with the markers the score asked for on
/// the way (each change of `MusicLastJumpRequest`).
fn settle(markers: &mut Vec<i32>) -> Walker { settle_with(markers, true) }

fn settle_with(markers: &mut Vec<i32>, skip: bool) -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "07000004.bin", false).expect("load 07000004"));
    w.skip_cutscenes = skip;
    for _ in 0..6000u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let r = sim.drones.music.last_request;
        if markers.last() != Some(&r) { markers.push(r); }
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { return w; }
    }
    panic!("the intro and story scenes settle");
}

fn ninja(w: &Walker) -> usize {
    w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| d.present && d.dtype == 0xf).expect("the assassin [D]")
}

/// Bond on open ground beside her, at `gap` along +x, and alive for the test.
fn stand_by(w: &mut Walker, ni: usize, gap: f32) {
    let sim = w.scene.sim.as_mut().unwrap();
    let p = sim.drones.list[ni].pos;
    let yaw = sim.bond.yaw;
    sim.bond = Bond::new([p[0] + gap, sim.bond.pos[1], p[2]], yaw);
    sim.drones.health.health = 100.0;
}

/// The story scene carries the score: its start jumps marker 2, the first 0x172 line marker 3,
/// the second marker 4, which is what plays through the fight [M 0x112bf8:41-65; D 0x06000082's
/// 0x172 lines at ticks 546 and 710].
#[test]
fn the_story_scene_walks_the_score_to_the_fight_section() {
    let mut markers = Vec::new();
    let w = settle_with(&mut markers, false);
    eprintln!("[04] markers {markers:?}");
    let want: Vec<i32> = markers.iter().copied().filter(|m| [2, 3, 4].contains(m)).collect();
    assert_eq!(want, vec![2, 3, 4], "the story scene's markers in order, saw {markers:?}");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.music.last_request, 4, "the fight runs on marker 4");
}

/// Skipping the scenes lands on the same section: the skip event takes the score to marker 4 once
/// the story scene has started [M 0x112bf8:46-49, 0x1dac78:55].
#[test]
fn skipping_the_scenes_still_reaches_the_fight_section() {
    let mut markers = Vec::new();
    let w = settle(&mut markers);
    eprintln!("[04] skipped markers {markers:?}");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.music.last_request, 4, "saw {markers:?}");
}

/// Mode 0xd's block runs after `DoModeSettingsNEW`, so the assassin fires with accuracy byte 0 —
/// `100 − 5·0` before the tuning factors — and carries properties 0x35/0x36/0x3c
/// [M 0x14b300:501 then :744-753].
#[test]
fn the_assassin_has_her_own_init_block() {
    let w = settle(&mut Vec::new());
    let ni = ninja(&w);
    let sim = w.scene.sim.as_ref().unwrap();
    let dr = &sim.drones.list[ni];
    assert_eq!(dr.mode, 0xd);
    assert_eq!(dr.accuracy, 0, "+0xB4 = 0 [M 0x14b300:752]");
    for p in [0x35, 0x36, 0x3c] {
        assert_ne!(dr.prop_value(&sim.props, p), 0, "property {p:#x} [M 0x14b300:746-748]");
    }
}

/// Fires at her with her held in `state` until one round lands, and returns the health it took
/// with the hit part.
fn one_round(w: &mut Walker, ni: usize, state: u8) -> (f32, i16) {
    w.scene.sim.as_mut().unwrap().drones.list[ni].health = 100.0;
    let spot = w.scene.sim.as_ref().unwrap().drones.list[ni].pos;
    for t in 0..1200u32 {
        let aim = w.exposed_aim(ni);
        let (input, on, hp) = {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.health.health = 100.0;
            sim.gun.spare = sim.gun.spare.max(30);
            let bp = sim.bond.pos;
            let dr = &mut sim.drones.list[ni];
            dr.state = state;
            dr.next_state = None;
            dr.pos = spot;
            let at = aim.unwrap_or([spot[0], spot[1] + 0.9, spot[2]]);
            let (dx, dz) = (at[0] - bp[0], at[2] - bp[2]);
            let dy = at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
            let err = walker::wrap_pi(dx.atan2(dz) - sim.bond.yaw);
            let perr = (dy / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch;
            (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
                err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0, sim.drones.list[ni].health)
        };
        if on { w.tick_fire(input); } else { w.tick(input, false); }
        let dr = &w.scene.sim.as_ref().unwrap().drones.list[ni];
        if dr.health < hp { return (hp - dr.health, dr.hit.map_or(0, |h| h.part)); }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    panic!("no round landed on her in state {state:#x}: weapon {} clip {} spare {} bond {:?} yaw {} pitch {} her {:?}", sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.bond.pos, sim.bond.yaw, sim.bond.pitch, spot);
}

/// The same round to the same part takes half as much mid-flip or mid-swing: those five states
/// call `NDrone2_DoHitEffects` with 0.5, the others with 1.0 [M 0x16cdd0:39,49, 0x16cf68:39,49,
/// 0x16d100:44,54, 0x16d298:35,45, 0x16d430:35,45 against 0x16d5b8's 1.0].
#[test]
fn a_flip_or_a_swing_halves_the_round() {
    let mut w = settle(&mut Vec::new());
    let ni = ninja(&w);
    w.switch_to_armed("the round");
    // Open lawn west of the pond [the probe's fight ground]: she stands at the spot, Bond 4 north.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.list[ni].pos = [-9.5, 5.34, 33.0];
        let yaw = std::f32::consts::PI;
        sim.bond = Bond::new([-9.5, 5.4, 37.0], yaw);
    }
    w.wait(20);
    let mut full = std::collections::BTreeMap::new();
    for _ in 0..12 { let (loss, part) = one_round(&mut w, ni, kiko::NINJA_STAND_FIRE); full.insert(part, loss); }
    for st in [kiko::NINJA_SWORD, kiko::NINJA_SOMERSAULT, kiko::NINJA_BACKFLIP, kiko::NINJA_SIDEFLIP_LEFT, kiko::NINJA_SIDEFLIP_RIGHT] {
        let mut compared = false;
        for _ in 0..12 {
            let (loss, part) = one_round(&mut w, ni, st);
            if let Some(f) = full.get(&part) {
                eprintln!("[04] state {st:#x} part {part:#x} took {loss} of {f}");
                assert!((loss - f * 0.5).abs() < 1e-3, "state {st:#x} part {part:#x}: {loss} is not half of {f}");
                compared = true;
                break;
            }
        }
        assert!(compared, "state {st:#x}: a round to a compared part");
    }
}

/// Entering NinjaAttack inside the 2.0 band with room behind her, she backflips: the 10% sideflip
/// roll falls through to the −5 probe, and only a blocked probe reaches the sword-or-fire coin
/// [M 0x16bb50:48-77]. Twenty entries, every one a backflip or the sideflip roll.
#[test]
fn inside_the_band_she_backs_off_before_she_swings() {
    let mut w = settle(&mut Vec::new());
    let ni = ninja(&w);
    let mut swung = 0;
    let mut flipped = 0;
    for _ in 0..20 {
        stand_by(&mut w, ni, 1.2);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            let dr = &mut sim.drones.list[ni];
            let to_bond = 1.5707964f32; // Bond on +x: she faces him, her back to the open −x
            dr.yaw = to_bond;
            dr.desired_yaw = to_bond;
            dr.state = kiko::NINJA_STAND_FIRE;
            dr.next_state = Some(kiko::NINJA_ATTACK);
        }
        w.tick(Input::default(), false);
        let st = w.scene.sim.as_ref().unwrap().drones.list[ni].state;
        match st {
            kiko::NINJA_SWORD | kiko::NINJA_STAND_FIRE => swung += 1,
            kiko::NINJA_BACKFLIP | kiko::NINJA_SIDEFLIP | kiko::NINJA_SIDEFLIP_LEFT | kiko::NINJA_SIDEFLIP_RIGHT => flipped += 1,
            s => eprintln!("[04] entry routed to {s:#x}"),
        }
    }
    eprintln!("[04] twenty entries: {flipped} evasions, {swung} swings or stands");
    assert!(flipped >= 18 && swung == 0, "{flipped} evasions, {swung} swings");
}

/// On a direct route the run-in plays the advance anim 0x67 and closes: case 2 falls through into
/// the moving arm [M 0x16cb70:51-61].
#[test]
fn the_run_in_advances_on_a_direct_route() {
    let mut w = settle(&mut Vec::new());
    let ni = ninja(&w);
    stand_by(&mut w, ni, 4.5);
    {
        let dr = &mut w.scene.sim.as_mut().unwrap().drones.list[ni];
        dr.next_state = Some(kiko::NINJA_GET_CLOSE);
    }
    let d0 = w.scene.sim.as_ref().unwrap().drones.list[ni].dist;
    let mut advanced = 0;
    let mut close = 0;
    for _ in 0..60 {
        w.scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        w.tick(Input::default(), false);
        let dr = &w.scene.sim.as_ref().unwrap().drones.list[ni];
        if dr.state == kiko::NINJA_GET_CLOSE {
            close += 1;
            if dr.anim.state == 0x67 { advanced += 1; }
        }
    }
    let dr = &w.scene.sim.as_ref().unwrap().drones.list[ni];
    eprintln!("[04] run-in: {close} ticks in the state, {advanced} on the advance anim, dist {d0:.2} -> {:.2}", dr.dist);
    assert!(close > 5, "the run-in holds");
    assert!(advanced * 2 > close, "the advance anim plays: {advanced} of {close}");
}

/// A flip holds the heading it left on: only the somersault of the four evasions calls
/// `NDrone2_SetAngleToObj` [M 0x16d298 case 3 against 0x16cdd0, 0x16cf68, 0x16d430 case 3].
#[test]
fn a_flip_holds_its_heading() {
    let mut w = settle(&mut Vec::new());
    let ni = ninja(&w);
    stand_by(&mut w, ni, 5.0);
    for st in [kiko::NINJA_SIDEFLIP_LEFT, kiko::NINJA_SIDEFLIP_RIGHT, kiko::NINJA_BACKFLIP] {
        {
            let dr = &mut w.scene.sim.as_mut().unwrap().drones.list[ni];
            dr.next_state = Some(st);
        }
        w.tick(Input::default(), false);
        let held = w.scene.sim.as_ref().unwrap().drones.list[ni].desired_yaw;
        stand_by(&mut w, ni, -5.0); // Bond jumps to her far side mid-flip
        for _ in 0..6 {
            w.tick(Input::default(), false);
            let dr = &w.scene.sim.as_ref().unwrap().drones.list[ni];
            if dr.state != st { break; }
            assert_eq!(dr.desired_yaw, held, "state {st:#x} keeps its heading");
        }
        stand_by(&mut w, ni, 5.0);
    }
}

/// handback-opp-info: the intro's release reaches the assassin with Bond at the hand-back spot.
/// [L PCSX2 07000004 d1, drone 0x1b45420]: she holds 0xa9 through the intro; one frame after the
/// `Drone_EnableAll(1)` her distance reads 1.25 (Bond at (-3.87, 5.46, 30.30)) and she goes
/// straight to the backflip 0xae with obj+0x54 unchanged at -1.572, landing EAST at
/// (1.76, 5.31, 30.09) [the 30-frame trace]. The port used to read the pre-scene Bond (11.88 u),
/// run 0xa9's MoveToObject toward his intro spot and turn the flip WEST, onto Bond (-5.50, 28.15).
#[test]
fn the_release_reads_the_hand_back_bond_and_she_flips_away() {
    let iso = iso();
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000004.bin", false, None, 1).expect("load 07000004");
    let mut first: Option<(u8, f32, f32)> = None;
    let mut landed = None;
    for _ in 0..1600u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let n = sim.drones.list.iter().find(|d| d.present && d.dtype == 0xf).expect("the assassin [D]");
        if first.is_none() && n.state != 0 { first = Some((n.state, n.dist, n.yaw)); }
        if first.is_some() && landed.is_none() && n.state != kiko::NINJA_BACKFLIP { landed = Some(n.pos); }
    }
    let (st, dist, yaw) = first.expect("the assassin wakes after the intro");
    eprintln!("[04] first update: state {st:#x} dist {dist:.2} yaw {yaw:.3}; landed {landed:.2?}");
    assert_eq!(st, kiko::NINJA_BACKFLIP, "her first update after the release is the backflip, like the game [L]");
    assert!((dist - 1.25).abs() < 0.2, "she reads Bond at the hand-back spot: dist {dist} (game 1.25, pre-scene Bond 11.88) [L]");
    assert!((yaw + 1.572).abs() < 0.05, "the flip keeps her heading: yaw {yaw} (game -1.572) [L; M 0x16d430 sets no angle]");
    let p = landed.expect("the flip ends");
    assert!(p[0] > 1.0 && (p[2] - 30.08).abs() < 0.6, "she lands east, away from Bond: {p:?} (game (1.76, 30.09)) [L]");
    // Sister: the pre-fix landing spot fails this pin.
    assert!(!([-5.50f32, 5.34, 28.15][0] > 1.0), "the old west landing is rejected");
}

/// Her ranged rounds are weapon 0x14, not the placement's 0x12: the weapon switch's 0x12/0x14/0x15
/// arm gives a kind-0xB body weapon 0x14 [M NDrone2_DefaultInit 0x14b300:294-300]. Live, at
/// difficulty 1 every one of her ranged hits costs Bond 1.25 (raw 2.5 × the level's 0.5) and the
/// sword 25 [L PCSX2 07000004 d1, `Drone_ModBulletDamage` 0x139f50 entry: weapon 0x14 raw 2.5
/// mul 1.0 at 5.0-6.2 u; weapon 0x63 raw 50 in 0xac].
#[test]
fn her_ranged_rounds_cost_what_the_games_do() {
    let iso = iso();
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000004.bin", false, None, 1).expect("load 07000004");
    let hp = 100.0f32;
    let (mut ranged, mut sword) = (Vec::new(), Vec::new());
    for _ in 0..6000u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_mut().unwrap();
        let n = sim.drones.list.iter().find(|d| d.present && d.dtype == 0xf).expect("the assassin [D]");
        assert_eq!(n.weapon, 0x14, "her held weapon is 0x14 [L anim+0x62 / the round's weapon]");
        let h = sim.drones.health.health;
        if h < hp { if n.state == kiko::NINJA_SWORD { sword.push(hp - h) } else { ranged.push(hp - h) } }
        // Bond stays alive for the whole exchange.
        sim.drones.health.health = 100.0;
        if ranged.len() >= 4 { break; }
    }
    eprintln!("[04] ranged {ranged:?} sword {sword:?}");
    assert!(!ranged.is_empty(), "she lands ranged rounds in the opening fight");
    assert!(ranged.iter().all(|d| (d - 1.25).abs() < 1e-3), "each ranged hit costs 1.25 at d1 [L]: {ranged:?}");
    assert!(sword.iter().all(|d| (d - 25.0).abs() < 1e-3), "each sword hit costs 25 [L]: {sword:?}");
    // Sister: the old 0x12 round (raw 1.5 × 0.5) is rejected.
    assert!(!((0.75f32 - 1.25).abs() < 1e-3));
}
