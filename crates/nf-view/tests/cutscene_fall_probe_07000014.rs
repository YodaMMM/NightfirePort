//! level-14-cutscene-fall probe: which script plays at the broken cut-scene spot
//! (40.87, 2.98, 40.85) [frames 1931/2300] and at the fall spot (-49.65, 0.65, 36.85)
//! [frame 20889 black → 21799 below map at (-107.07, -70.14, 73.28)].
//!
//! Dumps (read-only):
//! 1. every ScriptPlayer placement (kind 0xd9) on 07000014: index, scripts, gate, out, base;
//! 2. the trigger/0xea-0xeb + mux placements near both spots, so the firing channel can be traced;
//! 3. the NIS scripts' streams + keys (camera streams, entity streams, events) for the suspects;
//! 4. a sim run of each scene staging: camera positions, NF_SCRIPTLOG event lines, the hand-back
//!    enable matrix and Bond's path after it (the fall).
//!
//! Opt-in via `NF_ISO`. Run with `--nocapture --test-threads=1`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn probe_placements_and_scripts() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== ScriptPlayer placements (kind 0xd9) ===");
    for p in &sim.drones.scripts.players {
        eprintln!(
            "SP #{} scripts {:08x}/{:08x} modes {:?} gate {:#x} out {:#x} stay {} t ({:.2},{:.2},{:.2}) r {:?}",
            p.placement, p.scripts[0], p.scripts[1], p.modes, p.gate, p.out, p.stay,
            p.matrix[3][0], p.matrix[3][1], p.matrix[3][2], p.matrix[2]
        );
    }

    // The trigger records (0xea/0xeb touch families + the muxes 0xec/0xed/0xef whose volumes/
    // channel lists sit near either spot): kind, out, list, bounds.
    eprintln!("=== triggers near (40.87, 2.98, 40.85) / (-49.65, 0.65, 36.85) ===");
    let mid = |lo: [f32; 3], hi: [f32; 3]| [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
    for t in &sim.drones.triggers {
        let c = mid(t.lo, t.hi);
        let c1 = (c[0] - 40.87).powi(2) + (c[2] - 40.85).powi(2) < 144.0;
        let c2 = (c[0] + 49.65).powi(2) + (c[2] - 36.85).powi(2) < 144.0;
        let inside1 = t.lo[0] <= 40.87 && 40.87 <= t.hi[0] && t.lo[2] <= 40.85 && 40.85 <= t.hi[2];
        let inside2 = t.lo[0] <= -49.65 && -49.65 <= t.hi[0] && t.lo[2] <= 36.85 && 36.85 <= t.hi[2];
        if c1 || c2 || inside1 || inside2 {
            eprintln!(
                "trig #{} kind {} out {:#x} list {:x?} lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2}){}{}",
                t.placement, t.kind, t.out, t.list,
                t.lo[0], t.lo[1], t.lo[2], t.hi[0], t.hi[1], t.hi[2],
                if inside1 { " [INSIDE cam-scene spot]" } else { "" },
                if inside2 { " [INSIDE fall spot]" } else { "" },
            );
        }
    }
    // The SPs whose gate the found triggers could feed: dumped above; re-list the 0xd9 players'
    // gate/out pairs for the cross-reference.
    eprintln!("=== SP gate/out quick list ===");
    for p in &sim.drones.scripts.players {
        eprintln!("SP #{} s {:08x} gate {:#x} out {:#x}", p.placement, p.scripts[0], p.gate, p.out);
    }

    // The three suspect scripts' full stream + key dump.
    for h in [0x0600_0833u32, 0x0600_062f, 0x0600_0848, 0x0600_0a1b, 0x0600_0708] {
        if let Some(n) = sim.drones.scripts.lib.get(&h) {
            eprintln!("HEADER {h:08x}: camera-flag {} streams {} length {}", n.camera, n.streams.len(), n.length);
        }
    }
    for h in [0x0600_0833u32, 0x0600_062f, 0x0600_0848] {
        let Some(n) = sim.drones.scripts.lib.get(&h) else { eprintln!("NIS {h:08x} NOT in lib"); continue };
        eprintln!("=== NIS {:08x} length {} camera-flag {} streams {} keys {} ===", h, n.length, n.camera, n.streams.len(), n.keys.len());
        for (i, s) in n.streams.iter().enumerate() {
            eprintln!(" stream {i}: keys {}..{} cmds {:?}", s.first_key, s.last_key, s.decode());
        }
        for (k, key) in n.keys.iter().enumerate() {
            eprintln!(
                " key {k}: pos ({:.4},{:.4},{:.4},{:.4}) quat ({:.5},{:.5},{:.5},{:.5}) scale {:.6} f24 {:.6} f28 {:.4} time {:.3}",
                key.pos[0], key.pos[1], key.pos[2], key.pos[3],
                key.quat[0], key.quat[1], key.quat[2], key.quat[3], key.scale, key.f24(), key.w28, key.time
            );
        }
    }
}

