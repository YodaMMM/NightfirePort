//! ronin-live: the Phoenix Ronin (the deployable suitcase turret) through the real paths.
//!
//! The game's rules, cited:
//! - The campaign gives it as a placed pickup: 0700000c (Chain Reaction) #561
//!   `oPickUp_Briefcase_3rd_LOD0` (53.98, 1.26, −173.46) and 07000012 (Phoenix Fire, part 2) #753
//!   `oPickUp_Briefcase_LOD0` (13.07, 0.39, 15.71), both type 0xF0 kind 0, id 0x52 (82), 1 round
//!   [D placements dump]. No SP loadout equips 0x52/0x53 [M Player_InitWeapon 0x1a7ef8].
//! - Fire with 0x52: `Player_WeaponInitBullet` 0x1a4bf0:265-282 → `GT_DeployMiniGun` 0x181560 →
//!   `Collide_PlaceObject(0.5, 0.6)` + `GT_Create` (facing 0, scripts 0x06000045/44, owner = the
//!   player) [M; disasm 0x1815e0-0x181614]. The emptied 0x52 then becomes 0x53 through
//!   `Player_HandleHasNoAmmo` 0x1a2ba8's 'R' arm (owned[0x52] = 0, equip 0x53 999, request 0x53).
//! - The turret: `GT_Update` 0x180498 — dormant → arming (both scripts parked) → combat;
//!   `GT_Track` 0x17f9c8 (SP range² 2500, a player-owned turret skips the player [M :62-64]); a
//!   tracked frame fires every 0.1 s (`Bullet_init(−1, ext+0xB8 = the player, …, 0x2c55d0)`).
//! - Fire with 0x53: `GT_TakeControl` 0x181740 — state 7, cam mode 0xf (`Player_SetCamMode` →
//!   HUD pane 18) [M]; the action-0xE press → `GT_LoseControl` 0x181408 → weapon 0x53 back.
//! - Pick-up: E on it → `GT_Activate` 0x1816f0 (state 2) → 1.2 s → 3 → both scripts done →
//!   facing 0 → `Pickup_Create(…, 0x2000366, kind 0, id 0x52, 1 round)` + self-delete [M 0x180498
//!   case 3, disasm 0x18092c-0x18098c].
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_game::gt::{gt_state, RONIN_CONTROL, RONIN_DEPLOY};
use nf_game::player::Input;
use nf_view::scene::Sim;

const LEVEL: &str = "0700000c.bin";
/// #561's translation [D].
const CASE_AT: [f32; 3] = [53.980_007, 1.263_983, -173.459_2];

fn load() -> Option<nf_view::scene::Scene> {
    let iso = std::env::var("NF_ISO").ok()?;
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, LEVEL, false).expect("load 0700000c");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 {
        if !sim.disabled { break; }
        sim.drones.scripts.skip_request = true;
        sim.step_bond(Default::default(), false, false, false);
    }
    Some(scene)
}

fn idle(sim: &mut Sim, n: u32) {
    for _ in 0..n { sim.step_bond(Input::default(), false, false, false); }
}

/// The real take: Bond stands on #561's spot and the pickup walk grants the case.
fn take_case(sim: &mut Sim) {
    sim.bond.pos = [CASE_AT[0], CASE_AT[1] + 0.4, CASE_AT[2]];
    for _ in 0..120 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.inventory.owned[RONIN_DEPLOY as usize] { break; }
    }
    assert!(sim.drones.inventory.owned[RONIN_DEPLOY as usize], "walking onto #561 takes the Ronin");
}

/// Draw 0x52 through the switch request and wait for the draw to finish.
fn draw(sim: &mut Sim, id: u8) {
    for _ in 0..400 {
        if sim.gun.weapon == id && sim.gun.switch_state == 0 { break; }
        let ask = sim.gun.weapon != id && sim.gun.switch_state == 0;
        sim.step_bond(Input { switch_to: ask.then_some(id), ..Default::default() }, false, false, false);
    }
    assert_eq!(sim.gun.weapon, id, "the switch landed on {id:#x} (state {}, request {:#x}, owned {}, fp {:#x})",
        sim.gun.switch_state, sim.gun.switch_request, sim.drones.inventory.owned[id as usize], sim.weapons.first_person(id as usize));
}

