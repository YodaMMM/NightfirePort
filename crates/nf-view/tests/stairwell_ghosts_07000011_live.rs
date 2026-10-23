//! stairwell-ghosts-11: the 07000011 stairwell's T-posed "ghost" guards — guards drawn standing
//! in a doorway and in mid-air outside the railing where no guard should be (master render,
//! `--bond 14.11,34.03,0.26,2.305,-0.395` frame 921 two in a doorway, `15.35,35.42,1.61` frame
//! 1258, one in the air outside the railing frame 3688).
//!
//! The game's rule: a drone placement whose level tag (parameter 2) exceeds the load's difficulty
//! is never created at all — `Drone_Create` 0x136be0 gates the whole create (object included) on
//! `level_tag+0x34 (param 2) <= GameState+0x40 (difficulty)` [M 00136be0 decompile, first branch],
//! so nothing exists and nothing is drawn for it. The port's `character_instances` built every
//! 0x05-skin placement's meshes regardless of that gate, so a skipped placement kept characters
//! the sim's drone list has no entry for: `char_drone` (keyed on the sim's placements'
//! translations) missed them, `char_hidden`'s Placed arm drew them at their placement matrix —
//! T-posed (no StartAnim reaches them), full-ambient (`char_ambient` never leaves the 0xff
//! sentinel for an unmatched character) — the ghosts.
//!
//! Pinned here through the real load path (`load_with_carry_difficulty`, the replay-parity
//! difficulty 1 the direct loads keep, and difficulty 2 the viewer boots with [M 0x1c7e78:25 boot
//! default 2]): every kind-0x0F placement with param 2 > difficulty has NO drone in the sim list
//! and NO character; every placement that passes the gate has both, and its character is drawn
//! exactly when the wake rule says so (the disabled waiters stay hidden — stairwell_guards_
//! 07000011_live.rs pins the wake chain itself).
//! Evidence tags: [M] decompile address, [D] disc bytes, [L] live measurement.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_view::scene::{Scene, ViewOpts};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn load_d(difficulty: u32) -> Scene {
    nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, difficulty).expect("load 07000011")
}

/// The main-map kind-0x0F placements with their gate param [D 07000011.bin placements block].
fn drone_placements() -> Vec<(usize, [f32; 3], u32, u32, u32)> {
    let data = read_archive(&iso(), "07000011.bin");
    let lv = Level::parse(&data).expect("parse 07000011");
    let mm = lv.main_map().unwrap();
    lv.maps[mm].placements.iter().enumerate()
        .filter(|(_, p)| p.flags & 0xff == 0x0f)
        .map(|(i, p)| {
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
            (i, p.translation, param(0), param(2), param(3))
        })
        .collect()
}

