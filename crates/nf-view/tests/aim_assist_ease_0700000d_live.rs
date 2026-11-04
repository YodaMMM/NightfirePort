//! aim-assist EASE pin (0700000d): the aim-mode half of `Player_AutoAim` 0x1a7c98 on real input.
//!
//! The game's law [M Player_AutoAim 0x1a7c98:50-59 + the tail :112-115]: the function runs ONCE
//! per tick (the only `jal Player_AutoAim` in ACTION.ELF sits at the end of `Player_Weapon`
//! 0x1a2b70-279, called from `Player_Update` 0x193600:412), and its two arms read the SAME gate —
//! bit0 of `*(ushort*)(obj+0xdc+0x60)`, the aim bit:
//!   * bit SET (aimed): the assist offsets (+0x120/+0x124) ease toward the MANUAL offsets
//!     (+0x118/+0x11c, `Player_Aiming` 0x1a1d28's cursor accumulation, updated EARLIER the same
//!     tick [M Player_Update 0x193600:282 → :412]) at exactly 0.5 per tick, the `Check_AutoAim`
//!     scan does NOT run (the lock pointer is untouched — it survives a dead or out-of-cone
//!     target), and the offsets do NOT halve;
//!   * bit CLEAR: the `Check_AutoAim` scan runs (auto-aim on), re-snapping the offsets onto the
//!     best target, and ALL FOUR offsets halve ×0.5 — the scan still runs with the button up.
//! The fired round consumes the offsets where they stand after that tick's ease
//! (`Player_GetAimingPoint` 0x1a4a98 reads +0x120/+0x124), so a residual assist carries into the
//! first aimed rounds and a converged one puts the round exactly where the view points.
//!
//! Pinned on the real input path from the level start (no teleports), with the Wolfram P2K 0x02:
//! the one weapon whose record removes both confounds — no aim-raise script (+0xB8 = 0 [L
//! ee_start_05.bin weapon_data 0x2bf150 + 0x02·0x10c + 0xB8], so the aim bit follows the raw
//! button at once [M 0x1a24c8:117-119] and there is no raise window) and zero spread (+0x24 = 0.0,
//! +0x98 = 0.0 [L same record], so the spawned round's direction is the aim direction exactly).
//! The route legs are the walkreal-0d ones to the ladder base; the lock is taken hip-firing
//! (button up), then the pin holds the aim button and asserts, per tick, the exact 0.5 ease toward
//! the centred cursor's manual offsets (0, 0), fires one converged round and asserts its
//! direction equals the view direction, then releases the button and asserts the scan re-snaps
//! (the offsets jump back up) — the sister.

#[path = "common/walker_b2.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn bond_pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

/// The scan's own filter [M Check_AutoAim 0x1a73f8, the port's `auto_aim` drone loop]: drawn,
/// alive, enemy-team drones. Returns (index, dist from the eye), nearest first.
fn lockable(w: &Walker) -> Vec<(usize, f32)> {
    let sim = w.scene.sim.as_ref().unwrap();
    let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
    let mut v: Vec<(usize, f32)> = sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && d.enabled && d.drawn && !d.corpse && d.health > 0.0 && d.team == 1 && d.kind != 0xb)
        .map(|(i, d)| {
            let dv = [d.pos[0] - eye[0], d.pos[1] - eye[1], d.pos[2] - eye[2]];
            (i, (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt())
        })
        .filter(|(_, dist)| *dist < 24.0)
        .collect();
    v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    v
}