/// The fire press on the real input path; returns the deployed turret's index.
fn deploy(sim: &mut Sim) -> Option<usize> {
    let n0 = sim.drones.gt_turrets.len();
    for t in 0..120 {
        sim.step_bond(Input::default(), t < 4, false, false);
        if sim.drones.gt_turrets.len() > n0 { return sim.drones.ronin_turret(); }
    }
    None
}

/// Every guard out of the room (the "empty room" fixture) or none touched.
fn clear_guards(sim: &mut Sim) {
    for d in sim.drones.list.iter_mut() { d.present = false; }
}

/// Take, draw and deploy facing the yaw that leaves the case room (tries the four headings).
fn deployed(sim: &mut Sim) -> usize {
    take_case(sim);
    draw(sim, RONIN_DEPLOY);
    for k in 0..8 {
        sim.bond.yaw = k as f32 * std::f32::consts::FRAC_PI_4;
        if let Some(i) = deploy(sim) { return i; }
        assert!(sim.gun.clip >= 1, "a refused deploy refunds the round (BLData+0x590 += 1)");
    }
    panic!("no heading near #561 seats the turret");
}

#[test]
fn ronin_deploys_on_real_input() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    let sim = scene.sim.as_mut().unwrap();
    assert!(sim.drones.ronin_turret().is_none(), "no turret before the press");
    // Sister (can fail): holding the case without firing deploys nothing.
    take_case(sim);
    draw(sim, RONIN_DEPLOY);
    idle(sim, 120);
    assert!(sim.drones.ronin_turret().is_none(), "no press → no turret");
    let mut ti = None;
    for k in 0..8 {
        sim.bond.yaw = k as f32 * std::f32::consts::FRAC_PI_4;
        if let Some(i) = deploy(sim) { ti = Some(i); break; }
    }
    let ti = ti.expect("the press deployed a turret");
    let t = &sim.drones.gt_turrets[ti];
    assert_eq!((t.facing, t.hp, t.owner), (0, 200, nf_game::gt::PLAYER_OWNER), "the GT_DeployMiniGun record");
    let b = sim.bond.pos;
    let d = ((t.pos[0] - b[0]).powi(2) + (t.pos[2] - b[2]).powi(2)).sqrt();
    assert!((0.5..1.6).contains(&d), "seated about 1.0 ahead of Bond (got {d:.2})");
    assert!(t.script_player_a != usize::MAX && t.script_player_b != usize::MAX, "the script pair mounted");
    // The emptied case becomes its remote (`Player_HandleHasNoAmmo` 'R' arm).
    idle(sim, 200);
    assert!(!sim.drones.inventory.owned[RONIN_DEPLOY as usize], "owned[0x52] cleared");
    assert!(sim.drones.inventory.owned[RONIN_CONTROL as usize], "0x53 equipped");
    assert_eq!(sim.gun.weapon, RONIN_CONTROL, "the hand switched to the remote");
    // It arms: the unfold scripts park → combat.
    let mut armed = false;
    for _ in 0..600 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.gt_turrets.get(ti).is_some_and(|t| t.state == gt_state::COMBAT) { armed = true; break; }
    }
    assert!(armed, "the deployed Ronin reaches COMBAT (state {:?})", sim.drones.gt_turrets.get(ti).map(|t| t.state));
}