/// The fall: stage Bond at the Kiko touch area and run — record camera ticks, the hand-back
/// (via NF_SCRIPTLOG's free lines on stderr) and Bond's path after re-enable.
#[test]
fn probe_the_fall_run() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SCRIPTLOG", "1");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [-49.2, 0.9, 38.4];
    let mut cam_frames: Vec<(u32, [f32; 3], f32)> = Vec::new();
    let mut cam_tick = 0u32;
    let mut enabled_frame: Option<u32> = None;
    let mut disabled = false;
    for f in 0..3600u32 {
        sim.step_bond(Input::default(), false, false, false);
        let cam = sim.script_camera();
        if let Some((pos, _y, _p, fov)) = cam {
            if cam_tick < 4 || cam_tick % 60 == 0 { cam_frames.push((cam_tick, pos, fov)); }
            cam_tick += 1;
        }
        if sim.disabled { disabled = true; }
        else if disabled {
            disabled = false;
            enabled_frame.get_or_insert(f);
            eprintln!("trace: re-enabled at frame {f} bond ({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
        }
        if enabled_frame == Some(f) {
            // the floor under Bond at the hand-back + the next 300 ticks of fall
            for k in 0..15u32 {
                let p = sim.bond.pos;
                let floor = sim.world.segment(p, [p[0], p[1] - 80.0, p[2]]).map(|t| p[1] - 80.0 * t);
                eprintln!("trace: +{:3} bond ({:.2},{:.2},{:.2}) floor-under {:?} disabled {}", k * 20, p[0], p[1], p[2], floor, sim.disabled);
                for _ in 0..20 { sim.step_bond(Input::default(), false, false, false); }
                if sim.disabled { eprintln!("trace: re-disabled at +{}", k * 20); break; }
            }
        }
        if enabled_frame.is_some() && f > enabled_frame.unwrap() + 320 { break; }
    }
    eprintln!("trace: camera ran {cam_tick} ticks; disabled_seen {disabled}");
    for (t, pos, fov) in &cam_frames { eprintln!("trace: cam tick {t} pos ({:.2},{:.2},{:.2}) fov {fov:.4}", pos[0], pos[1], pos[2]); }
}

