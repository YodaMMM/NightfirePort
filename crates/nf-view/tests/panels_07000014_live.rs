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

/// The wire sub-objects behind the fusebox panel. CORRECTED against the decompile (the original
/// fail-first repro asserted a transcription slip and the wrong spawning agent — see the report):
///
/// - The spawn agent is the box's mode-0x40 PRELOAD, not the use: `FuseBox_Create` 0x1b0448's tail
///   `SP_Create(param 0, mode 0x40)` runs the script ONCE at level load ([M] SP_LoadScript
///   0x1dbc08:46-54 — any mode outside {1,8} runs Play+Update+Stop, state 1), and the key-1
///   `Entity` commands of streams 0/1/3/5 spawn the wires 0x02000AA2 / 0x02000AA1 / 0x02000AB3 /
///   0x02000AA4 (obj_type 0x31) [D 0x06000A1B decode — the raw disc bytes e.g. stream 0
///   `07 a2 0a 00 02` big-endian-fold to 0x02000AA2; the "0x02000A62/61/73" in the hand-off notes
///   drop the 0x40 bit]. `Script_CreateEntity` 0x1d8968 creates by type [M].
/// - The use press is INERT at the box: `Player_Activate` 0x1a5170:95-160 has no class-0x3C arm,
///   and the box's SP carries mask 1 (SP_Create's param_6 = 0 [M 0x1dc7e0]; the port primes it
///   with mask 1), so `SP_Activate` 0x1dc338's bit-4 gate refuses — the E press neither spawns nor
///   removes the wires (the sister pin on 07000008: fix_fusebox_cut_live::e_presses_do_nothing_to_the_fusebox).
///   The wires take the DAMAGE path instead — `FuseBox_Update` 0x1adfe8:88-96 cuts a registered
///   wire whose hit list survives `Collide_FilterBullets(list, 0xfffd)` (the Laser's 0x0002 cuts,
///   gunfire's 0x0008 is dropped [L weapon_data]); the full sabotage chain on THIS level is pinned
///   green by fix_fusebox_cut_live::the_07000014_sabotage_cuts_through_the_real_laser.
/// - The registrations follow the decode: stream 2's event 0xF (group 0) hands over stream 1's
///   entity and stream 6's (group 1) stream 5's ([M] Script_EventHandler 0x1d94c0:172-175 →
///   FuseBox_Register 0x1b0970), i.e. group 0 holds 0x02000AA1 and group 1 holds 0x02000AA4.
/// - "Behind the panel": the wall panel is the small door object #386 at (−89.58, −0.68, 49.17)
///   [D placement]; the wires sit at the box #385 (−89.30, −2.22, 49.36) [D] — east of the panel,
///   the room side. All four wire glists resolve to models (the meshes exist and the entities join
///   the render set); the visible open-panel check is the named headless render in the report.
#[test]
fn the_fusebox_use_spawns_the_wire_sub_objects_behind_the_panel() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    const WIRES: [u32; 4] = [0x0200_0aa2, 0x0200_0aa1, 0x0200_0ab3, 0x0200_0aa4];
    let box385: [f32; 3] = [-89.30, -2.22, 49.36]; // [D placement #385]
    let panel386_x: f32 = -89.58;                  // [D placement #386, the wall panel]
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
        // At load: all four wires exist AT the box, behind the panel, with drawable meshes —
        // the preload's one-tick play reached the key-1 Entity commands [M 0x1dbc08:46-54].
        let near385 = |p: [f32; 3]| ((p[0] - box385[0]).powi(2) + (p[1] - box385[1]).powi(2) + (p[2] - box385[2]).powi(2)).sqrt() < 3.0;
        let pre = entities_near(sim, &WIRES);
        eprintln!("[fusebox] at load: {pre:?}");
        for h in WIRES {
            let e = pre.iter().find(|(hh, _, p)| *hh == h && near385(*p))
                .unwrap_or_else(|| panic!("wire {h:08x} exists at load at the fusebox #385 (the mode-0x40 preload runs the script once [M 0x1dbc08:46-54; D 0x06000A1B s0/s1/s3/s5 key-1 Entity cmds]) — got {pre:?}"));
            assert!(e.2[0] > panel386_x, "wire {h:08x} is behind the panel #386 (east of x {panel386_x}, the room side), got {:?}", e.2);
        }
        assert_eq!(pre.iter().filter(|(_, _, p)| near385(*p)).count(), 4, "all four decode wires sit at box #385 [D decode]");
        // The decode's registrations: group 0 = stream 1's wire, group 1 = stream 5's.
        use nf_game::nis::World;
        let data = sim.data();
        let fb = sim.drones.fuseboxes.iter().find(|f| f.placement == 385).expect("fusebox #385 exists [D]");
        let group_hash = |g: usize| fb.groups[g].slots.iter().find_map(|s| s.and_then(|obj| match sim.drones.scripts.objects.list.get(obj) {
            Some(Some(nf_game::nis::Obj::Entity { hash, .. })) => Some(*hash), _ => None }));
        assert_eq!(group_hash(0), Some(0x0200_0aa1), "group 0 holds stream 1's wire [M 0x1d94c0:172-175 → 0x1b0970; D decode]");
        assert_eq!(group_hash(1), Some(0x0200_0aa4), "group 1 holds stream 5's wire [M 0x1d94c0:172-175 → 0x1b0970; D decode]");
        for h in WIRES {
            assert!(data.glist_sphere(h).is_some(), "the wire {h:08x} glist resolves to a model (the mesh draws)");
        }
        // The real-input drive: Bond at the panel, the use press through step_bond (the SP's mask
        // 1 has no bit 4 — SP_Activate refuses [M 0x1dc338]; the press pins the inertness here).
        sim.bond.pos = [-88.40, -2.20, 48.60];
        sim.bond.yaw = -0.87; // face the box: atan2(dx, dz) [D the walker's convention]
        for k in 0..6 { sim.step_bond(Input::default(), false, false, k == 0); }
    }
    for t in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    let post = entities_near(sim, &WIRES);
    eprintln!("[fusebox] after the use + 120 ticks: {post:?}");
    let near385 = |p: [f32; 3]| ((p[0] - box385[0]).powi(2) + (p[1] - box385[1]).powi(2) + (p[2] - box385[2]).powi(2)).sqrt() < 3.0;
    assert_eq!(post.iter().filter(|(_, _, p)| near385(*p)).count(), 4, "the use press neither spawns nor removes the wires ([M 0x1a5170:95-160 no 0x3C arm; SP mask 1 has no bit 4]) — got {post:?}");
    let fb = sim.drones.fuseboxes.iter().find(|f| f.placement == 385).expect("fusebox #385");
    assert_eq!(fb.alive_wires().count(), 2, "both groups still hold their registered wire [M FuseBox_Register 0x1b0970]");
}