#[test]
fn ronin_shoots_a_guard_in_range() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    let sim = scene.sim.as_mut().unwrap();
    let ti = deployed(sim);
    for _ in 0..600 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.gt_turrets[ti].state == gt_state::COMBAT { break; }
    }
    // One guard, 6 units in front of the case (the drone origin sits 1.0 above its feet — `head()`
    // = pos + 0.6, the body capsule from pos − 1.0; the case seats 0.5 above the floor), every
    // other guard out of the room.
    let gi = sim.drones.list.iter().position(|d| d.present && d.enabled && d.health > 0.0 && !d.corpse).expect("a live, enabled guard");
    clear_guards(sim);
    let t = sim.drones.gt_turrets[ti].clone();
    let spot = [t.pos[0] + t.fire_dir[0] * 6.0, t.pos[1] + 0.5, t.pos[2] + t.fire_dir[2] * 6.0];
    let h0 = sim.drones.list[gi].health;
    let mut fired = false;
    let mut hit = false;
    for _ in 0..240 {
        let g = &mut sim.drones.list[gi];
        g.present = true;
        if !g.corpse { g.pos = spot; }
        sim.step_bond(Input::default(), false, false, false);
        let t = &sim.drones.gt_turrets[ti];
        fired |= t.hp < 200;
        let g = &sim.drones.list[gi];
        hit |= g.health < h0 || g.corpse;
        if fired && hit { break; }
    }
    let t = &sim.drones.gt_turrets[ti];
    assert!(fired, "the Ronin fired at the guard (shots left {}, target {})", t.hp, t.target);
    assert!(hit, "its rounds reached the guard (health {h0} → {})", sim.drones.list[gi].health);
    assert!(sim.drones.health.health >= 100.0 - 1e-3, "Bond, behind his own turret, is never its target");
}

#[test]
fn ronin_holds_fire_in_an_empty_room() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    let sim = scene.sim.as_mut().unwrap();
    let ti = deployed(sim);
    clear_guards(sim);
    let mut combat = false;
    for _ in 0..900 {
        sim.step_bond(Input::default(), false, false, false);
        combat |= sim.drones.gt_turrets[ti].state == gt_state::COMBAT;
    }
    let t = &sim.drones.gt_turrets[ti];
    assert!(combat, "it armed");
    assert_eq!(t.hp, 200, "no target → no shot (GT_Track found nothing) — shots left {}", t.hp);
    assert_eq!(t.target, 0);
    assert!(!sim.drones.player_bullets.iter().any(|b| b.weapon == nf_game::gt::TURRET_WEAPON_RECORD as u16), "no turret rounds in flight");
    // Sister (can fail): Bond walking in front of it draws no fire either (the owner skip), but a
    // guard put in the same spot does.
    let tt = sim.drones.gt_turrets[ti].clone();
    sim.bond.pos = [tt.pos[0] + tt.fire_dir[0] * 3.0, tt.pos[1], tt.pos[2] + tt.fire_dir[2] * 3.0];
    idle(sim, 120);
    assert_eq!(sim.drones.gt_turrets[ti].hp, 200, "Bond in front of his own Ronin draws no fire");
    let gi = 0;
    for _ in 0..120 {
        let g = &mut sim.drones.list[gi];
        g.present = true;
        g.pos = [tt.pos[0] + tt.fire_dir[0] * 8.0, tt.pos[1] + 0.5, tt.pos[2] + tt.fire_dir[2] * 8.0];
        sim.step_bond(Input::default(), false, false, false);
    }
    assert!(sim.drones.gt_turrets[ti].hp < 200, "a guard in the same room is fired on");
}

