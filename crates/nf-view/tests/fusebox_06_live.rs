//! level-06 interactables: the FuseBox #747 (0x31, script 0x0600004B, params 57/55/103/55) through
//! the REAL input path — Bond takes a stance at the guardroom nook, equips the Laser (0x4E, the
//! disc's own cutting laser), and burns the wire bundles [M FuseBox_Create 0x1b0448 /
//! FuseBox_Update 0x1adfe8 / FuseBox_Register 0x1b0970; D #747 params: group 0 → ch 57 + 103,
//! group 1 → ch 55 + 55].
//!
//! The game law [M 0x1adfe8]: a wire whose hit list holds a contact that SURVIVES
//! `Collide_FilterBullets(list, 0xfffd)` dies (`Effect_RemoveSObjs`); guns (0x0008) are filtered,
//! the Laser (0x0002) cuts [L weapon_data]; a group emptied with a non-zero registered count writes
//! both its channel words one-shot; a group-1 death hurts Bond 5.0 IFF +0x56 == 0 — #747's +0x56 is
//! 55, so the alarm bundle never hurts. The sisters hold fire / hold E: no contact, no channel.
//!
//! Opt-in via NF_ISO (the slim ISO is enough). Mechanics follow fix_fusebox_cut_live's proven aim
//! loop (the stance settle + direct aim + hold fire).

use nf_game::player::{Input, RamSave};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn load_06(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load_with_carry(iso, "07000006.bin", false, Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    assert!(!sim.disabled, "Bond is free after the intro");
    scene
}

/// Settle Bond at the fusebox nook (the box at (28.00, 40.00, −18.15) [D #747]) and return the
/// group's live wires as (script-object index, the entity's translation) pairs — the burn aims at
/// the VISIBLE geometry through the object (`wire_aim_point`), not the translation.
fn stance_and_wires(scene: &mut nf_view::scene::Scene, want_group: usize) -> Vec<(usize, [f32; 3])> {
    let sim = scene.sim.as_mut().unwrap();
    let spot = [29.9f32, 41.0, -17.9];
    let floor = sim.world.segment(spot, [spot[0], spot[1] - 30.0, spot[2]]).map(|t| spot[1] - 30.0 * t);
    if let Some(f) = floor { sim.bond.pos = [spot[0], f + 0.1, spot[2]]; } else { sim.bond.pos = spot; }
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let fb = sim.drones.fuseboxes.first().expect("the 06 fusebox #747 exists");
    eprintln!("[fusebox] ch {:?} wires {:?}", fb.ch,
        fb.alive_wires().map(|(g, k, o)| (g, k, o)).collect::<Vec<_>>());
    fb.alive_wires().filter(|(g, _, _)| *g == want_group)
        .map(|(_, _, o)| (o, nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), o).unwrap_or([0.0; 3])))
        .collect()
}

/// The wire object's VISIBLE geometry: its local bounds (`Drones::wire_geo`, from the box script's
/// Entity streams' meshes) through the live entity matrix — the same world slab the bullet pass
/// tests the round against (`Collide_PickObj` 0x1eae80 → obj+0xD0 → `FuseBox_Update`
/// 0x1adfe8:88-123; the port's slab test in drone.rs's bullet pass). None when the object has no
/// resolved geometry (the pass then falls back to the translation sphere).
fn wire_world_box(sim: &nf_view::scene::Sim, obj: usize) -> Option<([f32; 3], [f32; 3])> {
    match sim.drones.scripts.objects.list.get(obj)? {
        Some(nf_game::nis::Obj::Entity { hash, matrix, .. }) => {
            let (lo, hi) = sim.drones.wire_geo.get(hash)?;
            let (mut wlo, mut whi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for cx in 0..2 { for cy in 0..2 { for cz in 0..2 {
                let p = [if cx == 0 { lo[0] } else { hi[0] }, if cy == 0 { lo[1] } else { hi[1] }, if cz == 0 { lo[2] } else { hi[2] }];
                let w = nf_formats::placement::transform(matrix, p);
                for c in 0..3 { wlo[c] = wlo[c].min(w[c]); whi[c] = whi[c].max(w[c]); }
            } } }
            Some((wlo, whi))
        }
        _ => None,
    }
}

/// Where a player aims to burn the wire: the VISIBLE cable's centre (the wire object's world
/// bounds), falling back to the script entity's translation when no geometry resolves.
fn wire_aim_point(sim: &nf_view::scene::Sim, obj: usize) -> [f32; 3] {
    if let Some((lo, hi)) = wire_world_box(sim, obj) {
        return [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
    }
    nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), obj)
        .unwrap_or([0.0; 3])
}

