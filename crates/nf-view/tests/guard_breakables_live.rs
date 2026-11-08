//! guard-breakables: a guard's round damages and breaks the breakables its flight crosses, the
//! same way Bond's rounds do. The game's pick and contact chain never reads the round's owner:
//! `Bullet_update` 0x12e7b0:388-443 arms the same HITTEST pick for every round, `Collide_Update`
//! 0x1ea398:216-229 ends the sorted walk on the first solid node, and `Break_Update` 0x12bfd8:15-37
//! drains every class-5 contact that survives `Collide_FilterBullets(filter | 0x106)`
//! [M 0x1ece30:24-33]; health < 0 → `Break_Kill` 0x12b970. A pane whose shape carries surface byte
//! 0x10 passes a gun round without `weapon_data+0x6C & 0x800` [M Bullet_CollisionHandler 0x12ce80
//! 0x12cf54-0x12cf60, the player walk's rule]; any other breakable is solid and ends the round
//! [M 0x12cff0-4]. Either way the contact drains the breakable.
//!
//! 07000001 carries 55 placed breakables: 12 kind-2 panes, 41 kind-0 and 2 kind-1 pieces, the pools 0.0 —
//! the first positive-damage contact kills [D placements; M 0x12c230:21-25]. The round is staged
//! from a real guard of the level with that guard's own weapon record and owner index (the shape
//! `Drone::fire` builds), 1.5 units above the target, flying straight down.
//!
//! Opt-in via NF_ISO.

use nf_game::bullet::Bullet;
use nf_game::player::Input;

fn load() -> Option<nf_view::scene::Scene> {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let iso = std::env::var("NF_ISO").ok()?;
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    Some(scene)
}

/// A real guard of the level holding a gun whose record deals contact damage: (index, weapon).
fn guard_gun(sim: &nf_view::scene::Sim) -> (usize, u16) {
    let w = &sim.weapons;
    sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present)
        .map(|(i, d)| (i, d.weapon as u16))
        .find(|&(_, wp)| w.bullet_contact_damage(wp) > 0.0
            && w.records.get(wp as usize).is_some_and(|r| u16::from_le_bytes([r[0x10], r[0x11]]) & 0x106 == 0))
        .expect("a 07000001 guard holds a gun [D]")
}

/// The first breakable of `kind` (skipping the hall window #130 the opening dive breaks) whose
/// filter passes the round, and one unit of clear air above it to stage the round from.
fn target(sim: &nf_view::scene::Sim, kind: u8, word: u16) -> usize {
    sim.drones.breaks.list.iter().enumerate()
        .filter(|(_, b)| b.kind == kind && !b.dead && b.placement != 130 && word & (b.filter | 0x106) == 0)
        .find(|(_, b)| {
            let c = centre(b);
            // Clear air between the staging point and the target's top (no world stop first).
            let from = [c[0], b.hi[1] + 1.5, c[2]];
            sim.world.segment_bullet(from, [c[0], b.hi[1], c[2]]).is_none()
        })
        .map(|(i, _)| i)
        .unwrap_or_else(|| panic!("a kind-{kind} breakable with clear air above it [D 07000001]"))
}

fn centre(b: &nf_game::breakable::Break) -> [f32; 3] {
    [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0]
}

/// Stages the guard's round `dx` to the side of the target's centre column, 1.5 above its top,
/// flying straight down, and steps ten ticks.
fn shoot_down(scene: &mut nf_view::scene::Scene, guard: usize, weapon: u16, i: usize, dx: f32) {
    let sim = scene.sim.as_mut().unwrap();
    let b = &sim.drones.breaks.list[i];
    let c = centre(b);
    let r = &sim.weapons.records[weapon as usize];
    let f = |o: usize| f32::from_le_bytes(r[o..o + 4].try_into().unwrap());
    let pos = [c[0] + dx, b.hi[1] + 1.5, c[2]];
    sim.drones.bullets.push(Bullet {
        pos, dir: [0.0, -1.0, 0.0], speed: f(0x20), travelled: 0.0, range: f(0x1c),
        damage: f(0x0c), owner: guard, state: 0, weapon, flyby: false, bounces: 0, ..Bullet::TAIL
    });
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
}

#[test]
fn a_guard_round_breaks_the_vase_it_crosses_and_dies_on_it() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    let (guard, weapon) = guard_gun(scene.sim.as_ref().unwrap());
    let word = { let r = &scene.sim.as_ref().unwrap().weapons.records[weapon as usize]; u16::from_le_bytes([r[0x10], r[0x11]]) };
    let vase = target(scene.sim.as_ref().unwrap(), 0, word);
    let placement = scene.sim.as_ref().unwrap().drones.breaks.list[vase].placement;
    let kills0 = scene.sim.as_ref().unwrap().drones.break_kills.len();
    // The sister first: the same round one unit beside the vase's column touches nothing.
    shoot_down(&mut scene, guard, weapon, vase, 1.2);
    assert!(!scene.sim.as_ref().unwrap().drones.breaks.list[vase].dead, "a round that misses the vase leaves it whole");
    shoot_down(&mut scene, guard, weapon, vase, 0.0);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.breaks.list[vase].dead,
        "guard {guard}'s weapon-{weapon:#x} round crossed vase #{placement} and must break it [M 0x12bfd8:15-37, 0x12b970]");
    assert!(sim.drones.break_kills[kills0..].iter().any(|k| k.0 == placement && k.1 == 0),
        "the kill records for the view [M 0x12b970]");
    assert!(!sim.drones.bullets.iter().any(|b| b.owner == guard && b.weapon == weapon && b.dir == [0.0, -1.0, 0.0]),
        "the round dies on the solid vase [M 0x12cff0-4]");
}

#[test]
fn a_guard_round_shatters_the_glass_pane_it_crosses() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    let (guard, weapon) = guard_gun(scene.sim.as_ref().unwrap());
    let word = { let r = &scene.sim.as_ref().unwrap().weapons.records[weapon as usize]; u16::from_le_bytes([r[0x10], r[0x11]]) };
    let pane = target(scene.sim.as_ref().unwrap(), 2, word);
    let placement = scene.sim.as_ref().unwrap().drones.breaks.list[pane].placement;
    shoot_down(&mut scene, guard, weapon, pane, 0.0);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.breaks.list[pane].dead,
        "guard {guard}'s weapon-{weapon:#x} round crossed pane #{placement} and must shatter it [M 0x12bfd8:15-37, 0x12b970]");
    assert!(sim.drones.break_kills.iter().any(|k| k.0 == placement && k.1 == 2 && k.2 == 0x2e),
        "the glass kill plays 0x2e [M 0x12b970:65-69]");
}
