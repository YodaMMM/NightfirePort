//! 07000006 fire-path repro (level-06-castle-fire): the walked route's fight wedge, driven through
//! the real input path on the real level — the same carry, the same walked lane to the guardhouse
//! courtyard, and the same real fire path (`Sim::step_bond` trigger → `Gun::update_with` → the shot
//! block) the walkreal fights use.
//!
//! The measured faults (walkreal_07000006_live's doc comment, probe logs): on the FIRST press the
//! round spawns on the press tick but the body ends the tick turned −1.38 rad from the aimed yaw
//! (bond.yaw −3.3134 vs the sampled −1.9311), and every later press — taps, held fire, idle-gated
//! presses — spawns nothing (12 presses → 1 bullet).
//!
//! The game's rules pinned here:
//! • Nothing turns the body on the press tick. The fire path writes no body yaw —
//!   `Player_WeaponFiring` 0x1a5460's fire branch spends the round and calls
//!   `Player_WeaponInitBullet` (the bullet flies from the eye along the view); `Player_SetFiringAnim`
//!   0x1a3c70 only queues the first-person fire script (view state 9); `Player_SetWeaponAnimObj`
//!   0x1a2e38 case 9 is the view-state machine. The body's yaw moves only from the stick
//!   (`Player_Update`'s walk turn) — and this test steers nothing on the press tick.
//! • A semi-automatic fires once per fresh press while the refire interval has run out:
//!   the fresh-press edge (`Input_Action(9,4)`) arms `shotsLeftPull = +0x28` when `+0x92C <= 0`
//!   [M 0x1a5460:601-655]; the P2K (0x02) record is spp 1 / interval 8 [L ee_start_05.bin
//!   weapon_data @ 0x2bf150, stride 0x10C]. 12 presses ≥ 8 ticks apart → 12 rounds.
//! • A held semi-auto does NOT re-fire: the pull is armed only on the press edge [M 0x1a5460
//!   fresh-press edge; L the P2K's spp = 1]. Held fire → exactly 1 round.
//!
//! Opt-in via NF_ISO (the slim ISO is enough). NF_06FIRE=1 prints the per-press state dump.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, RamSave};
use walker::{dxz, Walker, Wp};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The save the truck route's Trigger_Activate would make at 07000005's exit — the walkreal's own
/// carry [M 0x1b2318:108-116: ch 0x1D selects the truck-ride start #496].
fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 240), &ch)
}

/// The walked lane from the truck-ride spawn to just short of the idle guard at
/// (27.51, 39.18, −10.19) [D census: placement #438] — the walkreal's own proven legs, stopped
/// before the guard's body-block.
fn walk_to_the_guard_site(w: &mut Walker) {
    w.wait_enabled(3000, "the intro NIS");
    w.wait(30);
    w.walk_nav_to([60.0, 38.3, 31.5], "off the truck, south", 1.6);
    w.walk_nav_to([45.0, 38.3, 24.0], "across the greeting plates", 1.6);
    w.walk_nav_to([24.3, 38.3, 27.6], "the dock's west end", 1.6);
    w.walk_nav_to([24.6, 38.5, 21.5], "down the west lane", 1.6);
    w.walk_nav_to([26.5, 39.1, 13.1], "the well courtyard's west side", 1.6);
    w.walk_to(Wp::new(23.2, 39.1, 7.0), "the hedge gap");
    w.walk_to(Wp::new(27.2, 39.1, -2.4), "around the hedge end");
    // Stop ~5 u short of the guard's placement — inside the fight's 8-u engagement band.
    w.walk_to(Wp::new(31.0, 39.1, -6.5), "before the idle guard");
}

/// The live team-1 guard nearest the #438 placement.
fn guard_near(w: &Walker) -> usize {
    let sim = w.scene.sim.as_ref().unwrap();
    (0..sim.drones.list.len()).filter(|&i| {
        let d = &sim.drones.list[i];
        d.present && d.team == 1 && !d.corpse && d.health > 0.0
    }).min_by(|&a, &b| {
        let da = dxz(sim.drones.list[a].pos, [27.51, 39.18, -10.19]);
        let db = dxz(sim.drones.list[b].pos, [27.51, 39.18, -10.19]);
        da.total_cmp(&db)
    }).expect("the idle guard at the guardhouse courtyard lane [D #438]")
}