#[test]
fn ronin_remote_view_and_pick_up() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    let sim = scene.sim.as_mut().unwrap();
    let ti = deployed(sim);
    clear_guards(sim);
    for _ in 0..600 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.gt_turrets[ti].state == gt_state::COMBAT && sim.gun.weapon == RONIN_CONTROL && sim.gun.switch_state == 0 { break; }
    }
    assert_eq!(sim.gun.weapon, RONIN_CONTROL);
    // Fire 0x53 → GT_TakeControl: state 7, the remote view (cam mode 0xf).
    for t in 0..60 {
        sim.step_bond(Input::default(), t < 4, false, false);
        if sim.drones.ronin_mounted().is_some() { break; }
    }
    assert!(sim.drones.ronin_mounted().is_some(), "the remote press mounted the turret");
    let (eye, dir) = sim.view_override().expect("the view rides the turret");
    let tp = sim.drones.gt_turrets[ti].pos;
    assert!(((eye[0] - tp[0]).powi(2) + (eye[2] - tp[2]).powi(2)).sqrt() < 1.5, "the eye sits on the turret");
    // The stick turns the gun (case 7's aim).
    let yaw0 = sim.drones.gt_turrets[ti].aim[1];
    for _ in 0..30 { sim.step_bond(Input { turn: 0.02, ..Default::default() }, false, false, false); }
    assert!(sim.drones.gt_turrets[ti].aim[1] > yaw0, "the turn input moves ext+0x94");
    let _ = dir;
    // E dismounts → weapon 0x53 back.
    sim.step_bond(Input::default(), false, false, true);
    idle(sim, 200);
    assert!(sim.drones.ronin_mounted().is_none(), "action 0xE dismounted");
    assert_eq!(sim.drones.gt_turrets[ti].state, gt_state::COMBAT, "7 → 4");
    assert!(sim.view_override().is_none(), "the view is Bond's again");
    // Pick-up: face the case, E → the fold → the 0x52 case back on the floor → walk over it.
    let t = sim.drones.gt_turrets[ti].clone();
    sim.bond.pos = [t.pos[0] - t.fire_dir[0] * 1.2, sim.bond.pos[1], t.pos[2] - t.fire_dir[2] * 1.2];
    sim.bond.yaw = t.fire_dir[0].atan2(t.fire_dir[2]);
    sim.bond.pitch = -0.6;
    // Sister (can fail): without the use press the turret stays.
    idle(sim, 120);
    assert!(sim.drones.ronin_turret().is_some(), "no E → still deployed");
    let mut pressed = false;
    for k in 0..12 {
        sim.step_bond(Input::default(), false, false, true);
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.gt_turrets.get(ti).is_some_and(|t| t.state == gt_state::RESTORING || t.state == gt_state::FINISHED) { pressed = true; break; }
        sim.bond.pitch = -0.6 + k as f32 * 0.1;
    }
    assert!(pressed, "E on the case started the fold (GT_Activate → state 2)");
    let mut dropped = false;
    for _ in 0..900 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.ronin_turret().is_none() { dropped = true; break; }
    }
    assert!(dropped, "the fold ended with the turret gone");
    let case = sim.drones.pickups.iter().find(|p| p.weapon == RONIN_DEPLOY as u16 && !p.gone).expect("the 0x52 case is back on the floor");
    let at = [case.m[3][0], case.m[3][1], case.m[3][2]];
    assert_eq!(case.rounds, 1);
    sim.bond.pos = [at[0], at[1] + 0.4, at[2]];
    for _ in 0..120 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.inventory.owned[RONIN_DEPLOY as usize] { break; }
    }
    assert!(sim.drones.inventory.owned[RONIN_DEPLOY as usize], "walking over the case gives the Ronin back");
    // `Player_EquipWeapon`'s 0x52 tail [M 0x1a8a58:81-85]: the equip clears BLData+0x59E (the
    // remote 0x53's owned byte) and, with the remote in hand (+0x62 == 0x53), requests 0x52.
    assert!(!sim.drones.inventory.owned[RONIN_CONTROL as usize], "the remote leaves the inventory with the case re-taken");
    for _ in 0..120 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.gun.weapon == RONIN_DEPLOY && sim.gun.switch_state == 0 { break; }
    }
    assert_eq!(sim.gun.weapon, RONIN_DEPLOY, "the hand goes from the remote back to the case");
}

