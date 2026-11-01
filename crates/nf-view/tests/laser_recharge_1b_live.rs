//! The laser rifle's recharge on 0700001b (Equinox), through the REAL Sim (the Walker harness).
//!
//! The game's rule [M `Player_Update` 0x193600:378-409, disasm 0x193eb8-0x19405c]: with the
//! player in state 1 or 4 and `BL+0x94B == 0`, while the view model's anim state
//! (`BL[0x1FA]+0xF4`) is NOT 9, on `GameState+0x34 & 3 == 0` the clip slot `BL+0x41C`
//! (clip[0x33] = `0x33·0xC + BL + 0x1B8`) gains `FRAME_RATE_MUL` up to `weapon_data[0x33]+0x92`
//! (`weapon_data._13814_2_` = 0x33·0x10C + 0x92). `GameState+0x34` counts one per update
//! [M `GameFlow_Main` 0x1c98c8:27-28] and `FRAME_RATE_MUL` = 60 / FRAME_RATE
//! [M `GS_SetRefreshRate` 0x17d2b8], so the rate is +1 per 4 updates at 60 Hz and +2 per 4 at the
//! 30 Hz step (`main` 0x109080:59-99 picks 1 or 2 vblanks per update): 15 rounds a second either
//! way. The port's fixed 60 Hz step has `FRAME_RATE_MUL` = 1.
//!
//! The pause: a shot enters view state 9 (`Player_SetFiringAnim` 0x1a3c70: +0xDC != 0 and a fire
//! hash → `+0xF4 = 9`, `AnimScriptAdd(+0xB0)` at speed 1); `Player_SetWeaponAnimObj` 0x1a2e38
//! case 9 leaves it only when the script is stopped (+0x68 & 0x400 and 0x100000 clear, +0xDC != 0
//! → `AnimScriptIsStopped` → `+0xF4 = 0`, :438-456). So every tap stops the recharge for the
//! whole fire script. The spend [M `Player_WeaponFiring` 0x1a5460:651-656 →
//! `Player_RoundToFire` 0x1a18e0]: needs and spends `+0x91` from the shared slot (`+0x05` != 0
//! and the +0x90 ammo types match → slot `+0x02` = 0x33). Running dry does not switch away:
//! `Player_HandleHasNoAmmo` 0x1a2ba8 returns for weapons 0x32..0x33.
//!
//! Record 0x33 [D ACTION.ELF `weapon_data` static init, read through `Sim::weapons`]: +0x02 0x33,
//! +0x05 −1, +0x28 999 shots a pull, +0x40 30 (the refire), +0x68 0x12040, +0x90 31, +0x91 12,
//! +0x92 100, +0xB0 0x060007eb (the fire script, 100 frames in the level's lib), +0xB4 0,
//! +0xDC 0x0500007f.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// One traced tick: the clip after the tick and whether the fire script (view state 9) holds.
#[derive(Clone, Copy, Debug)]
struct Tick { clip: u32, state9: bool }

/// Checks a trace against the game's rule: no gain while state 9 holds (or on the tick it ends —
/// the regen reads the state before `Player_Weapon` runs), and otherwise exactly +1 every 4th
/// tick up to `cap`, gains spaced exactly 4 ticks apart.
fn check_regen(trace: &[Tick], cap: u32) -> Result<(), String> {
    let mut last_gain: Option<usize> = None;
    for i in 1..trace.len() {
        let (a, b) = (trace[i - 1], trace[i]);
        if b.clip < a.clip { return Err(format!("tick {i}: the clip fell {} → {} with no shot", a.clip, b.clip)); }
        let d = b.clip - a.clip;
        if d == 0 { continue; }
        if a.state9 { return Err(format!("tick {i}: +{d} while the fire script held")); }
        if d != 1 { return Err(format!("tick {i}: +{d} in one tick (FRAME_RATE_MUL is 1)")); }
        if b.clip > cap { return Err(format!("tick {i}: {} past the cap {cap}", b.clip)); }
        if let Some(g) = last_gain {
            if i - g != 4 { return Err(format!("tick {i}: gains {} ticks apart, not 4", i - g)); }
        }
        last_gain = Some(i);
    }
    // A clip below the cap with no state 9 must still be gaining: 5 idle ticks with no gain fail.
    let mut idle_flat = 0;
    for i in 1..trace.len() {
        if trace[i - 1].state9 || trace[i].clip >= cap { idle_flat = 0; continue; }
        if trace[i].clip == trace[i - 1].clip { idle_flat += 1 } else { idle_flat = 0 }
        if idle_flat > 3 { return Err(format!("tick {i}: below the cap and idle, but no gain in 4 ticks")); }
    }
    Ok(())
}

