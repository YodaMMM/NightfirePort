//! level-06-searchlight-state: the 07000006 courtyard searchlights' OFF state after the fusebox
//! Laser cut, through the REAL input path (the guardroom stance, the Laser drawn, the group-0 wire
//! burned — `FuseBox_Update` 0x1adfe8 writes ch 57 + 103 [M]).
//!
//! The game's off look [M Searchlight_Update 0x1cb900:92-138]: the head's off-channel kill sets
//! state 3, deletes the spot/beam/corona objects (obj+0xFE bit 0 — the movement pass removes them,
//! 0x1336f8:88-91), clears the spot's psiLight node, zeroes the head's on/off/alarm bytes
//! (data+0x34/35/36) and runs `Control_NextLOD` on the head [M :138 → 0x1349b0: obj+0xD8 ←
//! *(glist+0x40)] — the block-0x27 chain target, which for every 07000006 head is map object 131
//! OBJECT_SearchLightBroken [D: four head placements #2/#4/#6/#8 all resolve object 2, its lod
//! entry is 131; the mounts/beams/coronas/spots carry −1]. The MOUNT and the HEAD keep drawing —
//! the housing stays, with the dark-lens variant on the head. No light re-lights afterwards: the
//! state-3 arm reads the head's own on-byte, which the kill zeroed [M :399].
//!
//! What the renderer must do differently (the draw half, listed for the report): a
//! lod_next head draws OBJECT_SearchLightBroken's mesh (the dark lens) at the head's matrix
//! instead of OBJECT_SearchLight1's lit-lens mesh; the mount keeps its mesh; the
//! spot/beam/corona stay collapsed. The sim side pins all of it below.
//!
//! Opt-in via NF_ISO (the slim ISO is enough). Mechanics follow fusebox_06_live's proven aim loop.