/// The remote view's eye and parts, against a live capture: 0700000c, the case deployed from #561
/// at yaw 0 (seated at (53.98, 1.402, −172.46) in both), weapon 0x53 fired → state 7. Measured
/// [L PCSX2 audits/render-1101-roninview/live.txt]: ext+0xC0/C4/C8 = the script-A objects of
/// streams 2/4/0 (0x2000361 / 0x2000362 / 0x200035a); the viewer at (53.986, 1.552, −172.366) =
/// registered[0] (53.994, 1.402, −171.966) + (0, 0.15, −0.4) in its frame — (0.006, 0.15, 0.094)
/// from the turret; registered[0]'s obj+0xF0 = 0x231 (bit 0x10 set) with no shot fired.
#[test]
fn ronin_remote_view_eye_and_flash() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    let sim = scene.sim.as_mut().unwrap();
    let ti = deployed(sim);
    clear_guards(sim);
    let t = &sim.drones.gt_turrets[ti];
    let hash = |sim: &Sim, slot: u32| match slot.checked_sub(1).and_then(|o| sim.drones.scripts.objects.list.get(o as usize)) {
        Some(Some(nf_game::nis::Obj::Entity { hash, src, .. })) => Some((*hash, *src)),
        _ => None,
    };
    // GT_Register runs inside GT_Create's Script_Load: the slots are filled on the deploy tick.
    assert_eq!(t.registered.map(|r| hash(sim, r)), [
        Some((0x0200_0361, (0x0600_0045, 2))), Some((0x0200_0362, (0x0600_0045, 4))), Some((0x0200_035a, (0x0600_0045, 0))),
    ], "the GT_Register slots");
    for _ in 0..600 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.gt_turrets[ti].state == gt_state::COMBAT && sim.gun.weapon == RONIN_CONTROL && sim.gun.switch_state == 0 { break; }
    }
    for t in 0..60 {
        sim.step_bond(Input::default(), t < 4, false, false);
        if sim.drones.ronin_mounted().is_some() { break; }
    }
    assert!(sim.drones.ronin_mounted().is_some(), "mounted");
    idle(sim, 30);
    let t = sim.drones.gt_turrets[ti].clone();
    let (eye, _) = sim.view_override().expect("the remote view");
    let rel = [eye[0] - t.pos[0], eye[1] - t.pos[1], eye[2] - t.pos[2]];
    let live = [0.006f32, 0.15, 0.094];
    let off = |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    assert!(off(rel, live) < 0.02, "the eye sits where the game's viewer does: {rel:?} vs {live:?}");
    // Sister (can fail): the turret's own pose + the same offset (the eye without registered[0])
    // lands 0.49 behind the game's eye, inside the case.
    let pm = t.pose_matrix();
    let e = nf_game::gt::RONIN_EYE;
    let fallback = [pm[3][0] + e[0] * pm[0][0] + e[1] * pm[1][0] + e[2] * pm[2][0] - t.pos[0],
        pm[3][1] + e[0] * pm[0][1] + e[1] * pm[1][1] + e[2] * pm[2][1] - t.pos[1],
        pm[3][2] + e[0] * pm[0][2] + e[1] * pm[1][2] + e[2] * pm[2][2] - t.pos[2]];
    assert!(off(fallback, live) > 0.4, "the pose-only eye would fail the comparison ({fallback:?})");
    // The muzzle flash (registered[0]) is hidden while the gun is idle ...
    let flash_obj = t.registered[0] as usize - 1;
    assert!(t.flash_hidden && sim.drones.scripts.hidden_stream_objects().contains(&flash_obj), "the idle flash is hidden (obj+0xF0 0x231)");
    // ... shows on a shot (reg0+0xF4 = 2, bit 0x10 cleared), and hides again two ticks after the last one.
    let mut shown = false;
    for _ in 0..12 {
        sim.step_bond(Input::default(), true, false, false);
        shown |= !sim.drones.gt_turrets[ti].flash_hidden && !sim.drones.scripts.hidden_stream_objects().contains(&flash_obj);
    }
    assert!(shown, "a mounted shot showed the flash");
    idle(sim, 3);
    assert!(sim.drones.gt_turrets[ti].flash_hidden && sim.drones.scripts.hidden_stream_objects().contains(&flash_obj), "the flash hides after the burst");
    // Sister: only registered[0] is hidden — the gun and barrel stay drawn.
    let t = &sim.drones.gt_turrets[ti];
    let hidden = sim.drones.scripts.hidden_stream_objects();
    assert!(!hidden.contains(&(t.registered[1] as usize - 1)) && !hidden.contains(&(t.registered[2] as usize - 1)), "barrel and gun drawn");
}

/// Screen-space boxes (x0, y0, x1, y1) of the HUD quads drawn with texture `hash`, in the 512×448 frame.
fn hud_boxes(scene: &nf_view::scene::Scene, hash: u32) -> Vec<[f32; 4]> {
    let f = nf_view::hud::overlay(scene, 4.0 / 3.0);
    let Some(ti) = scene.hud_textures.iter().position(|(h, _)| *h == hash) else { return Vec::new() };
    let mut out = Vec::new();
    for r in f.runs.iter().filter(|r| r.0 == ti + 1) {
        for q in f.v[r.2 as usize..(r.2 + r.3) as usize].chunks(6) {
            let (a, b) = (q[0].p, q[2].p);
            out.push([(a[0] + 1.0) * 256.0, (1.0 - a[1]) * 224.0, (b[0] + 1.0) * 256.0, (1.0 - b[1]) * 224.0]);
        }
    }
    out
}