fn tick(w: &mut Walker, fire: bool) -> Tick {
    if fire { w.tick_fire(Default::default()); } else { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    Tick { clip: sim.gun.clip, state9: sim.gun.fire_state }
}

/// The record bytes, a tap's spend, the full-script pause, +1 per 4 ticks after it, the 100 cap.
#[test]
fn the_laser_rifle_recharges_one_round_per_4_ticks_outside_the_fire_script() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b"));
    w.wait_enabled(3000, "the station start");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.gun.weapon, 0x33, "the level starts on the laser rifle");
        let r = &sim.weapons.records[0x33];
        let u16a = |o: usize| u16::from_le_bytes([r[o], r[o + 1]]);
        let u32a = |o: usize| u32::from_le_bytes(r[o..o + 4].try_into().unwrap());
        assert_eq!((u16a(0x02), r[0x05] as i8, u16a(0x28), u32a(0x40)), (0x33, -1, 999, 30), "0x33's slot, twin, pull and refire [D]");
        assert_eq!((u32a(0x68), r[0x90], r[0x91], u16a(0x92)), (0x12040, 31, 12, 100), "0x33's flags, ammo type, spend and clip [D]");
        assert_eq!((u32a(0xb0), u32a(0xb4), u32a(0xdc)), (0x060007eb, 0, 0x0500007f), "0x33's fire script and the +0xDC case-9 arm [D]");
        assert_eq!(sim.scripts.get(&0x060007eb).map(|s| s.length), Some(100), "the fire script's lib length [D]");
        assert_eq!(sim.gun.clip, 100, "a full clip at the start");
    }
    // Let the draw script settle, then one tap.
    let mut trace = Vec::new();
    for _ in 0..40 { trace.push(tick(&mut w, false)); }
    let before = trace.last().unwrap().clip;
    let shot = tick(&mut w, true);
    assert_eq!(before - shot.clip, 12, "a tap spends +0x91 = 12 [M 0x1a18e0]");
    assert!(shot.state9, "the tap enters state 9 [M 0x1a3c70]");
    // The pause: the fire script's 100 frames, no gain.
    let mut held = 0;
    let mut t = shot;
    let mut after = vec![shot];
    while t.state9 && held < 200 { t = tick(&mut w, false); after.push(t); held += 1; }
    assert!((98..=101).contains(&held), "state 9 holds for the fire script's 100 frames (held {held})");
    assert_eq!(after[after.len() - 2].clip, shot.clip, "no recharge while the fire script holds");
    // The recharge: 12 rounds back in 48 ticks, then on to the cap and no further.
    let start = t.clip;
    for _ in 0..48 { after.push(tick(&mut w, false)); }
    let got = after.last().unwrap().clip - start;
    assert!((11..=12).contains(&got), "+1 per 4 ticks: 12 rounds in 48 ticks (got {got})");
    for _ in 0..40 { after.push(tick(&mut w, false)); }
    assert_eq!(after.last().unwrap().clip, 100, "refilled to +0x92 = 100");
    for _ in 0..8 { after.push(tick(&mut w, false)); }
    assert_eq!(after.last().unwrap().clip, 100, "held at the cap");
    check_regen(&after, 100).unwrap();
}

/// A held trigger refires every +0x40 = 30 ticks, each refire restarting the 100-frame fire
/// script: the clip only ever falls (no recharge between refires) and the dry clip stays on the
/// rifle [M 0x1a2ba8: weapons 0x32..0x33 return].
#[test]
fn a_held_trigger_drains_with_no_recharge_and_stays_on_the_rifle() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b"));
    w.wait_enabled(3000, "the station start");
    for _ in 0..40 { tick(&mut w, false); }
    let mut shots = Vec::new();
    let mut prev = w.scene.sim.as_ref().unwrap().gun.clip;
    for i in 0..300u32 {
        let t = tick(&mut w, true);
        assert!(t.clip <= prev, "tick {i}: the clip rose {prev} → {} under a held trigger", t.clip);
        if t.clip < prev { shots.push(i); }
        prev = t.clip;
    }
    assert_eq!(prev, 100 % 12, "8 shots of 12 leave 4: below +0x91, no ninth");
    assert_eq!(shots.len(), 8, "8 shots from a full clip");
    assert!(shots.windows(2).all(|p| p[1] - p[0] == 30), "the refire is +0x40 = 30 ticks: {shots:?}");
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x33, "a dry rifle is kept [M 0x1a2ba8]");
}

/// Sister check: the rule checker rejects the other plausible rules.
#[test]
fn the_regen_checker_rejects_other_rules() {
    let mk = |v: &[(u32, bool)]| v.iter().map(|&(clip, state9)| Tick { clip, state9 }).collect::<Vec<_>>();
    let ok: Vec<(u32, bool)> = (0..20u32).map(|i| (88 + i / 4, false)).collect();
    assert!(check_regen(&mk(&ok), 100).is_ok(), "the game's rule passes");
    let fast: Vec<(u32, bool)> = (0..20u32).map(|i| (88 + i / 2, false)).collect();
    assert!(check_regen(&mk(&fast), 100).is_err(), "+1 per 2 ticks fails");
    let double: Vec<(u32, bool)> = (0..20u32).map(|i| (88 + 2 * (i / 4), false)).collect();
    assert!(check_regen(&mk(&double), 100).is_err(), "+2 per 4 ticks (the 30 Hz step at 60 Hz) fails");
    let mut during: Vec<(u32, bool)> = (0..20u32).map(|i| (88 + i / 4, true)).collect();
    during[19].1 = false;
    assert!(check_regen(&mk(&during), 100).is_err(), "a gain inside state 9 fails");
    let slow: Vec<(u32, bool)> = (0..20u32).map(|i| (88 + i / 8, false)).collect();
    assert!(check_regen(&mk(&slow), 100).is_err(), "+1 per 8 ticks fails");
    let over: Vec<(u32, bool)> = (0..12u32).map(|i| (99 + i / 4, false)).collect();
    assert!(check_regen(&mk(&over), 100).is_err(), "past the cap fails");
}