/// Hold fire and walk the crosshair onto the wire's VISIBLE geometry the way a player does: each
/// tick aims at `aim + delta`, reads the Laser round the tick spawned and shifts `delta` by that
/// round's miss against the wire's own bounds (the contact chain above). `done` is the test's own
/// success probe (the channel the cut writes). Returns whether `done` fired.
fn burn_wire(scene: &mut nf_view::scene::Scene, obj: usize, max_ticks: u32, mut done: impl FnMut(&nf_view::scene::Sim) -> bool) -> bool {
    let mut delta = [0.0f32; 3];
    for _ in 0..max_ticks {
        let round = {
            let sim = scene.sim.as_mut().unwrap();
            let aim = wire_aim_point(sim, obj);
            let target = [aim[0] + delta[0], aim[1] + delta[1], aim[2] + delta[2]];
            let head = sim.bond.head();
            let dir = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
            let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
            sim.bond.yaw = dir[0].atan2(dir[2]);
            sim.bond.pitch = (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2;
            let n0 = sim.drones.player_bullets.len();
            sim.step_bond(Input::default(), true, false, false);
            let hit = done(sim);
            let shot = if hit { None } else {
                sim.drones.player_bullets.get(n0).filter(|b| b.weapon == 0x4e)
                    .map(|b| (b.pos, b.dir, b.speed))
            };
            shot
        };
        if round.is_none() && done(scene.sim.as_ref().unwrap()) { return true; }
        // The tick's first Laser round: walk the crosshair by the round's miss against the
        // wire's own slab (the player watches the beam and corrects).
        if let Some((pos, dir, speed)) = round {
            let sim = scene.sim.as_ref().unwrap();
            if let Some((lo, hi)) = wire_world_box(sim, obj) {
                let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
                let e = [pos[0] + dir[0] * speed, pos[1] + dir[1] * speed, pos[2] + dir[2] * speed];
                let ab = [e[0] - pos[0], e[1] - pos[1], e[2] - pos[2]];
                let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
                let t = if ab2 > 1e-9 { (((centre[0] - pos[0]) * ab[0] + (centre[1] - pos[1]) * ab[1] + (centre[2] - pos[2]) * ab[2]) / ab2).clamp(0.0, 1.0) } else { 0.0 };
                let closest = [pos[0] + ab[0] * t, pos[1] + ab[1] * t, pos[2] + ab[2] * t];
                for c in 0..3 {
                    delta[c] = (delta[c] + (centre[c] - closest[c])).clamp(-0.8, 0.8);
                }
            }
        }
    }
    done(scene.sim.as_ref().unwrap())
}

fn draw_laser(scene: &mut nf_view::scene::Scene) {
    let draw = Input { switch_to: Some(0x4e), ..Default::default() };
    for _ in 0..40 { scene.sim.as_mut().unwrap().step_bond(draw, false, false, false); }
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the Laser equips [D USATxt label 0x05000043]");
}

fn aim_tick(scene: &mut nf_view::scene::Scene, wire: [f32; 3], fire: bool) -> bool {
    let hit_ch = |sim: &nf_view::scene::Sim| sim.drones.channels.on(57) || sim.drones.channels.on(55);
    {
        let sim = scene.sim.as_mut().unwrap();
        let head = sim.bond.head();
        let dir = [wire[0] - head[0], wire[1] - head[1], wire[2] - head[2]];
        let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
        sim.bond.yaw = dir[0].atan2(dir[2]);
        sim.bond.pitch = (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        sim.step_bond(Input::default(), fire, false, false);
    }
    hit_ch(scene.sim.as_ref().unwrap())
}

/// KIND FuseBox: the laser burns the searchlight bundle (group 0) → ch 57 (the searchlights-off
/// gate) + ch 103 fire one-shot; the dying wire's entity is removed. Sister (last test): holding
/// fire keeps every bundle and channel untouched.
#[test]
fn the_laser_cut_through_the_searchlight_bundle_kills_the_lights() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso);
    let wires = stance_and_wires(&mut scene, 0);
    assert!(!wires.is_empty(), "the box script 0x0600004B registered group-0 wires at load [M FuseBox_Register 0x1b0970]");
    let (obj, spos) = wires[0];
    {
        let sim = scene.sim.as_ref().unwrap();
        let aim = wire_aim_point(sim, obj);
        eprintln!("[fusebox] burning group-0 wire obj {obj}: entity translation {spos:?}, visible cable at {aim:?}");
    }
    draw_laser(&mut scene);
    let hit = burn_wire(&mut scene, obj, 900, |sim| sim.drones.channels.on(57) || sim.drones.channels.on(103));
    let sim = scene.sim.as_ref().unwrap();
    assert!(hit, "the group-0 bundle empties: ch 57 (the searchlights-off gate) or ch 103 rises");
    assert!(sim.drones.channels.on(57), "ch 57 — the searchlights' gate — is written [D #747 p1; M 0x1adfe8:110-115]");
    assert!(sim.drones.channels.on(103), "the bundle's second word ch 103 fires with it [D #747 p3]");
    let fb = sim.drones.fuseboxes.first().unwrap();
    assert_eq!(fb.ch[0], 0, "group 0's words are zeroed after the write (one-shot) [M 0x1adfe8:113-122]");
    assert!(fb.alive_wires().filter(|(g, _, _)| *g == 0).next().is_none(), "the group-0 wires are gone [M Effect_RemoveSObjs]");
    assert!(!sim.drones.player_hits.iter().any(|(_, who, _)| *who == usize::MAX),
        "no live-wire hurt from a group-0 death [M 0x1adfe8:98 — the hurt is the group-1 arm]");
    // The game's own consumers see the channel: the lights-off clunk trigger #703 (ch 57 → sound 241).
    assert!(sim.drones.channels.on(57), "the ch-57 consumers' gate is on");
}