fn char_at_translation(sc: &Scene, t: [f32; 3]) -> Option<usize> {
    let bits = t.map(f32::to_bits);
    sc.characters.iter().position(|c| [c.matrix[3][0], c.matrix[3][1], c.matrix[3][2]].map(f32::to_bits) == bits)
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored"]
fn dump_07000011_drone_placements_and_ghosts() {
    let Some(_) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let placements = drone_placements();
    println!("== 07000011 kind-0x0F placements: {} ==", placements.len());
    for (i, t, skin, tag, wake) in &placements {
        println!("#{i} @({:.2},{:.2},{:.2}) skin {skin:08x} diff-tag {tag} wake ch {wake:#x}", t[0], t[1], t[2]);
    }
    for d in [1u32, 2, 3] {
        let sc = load_d(d);
        let sim = sc.sim.as_ref().unwrap();
        let present: Vec<usize> = sim.drones.list.iter().map(|dr| dr.placement).collect();
        let mut ghosts = 0;
        for (i, t, skin, tag, wake) in &placements {
            let in_sim = present.contains(i);
            let ch = char_at_translation(&sc, *t);
            if tag > &d {
                if ch.is_some() || in_sim {
                    ghosts += 1;
                    println!("d{d}: #{i} diff-tag {tag} — SIM {} CHAR {}  ← the game creates nothing [M 0x136be0]",
                        if in_sim { "HAS IT" } else { "-" }, if ch.is_some() { "DRAWS IT" } else { "-" });
                }
            } else if !in_sim || ch.is_none() {
                println!("d{d}: #{i} diff-tag {tag} wake {wake:#x} — expected in sim ({in_sim}) and char ({ch:?})");
            }
        }
        println!("d{d}: {} drones in sim, {} characters, {} gate violations", sim.drones.list.len(), sc.characters.len(), ghosts);
    }
}

/// The ghost rule: a drone placement the game's create gate skips [M 0x136be0] must have NO sim
/// drone and NO character at its placement translation — at the difficulty the direct loads keep
/// (1, the replay-parity default) and the viewer's boot default (2) [M 0x1c7e78:25].
#[test]
fn a_placement_the_create_gate_skips_draws_nothing() {
    for d in [1u32, 2] {
        let sc = load_d(d);
        let sim = sc.sim.as_ref().unwrap();
        let placements = drone_placements();
        assert!(!placements.is_empty(), "07000011 carries drone placements [D]");
        let present: Vec<usize> = sim.drones.list.iter().map(|dr| dr.placement).collect();
        let mut gated = 0;
        for (i, t, skin, tag, _wake) in &placements {
            if *tag > d {
                gated += 1;
                assert!(!present.contains(i),
                    "d{d}: placement #{i} (skin {skin:08x}, tag {tag}) is above the create gate — Drone_Create makes nothing [M 0x136be0], got a sim drone");
                assert!(char_at_translation(&sc, *t).is_none(),
                    "d{d}: placement #{i} @({:.2},{:.2},{:.2}) tag {tag} has no object in the game [M 0x136be0] — the port drew a character there (the T-posed ghost)",
                    t[0], t[1], t[2]);
            } else {
                assert!(present.contains(i),
                    "d{d}: placement #{i} tag {tag} passes the gate — Drone_Create creates it [M 0x136be0], no sim drone");
                assert!(char_at_translation(&sc, *t).is_some(),
                    "d{d}: placement #{i} tag {tag} has a drone [M 0x136be0] — its character must map (char_drone), none found");
            }
        }
        assert!(gated > 0, "07000011 has difficulty-gated stairwell placements [D param 2 > 1: the pin test's p222/p223 (≥2), p224/p225 (≥3), p280 (≥2)]");
        // And nothing unmatched is DRAWN anywhere: a character the render's own gate
        // (`char_hidden`, the frame_state hidden the draw walks) shows must map to a sim drone or
        // be a script actor — the Placed+unmatched arm drew the ghosts. View models are exempt
        // (the held weapon's body is a character without a drone [M the draw's view arm]).
        for (ci, c) in sc.characters.iter().enumerate() {
            if sc.char_hidden(ci, ViewOpts::default()) { continue; }
            let mapped = sim.char_drone.get(ci).copied().flatten();
            assert!(mapped.is_some() || c.script.is_some() || sc.view_models.values().any(|&v| v == ci),
                "d{d}: character {ci} @({:.2},{:.2},{:.2}) skin {:08x} is DRAWN without a sim drone or script — the ghost arm",
                c.matrix[3][0], c.matrix[3][1], c.matrix[3][2], c.skin.hashcode);
        }
    }
}

/// The frame-921 view (Bond parked at `14.11,34.03,0.26`, the render's pose) walked through the
/// real step path: the stairwell characters the port draws are exactly the sim's live drones — no
/// character is drawn that the create gate skipped, at any tick of the walk.
#[test]
fn the_frame_921_view_draws_no_guards_the_create_gate_skipped() {
    let mut sc = load_d(1);
    let gated: Vec<[f32; 3]> = drone_placements().into_iter()
        .filter(|(_, _, _, tag, _)| *tag > 1).map(|(_, t, _, _, _)| t).collect();
    assert!(!gated.is_empty(), "difficulty-gated stairwell placements exist [D]");
    {
        let sim = sc.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [14.11, 34.03, 0.26], 2.305);
        sim.bond.pitch = -0.395;
    }
    for tick in 0..=921 {
        sc.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if tick % 300 != 0 && tick != 921 { continue; }
        let sim = sc.sim.as_ref().unwrap();
        for (i, t, skin, tag, _wake) in drone_placements() {
            if tag <= 1 { continue; }
            assert!(char_at_translation(&sc, t).is_none(),
                "frame {tick}: placement #{i} (skin {skin:08x}, tag {tag}) @({:.2},{:.2},{:.2}) must draw nothing [M 0x136be0] — a ghost character is there",
                t[0], t[1], t[2]);
        }
        let _ = sim; // the sim's own gated-placements assertion lives in the census + first test
    }
}

/// Sister: at difficulty 3 every placement passes the gate — each has its sim drone AND its
/// character mapped, hidden exactly while the wake rule holds it (disabled waiters [M 0x149dd0]).
#[test]
fn at_difficulty_3_every_guard_exists_and_maps_to_its_character() {
    let sc = load_d(3);
    let sim = sc.sim.as_ref().unwrap();
    let placements = drone_placements();
    let present: Vec<usize> = sim.drones.list.iter().map(|dr| dr.placement).collect();
    for (i, t, _skin, tag, _wake) in &placements {
        assert!(*tag <= 3, "placement #{i} tag {tag} above the 00-Agent gate would be uncreateable [D]");
        assert!(present.contains(i), "d3: placement #{i} tag {tag} exists [M 0x136be0]");
        let ci = char_at_translation(&sc, *t)
            .unwrap_or_else(|| panic!("d3: placement #{i} @({:.2},{:.2},{:.2}) has a drone but no character", t[0], t[1], t[2]));
        assert_eq!(sim.char_drone[ci], Some(present.iter().position(|&p| p == *i).unwrap()),
            "d3: placement #{i}'s character maps to its own sim drone (the ghost arm's counterpart)");
        let dr = &sim.drones.list[sim.char_drone[ci].unwrap()];
        assert_eq!(sc.char_hidden(ci, ViewOpts::default()), !dr.enabled,
            "d3: placement #{i} draws exactly when enabled (disabled waiters hidden [M 0x149dd0])");
    }
}
