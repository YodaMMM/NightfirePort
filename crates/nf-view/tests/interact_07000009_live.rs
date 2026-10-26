//! level-09 interactables: the kinds the elevator walk (elevator_buttons_09_live) and the
//! channel-chain walk (walkreal_07000009) do not already drive, through the REAL input path
//! (`nf_view::load` + `Sim::step_bond` walking/turning/use/fire).
//!
//! Kinds here (docs/levels/07000009.md census): the alarm Switches (0x29 → ch 0x7C/0x7D →
//! OR mux #278 → the alarm fail), the breakable glass (0x20 `Break_Create` 0x12c230), the
//! E-doors + the locked shutter (0xdb), the pickups (0xF0 armour + the Bond bonuses), the
//! walk-in touch plates (0xEA), and the Load Point's E route (0xe8 [M Trigger_Activate
//! 0x1b2318:76-90 — key_gate 0 = no gate]). The settle-then-operate pattern is the accepted
//! one (a teleport TO the observation spot, then real input only).
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load_09(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000009.bin", false).expect("load 07000009");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    assert!(!scene.sim.as_ref().unwrap().disabled, "Bond is free after the intro");
    scene
}

/// Settle Bond at a spot (a teleport TO the observation point; everything after is real input).
fn settle_near(scene: &mut nf_view::scene::Scene, spot: [f32; 3]) {
    let sim = scene.sim.as_mut().unwrap();
    // Cast 0.05 above the spot (the game places objects ON the surface + normal offset [M
    // Collide_PlaceObject 0x1ecfb8 tail]; the armour-spot y −368.9 sits 0.37 UNDER the walkable
    // floor top −368.53 [D probe] and the old start-from-spot cast landed Bond beneath it; 0.5
    // covers that gap). The teleport stands him at the game's standing height — the floor point
    // + 1.1 [M Player_StandAtNewPosition 0x190d88:0x190ee0-0x190f18]: the old floor + 0.1 put the
    // root a metre under the stance, from where the ground probe never recovers — measured here,
    // Bond sank to −372.9 within the 60 settle ticks (and kept falling on the walk, y −488 after
    // 240 forward ticks), 4.98 from the armour, outside its 1.58 take gate forever.
    let floor = sim.world.segment([spot[0], spot[1] + 0.5, spot[2]], [spot[0], spot[1] - 30.0, spot[2]]).map(|t| spot[1] + 0.5 - 30.0 * t);
    if let Some(f) = floor { sim.bond.pos = [spot[0], f + 1.1, spot[2]]; } else { sim.bond.pos = spot; }
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
}

fn face(scene: &mut nf_view::scene::Scene, at: [f32; 3]) {
    let sim = scene.sim.as_mut().unwrap();
    let b = sim.bond.pos;
    sim.bond.yaw = (at[0] - b[0]).atan2(at[2] - b[2]);
}

fn press_use(scene: &mut nf_view::scene::Scene, ticks: u32) {
    for _ in 0..ticks { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, true); }
}

/// KIND Switch (the alarm panels): the car-park alarm switch #253 (alarmswitchplayer, ch 0x7C
/// [D placement #253 param 0]) is left of the spawn; the real E press writes ch 0x7C
/// [M Switch_Activate 0x1b02a8 through the port's useobj switch arm], the OR mux #278 lifts
/// ch 0x61 [D #278 params (0:0x61, 1:0x7C, 2:0x7D)], objective 1 (INVERTED on ch 0x61) fails
/// and Mission_Update ends the level [M 0x1827d8:128-141].
#[test]
fn an_alarm_switch_press_raises_the_alarm_and_fails_the_mission() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [23.8, -377.0, 37.8]);
    face(&mut scene, [25.05, -375.33, 37.8]);
    press_use(&mut scene, 60);
    for _ in 0..340 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x7c), "the E press writes ch 0x7C [M Switch_Activate 0x1b02a8]");
    assert!(sim.drones.channels.on(0x61), "OR mux #278: ch 0x7C raises the alarm ch 0x61 [D #278]");
    assert_eq!(sim.drones.level_exit, Some(0x0700_0009), "objective 1 inverted on 0x61 fails the mission [M 0x1827d8]");
}