use nf_game::player::{Input, RamSave};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn load_06(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load_with_carry(iso, "07000006.bin", false, Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    assert!(!sim.disabled, "Bond is free after the intro");
    scene
}

/// The four head parts by placement order (#2/#4/#6/#8, the kind-1 records).
fn heads(sim: &nf_view::scene::Sim) -> Vec<usize> {
    sim.drones.searchlights.parts.iter().enumerate().filter(|(_, p)| p.kind == 1).map(|(i, _)| i).collect()
}

fn stance(scene: &mut nf_view::scene::Scene) {
    let sim = scene.sim.as_mut().unwrap();
    let spot = [29.9f32, 41.0, -17.9];
    let floor = sim.world.segment(spot, [spot[0], spot[1] - 30.0, spot[2]]).map(|t| spot[1] - 30.0 * t);
    if let Some(f) = floor { sim.bond.pos = [spot[0], f + 0.1, spot[2]]; } else { sim.bond.pos = spot; }
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
}

/// The wire object's VISIBLE geometry (fusebox_06_live's helper verbatim): the script entity's
/// local bounds through its live entity matrix.
fn wire_world_box(sim: &nf_view::scene::Sim, obj: usize) -> Option<([f32; 3], [f32; 3])> {
    match sim.drones.scripts.objects.list.get(obj)? {
        Some(nf_game::nis::Obj::Entity { hash, matrix, .. }) => {
            let (lo, hi) = sim.drones.wire_geo.get(hash)?;
            let (mut wlo, mut whi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for cx in 0..2 { for cy in 0..2 { for cz in 0..2 {
                let p = [if cx == 0 { lo[0] } else { hi[0] }, if cy == 0 { lo[1] } else { hi[1] }, if cz == 0 { lo[2] } else { hi[2] }];
                let w = nf_formats::placement::transform(matrix, p);
                for c in 0..3 { wlo[c] = wlo[c].min(w[c]); whi[c] = whi[c].max(w[c]); }
            } } }
            Some((wlo, whi))
        }
        _ => None,
    }
}

/// Where a player aims to burn the wire (fusebox_06_live's helper verbatim).
fn wire_aim_point(sim: &nf_view::scene::Sim, obj: usize) -> [f32; 3] {
    if let Some((lo, hi)) = wire_world_box(sim, obj) {
        return [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
    }
    nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), obj)
        .unwrap_or([0.0; 3])
}

/// fusebox_06_live's proven burn loop: hold fire, read the tick's Laser round, shift the aim by
/// the round's miss against the wire's own slab.
fn burn_wire(scene: &mut nf_view::scene::Scene, obj: usize, max_ticks: u32) -> bool {
    let mut delta = [0.0f32; 3];
    for _ in 0..max_ticks {
        let round = {
            let sim = scene.sim.as_mut().unwrap();
            let aim = wire_aim_point(sim, obj);
            let target = [aim[0] + delta[0], aim[1] + delta[1], aim[2] + delta[2]];
            let head = sim.bond.head();
            let dir = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
            let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
            sim.bond.yaw = dir[0].atan2(dir[2]);
            sim.bond.pitch = (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2;
            let n0 = sim.drones.player_bullets.len();
            let hit = sim.drones.channels.on(57) || sim.drones.channels.on(103);
            sim.step_bond(Input::default(), !hit, false, false);
            if hit { None } else {
                sim.drones.player_bullets.get(n0).filter(|b| b.weapon == 0x4e)
                    .map(|b| (b.pos, b.dir, b.speed))
            }
        };
        if round.is_none() && scene.sim.as_ref().unwrap().drones.channels.on(57) { return true; }
        if let Some((pos, dir, speed)) = round {
            let sim = scene.sim.as_ref().unwrap();
            if let Some((lo, hi)) = wire_world_box(sim, obj) {
                let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
                let e = [pos[0] + dir[0] * speed, pos[1] + dir[1] * speed, pos[2] + dir[2] * speed];
                let ab = [e[0] - pos[0], e[1] - pos[1], e[2] - pos[2]];
                let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
                let t = if ab2 > 1e-9 { (((centre[0] - pos[0]) * ab[0] + (centre[1] - pos[1]) * ab[1] + (centre[2] - pos[2]) * ab[2]) / ab2).clamp(0.0, 1.0) } else { 0.0 };
                let closest = [pos[0] + ab[0] * t, pos[1] + ab[1] * t, pos[2] + ab[2] * t];
                for c in 0..3 {
                    delta[c] = (delta[c] + (centre[c] - closest[c])).clamp(-0.8, 0.8);
                }
            }
        }
    }
    scene.sim.as_ref().unwrap().drones.channels.on(57)
}

/// The disc pin [D]: every 07000006 head placement resolves OBJECT_SearchLight1 whose block-0x27
/// chain entry points at OBJECT_SearchLightBroken — the game's off/broken model swap target
/// (`parsemap_block_LOD` 0x1d1f10 stores it into glist+0x40; `Control_NextLOD` 0x1349b0 walks it).
/// The mounts/beams/coronas/spots carry −1 (no swap target).
#[test]
fn every_06_head_placement_chains_to_the_searchlight_broken_variant() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000006.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let mi = lv.main_map().unwrap();
    let m = &lv.maps[mi];
    let mut checked = 0;
    for p in m.placements.iter().filter(|p| p.flags & 0xff == 0xde && p.params.iter().any(|&(k, v)| k == 0 && v == 1)) {
        let oi = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied().unwrap_or((mi, usize::MAX)).1 } else { p.index as usize };
        let name = m.objects.get(oi).map(|o| o.name.trim().to_string()).unwrap_or_default();
        assert_eq!(name, "OBJECT_SearchLight1", "the head object [D census]");
        let t = m.lod_target(oi).expect("the head's block-0x27 chain entry is set [D]");
        assert_eq!(m.objects[t].name.trim(), "OBJECT_SearchLightBroken",
            "the head's chain target is the broken/dark variant [D block 0x27]");
        checked += 1;
    }
    assert_eq!(checked, 4, "the four courtyard heads [D census: #2/#4/#6/#8]");
    // The non-head parts: no chain (a Control_NextLOD on them would no-op [M 0x1349b0's != 0 guard]).
    for p in m.placements.iter().filter(|p| p.flags & 0xff == 0xde && !p.params.iter().any(|&(k, v)| k == 0 && v == 1)) {
        let oi = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied().unwrap_or((mi, usize::MAX)).1 } else { p.index as usize };
        assert!(m.lod.get(oi).copied().map_or(true, |x| x < 0), "mount/beam/corona/spot object {oi} has no chain entry [D]");
    }
}

