//! s/collide-push-2: Bond's wall push against measured PCSX2 holds. Each case places Bond with a raw
//! position write (the game's obj+0x30, yaw obj+0x54; the port's pos/yaw with the cel re-found),
//! waits, then holds full forward and compares the resting spot with the game's. Opt-in: NF_ISO.
//!
//! The live runs: PCSX2 (GDB build), a fresh level load, god mode held, the left stick held at
//! full forward through the GDB `input_set`, Bond's obj+0x30 read every frame over PINE.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Idle ticks after the load, before the placement, so the spawn settle hold has released.
const SPAWN_IDLE: u32 = 200;

/// Loads `level`, idles `SPAWN_IDLE` ticks, places Bond at `at` facing `yaw`, waits `settle` idle ticks, holds the use key
/// `use_ticks` ticks then waits `after_use`, then holds forward for `hold` ticks. Returns the
/// position after the hold.
fn hold(level: &str, at: [f32; 3], yaw: f32, settle: u32, use_ticks: u32, after_use: u32, hold: u32) -> [f32; 3] {
    let mut w = Walker::new(nf_view::load(&iso(), level, false).unwrap());
    w.wait_enabled(3000, "load");
    // Idle out the spawn settle hold first: placing Bond while it still runs lets its release
    // integrate the held velocity once (0.0082 off a placed height).
    for _ in 0..SPAWN_IDLE { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let cel = sim.world.find_cel(at);
        sim.bond.pos = at;
        sim.bond.prev_pos = at;
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = yaw;
        sim.bond.cel = cel;
    }
    for _ in 0..settle { w.tick(Input::default(), false); }
    for _ in 0..use_ticks { w.tick(Input::default(), true); }
    for _ in 0..after_use { w.tick(Input::default(), false); }
    for _ in 0..hold { w.tick(Input { forward: 1.0, ..Default::default() }, false); }
    w.scene.sim.as_ref().unwrap().bond.pos
}

/// 07000006, the shelf under the gallery: held north into the balustrade post at x 36. The post's
/// south face carries the stored plane n.z −0.99994, d −62.1297 [D the record], 0.0043 in front
/// of its own vertices (z −62.129166). The game rests Bond at z −62.6835 [L 2026-11-21: placed
/// (36.0, 46.2, −63.3) yaw 0, 400 frames forward, the contact z −62.6835 from frame 12 on], which
/// is that record plane + 0.55; the vertex plane gives −62.6792. So the push reads the stored
/// record [M Intersect_CylGeom 0x109da0 disasm 0x10a1d8-0x10a1e8 → Dist2Tri 0x109770's plane].
/// Known gap: the game's Bond also creeps east along the post (x 36.0 → 36.49 over ~100 frames,
/// with y 46.20-46.23); the port's stays at x 36.0. Only z is pinned here.
#[test]
fn the_post_face_rests_bond_on_the_stored_plane() {
    let p = hold("07000006.bin", [36.0, 46.2, -63.3], 0.0, 20, 0, 0, 60);
    assert!((p[2] - (-62.6835)).abs() < 3e-4, "Bond rests on the record plane + r like the game (z −62.6835 [L]), got {p:?}");
}

/// 07000006, the same shelf held east into the room's east wall: the game stops Bond at (36.7021,
/// 46.3022, −63.4152) [L 2026-11-21: placed (30.0, 46.2, −63.5) yaw π/2, 120 frames forward,
/// resting from frame ~75]. The east wall's record d is 37.0000-37.0018 with n.x −0.99994 (the
/// x 36.7021 contact); the old winding planes stopped Bond at (36.7000, 46.3012, −63.4168).
#[test]
fn the_shelf_east_corner_matches_the_game() {
    let p = hold("07000006.bin", [30.0, 46.2, -63.5], std::f32::consts::FRAC_PI_2, 20, 0, 0, 120);
    assert!((p[0] - 36.7021).abs() < 3e-4 && (p[2] - (-63.4152)).abs() < 5e-4,
        "the corner rest matches the game (36.7021, _, −63.4152 [L]), got {p:?}");
}

/// 0700000a, the west office door: Bond at (−20.5, 1.2, 4.3) facing east opens the door with the
/// use key and holds forward into the doorway. The opened leaf swings into his path and the game
/// wedges him between it and the jamb at (−20.2343, _, 4.4970) [L 2026-11-21: Square 10 frames,
/// 90 frames idle, 150 frames forward; the rest spot holds from frame ~30]. The old two-sided push
/// rested him at (−20.2329, 1.0327, 4.4834): 0.014 off in z and sunk 0.09.
#[test]
fn the_opened_west_office_door_wedges_bond_like_the_game() {
    let p = hold("0700000a.bin", [-20.5, 1.2, 4.3], std::f32::consts::FRAC_PI_2, 20, 10, 90, 150);
    assert!((p[0] - (-20.2343)).abs() < 5e-4 && (p[2] - 4.4970).abs() < 5e-4,
        "the door leaf wedges Bond where the game's does ((−20.2343, _, 4.4970) [L]), got {p:?}");
    assert!(p[1] > 1.1, "Bond stands on the floor at the wedge (the game's y 1.1237 [L]), got {:.4}", p[1]);
}