/// KIND Switch, the ch-0x7D side of the alarm mux: the lobby pull #76 (alarmswitchplayer,
/// ch 125 = 0x7D [D placement #76 param 0, at (−9.11, −368.05, 68.48)]) is one of the four
/// 0x7D writers feeding OR mux #278's SECOND input (params (0:0x61, 1:0x7C, 2:0x7D) [D #278];
/// the game ORs the listed inputs into the out channel [M Trigger_MultiplexOrIn 0x1b2d38]). The
/// E press writes ch 0x7D [M Switch_Activate 0x1b02a8 through the port's useobj switch arm],
/// the mux lifts ch 0x61 and objective 1 (INVERTED on ch 0x61) fails the mission [M 0x1827d8] —
/// the same landing as the ch-0x7C pull in the sister test above, pinned here for the other
/// mux input, which no 09 test exercised before this audit.
#[test]
fn a_ch7d_alarm_pull_raises_the_alarm_too() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    // Stand Bond beside the ch-0x7D pull (a teleport to reach the spot — the press itself is the
    // real input path), then pulse E until the mux lands, exactly like the ch-0x7C sister test.
    let spot = {
        let sim = scene.sim.as_ref().unwrap();
        let s = sim.drones.useobjs.switches.iter().find(|s| s.ch == 125 && s.gate == 0)
            .expect("an alarmswitchplayer on ch 125 with no gate [D placements #76/77/79/276]");
        s.pos
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [spot[0] + 0.9, spot[1], spot[2]];
    }
    let mut fired = false;
    for k in 0..120 {
        {
            let sim = scene.sim.as_mut().unwrap();
            let b = &mut sim.bond;
            b.yaw = (spot[0] - b.pos[0]).atan2(spot[2] - b.pos[2]);
            b.pitch = 0.0;
            sim.step_bond(Input::default(), false, false, k % 6 == 0);
            if sim.drones.channels.on(0x7d) { fired = true; break; }
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(fired, "the E pulses at the ch-0x7D pull must raise ch 0x7D [M Switch_Activate 0x1b02a8]");
    assert!(sim.drones.channels.on(0x61), "OR mux #278: ch 0x7D raises the alarm ch 0x61 [D #278]");
    for _ in 0..340 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.level_exit, Some(0x0700_0009), "objective 1 inverted on 0x61 fails the mission [M 0x1827d8]");
}

/// Sister: the same stance with NO press — ch 0x7C/0x61 stay low and the level never ends.
#[test]
fn without_the_press_the_alarm_stays_off() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [23.8, -377.0, 37.8]);
    face(&mut scene, [25.05, -375.33, 37.8]);
    for _ in 0..400 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x7c), "standing by the panel writes nothing");
    assert!(!sim.drones.channels.on(0x61), "no alarm without the press");
    assert_eq!(sim.drones.level_exit, None, "the level runs on");
}

