//! level-14-invisible-guard: every drone on 07000014 that is present/enabled at any point of the
//! level must have a DRAWN character — a resolved skin, a released (non-held-back) mesh and a
//! posed (non-collapsed, non-FAR) palette — through the whole mission chain.
//!
//! The lanes the census covers (docs/reports/level-14-invisible-guard.md):
//! - no skin resolved: the raw 0x050000b1 placements never reach the skin table (the skin rides
//!   only 07000015/16 [D carriers scan]) — but `NDrone2_DefaultInit` 0x14b300:82-95 rewrites the
//!   param-0 to the own-archive 0x0500008c in Drone+0xe0 BEFORE the `AnimObjectNew` request
//!   (:563-565), and the captain swap (:536-555: kind 6 → 0x05000093, kind 7 → 0x050000b2, both
//!   own-archive [D]) swaps on top. `0x05000090` substitutes to `0x05000093` on 0x07000014
//!   [M AnimObjectNew 0x1be910:33-43].
//! - a held-back spawner mesh never released: a template drone's character draws only once the
//!   spawner brings it in (`char_hidden`'s drone-link arm — present && enabled).
//! - a palette slot left FAR: a drone whose pose collapses (`object_pose` None) draws nothing
//!   while its AI still acts — the palette pass collapses the bones to the placement point.
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test invisible_guard_07000014_live -- --test-threads=1`

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn tick_until(scene: &mut nf_view::scene::Scene, ticks: u32, done: impl Fn(&nf_view::scene::Sim) -> bool) {
    for _ in 0..ticks {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if done(scene.sim.as_ref().unwrap()) { break; }
    }
}

/// Every present/enabled/alive drone must have a bound character that is neither hidden nor
/// pose-collapsed, with a resolved skin. Returns the offenders (the census rows).
fn offenders(scene: &nf_view::scene::Scene) -> Vec<String> {
    let s = scene.sim.as_ref().unwrap();
    let opts = nf_view::scene::ViewOpts::default();
    let mut out = Vec::new();
    for (di, dr) in s.drones.list.iter().enumerate() {
        if !(dr.present && dr.enabled && !dr.corpse && dr.health > 0.0) { continue; }
        let chars: Vec<usize> = (0..scene.characters.len())
            .filter(|&ci| s.char_drone.get(ci).copied().flatten() == Some(di)).collect();
        let pos = format!("({:.2},{:.2},{:.2})", dr.pos[0], dr.pos[1], dr.pos[2]);
        if chars.is_empty() {
            out.push(format!("drone#{di} plc={} skin=0x{:08x} NO CHARACTER pos={pos}", dr.placement, dr.skin));
            continue;
        }
        let skin_ok = s.skins.get(&dr.skin).map(|sk| s.skeletons.get(&sk.skeleton).map(|x| x.bones > 0).unwrap_or(false)).unwrap_or(false);
        for &ci in &chars {
            if scene.char_hidden(ci, opts) {
                out.push(format!("drone#{di} plc={} skin=0x{:08x} char#{ci} HELD BACK (hidden) pos={pos}", dr.placement, dr.skin));
            } else if dr.object_pose(&s.data()).is_none() {
                out.push(format!("drone#{di} plc={} skin=0x{:08x} char#{ci} PALETTE COLLAPSED (object_pose None) pos={pos}", dr.placement, dr.skin));
            } else if !skin_ok || scene.characters[ci].skin.hashcode != dr.skin {
                out.push(format!("drone#{di} plc={} skin=0x{:08x} char#{ci} skin=0x{:08x} UNRESOLVED SKIN pos={pos}",
                    dr.placement, dr.skin, scene.characters[ci].skin.hashcode));
            }
        }
    }
    out
}

/// The full mission chain (the walk_07000014_live route, channels standing in for the wire
/// burns the way `FuseBox_Update` 0x1adfe8 writes them): at every stage the census must be empty.
#[test]
fn every_present_drone_draws_through_the_whole_mission() {
    let Some(iso) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(iso.to_str().unwrap(), "07000014.bin", false).expect("load 07000014");
    let mut stage = |scene: &mut nf_view::scene::Scene, tag: &str| {
        let off = offenders(scene);
        assert!(off.is_empty(), "{tag}: drones present with no drawn character:\n  {}", off.join("\n  "));
    };
    stage(&mut scene, "t=0");

    // Stage 1: the vent drop — TouchOnce #301 → the 0x06000833 cut-scene writes ch 0x47 [D].
    scene.sim.as_mut().unwrap().bond.pos = [40.7, 2.6, 39.0];
    tick_until(&mut scene, 1700, |s| s.drones.channels.on(0x47));
    assert!(scene.sim.as_ref().unwrap().drones.channels.on(0x47), "the vent cut-scene writes ch 0x47 [D]");
    { let s = scene.sim.as_mut().unwrap(); for _ in 0..900 { s.step_bond(Input::default(), false, false, false); } }
    stage(&mut scene, "after ch 0x47");

    // Stage 2: Kiko's walk — TouchOnce #381 → 0x0600062F writes ch 0x48 [D].
    scene.sim.as_mut().unwrap().bond.pos = [-49.2, 0.9, 38.4];
    tick_until(&mut scene, 1500, |s| s.drones.channels.on(0x48));
    assert!(scene.sim.as_ref().unwrap().drones.channels.on(0x48), "the Kiko cut-scene writes ch 0x48 [D]");
    { let s = scene.sim.as_mut().unwrap(); for _ in 0..900 { s.step_bond(Input::default(), false, false, false); } }
    stage(&mut scene, "after ch 0x48");

    // Stage 3: the sabotage — the west generator-room fuseboxes #383/#385 write 0x87/0x88 when cut
    // [M FuseBox_Update 0x1adfe8]; AND #391 [0x87, 0x88] → 0x8C [D] — the ch-0x8C spawner wave's arm.
    {
        let s = scene.sim.as_mut().unwrap();
        s.drones.channels.set(0x87, 1, 0);
        s.drones.channels.set(0x88, 1, 0);
        for _ in 0..600 { s.step_bond(Input::default(), false, false, false); }
        assert!(s.drones.channels.on(0x8c), "AND #391 raises 0x8C from the two cut groups [D]");
    }
    stage(&mut scene, "after ch 0x8C");

    // Stage 4: the room escape — Touch #456 (out 0x0C) with 0x8C → SEQ_AND #445 → 0x09 [D]. The
    // ch-0x09 spawner wave holds the 0x050000b1/8c templates behind the room; the game spawns it as
    // the player walks ON (the visible-spot screen [M 0x137cb0:43] aborts while Bond stands and
    // stares), so the census rounds rotate the look and move down the escape route.
    scene.sim.as_mut().unwrap().bond.pos = [-33.1, 0.9, 26.8];
    tick_until(&mut scene, 400, |s| s.drones.channels.on(0x0c));
    assert!(scene.sim.as_ref().unwrap().drones.channels.on(0x0c), "the room touch #456 writes 0x0C [D]");
    {
        let s = scene.sim.as_mut().unwrap();
        s.bond.pos = [-30.0, 0.9, 10.0];
        for round in 0..6u32 {
            s.bond.yaw = round as f32;
            for _ in 0..200 { s.step_bond(Input::default(), false, false, false); }
        }
    }
    stage(&mut scene, "after ch 0x09 (the escape walk)");

    // Stage 5: Rook — Touch #476 (out 0x0F) with 0x8C → SEQ_AND #475 → 0x0D [D]; his death ch 0x54.
    scene.sim.as_mut().unwrap().bond.pos = [-40.0, 0.9, 45.0];
    tick_until(&mut scene, 400, |s| s.drones.channels.on(0x0d));
    { let s = scene.sim.as_mut().unwrap(); for _ in 0..900 { s.step_bond(Input::default(), false, false, false); } }
    stage(&mut scene, "after ch 0x0D (Rook awake)");
    {
        let s = scene.sim.as_mut().unwrap();
        s.drones.channels.set(0x54, 1, 0);
        for _ in 0..900 { s.step_bond(Input::default(), false, false, false); }
    }
    stage(&mut scene, "after ch 0x54");
}

/// The raw-archive census [D], pinned: the 11 param-0 0x050000b1 placements, the 0x05000090
/// placements (the 0x1be910:33-43 substitution), and the fact that every EFFECTIVE skin (the
/// Drone+0xe0 rewrite + the captain swap) sits in 07000014's own archive.
#[test]
fn the_b1_and_90_placements_resolve_to_own_archive_skins() {
    let Some(iso) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let img = nf_formats::iso9660::IsoImage::open(iso.to_str().unwrap()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("07000014.bin on the disc");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = nf_formats::level::Level::parse(&data).unwrap();
    let mm = level.main_map().unwrap();
    let (mut b1, mut s90) = (Vec::new(), Vec::new());
    for (i, p) in level.maps[mm].placements.iter().enumerate() {
        if p.flags & 0xff != 0x0f { continue; }
        let p0 = p.params.iter().find(|(j, _)| *j == 0).map(|&(_, v)| v).unwrap_or(0);
        match p0 {
            0x0500_00b1 => b1.push(i),
            0x0500_0090 => s90.push(i),
            _ => {}
        }
    }
    // [D probe this session] 11 b1 placements (groups 10/12/14/15, p9=26) and 20 0x05000090 ones.
    assert_eq!(b1, vec![448, 449, 465, 468, 471, 472, 479, 703, 704, 1029, 1030],
        "the raw 0x050000b1 placements [D]");
    assert_eq!(s90.len(), 15, "the 0x05000090 placements [D]");
    // Every effective skin is own-archive [D probe: level.skins holds 8c/93/b2; 90 rides the
    // substitution 0x1be910:33-43 → 0x05000093 and the b1 rewrite 0x14b300:82-95 → 0x0500008c].
    for h in [0x0500_008cu32, 0x0500_0093, 0x0500_00b2] {
        assert!(level.skins.contains_key(&h), "0x{h:08x} is a 07000014 own-archive skin [D]");
    }
    assert!(!level.skins.contains_key(&0x0500_00b1), "the raw b1 rides only 07000015/16 [D carriers scan]");
}
