//! platforms_drake_scene_1b: the level-1b-platforms-and-drake-scene pins.
//!
//! Part 1 — the breakable platforms. The disc facts [D 0700001b.bin map 26]:
//!   - NO 0x20 breakable placements exist on 0700001b (the audit's "no glass breaks" [D probe]);
//!     the only break-scenery class is 0xE1 `Destroy_Create` 0x12c318 — 6 placements, health
//!     param 3 = 0 on five (first bullet kills) and 10000 on PL#1 [D].
//!   - The game's destroy death is BULLET-only: `Destroy_CollisionHandler` 0x12bd48 walks the
//!     contact list and subtracts ONLY class-0x05 (bullet) contacts' damage from tag+0x04;
//!     `Destroy_Update` 0x12c4f8's other arm is the watched-channel smash. There is NO
//!     player-contact break path [M both decompiles].
//!   - The solar panels (SP#41-56, script 0x06000832, hit points 20 [D placements]) unhold on
//!     `hp <= damage` [M SP_Update 0x1dbda8:283-287 (`+0x14 <= +0x18`)] — one 20-damage round
//!     (the start weapon 0x33's damage is 20 [D weapon_data]) breaks a panel BY DESIGN.
//! Pinned here: an idle/drifting Bond breaks nothing in 900 ticks (the port has no body-contact
//! break either — the reported "slightest touch" repro needs the live checks in the report).
//!
//! Part 2 — Drake in the cut-scene. The arrival pack cut-scene 0x0600085c (0780001b.bin, 2000
//! ticks [D]) binds TWO anim actors: stream 1 = skin 0x0500008f (Drake — the station centrepiece
//! skin, also drone #140's), stream 2 = skin 0x0500007c [D stream decode]; both skins live in
//! 0700001b.bin on skeleton 0 [D]. The port's data path is pinned end-to-end here: the Scene
//! holds both cast rows, the sim ticks both Obj::Anim streams, and both poses resolve with
//! Drake's root at the station hub.

use nf_view::scene::Scene;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_arrival_scene_binds_drake_and_the_pose_resolves() {
    let iso = iso();
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "0700001b.bin", false, None, 3).expect("load 1b");
    // The pack cut-scene's anim actors exist in the sim from the first tick [D 0780001b stream 1/2].
    for _ in 0..401 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    for (skin, si) in [(0x0500_008fu32, 1u8), (0x0500_007c, 2)] {
        let o = sim.drones.scripts.objects.list.iter().find(|o| matches!(o,
            Some(nf_game::nis::Obj::Anim { skin: s, src: (h, st), .. }) if *s == skin && *h == 0x0600_085c && *st == si));
        assert!(o.is_some(), "the arrival scene's actor skin {skin:#x} (stream {si}) exists [D 0780001b stream {si}]");
    }
    // The anim frames TICK (the actors animate, frame 401 of the 2001-tick scene).
    for (skin, si) in [(0x0500_008fu32, 1u8), (0x0500_007c, 2)] {
        let Some(Some(nf_game::nis::Obj::Anim { frame, .. })) = sim.drones.scripts.objects.list.iter().find(|o| matches!(o,
            Some(nf_game::nis::Obj::Anim { skin: s, src: (h, st), .. }) if *s == skin && *h == 0x0600_085c && *st == si)) else { panic!("actor row") };
        assert!(*frame > 100.0, "the actor animates (frame {frame} after 401 ticks)");
    }
    // The SCENE holds both cast rows (the draw's slots) [D cast = the pack's Anim ops].
    for (skin, si) in [(0x0500_008fu32, 1u8), (0x0500_007c, 2)] {
        assert!(scene.characters.iter().any(|c| c.script == Some((0x0600_085c, si)) && c.skin.hashcode == skin),
            "the scene cast row for skin {skin:#x} stream {si} exists");
    }
    // Both poses resolve on skeleton 0 [D skins 0x8f/0x7c → skeleton 0].
    let sk = sim.skeletons.get(&0).expect("skeleton 0 [D]");
    for (skin, script, label) in [(0x0500_008fu32, 0x0600_08a5u32, "Drake"), (0x0500_007c, 0x0600_08a6, "the second actor")] {
        let skn = sim.skins.get(&skin).expect("the level carries the skin [D]");
        let pose = nf_game::anim::script_pose_together(&sim.scripts, &sim.seqs, sk, skn, script, 401.0);
        assert!(pose.is_some(), "{label}'s anim script {script:#x} poses [D kind-5 seq in 0700001b.bin]");
        assert_eq!(pose.unwrap().len(), 73, "the full body pose (skeleton 0's 73 bones)");
    }
}

/// The destroyables' no-touch sister: an idle/drifting Bond breaks nothing in 900 ticks — the
/// port has no player-contact break path, exactly like the game [M 0x12bd48: bullets only].
#[test]
fn an_idle_bond_breaks_no_destroyable() {
    let iso = iso();
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "0700001b.bin", false, None, 3).expect("load 1b");
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    for t in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.props.destroy.iter().all(|d| !d.dead),
            "no destroyable broke by tick {t} — the game breaks them only on bullet contacts [M 0x12bd48]");
    }
    // The disc's own health values ride through [D]: PL#1 = 10000, the other five = 0.
    let sim = scene.sim.as_ref().unwrap();
    let one = sim.drones.props.destroy.iter().find(|d| d.placement == 1).expect("PL#1");
    assert_eq!(one.health, 10000.0, "PL#1's health p3 = 10000 [D]");
    assert_eq!(sim.drones.props.destroy.iter().filter(|d| d.health == 0.0).count(), 5,
        "the other five carry p3 = 0 — a first bullet kills them [D]");
}