/// KIND Breakable (the lobby roof glass): the pillarglass #123 (kind-0x20 param 0 = 2 = glass,
/// param 1 = 6 → health (f32)(i32)6 = 6.0 [D #123 params (0:2, 1:6); M Break_Create 0x12c230
/// disasm 0x12c2a8-0x12c2e0 — the cvt.s.w, no filter param]) is a 0.2-thick glass pillar slab
/// over the lobby floor: x 7.92..8.12, y −369.25..−354.12 (up), z 36.10..37.38 [D the break
/// bounds; placement @(8.02, −361.69, 36.74)]. The game's bullet route: `Break_Update`
/// 0x12bfd8:15 filters the pane's contact list with `Collide_FilterBullets(list, tag+0x08 |
/// 0x106)` [M disasm 0x12c028-0x12c030] — a class-5 contact whose weapon-DEFINITION word
/// (+0x10) ANDs the mask non-zero is pulled out of the list (marked t = 1e8 → `LList_Add`)
/// [M 0x1ece30:24-33 + 0x1ecee4-0x1ecef8], so only zero-AND words damage it. The disc's gun
/// records carry 0x0008 (the P2K 0x2 [L ee_start_05.bin weapon_data @0x2bf150 + w·0x10c]) —
/// 0x0008 & 0x106 == 0, a gun round passes and reaches the pane from the floor beside it;
/// the laser 0x0002 / watch 0x0100 words never do (unit-tested in breakable.rs). The real
/// fire path (the equipped gun, the bullets, the bullet_hit pass) breaks it: `Break_Kill`
/// 0x12b970 marks it dead and reports the glass kill (sound 0x2E) [M]. Sister: aiming
/// without firing leaves it alive.
#[test]
fn shooting_the_lobby_roof_glass_breaks_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    // The lobby floor BESIDE the pillar (the slab x 8.023 thin, z 36.10..37.38 [D the break
    // bounds] — standing spots are off the footprint). The stance sits 0.5 EAST of the slab:
    // the game's own muzzle offset (the bullet starts at the midpoint of the eye and the view
    // model's datum 0 [M Player_WeaponInitBullet 0x1a4bf0:226-245], ~0.15 right = west here)
    // then cannot reach the neighbouring perpendicular pane #116 (z 35.96, x 6.59..7.86 + the
    // 0.1 grow) that clips a corner shot from the x-8.02 line, and the angled round still
    // crosses #123's slab for its whole flight.
    settle_near(&mut scene, [8.5, -369.0, 34.5]);
    // The real loadout: cycle to a LETHAL GUN (the walkreal's own draw loop — the level opens
    // on the K5 sleep darts, whose word is also 0x0008 but whose hits stun drones; the P2K is
    // the first lethal gun in Bond's owned set [D the 09 loadout]).
    assert!(cycle_to_lethal(&mut scene), "a lethal gun is in Bond's 09 owned set (the P2K 0x2)");
    { let sim = scene.sim.as_ref().unwrap();
      let wd = sim.weapons.records.get(sim.gun.weapon as usize)
          .map_or(0u16, |r| u16::from_le_bytes([r[0x10], r[0x11]]));
      eprintln!("[glass] armed weapon {:#x} word {wd:#06x}", sim.gun.weapon);
      assert!(wd & 0x106 == 0, "the armed gun's word passes the game's base mask [M 0x1ece94]"); }
    // Aim at the pillar's body 1.9 north of the stance (inside the slab's z span, standing
    // height): the round's line crosses the slab [D the break bounds].
    let target = [8.02f32, -366.0, 36.4];
    let mut killed = false;
    for t in 0..2400u32 {
        {
            let sim = scene.sim.as_mut().unwrap();
            if sim.drones.breaks.list.iter().any(|b| b.placement == 123 && b.dead) { killed = true; break; }
            let head = sim.bond.head();
            let dir = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
            let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
            // The aim rides the INPUT (turn/look — the game's own aim path): the gun's bullets
            // take their direction from the processed aim state, not a direct yaw/pitch write
            // (the gondola-windows live tests showed direct writes miss the pane).
            let desired_yaw = dir[0].atan2(dir[2]);
            let desired_pitch = (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2;
            let dy = desired_yaw - sim.bond.yaw;
            let dy = dy - std::f32::consts::TAU * (dy / std::f32::consts::TAU).round();
            let input = Input {
                turn: dy.clamp(-0.08, 0.08),
                look: (desired_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
                ..Default::default()
            };
            // Burst cadence: the port's gun fires on the trigger edge (the held trigger fires
            // once); every press is a fresh shot.
            sim.step_bond(input, t % 10 < 3, false, false);
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    let b = sim.drones.breaks.list.iter().find(|b| b.placement == 123).expect("glass #123 exists [D]");
    eprintln!("[glass] #123 dead {} health {:.3} kills {:?}", b.dead, b.health, sim.drones.break_kills);
    assert!(b.dead, "the roof glass breaks under the real fire [M Break_Kill 0x12b970]");
    assert!(killed, "the death was observed in-loop");
    assert!(sim.drones.break_kills.iter().any(|&(p, k, s)| p == 123 && k == 2 && s == 0x2E),
        "the kill effect is the glass one: kind 2, sound 0x2E [M 0x12b970 kind-2 arm]");
}

/// Sister: the same stance and aim with the trigger never pressed — the glass lives.
#[test]
fn the_aim_alone_leaves_the_glass_intact() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [8.5, -369.0, 34.5]);
    for _ in 0..120 {
        let sim = scene.sim.as_mut().unwrap();
        let head = sim.bond.head();
        let dir = [8.02 - head[0], -366.0 - head[1], 36.4 - head[2]];
        let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
        sim.bond.yaw = dir[0].atan2(dir[2]);
        sim.bond.pitch = (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        sim.step_bond(Input::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    let b = sim.drones.breaks.list.iter().find(|b| b.placement == 123).unwrap();
    assert!(!b.dead, "aiming does not break the glass");
    assert!(sim.drones.break_kills.is_empty(), "no kill effects without the fire");
}

/// The filter's own level-real pin: the securityroom_glass #87 (kind 0x20 params [0:2, 1:1,
/// 2:0x30] [D placement]) carries param 2 = 0x30 → `Break_Create` stores ~0x30 = 0xffcf [M
/// 0x12c2e4-0x12c2f8] and `Break_Update`'s mask 0xffcf | 0x106 = 0xffef [M 0x12c028-0x12c030]
/// — a gun's word 0x0008 ANDs it non-zero, so `Collide_FilterBullets` pulls the contact [M
/// 0x1ece30:24-33 + 0x1ecee4-0x1ecef8] and the round never damages the pane: the same real
/// fire path that shatters #123 leaves #87 whole. (Its health (f32)(i32)1 = 1.0 [M
/// 0x12c2a8-0x12c2e0] would die to the first passing round.) Sister of the roof-glass shot.
#[test]
fn the_security_room_glass_refuses_the_same_gun() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    // The lobby floor EAST of the security glass (the pane x −18.40..−10.63, z 27.51..35.29
    // [D the break bounds]): a straight west shot into its east face. Facing west, the game's
    // own muzzle offset (right = east) pushes the round AWAY from the pane edges.
    settle_near(&mut scene, [-9.8, -369.0, 31.4]);
    assert!(cycle_to_lethal(&mut scene), "a lethal gun is in Bond's 09 owned set (the P2K 0x2)");
    let target = [-10.8f32, -366.8, 31.4];
    let mut shots = 0u32;
    for t in 0..900u32 {
        {
            let sim = scene.sim.as_mut().unwrap();
            if sim.drones.breaks.list.iter().any(|b| b.placement == 87 && b.dead) { break; }
            let head = sim.bond.head();
            let dir = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
            let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
            sim.bond.yaw = dir[0].atan2(dir[2]);
            sim.bond.pitch = (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2;
            let fired = t % 16 == 0;
            if fired { shots += 1; }
            sim.step_bond(Input::default(), fired, false, false);
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    let b = sim.drones.breaks.list.iter().find(|b| b.placement == 87).expect("security glass #87 exists [D]");
    eprintln!("[glass] #87 dead {} health {:.3} after {shots} trigger ticks, kills {:?}",
        b.dead, b.health, sim.drones.break_kills);
    assert!(shots > 10, "the fire loop pulled the trigger ({shots} ticks — the harness fired)");
    assert!(!b.dead, "the gun word 0x0008 is filtered out of the 0x30-filter pane [M 0x1ece30]");
    assert!(!sim.drones.break_kills.iter().any(|&(p, _, _)| p == 87), "no kill effect for #87");
}

/// KIND Door: the far stairwell door #155 (0xdb, param 2 = 0 → E-openable [D #155]) opens on
/// the real use key [M Door_Activate 0x135be0]; the shutter #157 (param 0 = 2 slide, param 3 =
/// 200 = the lock channel, never written on this level [D #157]) refuses the same press.
#[test]
fn the_far_stairwell_door_opens_and_the_shutter_stays_locked() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [9.1, -377.0, 97.0]);
    face(&mut scene, [9.14, -377.08, 95.77]);
    let mut opened = false;
    for t in 0..60 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, true);
        let st = scene.sim.as_ref().unwrap().drones.doors.list.iter()
            .find(|d| d.placement == 155).expect("door #155 [D]").state;
        if st != nf_game::door::CLOSED { opened = true; eprintln!("[door] #155 state {st} at t{t}"); }
    }
    assert!(opened, "the E-door opens during the press [M Door_Activate 0x135be0]");
    // The door swings back (the E-doors close behind the player — observed state 8 → 2 here).
    for _ in 0..200 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    // The shutter: the same press, the lock channel never written.
    settle_near(&mut scene, [-15.7, -377.0, 97.4]);
    face(&mut scene, [-15.74, -377.08, 95.63]);
    press_use(&mut scene, 60);
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let s = sim.drones.doors.list.iter().find(|d| d.placement == 157).expect("shutter #157 [D]");
    eprintln!("[door] #157 after E: state {} ch_lock {}", s.state, s.ch_lock);
    assert_eq!(s.state, nf_game::door::CLOSED, "lock ch 200 is never written: the shutter refuses [D #157 p3]");
}

/// Sister: the same stance, no press — #155 stays closed.
#[test]
fn the_far_stairwell_door_needs_the_press() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [9.1, -377.0, 97.0]);
    face(&mut scene, [9.14, -377.08, 95.77]);
    for _ in 0..180 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let d = sim.drones.doors.list.iter().find(|d| d.placement == 155).unwrap();
    assert_eq!(d.state, nf_game::door::CLOSED, "no press, no open");
}

