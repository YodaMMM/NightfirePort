//! level-05-cable-full-length — the 07000005 castle zip cable ("Wire_Cable" #251, placement
//! type 0x3E, class 0x2C) reachable span, pinned both ways through the real input path.
//!
//! The game's law (the decompile):
//! - The travel segment is the wire's world sphere centre ± `Spherical(radius − 0.5, yaw + π/2, 0)`
//!   [M Player_Wire 0x197658: 0x1976dc-0x197730 — `Vec_Spherical_2_Cartesian(obj+0x8C − 0.5,
//!   obj+0x54 + 1.5707964, 0)`]; there are no endpoint params on the wire [M Wire_Create 0x1981d8
//!   — the 8-byte instance carries only the type/params/channel].
//! - The stop rule at the ends [M Player_Wire 0x1977c0-0x197834]: the strafe axis (action 1) is
//!   ZEROED when the squared 2D distance to the travelled-to end is ≤ 0.5 (≈0.707 u), or when the
//!   pre-snap squared distance to the opposite end exceeded the segment's own squared length
//!   (Bond came in from beyond that end). The per-tick snap [M 0x1976f0-0x197708] teleports him to
//!   an end when he is beyond it, so the ride can never leave [B, A].
//! - The grab teleport [M Player_CollWire 0x197130: the closest point of the GRAB segment
//!   (radius − 2.0)] puts Bond at the nearest grab-segment point of where he pressed jump.
//!
//! Disc/snapshot anchors: wire pos (−23.4855, 34.4676, 6.26292), yaw −1.5708, live sphere radius
//! 4.039 [L ee_start_05.bin 016dc430/016dc450/016dc4bc], instance bytes `00 00 02 00 00 00 00 00`
//! [L 016dc510]. The visible cable run is the Object_Cable_* CEL pieces plus OBJECT_CableEnds
//! [D placements]; the ride object is only #251 [D probe — one 0x3E placement on the level].
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test cable_span_live -- --nocapture`

use nf_game::player::{Input, SS_WALK, SS_WIRE};
use nf_formats::level::Level;
use nf_formats::iso9660::IsoImage;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

/// [D] the cable run's own placement extent: every world instance whose map object is named
/// Cable* (the Object_Cable_* CEL pieces, OBJECT_CableEnds), with the placement translation.
/// The ride object #251 "Wire_Cable" sits between them; this pins how far the visible cable
/// runs so the ride span is read against the real cable, not an invisible line.
#[test]
fn probe_the_cable_run_placements() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let data = read_archive(&iso, "07000005.bin");
    let level = Level::parse(&data).expect("parse");
    let mut n = 0;
    for (mi, m) in level.maps.iter().enumerate() {
        for p in &m.placements {
            let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let obj = obj.and_then(|(om, oi)| level.maps.get(om).and_then(|map| map.objects.get(oi)));
            let Some(o) = obj else { continue };
            if !o.name.to_ascii_uppercase().contains("CABLE") { continue; }
            n += 1;
            eprintln!("cable piece: map {mi} {:?} at ({:.3},{:.3},{:.3}) bounds r {:.3} centre {:?} min {:?} max {:?}",
                o.name, p.translation[0], p.translation[1], p.translation[2], o.radius, o.centre, o.min, o.max);
        }
    }
    eprintln!("cable-named placements: {n}");
    assert!(n > 0, "the level carries the Object_Cable_* pieces [D placements]");
}

/// The parsed wire's geometry against the snapshot values [L ee_start_05.bin 016dc430..016dc510]:
/// centre, yaw, radius, type/params. The radius is the ride extent's driver (radius − 0.5 [M]).
#[test]
fn the_parsed_wire_matches_the_snapshot_geometry() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let w = sim.wires.first().expect("07000005 has one wire (0x3E)").clone();
    eprintln!("parsed wire: centre {:?} origin {:?} radius {:.4} yaw {:.4} pitch {:.4} type {} params 0={},2={} ch {}",
        w.centre, w.origin, w.radius, w.yaw, w.pitch, w.wire_type, w.param0, w.param2, w.channel);
    assert_eq!((w.wire_type, w.param0, w.channel), (0, 2, 0), "the live instance bytes [L ee_start_05 016dc510]");
    assert!((w.centre[0] - (-23.4855)).abs() < 0.01 && (w.centre[1] - 34.4676).abs() < 0.01
        && (w.centre[2] - 6.26292).abs() < 0.01, "the wire centre [L 016dc430/016dc450]");
    assert!((w.yaw - (-1.5708)).abs() < 0.001, "the yaw [L 016dc450]");
    // The live sphere radius [L 016dc4bc = obj+0x8C = 4.039]. The port derives it from the disc
    // object's bounds radius (+0x14, the AABB half-diagonal — nf-formats::level::MapObject);
    // Control_BuildWorldSph 0x133070 rebuilds the same sphere through the yaw-only matrix.
    assert!((w.radius - 4.039).abs() < 0.02,
        "the disc-derived radius must match the live sphere radius [L 016dc4bc]: got {:.4}", w.radius);
    // The travel segment the game rides [M Player_Wire 0x1976dc-0x197730]: radius − 0.5 along Z.
    let (a, b) = w.travel_segment();
    eprintln!("travel segment: A z {:.3} .. B z {:.3} (span {:.3})", a[2], b[2], a[2] - b[2]);
    assert!((a[2] - (6.26292 + 4.039 - 0.5)).abs() < 0.02 && (b[2] - (6.26292 - 4.039 + 0.5)).abs() < 0.02,
        "the travel ends are centre.z ± (radius − 0.5) [M]");
}

/// The reachable span, BOTH ways, through the real input path (the icon-context jump grab
/// [M Player_HandleJump 0x1916b0:118-147]): grab near the B end, ride to A; without dropping,
/// turn the strafe the other way and ride back to B. The span pinned is the game's travel
/// segment: Bond reaches within the stop band (0.5² ≈ 0.707 + one hand cycle ≤ 0.8) of each end
/// [M 0x1977c0-0x197834] and never passes beyond an end [M the per-tick snap 0x1976f0-0x197708].
#[test]
fn the_cable_span_reaches_the_game_ends_both_ways() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let w = sim.wires.first().expect("wire").clone();
    let (a_t, b_t) = w.travel_segment();
    // Bond at the south icon spot (−23.58, 34.0, 2.65 [D probe]) — the jump grabs the nearest
    // wire and teleports him to the closest grab-segment point [M CollWire 0x197130].
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [-23.58, 34.0, 2.65], 0.0);
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.bond.icon_ctx, 3, "the wire icons arm context 3 [D the type-3 icons; M ThirdIcon_Update]");
    sim.step_bond(Input { jump: true, ..Default::default() }, false, false, false);
    assert_eq!(sim.bond.substate, SS_WIRE, "the jump grab attaches [M 0x1916b0]");
    for _ in 0..140 { sim.step_bond(Input::default(), false, false, false); } // the grab pose out
    let z_grab = sim.bond.pos[2];
    eprintln!("grab at z {:.3} (grab segment z {:.3}..{:.3})", z_grab, w.grab_segment().1[2], w.grab_segment().0[2]);

    // Ride B → A (strafe −1.0 [M 0x1977c0: negative = toward A]). Ride to the STALL (the stop
    // rule's idle settle): the game's stall point is the last hand cycle's end after the input
    // zeroed [M 0x1977c0-0x197834] — inside (end − 0.707 − sway, end] because the kill fires at
    // squared-2D 0.5 and the in-flight cycle still completes.
    let cross_a = Input { strafe: -1.0, ..Default::default() };
    let mut last_z = sim.bond.pos[2];
    let mut stall = 0u32;
    for t in 0..1500u32 {
        sim.step_bond(cross_a, false, false, false);
        assert_eq!(sim.bond.substate, SS_WIRE, "attached the whole ride (tick {t}: {:?})", sim.bond.pos);
        let z = sim.bond.pos[2];
        if (z - last_z).abs() < 0.005 { stall += 1; } else { stall = 0; }
        last_z = z;
        if stall > 60 { break; }
    }
    let za = last_z;
    eprintln!("stalled at A: z {:.3} (segment end {:.3}, shortfall {:.3})", za, a_t[2], a_t[2] - za);
    assert!(za > a_t[2] - 0.85 && za <= a_t[2] + 0.05,
        "the A stall must sit in the game's stop band (end − 0.707 − sway … end] [M 0x1977c0-0x197834]: got {za:.3}");

    // Hold the SAME direction: the clamp must hold him at the A end (the stop rule, not a stall).
    for _ in 0..200u32 { sim.step_bond(cross_a, false, false, false); }
    assert!(sim.bond.pos[2] >= a_t[2] - 0.9, "holding into the A end stays at the end [M 0x1977c0-0x197834]: {:?}",
        sim.bond.pos);

    // Ride back A → B without dropping (strafe +1.0): the mode byte turns Bond first (the turn
    // pair 5ee/5ea [M 0x1979d8-0x197a30]) and the move pair carries him to the B stall.
    let cross_b = Input { strafe: 1.0, ..Default::default() };
    let mut last_z = sim.bond.pos[2];
    let mut stall = 0u32;
    for t in 0..1800u32 {
        sim.step_bond(cross_b, false, false, false);
        assert_eq!(sim.bond.substate, SS_WIRE, "attached the ride back (tick {t}: {:?})", sim.bond.pos);
        let z = sim.bond.pos[2];
        if (z - last_z).abs() < 0.005 { stall += 1; } else { stall = 0; }
        last_z = z;
        if stall > 60 { break; }
    }
    let zb = last_z;
    eprintln!("stalled at B: z {:.3} (segment end {:.3}, shortfall {:.3})", zb, b_t[2], zb - b_t[2]);
    assert!(zb < b_t[2] + 0.85 && zb >= b_t[2] - 0.05,
        "the B stall must sit in the game's stop band [M 0x1977c0-0x197834]: got {zb:.3}");
    for _ in 0..200u32 { sim.step_bond(cross_b, false, false, false); }
    assert!(sim.bond.pos[2] <= b_t[2] + 0.9, "holding into the B end stays at the end: {:?}", sim.bond.pos);

    // The span covered in one ride both ways (the grab sat at the B side, so the round trip
    // covers the segment twice).
    let span = za - z_grab;
    eprintln!("span covered B->A: {:.3} (grab {:.3} to the A stall; the full segment {:.3})",
        span, z_grab, a_t[2] - b_t[2]);
    assert!(span > (a_t[2] - z_grab) * 0.9, "the ride covers the grab-to-A leg to its stop band");

    // Dismount at the B end (the action press [M 0x197758]) and live.
    sim.step_bond(Input { jump: true, ..Default::default() }, false, false, false);
    assert_eq!(sim.bond.substate, SS_WALK, "the press dismounts [M Player_Wire 0x197758]");
    assert!(sim.drones.health.health > 0.0, "alive after the dismount");
}
