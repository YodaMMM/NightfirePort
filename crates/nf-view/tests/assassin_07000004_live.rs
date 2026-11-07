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