/// The opening scene's stage room: which cel owns the eye (78.06, 4.79, 82.69), what the portal
/// walk reaches from it, what geometry exists near the eye and which room cel the stage's draws
/// were assigned. Explains "camera inside rock, the rest of the frame a flat blue void".
#[test]
fn probe_the_stage_room() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_ref().unwrap();
    let eye = [78.0615f32, 4.7936, 82.6934];
    let g = &sim.cel_graph;

    // 1. The room-class cells whose bbox contains the eye, and all room cells within 20 m.
    eprintln!("=== room cells near the eye ===");
    let mut eye_cell_idx = None;
    for (i, c) in g.cells.iter().enumerate() {
        if c.flags & 0x40000 == 0 { continue; }
        let inside = (0..3).all(|k| c.min[k] <= eye[k] && eye[k] <= c.max[k]);
        let near = (0..3).all(|k| c.min[k] - 20.0 <= eye[k] && eye[k] <= c.max[k] + 20.0);
        if inside || near {
            eprintln!("room cell {i}: min ({:.1},{:.1},{:.1}) max ({:.1},{:.1},{:.1}) flags {:x} {}",
                c.min[0], c.min[1], c.min[2], c.max[0], c.max[1], c.max[2], c.flags, if inside { "[CONTAINS EYE]" } else { "" });
            if inside { eye_cell_idx = Some(i); }
        }
    }

    // 2. eye_cell (the game's persistent-viewer FindCel + tie-break).
    let mut vc = nf_view::scene::ViewerCel::default();
    let ec = nf_view::scene::eye_cell(g, &mut vc, eye, Some(&sim.world));
    eprintln!("eye_cell(78.06,4.79,82.69) = {ec:?} (room_to_cell of 26 = {:?})", g.room_to_cell.get(26).copied().flatten());

    // 3. Portals touching the eye cell.
    if let Some(ec) = ec {
        for (pi, pe) in g.portals.iter().enumerate() {
            if pe.cel_a == ec || pe.cel_b == ec {
                eprintln!("portal {pi}: cel_a {} cel_b {} door {:?}", pe.cel_a, pe.cel_b, g.portal_doors[pi]);
            }
        }
    }

    // 4. The portal walk with all-pass planes (pure connectivity + door states): which cells
    //    become visible from the eye cell?
    if let Some(ec) = ec {
        let inside_plane = [[0.0f32, 0.0, 0.0, 1e9]; 4];
        let depth0 = [[0.0f32, 0.0, 0.0, 1e9]; 5];
        let game = nf_view::scene::portal_visible_cels_game_from_cell_doors(g, ec, eye, &depth0, &inside_plane, None);
        match game {
            Some(v) => {
                let n = v.iter().filter(|&&b| b).count();
                eprintln!("portal walk from cell {ec}: {n}/{} cells visible; eye cell visible = {}", v.len(), v[ec]);
                // which ROOM cels are visible
                for (ri, rc) in g.room_to_cell.iter().enumerate() {
                    if let Some(ci) = rc {
                        if *ci < v.len() && v[*ci] {
                            let c = &g.cells[*ci];
                            eprintln!("  visible room {ri} (cell {ci}) min ({:.1},{:.1},{:.1}) max ({:.1},{:.1},{:.1})",
                                c.min[0], c.min[1], c.min[2], c.max[0], c.max[1], c.max[2]);
                        }
                    }
                }
            }
            None => eprintln!("portal walk returned None"),
        }
    }

    // 5. The world's collision triangles near the stage box.
    let tris = sim.world.tris_near([70.0, -5.0, 70.0], [105.0, 10.0, 105.0]);
    eprintln!("collision tris in the stage box (70..105, -5..10, 70..105): {}", tris.len());

    // 6. The render materials near the stage: their assigned room cel.
    let mut by_cel: std::collections::BTreeMap<i32, usize> = Default::default();
    let mut near_any = 0usize;
    for m in &scene.materials {
        let c = m.centre;
        if (70.0..105.0).contains(&c[0]) && (-10.0..15.0).contains(&c[1]) && (70.0..105.0).contains(&c[2]) {
            near_any += 1;
            *by_cel.entry(m.cel.map(|v| v as i32).unwrap_or(-1)).or_default() += 1;
        }
    }
    eprintln!("render materials with centre in the stage box: {near_any}, by assigned cel {by_cel:?}");
    // The same census for the eye's own cel (26) — how many draws sit in the room that owns the eye.
    if let Some(c26) = by_cel.get(&26) { eprintln!("draws assigned cel 26: {c26}"); }
}


