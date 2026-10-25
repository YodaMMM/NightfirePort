//! level-14-empty-panels: the two "empty" spots from real play, through the real use path.
//!
//! THE VENT SPOT (54.21, 1.03, 46.15, frame 5739): "a vent/hatch opening with nothing beneath it".
//! The nearest dressing is the route lid SP #849 "Vent_StoreRoom_Graphic" (51.58, 3.58, 44.94)
//! [D placement] — one of the fifteen kind-0xd9 mode-0x40 preloads of script 0x06000A3B [D §1a].
//! The script's stream 0 spawns the fusebox prop 0x02000750 (type 0x31) at key 1 and the ladder
//! art 0x02000751 (type 0x19) at key 61 [D probe14_lidscript decode]; the mode-0x40 preload runs
//! ONE tick ([M] SP_LoadScript 0x1dbc08:46-54 — any mode outside {1,8} runs Play+Update+Stop,
//! state 1), so only the key-1 prop exists at load and the ladder art needs the SP's REPLAY:
//! [M] SP_Update 0x1dbda8 state 1 — on the trigger mask (bit 4 = the +0x2C activation word
//! SP_Activate 0x1dc338 sets on the player's use) the script re-plays from key 0 to its end.
//!
//! THE PANEL SPOT (−88.58, −1.10, 46.15, frame 25538): "a wall panel with nothing behind it" —
//! Bond faces the west generator room's fusebox wall (the real fusebox #385 "OBJECT_FuseBox1"
//! (−89.30, −2.22, 49.36) with the small door object #386 "OBJECT_Door" beside it [D]). The
//! fusebox's script 0x06000A1B [D placement params] spawns the wire sub-objects 0x02000A62 /
//! 0x02000A61 / 0x02000A73 (type 0x31) in streams 0/1/3 [D decode] — the art behind the fusebox
//! front the doc's fusebox.rs currently stands in for (docs/levels/07000014.md §5 "the wire
//! sub-objects are the documented stand-in").
//!
//! Assertion level: the script decodes and the preload rule are [D]/[M] as cited; whether the
//! port's real use path reaches each spawn is what this test pins.
#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn entities_near(sim: &nf_view::scene::Sim, hashes: &[u32]) -> Vec<(u32, u8, [f32; 3])> {
    sim.drones.scripts.objects.list.iter().flatten()
        .filter_map(|o| match o {
            nf_game::nis::Obj::Entity { hash, kind, matrix, .. } if hashes.contains(hash) =>
                Some((*hash, *kind, [matrix[3][0], matrix[3][1], matrix[3][2]])),
            _ => None,
        }).collect()
}

/// The route lid at (51.58, 3.58, 44.94): before any use, only the key-1 fusebox prop exists;
/// the use press FROM INSIDE THE DUCT (the SP's use sphere rides its live matrix at y 3.58
/// [M SP_Activate 0x1dc338]) replays the script and the ladder art 0x02000751 spawns at the
/// lid's base — the art beneath the opening. From the facility floor (y 1.03, the reported spot)
/// the sphere is out of reach (d ≈ 2.6): the crawl must E-open the lid before descending, or the
/// opening shows with nothing beneath it — the reported symptom and the route legs' open question
/// (docs/levels/07000014.md §1a).
#[test]
fn the_route_lid_use_spawns_the_ladder_art_beneath_the_opening() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
        // The lid SP records: scripts, modes, gate, out, mask — the replay chain's inputs.
        for (i, p) in sim.drones.scripts.players.iter().enumerate() {
            if p.scripts.contains(&0x0600_0a3b) {
                eprintln!("[lid] players[{i}] placement {} state {} gate {} out {:#x} mask {:#x} modes {:?} stay {}",
                    p.placement, p.state, p.gate, p.out, p.mask, p.modes, p.stay);
            }
        }
        let pre = entities_near(sim, &[0x0200_0750, 0x0200_0751]);
        eprintln!("[lid] at load: {pre:?}");
        assert!(pre.iter().any(|(h, _, _)| *h == 0x0200_0750), "the key-1 fusebox prop exists after the preload [M 0x1dbc08 mode-64 arm; D 0x06000A3B s0 t1]");
        assert!(!pre.iter().any(|(h, _, _)| *h == 0x0200_0751), "the ladder art (key 61) does not exist at load — the one-tick preload cannot reach it [M 0x1dbc08; D 0x06000A3B s0 t8]");
        // Bond INSIDE the duct at the lid opening (the lid SP's use sphere rides its live matrix
        // at (51.58, 3.58, 44.94) [M SP_Activate 0x1dc338]), facing it; the use press through the
        // real input path.
        sim.bond.pos = [51.58, 3.50, 45.60];
        sim.bond.yaw = std::f32::consts::PI; // face −z toward the lid at z 44.94
        for k in 0..4 { sim.step_bond(Input::default(), false, false, k == 0); }
    }
    for t in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    let post = entities_near(sim, &[0x0200_0750, 0x0200_0751]);
    eprintln!("[lid] after the use + 120 ticks: {post:?}");
    let ladder = post.iter().filter(|(h, _, _)| *h == 0x0200_0751).collect::<Vec<_>>();
    assert!(!ladder.is_empty(), "the use at the route lid replays script 0x06000A3B and the ladder art 0x02000751 spawns beneath the opening [M 0x1dbda8 state 1 replay; D 0x06000A3B s0 t8] — got {post:?}");
    let d = ((ladder[0].2[0] - 51.58).powi(2) + (ladder[0].2[2] - 44.94).powi(2)).sqrt();
    assert!(d < 6.0, "the ladder art spawns at the lid's base, got {:?}", ladder[0].2);
}

/// FAIL-FIRST repro, #[ignore]d until the fix lands (a red test must not ride the branch): the
/// use press at the real fusebox #385 runs its script and the wire sub-objects (0x02000A62 /
/// 0x02000A61 / 0x02000A73, type 0x31 [D 0x06000A1B s0/s1/s3 Entity cmds]) spawn behind the
/// front panel. The port's fusebox.rs stands in for them (docs/levels/07000014.md §5 "the wire
/// sub-objects are the documented stand-in — the SP-entity bullet path is not ported"), so the
/// reported "wall panel with nothing behind it" shows the recess empty. The fix: run the
/// fusebox script's Entity streams through the same SP-entity path the lid replay uses.
#[test]
#[ignore = "fail-first repro: the fusebox script's wire entities are a documented stand-in (docs/levels/07000014.md §5)"]
fn the_fusebox_use_spawns_the_wire_sub_objects_behind_the_panel() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
        let pre = entities_near(sim, &[0x0200_0a61, 0x0200_0a62, 0x0200_0a73]);
        eprintln!("[fusebox] at load: {pre:?}");
        // Bond at the fusebox wall, facing it (the fusebox at (−89.30, −2.22, 49.36)); the yaw is
        // atan2(dx, dz) [D the walker's convention]: dx −0.90, dz +0.76 → −0.87.
        sim.bond.pos = [-88.40, -2.20, 48.60];
        sim.bond.yaw = -0.87;
        for k in 0..6 { sim.step_bond(Input::default(), false, false, k == 0); }
    }
    for t in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    let post = entities_near(sim, &[0x0200_0a61, 0x0200_0a62, 0x0200_0a73]);
    eprintln!("[fusebox] after the use + 120 ticks: {post:?}");
    assert!(!post.is_empty(), "the use at the real fusebox #385 runs script 0x06000A1B and the wire sub-objects spawn [D 0x06000A1B s0/s1/s3 Entity cmds] — got {post:?}");
}