fn trace(w: &Walker, when: &str) {
    if std::env::var_os("NF_06FIRE").is_none() { return; }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[06fire] {when}: t{} weapon {:#x} clip {} spare {} yaw {:+.4} pitch {:+.4} pos {:?} view {:08x}@{:.1} speed {} fire_state {} reload {} switch {} latch {} held(id) disabled {} death {} bullets {}",
        w.total_ticks, sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.bond.yaw, sim.bond.pitch, sim.bond.pos,
        sim.view_script, sim.view_time, sim.view_speed, sim.gun.fire_state as u8, sim.gun.reload_state,
        sim.gun.switch_state, sim.gun.latch as u8, sim.disabled, sim.drones.death.active,
        sim.drones.player_bullets.len());
}

/// Aim at the guard's exposed part for `ticks` (the view steer only, no trigger), then report.
fn aim_at(w: &mut Walker, di: usize, ticks: u32) -> f32 {
    for _ in 0..ticks {
        let sim = w.scene.sim.as_ref().unwrap();
        let at = w.exposed_aim(di).unwrap_or_else(|| {
            let dp = sim.drones.list[di].pos;
            [dp[0], dp[1] + 0.3, dp[2]]
        });
        let bp = sim.bond.pos;
        let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let dy = at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let want_pitch = (dy / dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
        w.tick(input, false);
        trace(w, "aim");
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let dp = sim.drones.list[di].pos;
    let at = w.exposed_aim(di).unwrap_or([dp[0], dp[1] + 0.3, dp[2]]);
    let bp = sim.bond.pos;
    let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
    walker::wrap_pi(desired - sim.bond.yaw)
}

/// One press tick (trigger down, no steer — the aim is held), then `idle` aim-held ticks.
fn press(w: &mut Walker, di: usize, idle: u32) -> (bool, f32, f32) {
    let (yaw0, bullets0) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.bond.yaw, sim.drones.player_bullets.len())
    };
    // The press tick: no turn/look input at all — any body turn this tick is the engine's.
    let sim = w.scene.sim.as_ref().unwrap();
    let dp = sim.drones.list[di].pos;
    let at = w.exposed_aim(di).unwrap_or([dp[0], dp[1] + 0.3, dp[2]]);
    let bp = sim.bond.pos;
    let dy = at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
    let want_pitch = (dy / dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
    let perr = want_pitch - sim.bond.pitch;
    let input = Input { look: perr.clamp(-0.08, 0.08), ..Default::default() };
    w.tick(input, false);
    let fired_tick = w.scene.sim.as_ref().unwrap().shot_tick;
    let yaw1 = w.scene.sim.as_ref().unwrap().bond.yaw;
    trace(w, "PRESS");
    for _ in 0..idle {
        let sim = w.scene.sim.as_ref().unwrap();
        let dp = sim.drones.list[di].pos;
        let at = w.exposed_aim(di).unwrap_or([dp[0], dp[1] + 0.3, dp[2]]);
        let bp = sim.bond.pos;
        let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let dy = at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let want_pitch = (dy / dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
        w.tick(input, false);
    }
    let bullets1 = w.scene.sim.as_ref().unwrap().drones.player_bullets.len();
    (fired_tick, yaw1 - yaw0, (bullets1 - bullets0) as f32)
}

/// 12 real trigger presses, 20 ticks apart (the walk's own tap spacing), each counted by the
/// shot flag on its press tick AND by the bullets in flight after the tick.
#[test]
fn twelve_presses_spawn_twelve_rounds_and_nothing_turns_the_body() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false,
        Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006"));
    walk_to_the_guard_site(&mut w);
    let di = guard_near(&w);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[06fire] guard {di} at {:?} health {:.0} state {:#x} dist {:.1}",
            sim.drones.list[di].pos, sim.drones.list[di].health, sim.drones.list[di].state,
            dxz(sim.drones.list[di].pos, sim.bond.pos));
    }
    assert!(w.switch_to_armed("the repro"), "an armed weapon cycles in");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.gun.weapon, 2, "the carried P2K is the armed weapon [L level_start: P2K 7+41]");
        assert!(sim.gun.clip >= 12, "the clip holds the 12 presses' rounds");
    }
    // Aim on the guard; the aim-hold ticks settle the view before the first press.
    let err = aim_at(&mut w, di, 60);
    trace(&w, "aimed");
    assert!(err.abs() < 0.02, "the view is on the guard (err {err:+.4})");

    // ── Pin 1: the press tick turns the body nothing ──
    // [M the fire branch writes no body yaw: Player_WeaponFiring 0x1a5460 (spend + the spawn),
    // Player_SetFiringAnim 0x1a3c70 (the view script only), Player_SetWeaponAnimObj 0x1a2e38
    // case 9 (the view state); the walkreal probe measured the port turning −1.38 rad here.]
    let (fired0, dyaw0, _) = press(&mut w, di, 20);
    assert!(fired0, "the first press spawns on the press tick (the walkreal's own measurement)");
    assert!(dyaw0.abs() < 0.02, "the press tick turns the body nothing [M 0x1a5460: no body-yaw write], got {dyaw0:+.4}");

    // ── Pin 2: N presses → N rounds ──
    // [M Player_WeaponFiring 0x1a5460:601-655: the fresh-press edge re-arms the pull when the
    // refire timer has run out; L the P2K record: spp 1, interval 8 → a press every 20 ticks
    // fires every time. The walkreal probe measured 12 presses → 1 bullet.]
    let mut shots = 0u32;
    let mut starved = Vec::new();
    for k in 0..11 {
        let (fired, dyaw, _) = press(&mut w, di, 20);
        if fired { shots += 1; } else { starved.push((k + 2, dyaw)); }
        trace(&w, &format!("press {} done", k + 2));
    }
    eprintln!("[06fire] 12 presses → {} rounds (first press included); starved: {starved:?}", shots + 1);
    assert!(dyaw_starved_none(&starved), "no press turns the body [M 0x1a5460], starved: {starved:?}");
    assert_eq!(shots + 1, 12, "12 presses ≥ 8 ticks apart fire 12 rounds [M 0x1a5460:601-655; L P2K spp 1, interval 8]");
}