/// The [D] disc data: does ANY placement/object/glist exist at the two stage boxes?
/// (70..105, -10..15, 70..105) = the opening scene's composed camera/actors area;
/// (-120..-90, -5..10, 60..85) = the Kiko scene's composed camera + hand-back area.
#[test]
fn probe_the_disc_data_at_the_stages() {
    use nf_formats::iso9660::IsoImage;
    use nf_formats::level::Level;
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap_or(0);
    eprintln!("disc: {} maps, main m{}, placements main {}", lv.maps.len(), mm, lv.maps[mm].placements.len());

    let boxes = [
        ("OPENING-STAGE", [70.0f32, -10.0, 70.0], [105.0, 15.0, 105.0]),
        ("KIKO-STAGE", [-120.0, -5.0, 60.0], [-90.0, 10.0, 85.0]),
        ("OPENING-RAWKEY", [20.0, 0.0, 30.0], [33.0, 5.0, 50.0]),
        ("KIKO-RAWKEY", [-60.0, 0.0, 35.0], [-45.0, 4.0, 45.0]),
    ];
    for (name, lo, hi) in boxes {
        eprintln!("=== {name}: placements with translation inside the box ===");
        for (mi, m) in lv.maps.iter().enumerate() {
            for (pi, p) in m.placements.iter().enumerate() {
                let t = p.translation;
                if !(lo[0]..hi[0]).contains(&t[0]) || !(lo[1]..hi[1]).contains(&t[1]) || !(lo[2]..hi[2]).contains(&t[2]) { continue; }
                let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < lv.maps[mi].objects.len() { Some((mi, p.index as usize)) } else { None };
                let (oname, omin, omax, oflags) = target.map(|(m2, o2)| {
                    let o = &lv.maps[m2].objects[o2];
                    (o.name.trim().to_string(), o.min, o.max, o.flags)
                }).unwrap_or_else(|| (format!("UNRESOLVED {:08x}", p.hashcode), [0.0; 3], [0.0; 3], 0));
                eprintln!("m{mi} #{pi} kind {:#04x} @({:.2},{:.2},{:.2}) obj \"{}\" glist min ({:.1},{:.1},{:.1}) max ({:.1},{:.1},{:.1}) gflags {:#x} pflags {:#x}",
                    p.flags & 0xff, t[0], t[1], t[2], oname, omin[0], omin[1], omin[2], omax[0], omax[1], omax[2], oflags, p.flags);
            }
        }
    }
    for (name, lo, hi) in boxes {
        eprintln!("=== {name}: objects whose glist bbox intersects the box ===");
        for (mi, m) in lv.maps.iter().enumerate() {
            for (oi, o) in m.objects.iter().enumerate() {
                let hit = (0..3).all(|k| o.max[k] >= lo[k] && o.min[k] <= hi[k]);
                if hit {
                    eprintln!("m{mi} obj {oi} \"{}\" glist min ({:.1},{:.1},{:.1}) max ({:.1},{:.1},{:.1}) gflags {:#x} r {:.1}",
                        o.name.trim(), o.min[0], o.min[1], o.min[2], o.max[0], o.max[1], o.max[2], o.flags, o.radius);
                }
            }
        }
    }
    eprintln!("=== disc kind-0xd9 placements for the three suspect scripts (all maps) ===");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            if p.flags & 0xff != 0xd9 { continue; }
            let s0 = p.params.iter().find(|(k, _)| *k == 0).map(|(_, v)| *v).unwrap_or(0);
            if !matches!(s0, 0x0600_0833 | 0x0600_062f | 0x0600_0848) { continue; }
            eprintln!("m{mi} #{pi} @({:.2},{:.2},{:.2}) rot {:?} params {:x?} flags {:#x}",
                p.translation[0], p.translation[1], p.translation[2], p.rotation, p.params, p.flags);
        }
    }
}

/// The camera-flag (NIS header u16 at +2) of every script whose composed/raw spot a test pins:
/// the discriminator for the [M 0x1d916c] base reset (CameraStart → Mat_IdentityT(SCRIPTINFO)).
#[test]
fn probe_camera_flags_of_pinned_scripts() {
    use nf_formats::iso9660::IsoImage;
    use nf_formats::levelbin::LevelBin;
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let read = |archive: &str| {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect(archive);
        img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
    };
    let flag = |data: &[u8], hash: u32| -> Option<u16> {
        let lb = LevelBin::parse(data).ok()?;
        for e in lb.entries.iter().filter(|e| e.kind == 7) {
            if e.hash() == hash {
                let d = lb.data(data, e);
                return Some(u16::from_le_bytes([d[2], d[3]]));
            }
        }
        None
    };
    for (archive, hash, what) in [
        ("07800001.bin", 0x0600_0087u32, "07000001 intro cut-scene IN THE PACK"),
        ("0780004a.bin", 0x0600_06cf, "0700004a landing scene pack A"),
        ("0790004a.bin", 0x0600_06cf, "0700004a landing scene pack B"),
        ("07a0004a.bin", 0x0600_06cf, "0700004a landing scene pack C"),
        ("07000001.bin", 0x0600_0087u32, "07000001 intro cut-scene (the exit_aim pack s11)"),
        ("07000008.bin", 0x0600_09e2, "07000008 SafePickUps (the shelf-key re-pose)"),
        ("0700004a.bin", 0x0600_06cf, "0700004a the landing scene (the hash-0 kind-0x19 hand-back)"),
        ("07000003.bin", 0x0600_0563, "07000003 the safe-contents spawner"),
        ("07000005.bin", 0x0600_0a22, "07000005 the truck script"),
        ("07000005.bin", 0x0600_0994, "07000005 the ledge cut-scene"),
        ("07000014.bin", 0x0600_0a3b, "07000014 the vent lid script"),
    ] {
        let data = read(archive);
        eprintln!("CAMFLAG {hash:08x} ({what}): {:?}", flag(&data, hash));
    }
}