/// 07000016, up the ramp to the ring and into the north wall: the game rests Bond at (14.5530,
/// 7.1948, −74.6588) [L 2026-11-21: placed (14.5, 1.7, −87.5) yaw 0, 200 frames forward]; the
/// ramp climb y matches frame by frame (6.744 on the ring).
#[test]
fn the_ring_ramp_climb_ends_where_the_games_does() {
    let p = hold("07000016.bin", [14.5, 1.7, -87.5], 0.0, 20, 0, 0, 200);
    assert!((p[0] - 14.5530).abs() < 2e-3 && (p[1] - 7.1948).abs() < 2e-3 && (p[2] - (-74.6588)).abs() < 2e-3,
        "the ramp climb rests at the game's (14.5530, 7.1948, −74.6588) [L], got {p:?}");
}

/// 0700000a, the lobby doorway into the west hall: held west from (−8.0, 1.12, 16.29), Bond slides
/// north around the jamb base and runs the hall at z 16.8295 [L 2026-11-21: 80 frames forward,
/// z 16.8296-16.8295 from frame 17 on, x −14.93 at frame 80].
#[test]
fn the_west_hall_line_matches_the_game() {
    let p = hold("0700000a.bin", [-8.0, 1.12, 16.29], -std::f32::consts::FRAC_PI_2, 20, 0, 0, 60);
    assert!(p[0] < -11.0 && (p[2] - 16.8295).abs() < 3e-4, "the hall line is the game's z 16.8295 [L], got {p:?}");
}

/// The early exit inside `Intersect_CylGeom`'s triangle loop changes nothing [M + D]: after the
/// vertex decode, `vsub.x vf0, vf1, vf21` (vf1 = the query box max at sp+0x20, vf21 = the
/// triangle's vertex 0) sets the VU status S bit when vertex 0's x passes the box max, and
/// `cfc2 v0,$16` / `andi v0,v0,2` and the branch at 0x10a100 to 0x10a7d0 (disasm 0x10a0cc-0x10a100)
/// then leaves the box's triangle loop for the next box. On every level on the disc each leaf
/// box's triangles run in nondecreasing vertex-0 x and vertex 0 is each triangle's smallest x, so
/// every triangle the exit skips starts beyond the box max in x and fails the per-triangle box
/// test anyway. The port's flat triangle walk therefore needs no exit.
#[test]
fn every_leaf_runs_its_triangles_in_vertex0_x_order() {
    use nf_formats::levelbin::LevelBin;
    use nf_formats::map::MapFile;
    let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let (mut leaves, mut tris) = (0usize, 0usize);
    for lv in nf_view::dev::LEVELS {
        let name = format!("{lv:08x}.bin");
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)) else { continue };
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lb = LevelBin::parse(&data).expect("levelbin");
        for e in lb.entries.iter() {
            let edata = lb.data(&data, e);
            let Ok(mf) = MapFile::parse(edata) else { continue };
            for blk in mf.blocks.iter().filter(|b| b.id == 0x2e) {
                let Ok(c) = nf_formats::collision::Collision::parse(blk.payload(edata)) else { continue };
                for bx in c.boxes.iter().filter(|b| b.children.is_none()) {
                    leaves += 1;
                    let base = bx.vblock as usize * 0x20;
                    let x = |i: u16| c.words.get(base + i as usize).copied().unwrap_or(0);
                    let mut prev = i16::MIN;
                    for t in &c.tris[bx.tris.start as usize..bx.tris.end as usize] {
                        tris += 1;
                        let x0 = x(t.v[0]);
                        assert!(x0 >= prev, "{name}: a leaf's triangles are out of vertex-0 x order");
                        assert!(x0 <= x(t.v[1]) && x0 <= x(t.v[2]), "{name}: vertex 0 is not the smallest x");
                        prev = x0;
                    }
                }
            }
        }
    }
    assert!(leaves > 20_000 && tris > 2_000_000, "the disc-wide census ran ({leaves} leaves, {tris} triangles)");
}

/// 0700000a, the corner south of the west office door (the door closed): placed at (−17.87, 1.12,
/// 4.87) facing yaw 2.987 and held forward 60 frames, the game slides the corner and stops at
/// (−17.9715, 1.1159, −0.4162) [L 2026-10-26 fresh boot, the walkreal corner test's numbers].
#[test]
fn the_corner_line_matches_the_game() {
    let p = hold("0700000a.bin", [-17.87, 1.12, 4.87], 2.987, 20, 0, 0, 60);
    assert!((p[0] - (-17.9715)).abs() < 5e-3 && (p[1] - 1.1159).abs() < 5e-3 && (p[2] - (-0.4162)).abs() < 5e-3,
        "the corner line ends where the game's does ((−17.9715, 1.1159, −0.4162) [L]), got {p:?}");
}