/// The red bar on the right of the remote view: the ammo pane's kind-0x1f strip fed the turret's shot
/// counter. `GT_TakeControl` 0x181740 puts Bond in substate 0x10 and BL+0x878 = the turret, and
/// `HUD_UpdateAmmoPane` 0x19a7f0:146-151 then sets kind 0x1f, max 100, value (short)ext+0xF0 / 2.
/// Measured [L PCSX2 audits/render-1101-roninview/ps2_remote_view.png]: the bar spans x ≈ 480..493,
/// y ≈ 295..392 of the 512×448 frame with a fresh (200-shot) turret.
#[test]
fn ronin_remote_view_shot_gauge() {
    assert_eq!(nf_view::hud::ronin_shots_strip(200), (100.0, 100.0), "a fresh Ronin is full");
    assert_eq!(nf_view::hud::ronin_shots_strip(57), (28.0, 100.0), "halved toward zero");
    assert_eq!(nf_view::hud::ronin_shots_strip(0), (0.0, 100.0));
    // Sister: the raw counter is not the value (200 would draw a column twice the gauge).
    assert_ne!(nf_view::hud::ronin_shots_strip(200).0, 200.0);
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    const STRIP: u32 = 0x0300_0128;
    let ti = {
        let sim = scene.sim.as_mut().unwrap();
        let ti = deployed(sim);
        clear_guards(sim);
        for _ in 0..600 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.drones.gt_turrets[ti].state == gt_state::COMBAT && sim.gun.weapon == RONIN_CONTROL && sim.gun.switch_state == 0 { break; }
        }
        sim.hud.weapon_label_timer = 0;
        ti
    };
    // Sister: deployed but not mounted (0x53 in hand), the pane draws no 100-unit gauge.
    let tall = |b: &[[f32; 4]]| b.iter().any(|r| (r[3] - r[1] - 100.0).abs() < 0.5);
    assert!(!tall(&hud_boxes(&scene, STRIP)), "no shot gauge before the remote view: {:?}", hud_boxes(&scene, STRIP));
    {
        let sim = scene.sim.as_mut().unwrap();
        for t in 0..60 {
            sim.step_bond(Input::default(), t < 4, false, false);
            if sim.drones.ronin_mounted().is_some() { break; }
        }
        assert!(sim.drones.ronin_mounted().is_some(), "mounted");
        idle(sim, 30);
        sim.hud.weapon_label_timer = 0;
    }
    let hp0 = scene.sim.as_ref().unwrap().drones.gt_turrets[ti].hp;
    assert_eq!(hp0, 200, "no shot spent yet");
    let b = hud_boxes(&scene, STRIP);
    // BulletImg[0x1f] (1 px/unit, ×16 wide at max 100): the empty column 100 tall from y 294 and the
    // full fill the same, x 480..496 — the bar the PS2 frame shows.
    let cols: Vec<_> = b.iter().filter(|r| (r[3] - r[1] - 100.0).abs() < 0.5).collect();
    assert_eq!(cols.len(), 2, "the gauge's two columns, both full: {b:?}");
    for c in &cols {
        assert!((c[0] - 480.0).abs() < 0.5 && (c[2] - 496.0).abs() < 0.5 && (c[1] - 294.0).abs() < 0.5 && (c[3] - 394.0).abs() < 0.5, "{c:?}");
    }
    // Firing spends shots and the fill drops by half the spent count.
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..40 { sim.step_bond(Input::default(), true, false, false); }
        idle(sim, 2);
    }
    let hp = scene.sim.as_ref().unwrap().drones.gt_turrets[ti].hp;
    assert!(hp < hp0, "the mounted burst spent shots ({hp})");
    let want = (hp / 2) as f32;
    let b = hud_boxes(&scene, STRIP);
    assert!(b.iter().any(|r| (r[3] - r[1] - want).abs() < 0.5 && (r[3] - 394.0).abs() < 0.5), "the fill is {want} tall after the burst: {b:?}");
    assert!(tall(&b), "the empty column stays 100 tall");
}