/// THE symptom: through the real Laser cut, every searchlight ends in the SAME off state — the
/// game's field-for-field off-kill [M 0x1cb900:92-138] on all four lights, housing drawn (the
/// mount keeps drawing; the head stays drawn with its model swap pinned), no beam/corona/spot,
/// no light node, and no re-light when the boot NIS's staged on-channels (61..64 [D 0x06000066])
/// keep pulsing afterwards.
#[test]
fn the_laser_cut_leaves_every_06_searchlight_in_the_same_game_off_state() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso);
    // Wait for the boot NIS's staged reveal (the port raises 64/63/61/62 across ~167 ticks [D
    // 0x06000066 stream 0]) so the cut lands on LIT lights like the player's run.
    let mut lit_all = false;
    'outer: for _ in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        lit_all = heads(sim).iter().all(|&h| sim.drones.searchlights.parts[h].state == 0);
        if lit_all { break 'outer; }
    }
    assert!(lit_all, "all four courtyard heads sweep after the staged reveal");
    stance(&mut scene);
    // The burn: draw the Laser, hold fire on the group-0 wire through the real path.
    {
        let draw = Input { switch_to: Some(0x4e), ..Default::default() };
        for _ in 0..40 { scene.sim.as_mut().unwrap().step_bond(draw, false, false, false); }
        assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the Laser equips");
    }
    let obj = {
        let sim = scene.sim.as_ref().unwrap();
        let fb = sim.drones.fuseboxes.first().expect("the 06 fusebox #747 exists");
        fb.alive_wires().filter(|(g, _, _)| *g == 0)
            .filter_map(|(_, _, o)| nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), o).map(|_| o))
            .next().expect("the group-0 bundle is registered [M FuseBox_Register 0x1b0970]")
    };
    assert!(burn_wire(&mut scene, obj, 900), "the Laser cuts the group-0 bundle: ch 57 rises");
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(57) && sim.drones.channels.on(103), "ch 57 + 103 [D #747 p1/p3]");
    }
    // The boot NIS keeps running: let its remaining staged writes (and any later channel traffic)
    // pulse past before the end-state assert — the killed heads must not re-light.
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let hs = heads(sim);
    assert_eq!(hs.len(), 4, "the four courtyard heads [D census]");
    // The uniform off state — the mount link's own INDEX differs per head (each head links its own
    // mount), so the shape is asserted separately from the scalar fields.
    let mut anchor: Option<(u8, bool, u8, u8, u8, bool, bool)> = None;
    for &h in &hs {
        let p = &sim.drones.searchlights.parts[h];
        let now = (p.state, p.killed, p.on, p.off, p.alarm, p.lod_next, p.drawn);
        match anchor {
            None => anchor = Some(now),
            Some(ref a) => assert_eq!(*a, now, "every head ends in the SAME off state [M 0x1cb900:92-138]"),
        }
        assert_eq!(p.links[1..5], [None, None, None, None], "spot/beam/corona links cleared [M :114-133]");
        let mount = p.links[0].expect("the mount link survives [M the kill walk never touches data+0x20]");
        assert!(sim.drones.searchlights.parts[mount].drawn, "the housing's mount keeps drawing [M]");
        assert!(p.drawn, "the head keeps drawing — the housing is the swapped model [M :138]");
        assert!(p.lod_next, "Control_NextLOD ran — the head draws the broken variant [M :138; D → obj 131]");
    }
    for (i, p) in sim.drones.searchlights.parts.iter().enumerate() {
        match p.kind {
            2 | 3 | 4 => assert!(!p.drawn, "part {i} (spot/beam/corona) is gone [M :113-133]"),
            _ => {}
        }
        if p.kind == 2 { assert!(p.light.is_none(), "the spot's psiLight node freed [M :105-110]"); }
    }
}