fn dyaw_starved_none(starved: &[(usize, f32)]) -> bool { starved.iter().all(|(_, d)| d.abs() < 0.02) }

/// Sister: held fire with the semi-automatic P2K is ONE round — the pull is armed on the press
/// edge only [M 0x1a5460:601-655; L the P2K's spp = 1], and the hold turns the body nothing.
#[test]
fn held_fire_with_the_semi_auto_is_one_round() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false,
        Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006"));
    walk_to_the_guard_site(&mut w);
    let di = guard_near(&w);
    assert!(w.switch_to_armed("the held repro"), "an armed weapon cycles in");
    let err = aim_at(&mut w, di, 60);
    assert!(err.abs() < 0.02, "the view is on the guard (err {err:+.4})");
    let yaw0 = w.scene.sim.as_ref().unwrap().bond.yaw;
    let mut shots = 0u32;
    for _ in 0..90 {
        // The trigger HELD, the aim held (the walkreal's tick_fire shape).
        let sim = w.scene.sim.as_ref().unwrap();
        let dp = sim.drones.list[di].pos;
        let at = w.exposed_aim(di).unwrap_or([dp[0], dp[1] + 0.3, dp[2]]);
        let bp = sim.bond.pos;
        let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let dy = at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let want_pitch = (dy / dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
        w.tick(input, false);
        if w.scene.sim.as_ref().unwrap().shot_tick { shots += 1; }
    }
    let dyaw = w.scene.sim.as_ref().unwrap().bond.yaw - yaw0;
    eprintln!("[06fire] 90 held ticks → {shots} rounds, total yaw drift {dyaw:+.4}");
    assert_eq!(shots, 1, "held fire with the semi-auto fires the press tick's one round [M 0x1a5460:601-655; L P2K spp 1]");
    // The guard's return fire and the harness's aim ticks are the only yaw sources here —
    // nothing in the fire path may turn the body. (The engine must not have jumped the yaw.)
    assert!(dyaw.abs() < 1.0, "the hold turns the body nothing beyond steering [M 0x1a5460], got {dyaw:+.4}");
}