/// The barrel's spin on the real input path, against the live capture [L PCSX2
/// audits/render-1101-roninview2/live_ab.txt]: the 0x53 press held 8 frames takes control (and the
/// held trigger arms the spin), 45 frames later ext+0xD4 = −2.7410; the drawn barrel is the script
/// pose turned by RotZ(that angle) about its own axis [M GT_Update 0x180498:536-556].
#[test]
fn ronin_barrel_spin_on_the_press() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    let sim = scene.sim.as_mut().unwrap();
    let ti = deployed(sim);
    clear_guards(sim);
    for _ in 0..600 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.gt_turrets[ti].state == gt_state::COMBAT && sim.gun.weapon == RONIN_CONTROL && sim.gun.switch_state == 0 { break; }
    }
    assert_eq!(sim.drones.gt_turrets[ti].spin_draw, None, "an idle armed Ronin's barrel sits at its script pose");
    for _ in 0..8 { sim.step_bond(Input::default(), true, false, false); }
    assert!(sim.drones.ronin_mounted().is_some(), "mounted");
    idle(sim, 45);
    let t = &sim.drones.gt_turrets[ti];
    eprintln!("spin {} vel {} draw {:?} hp {}", t.spin, t.spin_vel, t.spin_draw, t.hp);
    assert!((t.spin - (-2.741)).abs() < 0.02, "ext+0xD4 {} vs the live −2.741", t.spin);
    assert_eq!(t.hp, 199, "the held press fired one shot (live ext+0xF0 199)");
    // Sister: one held frame fewer lands 0.21 rad short of the capture.
    assert!((t.spin - 0.209 - (-2.741)).abs() > 0.1);
}

/// The remote view on 0C drops the barrel's near-clipped tubes the way the 24-bit depth wrap does
/// [L PCSX2 audits/render-1101-roninview2: the stock view shows the muzzle ring's face and no tubes;
/// the eye moved 0.1 back shows them]. 0C's `Ps2Far1` 20475840 puts the wrap at 0.122 > the 0.1 near
/// plane (`nf_game::env::zbuf_wrap_distance`); the Ronin's script parts carry it in their palette
/// entries' m[3][3] (the world shader drops a triangle with a vertex nearer than that), and the
/// barrel's palette entry is the spun pose.
#[test]
fn ronin_remote_view_marks_the_wrapped_parts() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    let ti = {
        let sim = scene.sim.as_mut().unwrap();
        let ti = deployed(sim);
        clear_guards(sim);
        for _ in 0..600 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.drones.gt_turrets[ti].state == gt_state::COMBAT && sim.gun.weapon == RONIN_CONTROL && sim.gun.switch_state == 0 { break; }
        }
        ti
    };
    // Sister: not mounted → nothing marked.
    assert!(scene.ronin_view_wrap_parts().0.is_empty(), "no marks outside the remote view");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..8 { sim.step_bond(Input::default(), true, false, false); }
        idle(sim, 10);
        assert!(sim.drones.ronin_mounted().is_some());
    }
    let (parts, dw) = scene.ronin_view_wrap_parts();
    assert!((dw - 0.122).abs() < 0.001, "0C's wrap distance {dw}");
    let sim = scene.sim.as_ref().unwrap();
    let t = &sim.drones.gt_turrets[ti];
    let barrel = match sim.drones.scripts.objects.list.get(t.registered[1] as usize - 1) {
        Some(Some(nf_game::nis::Obj::Entity { hash, src, matrix, .. })) => (*hash, *src, *matrix),
        _ => panic!("registered[1] is a script entity"),
    };
    assert_eq!(barrel.0, 0x0200_0362, "registered[1] is the barrel");
    let att = parts.iter().copied().find(|&a| scene.attachments[a].script.is_some_and(|(sh, st, h, _)| (sh, st) == barrel.1 && h == barrel.0));
    let att = att.expect("the barrel's attachment is among the marked parts");
    let (l, _) = scene.skin_layout();
    let (pal, _) = scene.skin_palette(&l, None, 0.0, Default::default(), None);
    let e = pal[l.attach_base as usize + att];
    assert!((e[3][3] - (1.0 + dw)).abs() < 1e-6, "the barrel's entry carries the wrap distance: {}", e[3][3]);
    // The entry is RotZ(spin_draw) · the script pose: its x row = cos·pose.x + sin·pose.y.
    let a = t.spin_draw.expect("the press armed the spin");
    let (s, c) = a.sin_cos();
    let m = barrel.2;
    for k in 0..3 { assert!((e[0][k] - (c * m[0][k] + s * m[1][k])).abs() < 1e-4, "spun x row {:?} vs pose {:?} at {a}", e[0], m[0]); }
    assert!(a.abs() > 0.5, "the drawn barrel is visibly turned ({a})");
}