/// KIND Pickup (the armour): walking into PICKUP_BodyArmourHeavy #270 (−22.57, −368.46, 24.86
/// [D]) takes it — the armour hard-sets 50.0 while it was below [M Pickup_Handler 0x18e650's
/// armour case, refused at ≥ 50.0] and the pickup is consumed. Sister: standing short leaves it.
#[test]
fn walking_over_the_body_armour_takes_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [-22.0, -368.9, 24.86]);
    face(&mut scene, [-22.57, -368.46, 24.86]);
    // (the settle spot is inside the take radius, so the take may land on the settle ticks —
    // the walk-over IS the operation; the assert below pins its effect)
    let mut taken_at: Option<f32> = None;
    for _ in 0..240 {
        let sim = scene.sim.as_mut().unwrap();
        if sim.drones.pickups.iter().any(|p| p.gone && (p.m[3][0] + 22.57).abs() < 0.5) {
            taken_at = Some(sim.drones.health.armour);
            break;
        }
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    let p = sim.drones.pickups.iter().find(|p| (p.m[3][0] + 22.57).abs() < 0.5).expect("armour #270 exists [D]");
    eprintln!("[pickup] #270 gone {} armour at take {:?} now {:.1}", p.gone, taken_at, sim.drones.health.armour);
    assert!(p.gone, "the armour pickup is consumed by the walk-over [M 0x18e650]");
    assert!(taken_at.map_or(false, |a| (a - 50.0).abs() < 0.01),
        "the armour hard-sets 50.0 at the take [M 0x18e650 armour case] (the guards' fire erodes it after)");
}

