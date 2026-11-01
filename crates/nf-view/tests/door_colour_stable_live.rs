//! Doors keep their colour while pickups pile up during play (07000011 stairwell, 07000014).
//!
//! The GPU colour table is laid out once at build: [characters][attachments][pickups][searchlight
//! parts][door-region slots][col_one] (`Scene::skin_layout`). The sim's pickup list grows during
//! play: every guard death drops his gun as a NEW pickup (`DroneWeap_DropWeapon` → `Pickup_Create`
//! 0x18dda0 [M]). Those late pickups have no slot in the table (the drop draws as its attachment).
//! The palette used to write their ambient multiplier past the pickup section, into the door
//! colours (before the per-door section existed: into `col_one`, the entry every door shared), so
//! the doors went dark one kill after another, tinted the cel ambient where the gun landed. The
//! game has no such coupling: each door is its own object and `View_SetupRenderModes_Tweak`
//! (0x1e5888) tints it from its own obj+0x100..0x102 bytes, which only the door's own create/update
//! writes [M Door_Create 0x136100 → Control_SetGList 0x134960].
//!
//! These tests pin that state: a real kill on 07000011 and a flood of late pickups on 07000011 and
//! 07000014 leave every door-region colour and `col_one` exactly as they were, and a sister proves
//! that the unbounded write would have landed in that section with a non-white colour.
//!
//! Opt-in via NF_ISO.

#[path = "common/walker_b2.rs"]
mod walker;
use walker::Walker;
use nf_view::scene::{Scene, SkinLayout, ViewOpts};

fn settled(level: &str) -> Walker {
    let iso = std::env::var("NF_ISO").unwrap();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, level, false, None).unwrap());
    for _ in 0..4000 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        if !sim.disabled { break; }
    }
    for _ in 0..60 { w.tick(Default::default(), false); }
    w
}

/// The door-region colours and the shared `col_one`, as the GPU palette writes them this frame.
fn door_cols(s: &Scene, l: &SkinLayout) -> Vec<[f32; 4]> {
    let (_, cols) = s.skin_palette(l, None, 0.0, ViewOpts::default(), None);
    cols[l.col_door as usize..=l.col_one as usize].to_vec()
}

/// Late pickups (more than the whole door section) placed in a dimly lit cel; the ones past the
/// build's pickup section must not reach the door colours.
fn flood(level: &str) {
    if std::env::var("NF_ISO").is_err() { return }
    let mut w = settled(level);
    let (l, _) = w.scene.skin_layout();
    let n_pick = (l.col_sl - l.col_pick) as usize;
    let n_door = (l.col_one - l.col_door) as usize;
    assert!(n_door > 0, "{level} has door colour slots");
    let before = door_cols(&w.scene, &l);
    // A pickup template at a spot whose cel ambient is not white (the drop's tint).
    let sim = w.scene.sim.as_ref().unwrap();
    // A gun or ammo pickup (armour and the 007 bonus draw untinted [M Pickup_Create 0x18dda0]).
    let tmpl = sim.drones.pickups.iter()
        .find(|p| p.kind != nf_game::pickup::kind::ARMOUR && p.kind != nf_game::pickup::kind::BOND_BONUS)
        .cloned().expect("a placed tintable pickup");
    let dark = w.scene.cels.iter()
        .map(|c| [0, 1, 2].map(|k| (c.min[k] + c.max[k]) * 0.5))
        .find(|&p| sim.ambient_at(p).is_some_and(|a| a.iter().any(|&b| b < 0xf0)))
        .expect("a cel with a non-white ambient");
    let sim = w.scene.sim.as_mut().unwrap();
    for k in 0..n_door + 4 {
        let mut p = tmpl.clone();
        p.gone = false;
        p.hidden = false;
        p.m[3] = [dark[0] + 0.01 * k as f32, dark[1], dark[2], 1.0];
        sim.drones.pickups.push(p);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.pickups.len() > n_pick + n_door, "the flood outgrows the pickup section and the door section");
    let after = door_cols(&w.scene, &l);
    for (k, (a, b)) in before.iter().zip(after.iter()).enumerate() {
        assert_eq!(a, b, "{level}: door-region colour {k} (of {n_door}; {n_door} = col_one) changed after late pickups");
    }
    // Sister: the multipliers the old unbounded loop wrote for those late pickups are not white, and
    // their unbounded index (`col_pick + pk`) lands inside the door section — the check can fail.
    let (_, pm, _) = w.scene.colour_mults_dense();
    let hits: Vec<usize> = (n_pick..pm.len())
        .filter(|&pk| { let i = l.col_pick as usize + pk; i >= l.col_door as usize && i <= l.col_one as usize })
        .filter(|&pk| pm[pk] != [1.0; 3])
        .collect();
    assert!(!hits.is_empty(), "{level}: an unbounded pickup write would tint a door slot");
}

#[test]
fn late_pickups_leave_the_07000011_doors_alone() { flood("07000011.bin") }

#[test]
fn late_pickups_leave_the_07000014_doors_alone() { flood("07000014.bin") }

/// The real path on 07000011: kill guards on the fire path until one drops his gun as a new
/// pickup, then run on; the door colours stay as they were at the start.
#[test]
fn a_kill_on_07000011_leaves_the_doors_alone() {
    if std::env::var("NF_ISO").is_err() { return }
    let mut w = settled("07000011.bin");
    let (l, _) = w.scene.skin_layout();
    let before = door_cols(&w.scene, &l);
    let n0 = w.scene.sim.as_ref().unwrap().drones.pickups.len();
    let mut tried = Vec::new();
    for _ in 0..6 {
        if w.scene.sim.as_ref().unwrap().drones.pickups.len() > n0 { break; }
        let di = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            (0..sim.drones.list.len())
                .filter(|i| !tried.contains(i))
                .filter(|&i| { let d = &sim.drones.list[i]; d.present && d.enabled && !d.corpse && d.health > 0.0 })
                .min_by(|&a, &b| {
                    let f = |p: [f32; 3]| (p[0] - bp[0]).powi(2) + (p[2] - bp[2]).powi(2);
                    f(sim.drones.list[a].pos).total_cmp(&f(sim.drones.list[b].pos))
                })
        };
        let Some(di) = di else { break };
        tried.push(di);
        // Test setup, not the path under test: stand 3 units from the guard so the fire path reaches him.
        { let sim = w.scene.sim.as_mut().unwrap(); let dp = sim.drones.list[di].pos; sim.bond.pos = [dp[0] - 3.0, dp[1], dp[2]]; }
        let _ = w.resupply("ammo");
        w.shoot(di, "guard");
        for _ in 0..90 { w.tick(Default::default(), false); }
    }
    let n1 = w.scene.sim.as_ref().unwrap().drones.pickups.len();
    assert!(n1 > n0, "a guard death drops a gun as a new pickup ({n0} -> {n1}) [M 0x18dda0]");
    for _ in 0..600 { w.tick(Default::default(), false); }
    let after = door_cols(&w.scene, &l);
    assert_eq!(before, after, "07000011: the door colours stay as they were after the drop");
}
