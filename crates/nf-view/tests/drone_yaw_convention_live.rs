//! drone-yaw-convention — the placed-drone facing law, pinned through the real load path.
//!
//! The game's chain [M, ACTION.ELF]: `parsemap_create_dynamic_objects` case 0xf calls
//! `Drone_Create(translation*, rotation*, level_tag*)` with the placement row's own vectors
//! (0x1d0c30-0x1d0c40; the vectors are the dispatcher's stack copies of placement +0x0C/+0x18
//! [M 0x1d04f0:50, docs/fields/level-tag.md's +0x0C..+0x2B row]). `Drone_Create` 0x136be0 stores
//! the rotation vector into the DIVars' rot slot `Drone+0xC20` (the 128-bit store at 0x136cb0) —
//! the params block lands beside it at +0xC34 (amemcpy 0x136cac) — and `NDrone2_PostLoad_Init`
//! 0x14d890 hands `Drone+0xC00` to `NDrone2_DefaultInit` 0x14b300, whose `Vec_Copy2`
//! (0x1e2cf8 — three lwc1/swc1 pairs, a verbatim float copy) writes `DIVars+0x20` to BOTH
//! `obj+0x50` and `obj+0x60` (0x14b350-0x14b35c). The stored rotation vector's second component
//! is `obj+0x54` — the drone's facing, which the game's own yaw math reads [M 0x150ad0:27-35,
//! the +0x570 quad offsets around it]. So: **facing = placement rotation[1] (+0x1C f32) raw —
//! no negation, no component permutation anywhere in the chain**.
//!
//! The port spawns every drone with `p.rotation[1]` raw [M drone.rs, the placements pass's
//! `Drone::new_with_stats(.., p.rotation[1], ..)`] — the SAME law. This test pins it on
//! 07000012's real placements, whose corridor guards face AWAY from Bond's start (the guards at
//! (13.09, 1.00, 18.55) / (12.98, 1.00, 16.43) face west, yaw −1.89/−1.38, while Bond waits east
//! at (22.49, 19.16) — yaw_to_opp +1.50/+1.29): the disc authors faced them down the corridor,
//! and the game's wake comes through the alarm/hearing path, not the spawn sight cone.

use nf_formats::iso9660::IsoImage;

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn wrap(x: f32) -> f32 {
    let mut a = x;
    while a > std::f32::consts::PI { a -= 2.0 * std::f32::consts::PI; }
    while a < -std::f32::consts::PI { a += 2.0 * std::f32::consts::PI; }
    a
}

#[test]
fn placed_drones_face_their_rotation_y_component_raw() {
    let iso_path = iso();
    // The disc's own placement rows: the same bytes parsemap_create_dynamic_objects reads.
    let img = IsoImage::open(&iso_path).expect("iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000012.bin")).expect("07000012.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("archive bytes");
    let level = nf_formats::level::Level::parse(&data).expect("level parse");
    let rows: Vec<(f32, f32, f32, f32)> = level.maps.iter()
        .flat_map(|m| m.placements.iter())
        .filter(|p| p.flags & 0xff == 0x0f)
        .map(|p| (p.translation[0], p.translation[2], p.rotation[1], p.rotation[0]))
        .collect();
    assert!(rows.len() >= 2, "07000012 must carry its 0x0f drone placements, got {}", rows.len());

    // The sim: the port's real spawn pass over the same rows.
    let mut w = nf_view::load(&iso_path, "07000012.bin", false)
        .map(Walker::new).expect("load 07000012");
    assert!(w.wait_enabled(6000, "level start"), "the level never enabled");
    w.wait(120); // the intro scripts settle (the two seconds the corridor probe gives them)

    let sim = w.scene.sim.as_ref().unwrap();
    let mut checked = 0usize;
    let mut yawed = 0usize;
    for (px, pz, rot_y, _rot_x) in &rows {
        // The spawned drone for this row: position match (the spawn pass uses the translation,
        // floored — the port spawns at floor + 1.0 with the same x/z).
        let Some(dr) = sim.drones.list.iter().find(|d| {
            d.present && (d.pos[0] - px).abs() < 0.3 && (d.pos[2] - pz).abs() < 0.3 && (d.pos[1] - 1.0).abs() < 1.5
        }) else { continue }; // a difficulty-gated row spawns no drone — skip
        checked += 1;
        // THE LAW: the spawn yaw is the placement's rotation[1] VERBATIM — the Vec_Copy2
        // float copy 0x1e2cf8 lands it in obj+0x50..0x58 and the AI reads obj+0x54.
        assert!(
            wrap(dr.yaw - *rot_y).abs() < 1e-4,
            "drone at ({px:.2},{pz:.2}) must spawn facing rotation[1] raw [M 0x1e2cf8 → obj+0x50, \
             facing obj+0x54; 0x136cb0 rot slot; 0x1d0c30 case 0xf vectors]: yaw {} vs rotation[1] {}",
            dr.yaw, rot_y
        );
        // THE SISTER (fails on the opposite sign): wherever the disc authored a facing that is
        // not self-inverse under negation (not 0, not ±π — e.g. the corridor guards' −1.89 and
        // −1.38), the NEGATED law must NOT hold. A sign-flipped port would land −rot_y and trip.
        if wrap(*rot_y).abs() > 0.5 && (wrap(*rot_y) - std::f32::consts::PI).abs() > 0.5 {
            yawed += 1;
            assert!(
                wrap(dr.yaw + *rot_y).abs() > 1e-4,
                "drone at ({px:.2},{pz:.2}) spawned at −rotation[1] ({}) — the sign-flipped law the \
                 game does not use [M the verbatim Vec_Copy2 0x1e2cf8]: the disc authored {}",
                dr.yaw, rot_y
            );
        }
    }
    assert!(checked >= 2, "no sim drone matched the parsed 0x0f rows (checked {checked})");
    assert!(yawed >= 1, "no placement had a sign-discriminating rotation[1] — the sister is vacuous");
}
