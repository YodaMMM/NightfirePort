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
    // The walkreal's own guardhouse-court anchor — 6.3 u from the guard's placement, inside the
    // fight's 8-u band. (A direct line to a synthetic stop closer to the guard wedges on the
    // lane the real route threads around.)
    w.walk_to(Wp::new(33.7, 39.1, -8.9), "the guardhouse court");
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
    w.tick_fire(input); // the trigger DOWN for exactly this one tick
    let fired_tick = w.scene.sim.as_ref().unwrap().shot_tick;
    let yaw1 = w.scene.sim.as_ref().unwrap().bond.yaw;
    trace(w, "PRESS");
    for _ in 0..idle {
        aim_tick(w, di);
    }
    let bullets1 = w.scene.sim.as_ref().unwrap().drones.player_bullets.len();
    (fired_tick, yaw1 - yaw0, (bullets1 - bullets0) as f32)
}

/// One aim-held tick (the view steer toward the guard's exposed part, no trigger).
fn aim_tick(w: &mut Walker, di: usize) {
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

/// The real R key while the clip is dry, then wait the reload out (the game's reload: state 6's
/// +0xA4 at 1.35x, `reload_ticks` [M Player_WeaponFiring's shared tail 0x1a6250-0x1a631c]).
/// The auto-reload arms only from the fully idle view (`0xf4 == 0` [M 0x1a5460:447-450]) — wait
/// the playing fire script out first (the P2K's 0x06000823 is 24 ticks [L weapon_anims_live]).
fn reload_when_dry(w: &mut Walker) {
    {
        let sim = w.scene.sim.as_ref().unwrap();
        if !(sim.gun.clip == 0 && sim.gun.spare > 0) { return; }
    }
    for _ in 0..150 {
        if w.scene.sim.as_ref().unwrap().gun.fire_state { aim_tick(w, di_of(w)); } else { break; }
    }
    w.tick(Input::default(), false); // release the trigger: the reload arms on the released tick
    w.tick(Input::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.gun.reload_state != 0, "the reload started from the released trigger [M 0x1a5460 release arm] (fire_state {})", sim.gun.fire_state);
    }
    for _ in 0..300 {
        aim_tick(w, di_of(w));
        if w.scene.sim.as_ref().unwrap().gun.reload_state == 0 { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.gun.reload_state == 0 && sim.gun.clip > 0, "the reload completed (state {}, clip {})", sim.gun.reload_state, sim.gun.clip);
}

fn di_of(w: &Walker) -> usize {
    let sim = w.scene.sim.as_ref().unwrap();
    (0..sim.drones.list.len()).filter(|&i| {
        let d = &sim.drones.list[i];
        d.present && d.team == 1 && !d.corpse && d.health > 0.0
    }).min_by(|&a, &b| {
        let da = dxz(sim.drones.list[a].pos, [27.51, 39.18, -10.19]);
        let db = dxz(sim.drones.list[b].pos, [27.51, 39.18, -10.19]);
        da.total_cmp(&db)
    }).expect("the guard stays alive through the repro (10 hp dies to one hit — the presses starve instead)")
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
        assert!(sim.gun.clip + sim.gun.spare >= 12, "the P2K holds the 12 presses' rounds [L level_start: 7+41]");
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
        reload_when_dry(&mut w);
        let di = di_of(&w);
        let (fired, dyaw, _) = press(&mut w, di, 20);
        if fired { shots += 1; } else { starved.push((k + 2, dyaw)); }
        trace(&w, &format!("press {} done", k + 2));
    }
    eprintln!("[06fire] 12 presses → {} rounds (first press included); starved: {starved:?}", shots + 1);
    assert!(starved.iter().all(|(_, d)| d.abs() < 0.02), "no press turns the body [M 0x1a5460], starved: {starved:?}");
    assert_eq!(shots + 1, 12, "12 presses ≥ 8 ticks apart fire 12 rounds [M 0x1a5460:601-655; L P2K spp 1, interval 8]");
}

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
        w.tick_fire(input);
        if w.scene.sim.as_ref().unwrap().shot_tick { shots += 1; }
    }
    let dyaw = w.scene.sim.as_ref().unwrap().bond.yaw - yaw0;
    eprintln!("[06fire] 90 held ticks → {shots} rounds, total yaw drift {dyaw:+.4}");
    assert_eq!(shots, 1, "held fire with the semi-auto fires the press tick's one round [M 0x1a5460:601-655; L P2K spp 1]");
    // The guard's return fire and the harness's aim ticks are the only yaw sources here —
    // nothing in the fire path may turn the body. (The engine must not have jumped the yaw.)
    assert!(dyaw.abs() < 1.0, "the hold turns the body nothing beyond steering [M 0x1a5460], got {dyaw:+.4}");
}