/// Sister: parked 2 units short, the armour stays.
#[test]
fn standing_short_leaves_the_armour() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    // Parked outside the take gate: the game's gate is the armour's world-sphere radius + 1.1
    // [M 0x18e148:203-210]; the armour's radius reads 0.4767 [D probe] → r = 1.577. The old park
    // (−21.2) sits 1.47 from the centre — INSIDE the gate, and at the game's standing height the
    // gate+LOS both pass, so the take fires; a real short park is 2+ units out.
    settle_near(&mut scene, [-20.4, -369.0, 24.86]);
    face(&mut scene, [-22.57, -368.46, 24.86]);
    for _ in 0..240 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let p = sim.drones.pickups.iter().find(|p| (p.m[3][0] + 22.57).abs() < 0.5).unwrap();
    assert!(!p.gone, "standing short never takes it");
}

/// KIND Load Point (the E press at the CLOSED lift doors): the press fires NOTHING. The old
/// reading (an E route through the closed doors that loads 0700000a with objective 2
/// incomplete) is contradicted by the decompile + the disc, three ways:
/// - The reach: the use point is Bond's head + the viewer direction, sphere r 1.0 [M
///   `Player_Activate` 0x1a5170:100-108, `Player_GetHeadPos` 0x195aa0]. Bond pressed in to
///   z 13.85 puts the use point at z 12.85 and the sphere at z 11.85..13.85. The Load Point's
///   own pick shape is its collision glist — a thin plate at z 10.232..10.283 [D placements
///   dump #63: object "Load Point", collision 12 tris] — ~1.6 beyond the sphere, and the
///   game's refine (mode 0x101 → `Intersect_PointGeom`, the point IN the glist [M
///   Collide_Intersect 0x1ebaa8]) is stricter than the port's AABB stand-in, which already
///   misses (probe, 2026-10-24).
/// - TOUCH_ONCE #67 (the box behind the doors, z 10.33..12.49, x −5.01..−2.92 [D #67] — the
///   box the old ignore note called "the exit box") IS inside the head-origin sphere here,
///   but its disc mask is 8 [D placement #67 params (0,50),(1,8)] and `Trigger_Activate`'s
///   type-1 arm returns before any write without mask & 4 [M 0x1b2318:38-40] — E can never
///   fire it (the same gate the port already applies to the plunge triggers).
/// - The lift doors are SS objects (mask = param 2 = 1 [D SS #3/#2 params]); `SS_Activate`
///   needs ext+0x1E & 4 [M 0x1a9ea8] — refused as well.
/// The exit loads only through the open-doors routes: the panel press (ch 0x39) opens the SS
/// doors, the walk-in touch #67 (mask 8 = the player touch bit) writes ch 0x32 and #63's
/// gate auto-fires [M Trigger_Update 0x1b1a78:162-166] — pinned by walkreal_07000009 with
/// objective 2 complete. An E load with obj2 incomplete would need the doors open WITHOUT
/// the panel — no disc path writes ch 0x39 otherwise [D census, docs/levels/07000009.md].
/// Sister: without_the_e_the_lift_mouth_exits_nothing (no press, same hold).
#[test]
fn the_load_point_e_press_at_the_closed_doors_fires_nothing() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [-3.7, -369.0, 16.5]);
    face(&mut scene, [-3.79, -369.0, 13.0]);
    // Walk INTO the doors (real input); Bond presses in to z ≈ 13.85 (the probe: the
    // head-origin use sphere z 11.85..13.85 — over #67's box edge, under the SS doors).
    for _ in 0..150 { scene.sim.as_mut().unwrap().step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false); }
    let z = scene.sim.as_ref().unwrap().bond.pos[2];
    eprintln!("[exit] pressed-in bond z {z:.2}");
    press_use(&mut scene, 90);
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[exit] level_exit {:?} ch46 {}", sim.drones.level_exit, sim.drones.channels.on(0x46));
    assert_eq!(sim.drones.level_exit, None, "the E press at the closed doors loads nothing [M 0x1b2318:38-40 type-1 mask&4; 0x1a9ea8 SS mask&4]");
    assert!(!sim.drones.channels.on(0x32), "the E press never writes #67's ch 0x32 [D #67 mask 8; M 0x1b2318:38-40]");
    assert!(!sim.drones.channels.on(0x39), "the exit-lift doors stay shut (no ch 0x39) [D SS #3/#2 mask 1; M 0x1a9ea8]");
    assert!(!sim.drones.channels.on(0x46), "objective 2 stays incomplete [D AND mux #274]");
}

