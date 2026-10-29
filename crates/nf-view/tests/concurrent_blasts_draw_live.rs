//! Concurrent blasts of one script all get a draw slot.
//!
//! The blast effect scripts (0x6000052 and the Destroy_Smash type scripts) run on no placed
//! ScriptPlayer: the sim spawns one player per explosion (`Explode_Create` 0x17cbd0), so several
//! are live at once — 07000014's generator-hall cascade chains its boxes one tick apart while each
//! fireball burns ~165 ticks. With one slot set per script only the newest blast drew. The scene
//! now builds several slot sets per blast script; this pins that a SECOND live blast's entity lands
//! on a slot (k >= 1) during the cascade.
//!
//! Opt-in via NF_ISO.

#[test]
fn the_14_cascade_draws_more_than_one_fireball() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..4000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
        let f = sim.drones.frame;
        sim.drones.channels.set(0x51, 1, f); // the cascade root [D box #400 kill channel]
        for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    }
    let sim = scene.sim.as_ref().unwrap();
    let live = sim.drones.blasts.len();
    assert!(live >= 2, "the cascade has several live blasts ({live})");
    // The blast roots and the fireball sub-script 0x06000047 [L the frag blast's Obj::Entity src].
    let blast = [0x0600_0052u32, 0x0600_0027, 0x0600_004f, 0x0600_0047];
    let second = scene.attachments.iter().filter_map(|a| a.script)
        .filter(|(h, _, _, k)| *k >= 1 && blast.contains(h))
        .filter(|(h, si, hash, k)| sim.script_entity_k((*h, *si), *hash, *k as usize).is_some())
        .count();
    assert!(second > 0, "a second concurrent blast's entity has a draw slot");
}