/// The remote view shows the Ronin pane's reticle and no normal crosshair. `GT_TakeControl` 0x181740
/// calls `Player_SetCamMode(BL, 0xf)`, which stores BL+0x950 = 0xf and enables pane 18 [M 0x195928
/// case 0xf]; `HUD_UpdateCrossHair` 0x19e6d0:97-100 then ends at LAB_0019e9c8 (sprite layer 0xff,
/// not drawn) while BL+0x950 != 0. [L PCSX2 audits/render-1101-roninview2/sbs_remote_view.png: the
/// PS2 frame has the octagon reticle only.] Sister: on foot with 0x53 in hand the cross draws.
#[test]
fn ronin_remote_view_hides_the_crosshair() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    const CROSS_TEX: u32 = 0x0300_0026;
    {
        let sim = scene.sim.as_mut().unwrap();
        let ti = deployed(sim);
        clear_guards(sim);
        for _ in 0..600 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.drones.gt_turrets[ti].state == gt_state::COMBAT && sim.gun.weapon == RONIN_CONTROL && sim.gun.switch_state == 0 { break; }
        }
        assert_eq!(sim.gun.weapon, RONIN_CONTROL);
        assert!(sim.drones.ronin_mounted().is_none());
    }
    // The normal cross: cell 2 (11×11) centred on the screen centre (256, 224).
    let cross = |b: &[[f32; 4]]| b.iter().filter(|r| (r[2] - r[0] - 11.0).abs() < 0.5 && (r[3] - r[1] - 11.0).abs() < 0.5
        && ((r[0] + r[2]) / 2.0 - 256.0).abs() < 2.0 && ((r[1] + r[3]) / 2.0 - 224.0).abs() < 2.0).count();
    let before = hud_boxes(&scene, CROSS_TEX);
    assert_eq!(cross(&before), 1, "sister: on foot the crosshair draws at the centre: {before:?}");
    let reticle_tex = scene.hud.as_ref().expect("HUD tables").ronin[5].texture;
    assert!(hud_boxes(&scene, reticle_tex).is_empty(), "no pane-18 reticle before the remote view: {:?}", hud_boxes(&scene, reticle_tex));
    {
        let sim = scene.sim.as_mut().unwrap();
        for t in 0..60 {
            sim.step_bond(Input::default(), t < 4, false, false);
            if sim.drones.ronin_mounted().is_some() { break; }
        }
        assert!(sim.drones.ronin_mounted().is_some(), "mounted");
        idle(sim, 30);
    }
    let after = hud_boxes(&scene, CROSS_TEX);
    assert_eq!(cross(&after), 0, "the remote view draws no normal crosshair: {after:?}");
    // The pane's reticle (sprite 5, flag 0x800 → centred on (256, 224)) still draws: its quads
    // together cover a box wider than the cross, centred on the screen centre.
    let ret = hud_boxes(&scene, reticle_tex);
    let u = ret.iter().fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |a, r| [a[0].min(r[0]), a[1].min(r[1]), a[2].max(r[2]), a[3].max(r[3])]);
    assert!(!ret.is_empty() && ((u[0] + u[2]) / 2.0 - 256.0).abs() < 2.0 && ((u[1] + u[3]) / 2.0 - 224.0).abs() < 2.0 && u[2] - u[0] > 20.0,
        "the octagon reticle stays: tex {reticle_tex:#x} {ret:?}");
}