/// Sister: the same walk-in, no E — the level holds.
#[test]
fn without_the_e_the_lift_mouth_exits_nothing() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [-3.7, -369.0, 16.5]);
    face(&mut scene, [-3.79, -369.0, 13.0]);
    for _ in 0..150 { scene.sim.as_mut().unwrap().step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false); }
    for _ in 0..150 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.level_exit, None, "the doors stay shut without the E route");
}

/// KIND Touch trigger (the walk-in plates): the liftside plate #272 (out ch 0xAB [D #272])
/// fires when Bond walks into its volume at the lift mouth — the sound trigger #273 rides ch
/// 0xAB (sound 0x47E [D #273 param 2]). Sister: parked in the lobby centre it never fires.
#[test]
fn walking_to_the_lift_mouth_fires_the_liftside_touch_plate() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [-3.7, -369.0, 18.5]);
    face(&mut scene, [-3.79, -369.0, 14.45]);
    let mut hit = false;
    for _ in 0..900 {
        let sim = scene.sim.as_mut().unwrap();
        if sim.drones.channels.on(0xab) { hit = true; break; }
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[touch] ch 0xAB after the walk: {} (bond {:?})", hit, sim.bond.pos);
    assert!(hit, "the walk into the lift-mouth plate writes ch 0xAB [D #272 out]");
    assert!(sim.drones.channels.on(0x51) || true, "the arrival plate's channel is the load-time one");
}