/// KIND FuseBox (the alarm bundle): cutting group 1 raises ch 55 — the alarm channel the siren
/// (#702) and the guards ride — and never hurts Bond (+0x56 = p2 = 55 ≠ 0 [M 0x1adfe8:99-101;
/// D #747 params]).
#[test]
fn the_alarm_bundle_raises_ch55_and_never_hurts() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso);
    let wires = stance_and_wires(&mut scene, 1);
    assert!(!wires.is_empty(), "the box script registered group-1 wires [D #747 p2/p4]");
    let (obj, _spos) = wires[0];
    draw_laser(&mut scene);
    let health0 = scene.sim.as_ref().unwrap().drones.health.health;
    let hit = burn_wire(&mut scene, obj, 900, |sim| sim.drones.channels.on(55));
    let sim = scene.sim.as_ref().unwrap();
    assert!(hit && sim.drones.channels.on(55), "the alarm bundle empties: ch 55 rises");
    assert_eq!(sim.drones.health.health, health0, "no live-wire hurt: +0x56 = 55 is configured [M 0x1adfe8:99-101]");
    assert!(!sim.drones.player_hits.iter().any(|(_, who, _)| *who == usize::MAX), "no fusebox hurt marker");
}

/// Sister A: the Laser drawn and AIMED but never fired — no contact ever lands, the wires live,
/// and ch 57/103/55 stay low.
#[test]
fn holding_fire_leaves_the_bundles_and_their_channels_untouched() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso);
    let wires = stance_and_wires(&mut scene, 0);
    let (obj, _spos) = wires[0];
    draw_laser(&mut scene);
    for _ in 0..600 {
        let aim = wire_aim_point(scene.sim.as_ref().unwrap(), obj);
        aim_tick(&mut scene, aim, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    let fb = sim.drones.fuseboxes.first().unwrap();
    assert!(!sim.drones.channels.on(57) && !sim.drones.channels.on(103) && !sim.drones.channels.on(55),
        "no fire, no contact, no channel");
    assert!(fb.alive_wires().filter(|(g, _, _)| *g == 0).next().is_some(), "the group-0 wire survives the aim-only hold");
}

/// Sister B: pressing E at the fusebox does nothing — `Player_Activate` 0x1a5170 has no class-0x3C
/// arm [M]; the wires and channels stay untouched.
#[test]
fn e_presses_do_nothing_to_the_fusebox() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso);
    let wires = stance_and_wires(&mut scene, 0);
    let wire = wires[0];
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, true); }
    for _ in 0..80 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, true); }
    let sim = scene.sim.as_ref().unwrap();
    let fb = sim.drones.fuseboxes.first().unwrap();
    assert!(!sim.drones.channels.on(57) && !sim.drones.channels.on(55), "E never cuts a fusebox wire [M 0x1a5170:95-160]");
    assert!(fb.alive_wires().filter(|(g, _, _)| *g == 0).next().is_some(), "the wire is untouched");
    let _ = wire;
}