#[test]
fn the_aim_assist_eases_half_per_tick_and_the_aimed_round_goes_where_the_view_points() {
    let iso = iso();
    let cold = nf_view::load_with_carry(&iso, "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x23), "the start trigger #436 fires ch 0x23 [D]");

    // Auto Aim is the game's profile default [M Player_AutoAim 0x1a7c98:41; L ee_start_05
    // PlayerSetting+1] — the bare headless load leaves the mirror off.
    w.scene.sim.as_mut().unwrap().auto_aim = true;

    // The Wolfram P2K 0x02 through the panel (the fix-aim-raise give): the real switch input
    // rides the holster/draw cycle to the fully idle machine.
    let (status, sw) = nf_view::panel::apply(&mut w.scene, &[
        ("do".to_string(), "weapon".to_string()),
        ("id".to_string(), "0x02".to_string()),
        ("rounds".to_string(), "60".to_string()),
    ]);
    assert_eq!(sw, Some(0x02), "the give refused: {status}");
    for _ in 0..60 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x02 { break; }
        w.scene.sim.as_mut().unwrap().step_bond(Input { switch_to: Some(0x02), ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x02, "the switch never landed");
    {
        let start = w.total_ticks;
        while w.total_ticks - start < 600 {
            let s = w.scene.sim.as_ref().unwrap();
            if s.gun.switch_state == 0 && !s.gun.fire_state && s.gun.reload_state == 0 { break; }
            w.tick(Input::default(), false);
        }
        let s = w.scene.sim.as_ref().unwrap();
        assert!(s.gun.switch_state == 0 && !s.gun.fire_state && s.gun.reload_state == 0, "the P2K never settled");
    }
    // The walkreal-0d route legs to the ladder base, real input (the crossfire pool is the
    // walker's own top-up here — this pin is about the aim offsets, not survival).
    w.walk_to(Wp::new(180.6, 11.2, -208.8).r(0.9), "the BodyArmourHeavy #525");
    assert!(w.try_walk_to(Wp { at: [181.0, 7.5, -227.5], act: Act::None, arrive: 1.5 }, "the platform drop"), "the drop must reach the ground");
    for (at, r, label) in [
        ([188.0f32, 0.5, -228.0], 2.0, "the blind strip east 1"),
        ([196.0, 0.5, -228.5], 2.0, "the blind strip east 2"),
        ([204.0, 0.5, -229.0], 2.0, "the blind strip east 3"),
        ([212.0, 0.5, -230.0], 1.5, "the blind strip east 4"),
        ([215.0, 0.8, -233.2], 1.2, "east of the crane leg"),
        ([213.2, 1.5, -234.8], 0.8, "at the ladder base"),
    ] {
        assert!(w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label), "{label} must walk");
    }
    let cands = lockable(&w);
    eprintln!("[lock] bond {:?} lockable within 24: {:?}", bond_pos(&w), cands);
    assert!(!cands.is_empty(), "no enemy within the Autoaim_Range 25 at the ladder base — the pin needs a guard in range");

    // HIP-FIRE LOCK (button up): turn toward the nearest lockable guard and stop at a deliberate
    // ~0.04 rad residual so the scan's snap is non-zero; the scan runs every button-up tick and
    // locks the guard (cone + LOS proven by the lock itself).
    let mut t = cands[0].0;
    let mut locked = false;
    let mut v0 = [0.0f32; 2];
    'turn: for _ in 0..600 {
        let err = {
            let sim = w.scene.sim.as_ref().unwrap();
            if !lockable(&w).iter().any(|&(i, _)| i == t) {
                // The pick died or left the range: re-pick the nearest.
                match lockable(&w).first() { Some(&(i, _)) => t = i, None => break 'turn }
            }
            let dp = w.scene.sim.as_ref().unwrap().drones.list[t].pos;
            let desired = (dp[0] - sim.bond.pos[0]).atan2(dp[2] - sim.bond.pos[2]);
            let mut e = desired - sim.bond.yaw;
            while e > std::f32::consts::PI { e -= std::f32::consts::TAU; }
            while e < -std::f32::consts::PI { e += std::f32::consts::TAU; }
            e
        };
        if w.scene.sim.as_ref().unwrap().drones.aim_target == Some(t) { locked = true; break; }
        let turn = if err.abs() < 0.04 { 0.0 } else { err.clamp(-0.08, 0.08) };
        w.tick(Input { turn, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(cur) = sim.drones.aim_target {
            if lockable(&w).iter().any(|&(i, _)| i == cur) { t = cur; locked = true; v0 = sim.drones.aim_offset; break; }
        }
    }
    eprintln!("[lock] target #{t} locked {locked} v0 {:?}", v0);
    assert!(locked, "the hip-fire lock never landed on a guard (button up) [M Check_AutoAim 0x1a73f8]");
    assert!(v0[0].abs() > 0.004, "the lock snap must be non-zero for the ease to be observable, got {:?}", v0);

    // LAW 1 — the ease: hold the aim button (0x02 has no raise script: the bit sets on the press
    // tick) and assert, EVERY tick, the exact 0.5-per-tick ease toward the manual offsets. The
    // cursor is untouched (turn/look 0), so the manual offsets are (0, 0) and each tick's value is
    // v0·0.5^k [M Player_AutoAim 0x1a7c98:52-56, ONE call per tick]. A scanning port would re-snap
    // the offsets back up (the guard is still in the cone) or drop the pointer; a double-easing
    // port converges at 0.25 per tick. The lock pointer must survive untouched.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.aiming, "not aimed before the press");
    }
    for k in 1u32..=10 {
        w.tick(Input { aiming: true, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.aiming, "tick {k}: the aim bit did not set (0x02 has no raise script [M 0x1a24c8:117-119])");
        let want = [v0[0] * 0.5f32.powi(k as i32), v0[1] * 0.5f32.powi(k as i32)];
        let got = sim.drones.aim_offset;
        assert!((got[0] - want[0]).abs() < 1e-7 && (got[1] - want[1]).abs() < 1e-7,
            "tick {k}: the assist offsets must ease toward the manual offsets at 0.5 per tick [M Player_AutoAim 0x1a7c98:52-56 — ONE ease per tick; the only jal Player_AutoAim is Player_Weapon 0x1a2b70], want {want:?} got {got:?}");
        assert_eq!(sim.drones.aim_target, Some(t), "tick {k}: the scan must NOT run while the aim bit is set — the lock pointer survives [M 0x1a7c98:50-59]");
        assert_eq!(sim.cursor, [0.0, 0.0], "tick {k}: the cursor (the manual-offset source) stays centred with no stick input");
    }
    eprintln!("[ease] 10 ticks exact: v0 {v0:?} -> {:?}", w.scene.sim.as_ref().unwrap().drones.aim_offset);

    // LAW 2 — the converged round: fire ONE aimed round; with the offsets eased to v0·0.5^10
    // (~3e-5 rad) and the P2K's zero spread (+0x24 = 0.0 [L weapon_data 0x02 + 0x24]), the round's
    // direction is the view direction [M Player_GetAimingPoint 0x1a4a98 reads the eased offsets].
    let look0 = { let sim = w.scene.sim.as_ref().unwrap(); sim.bond.look() };
    let before = w.scene.sim.as_ref().unwrap().drones.player_bullets.len();
    w.tick_fire(Input { aiming: true, ..Default::default() });
    let mut dir = None;
    {
        let start = w.total_ticks;
        while w.total_ticks - start < 90 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.player_bullets.len() > before {
                dir = Some(sim.drones.player_bullets[sim.drones.player_bullets.len() - 1].dir);
                break;
            }
            w.tick(Input { aiming: true, ..Default::default() }, false);
        }
    }
    let dir = dir.expect("the aimed round never spawned (the P2K's fire path)");
    eprintln!("[round] dir {dir:?} vs view {look0:?}");
    for k in 0..3 {
        assert!((dir[k] - look0[k]).abs() < 2e-3,
            "the converged aimed round must go where the view points [M Player_GetAimingPoint 0x1a4a98 + the P2K's zero spread], component {k}: dir {} vs view {}", dir[k], look0[k]);
    }

    // The settle: the fire script plays out and the round flies, aim held throughout.
    {
        let start = w.total_ticks;
        while w.total_ticks - start < 400 {
            let s = w.scene.sim.as_ref().unwrap();
            if w.total_ticks - start > 30 && !s.gun.fire_state && s.gun.switch_state == 0 { break; }
            w.tick(Input { aiming: true, ..Default::default() }, false);
        }
    }

    // THE SISTER — button UP, the scan still runs: release the aim button; the very next tick the
    // `Check_AutoAim` branch runs and re-snaps the offsets onto the (still alive) guard — the
    // values jump back up from the ~converged residue.
    let pre = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.drones.aim_target, sim.drones.aim_offset, sim.drones.list[t].health)
    };
    eprintln!("[sister] pre {:?} target #{t} hp {:.1}", pre.1, pre.2);
    w.tick(Input::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let (tgt, off) = (sim.drones.aim_target, sim.drones.aim_offset);
        eprintln!("[sister] post target {:?} offsets {tgt:?} -> {off:?}", tgt);
        assert!(off[0].abs() > 0.002,
            "with the button up the scan must re-snap the offsets (they jumped {pre:?} -> {off:?}) [M Player_AutoAim 0x1a7c98:60-109 — the Check_AutoAim branch on the button-up ticks]");
        assert!(tgt.is_some(), "the scan re-locked a guard on the release tick (a guard is still in the cone) [M 0x1a73f8]");
    }
}