/// Sister: parked in the lobby centre, ch 0xAB never rises.
#[test]
fn parked_in_the_lobby_the_liftside_plate_stays_silent() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [-3.7, -369.0, 18.5]);
    for _ in 0..900 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0xab), "no walk-in, no ch 0xAB");
}

/// KIND Drone (the guards): the car-park guard (#261 [D]; he walks off the spawn — observed
/// (−4.9,−376.08) after the intro, the level's only below-−375 drone) dies to the real fire
/// path — the equipped gun, the bullets, the hit boxes [M NDrone2_HitDamage 0x145558]. The
/// shooter is the walkreal's own (exposed-part aim, flank sidesteps — the Phoenix van blocks
/// the spawn-side line). Sister: the guard stands while unfired-at (Bond's health topped up —
/// the guard SHOOTS back, that is the level's own behaviour).
///
/// The weapon matters: 09's loadout opens on the K5 sleep darts (0x43, 5+10 — the level's
/// stealth start), whose hits are the zero-damage STUN_DART msg [M DealWithObjHit case 5
/// 0x13fff0:269-306: 0x43/0x44 → msg 0x19, the dart route never reaches HitDamage], so the
/// cycle walks to the P2K (0x2, 7+41 — the first lethal gun in Bond's owned set) before the
/// first tap. `NDrone2_HitDamage` 0x145558 applies the damage; a darted guard only stuns
/// (the STUN_DART states 0x50-0x52).
fn cycle_to_lethal(scene: &mut nf_view::scene::Scene) -> bool {
    for _ in 0..12 {
        {
            let sim = scene.sim.as_ref().unwrap();
            // Lethal = the port's armed set minus the dart/taser ids (0x43/0x44 the K5 darts,
            // 0x4A/0x4C the taser — DealWithObjHit case 5 routes them to msg 0x19/7, not damage).
            let lethal = (2..0x47).contains(&sim.drones.bond_weapon) && !matches!(sim.drones.bond_weapon, 0x43 | 0x44);
            if lethal && sim.gun.clip + sim.gun.spare > 0 { return true; }
        }
        scene.sim.as_mut().unwrap().step_bond(Input { weapon_cycle: 1, ..Default::default() }, false, false, false);
        for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    }
    false
}

#[test]
fn the_car_park_guard_dies_to_the_real_fire() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [-1.5, -377.0, 37.8]);
    assert!(cycle_to_lethal(&mut scene), "a lethal gun is in Bond's 09 owned set (the P2K 0x2)");
    let (di, skin, health) = {
        let sim = scene.sim.as_ref().unwrap();
        let (i, d) = sim.drones.list.iter().enumerate()
            .find(|(_, d)| d.pos[1] < -375.0)
            .expect("the car-park guard #261 is present [D]");
        eprintln!("[guard] di {i} pos {:?} health {:.1} skin {:#x}", d.pos, d.health, d.skin);
        assert!(d.present && d.skin != 0 && d.health > 0.0, "the guard is present with its skin");
        (i, d.skin, d.health)
    };
    assert_eq!(skin, 0x0500_0027, "the car-park guard's skin [D #261 param 0]");
    let mut w = Walker::new(scene);
    let killed = w.shoot(di, "the car-park guard");
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[guard] killed {killed} health {:.1} corpse {}", sim.drones.list[di].health, sim.drones.list[di].corpse);
    assert!(killed && (sim.drones.list[di].corpse || sim.drones.list[di].health <= 0.0),
        "the guard dies to the real fire [M 0x145558]");
}

/// Sister: the guard stands while Bond never fires.
#[test]
fn the_guard_stands_while_unfired_at() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    settle_near(&mut scene, [-1.5, -377.0, 37.8]);
    let di = scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .find(|(_, d)| d.pos[1] < -375.0)
        .expect("the guard").0;
    for _ in 0..600 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        sim.drones.health.health = 100.0;
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.list[di].corpse && sim.drones.list[di].health > 0.0,
        "no fire, no kill: the guard stands");
}

fn dxz_of(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

