//! ai-spawned-objects: the AI-spawned object creators the call-tree audit listed, through the
//! real path (`nf_view::load` + `Sim::step_bond`).
//!
//! - The GT turrets: kind 0x27 placements → `GT_Create` 0x17ffa0's ext map (`crate::gt`).
//!   0700000a carries 6, 0700000b 5, 2 per MP map — 41 disc-wide [D docs/systems/script-census.md:141;
//!   docs/levels/0700000a.md:109, docs/levels/0700000b.md:160]. Every disc turret must carry BOTH
//!   script params (the game deletes a turret whose either Script_Load failed [M 0017ffa0:69,106-109]).
//! - The space laser: the single kind-0x38 placement on 0700001b (#108) → `Create_SpaceLaser`
//!   0x1aa8b8's ext map (`crate::space_laser`) [D script-census.md:144; docs/levels/0700001b.md:71].
//! - The body glow: `BodyGlow_Create` 0x1ac158 runs in every guard's `NDrone2_DefaultInit` tail
//!   [M 0x14b300; docs/atlas/drone.md:51] — 15 child sprites (the do-while counter runs 1..=15 — review 2026-10-21), owner = the guard [M 001ac158].
//!
//! Numbers and ids only. Opt-in via NF_ISO; run with NF_SILENT=1.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(iso: &str, level: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, level, false).expect("load");
    let idle = Input::default();
    for _ in 0..600 { if !scene.sim.as_ref().unwrap().disabled { break; } scene.sim.as_mut().unwrap().step_bond(idle, false, false, false); }
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(idle, false, false, false); }
    scene
}

#[test]
fn the_0700000a_turrets_carry_the_creator_data() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = load(&iso, "0700000a.bin");
    let t = &scene.sim.as_ref().unwrap().drones.gt_turrets;
    // 6 sky-bridge turrets [D docs/levels/0700000a.md:109; script-census.md:141].
    assert_eq!(t.len(), 6, "0700000a kind-0x27 count");
    // The game deletes a turret whose either script param is zero [M 0017ffa0:69] — disc data
    // ships none, so every surviving record carries both script hashes.
    assert!(t.iter().all(|g| g.script_a != 0 && g.script_b != 0), "both scripts loaded [M 0017ffa0:69]");
    // The HP selector: facing 0 → 200, else 0xffff [M 0017ffa0:92-96].
    assert!(t.iter().all(|g| g.hp == 200 || g.hp == 0xffff), "hp is the creator's two values only");
    // obj+0xf4 = 0 at creation [M 0017ffa0:117].
    assert!(t.iter().all(|g| g.state == 0));
    // Sister (can fail): the placements are distinct objects.
    let mut pl: Vec<usize> = t.iter().map(|g| g.placement).collect();
    pl.sort();
    pl.dedup();
    assert_eq!(pl.len(), t.len(), "one record per placement");
}

#[test]
fn the_0700000a_count_is_level_data_not_a_constant() {
    // Sister: 07000005 ships no kind-0x27 placement (script-census.md's 41 live only on
    // 0700000a/0700000b/MP maps) — the walk must find zero there.
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = load(&iso, "07000005.bin");
    assert_eq!(scene.sim.as_ref().unwrap().drones.gt_turrets.len(), 0, "07000005 has no GT turrets [D census]");
}

#[test]
fn the_0700001b_space_laser_record() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = load(&iso, "0700001b.bin");
    let l = &scene.sim.as_ref().unwrap().drones.space_lasers;
    // Exactly one kind-0x38 placement disc-wide: 0700001b #108 [D script-census.md:144].
    assert_eq!(l.len(), 1, "the station's one laser");
    let g = &l[0];
    // The creator's constants [M 001aa8b8]: ext+0x38 = 10.0f, obj+0xf4 = 0 at creation.
    assert_eq!(g.period, 10.0);
    assert_eq!(g.state, 0);
    assert!(g.placement > 0, "sister: the record keys on a real placement index");
}

#[test]
fn every_guard_spawns_with_the_body_glow() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = load(&iso, "07000005.bin");
    let list = &scene.sim.as_ref().unwrap().drones.list;
    assert!(!list.is_empty(), "07000005 starts with guards");
    // DefaultInit's tail creates the glow unconditionally on the shipped path
    // [M 0x14b300 + 001ac158:9 — switch_NO_DRONES is 0].
    assert!(list.iter().all(|d| d.body_glow.is_some()), "every guard glow");
    // The do-while makes exactly 15 children (counter 1..=15, `< 0xf`) [M 001ac158].
    assert!(list.iter().all(|d| d.body_glow.as_ref().unwrap().children == 15));
    // ext[0] = the owner object [M 001ac158] — the port keys it on the drone's list index.
    for (i, d) in list.iter().enumerate() {
        assert_eq!(d.body_glow.as_ref().unwrap().owner, i, "glow owner = drone index");
    }
    // Sister (can fail): the gfx hashes are the creator's two, not swapped.
    assert_ne!(nf_game::body_glow::MAIN_GFX, nf_game::body_glow::CHILD_GFX);
}