/// The walkreal's own fight sequence at the searchlight courtyard: the walked route through the
/// guardhouse (the idle guard #438 dies through the real fire path), doors #47/#59, then the
/// entry plates stage SP #808's boot NIS — the fight the walkreal wedges on. The same press
/// battery runs here: presses fire every time and turn the body nothing.
#[test]
fn the_searchlight_courtyard_fight_fires_every_press() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false,
        Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006"));
    walk_to_the_guard_site(&mut w);
    assert!(w.switch_to_armed("the mouth repro"), "an armed weapon cycles in");
    // The idle guard #438 dies through the real fire path (10 hp [D census]: one aimed round).
    let di = guard_near(&w);
    assert!(w.shoot(di, "the guardhouse guard"), "the guardhouse guard dies to the real fire path");
    // Through door #47 into the guardroom, out of the auto-closing door #59, to the mouth.
    {
        let (pos, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [36.5, 40.0, -16.25])
            .expect("door #47 Door_Top_I1 in the doors list [D census]");
        eprintln!("[06fire] door {di} at {pos:?}");
        w.use_door(di, "door #47");
    }
    w.walk_nav_to([29.9, 40.5, -17.5], "the guardroom", 1.4);
    w.walk_nav_to([27.9, 40.0, -21.0], "before door #59", 1.4);
    {
        let (pos, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [27.88, 40.0, -23.0])
            .expect("door #59 Door_Top_I2 in the doors list [D census]");
        w.use_door(di, "door #59");
    }
    // THROUGH the doorway (the auto-closing door #59), then west to the courtyard floor.
    w.walk_to(Wp::new(26.6, 40.6, -23.6), "through door #59");
    // West up the steps to the searchlight courtyard mouth (the entry plates #525/#526 stage
    // SP #808's boot NIS [D #525/#526 → 0x52]; the walker skips it with the game's own action).
    w.walk_nav_to([8.4, 42.0, -12.0], "the searchlight courtyard mouth", 1.6);
    w.wait_enabled(3000, "the boot NIS");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[06fire] at the mouth: ch52 {} ch55 {} yaw {:+.4} pos {:?} weapon {:#x} clip {} spare {}",
            sim.drones.channels.on(52) as u8, sim.drones.channels.on(55) as u8,
            sim.bond.yaw, sim.bond.pos, sim.gun.weapon, sim.gun.clip, sim.gun.spare);
        assert!(sim.drones.channels.on(52), "the entry plates staged the boot NIS [D #525/#526 out 0x52]");
    }
    // The fight: aim at the nearest live guard and run the press battery.
    let sim = w.scene.sim.as_ref().unwrap();
    let di = (0..sim.drones.list.len()).filter(|&i| {
        let d = &sim.drones.list[i];
        d.present && d.team == 1 && !d.corpse && d.health > 0.0
    }).min_by(|&a, &b| {
        let da = dxz(sim.drones.list[a].pos, sim.bond.pos);
        let db = dxz(sim.drones.list[b].pos, sim.bond.pos);
        da.total_cmp(&db)
    }).expect("a guard holds the searchlight courtyard");
    let dp = sim.drones.list[di].pos;
    eprintln!("[06fire] fighting guard {di} at {dp:?} health {:.0} state {:#x} dist {:.1}",
        sim.drones.list[di].health, sim.drones.list[di].state, dxz(dp, sim.bond.pos));
    drop(sim);
    let err = aim_at(&mut w, di, 60);
    trace(&w, "mouth aimed");
    assert!(err.abs() < 0.02, "the view is on the courtyard guard (err {err:+.4})");
    let mut shots = 0u32;
    let mut dyaws = Vec::new();
    for k in 0..6 {
        reload_when_dry(&mut w);
        let (fired, dyaw, _) = press(&mut w, di, 20);
        dyaws.push(dyaw);
        if fired { shots += 1; }
        trace(&w, &format!("mouth press {} done", k + 1));
    }
    eprintln!("[06fire] mouth: 6 presses → {shots} rounds, per-press dyaw {dyaws:?}");
    assert_eq!(shots, 6, "6 presses ≥ 8 ticks apart fire 6 rounds [M 0x1a5460:601-655; L P2K spp 1, interval 8]");
    assert!(dyaws.iter().all(|d| d.abs() < 0.02), "no press turns the body [M 0x1a5460], per-press dyaw {dyaws:?}");
}
