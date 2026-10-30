//! Countdown Part 1, the generator-hall blast aftermath (07000014): the Destroy leaves the
//! cascade smashes must SWAP to the game's destroyed glist, not vanish wholesale.
//!
//! The game's chain [M]: `Destroy_Create` 0x12c318:25-30 stores tag[0] = `*(glist+0x40)` when
//! the block-0x27 LOD-next word stands, else the parsemap fallback slot `pCelList +
//! idx*0x50 + 0x50` (0x1d1b68 computes it unconditionally; `parsemap_block_LOD` 0x1d1f10
//! writes `cel+0x40 = index*0x50 + pCelList`, so the fallback record is the NEXT main-map
//! entity's glist); `Destroy_Smash` 0x12bb60 swaps it in with `Control_SetGList(obj, tag[0])`
//! (the decomp dropped the arg — disassembly 0x12bbb8 `lw a1,0(s1)` → 0x12bbc4 `jal`). A
//! real-mesh target shows the wreck; an undrawable slot (no batches / the no-draw bit 0x10)
//! vanishes [M View_AddObjects 0x1e62d8:33's `(obj+0xF0 & 0x8010) == 0`, the FLAGS_COPYING
//! census]. Opt-in via NF_ISO.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn drawable(level: &Level, m: usize, o: usize) -> bool {
    let obj = &level.maps[m].objects[o];
    obj.flags & 0x10 == 0 && obj.gfx.leaves.iter().any(|l| !l.batches.is_empty())
}

/// The disc's destroyed-glist words for the three 0xe1 shapes on 07000014, and the port's
/// resolution of each [D probe census: 19× DESTROY_BoxSatellite (object 273), 25×
/// object_oilbarrel01 (290), 4× GENERATORS_Intact (291) — every row hashcode −1].
#[test]
fn the_destroyed_glists_resolve_per_the_game() {
    let iso = iso();
    let level = Level::parse(&read_archive(&iso, "07000014.bin")).expect("parse 07000014");
    let m = level.main_map().expect("main map");
    let find = |name: &str| (0..level.maps[m].objects.len()).find(|&o| level.maps[m].objects[o].name.trim() == name)
        .unwrap_or_else(|| panic!("{name} exists [D]"));
    let barrels = find("object_oilbarrel01");
    let barrel_wreck = find("object_oilbarrel01_destroyed");
    let gens = find("GENERATORS_Intact");
    let gen_wreck = find("GENERATORS_Destroyednew");
    let sat = find("DESTROY_BoxSatellite");
    let icon = find("3RD_Person_Icon");
    // The LOD words [D block 0x27]: the barrels and the generators carry a real destroyed
    // variant (the wreck's glist), the satellite box does not.
    assert_eq!(level.maps[m].lod.get(barrels).copied(), Some(barrel_wreck as i32),
        "object_oilbarrel01's block-0x27 word is the destroyed barrel [D]");
    assert!(drawable(&level, m, barrel_wreck), "the destroyed barrel's glist is a real mesh [D]");
    assert_eq!(level.maps[m].lod.get(gens).copied(), Some(gen_wreck as i32),
        "GENERATORS_Intact's block-0x27 word is GENERATORS_Destroyednew [D]");
    assert!(drawable(&level, m, gen_wreck), "GENERATORS_Destroyednew's glist is a real mesh [D]");
    assert_eq!(level.maps[m].lod.get(sat).copied(), Some(-1), "DESTROY_BoxSatellite has no LOD variant [D]");
    // The fallback slot for the satellite box: the next entity's glist record — an undrawable
    // slot (the no-draw bit 0x10), so the game's smash leaves nothing drawn [M 0x1d1b68 +
    // View_AddObjects 0x8010].
    assert_eq!(sat + 1, icon, "the satellite box's fallback record is 3RD_Person_Icon [D]");
    assert!(!drawable(&level, m, icon), "3RD_Person_Icon's glist is undrawable (flags {:#x}) [D]",
        level.maps[m].objects[icon].flags);
    // The port's bake: every DRAWN leaf with a drawable destroyed glist has a wreck entry, and
    // no satellite leaf got one (its target is undrawable).
    let scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    assert!(!scene.destroy_centres.is_empty(), "07000014 draws Destroy leaves [D]");
    let drawn: std::collections::HashSet<usize> = scene.destroy_centres.iter().map(|(_, pl)| *pl).collect();
    let wrecked: std::collections::HashSet<usize> = scene.destroy_wrecks.iter().map(|(_, pl)| *pl).collect();
    assert!(!wrecked.is_empty(), "the drawn barrel/generator leaves bake their wrecks");
    assert!(wrecked.is_subset(&drawn), "wreck entries exist only for drawn leaves");
    let sat_placements: Vec<usize> = level.maps[m].placements.iter().enumerate()
        .filter(|(_, p)| p.flags & 0xff == 0xe1 && !p.by_hash() && p.index as usize == sat)
        .map(|(i, _)| i).collect();
    for i in sat_placements { assert!(!wrecked.contains(&i), "no wreck baked for the satellite leaf #{i} (undrawable slot)"); }
}

/// The real-input path: the sabotage AND 0x8C (the real laser cut is the walk's leg; here the
/// two words seed the REAL AND mux [D AND plc (0x87,0x88)→0x8C]), the REAL E press on the
/// console plate [M Player_Activate 0x1a5170's class-0x1D arm → Trigger_Activate 0x1b2318
/// mask 0x4], the cascade → 0x85 [D] — then the cascade's Destroy leaves are dead and their
/// wreck draws are the only new draws (the intact draws drop — the swap).
#[test]
fn the_cascade_swaps_the_smashed_leaves_to_their_wrecks() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    let t_centre = {
        let sim = scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter()
            .find(|t| t.out == 0xa0 && t.mask & 4 != 0 && t.kind == nf_game::triggers::kind::TOUCH)
            .expect("the 0xA0 E-use touch exists [D placement 399]");
        [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
    };
    scene.sim.as_mut().unwrap().drones.channels.set(0x87, 1, 1);
    scene.sim.as_mut().unwrap().drones.channels.set(0x88, 1, 1);
    scene.sim.as_mut().unwrap().bond.pos = [-60.3, 0.9, 38.4];
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        let head = sim.bond.head();
        let d = [t_centre[0] - head[0], t_centre[1] - head[1], t_centre[2] - head[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.yaw = d[0].atan2(d[2]);
        sim.bond.pitch = (d[1] / l).asin() / std::f32::consts::FRAC_PI_2;
    }
    let mut fired = false;
    for _ in 0..12 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, true);
        if scene.sim.as_ref().unwrap().drones.channels.on(0xa0) { fired = true; break; }
    }
    assert!(fired, "the E press fires the mask-4 touch → 0xA0 [M 0x1b2318:23-33]");
    let mut ch85 = false;
    for _ in 0..300u32 {
        if scene.sim.as_ref().unwrap().drones.channels.on(0x85) { ch85 = true; }
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.health.health = 100.0;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        if ch85 { break; }
    }
    assert!(ch85, "the cascade reaches box #729's death 0x85 [D the kill→death pairs]");
    let sim = scene.sim.as_ref().unwrap();
    // The smash landed: some Destroy leaves are dead, and every drawn dead leaf is either
    // wrecked (the swap target drawable) or vanished (undrawable) — never intact-drawn.
    let dead: std::collections::HashSet<usize> = sim.drones.props.destroy.iter()
        .filter(|d| d.dead).map(|d| d.placement).collect();
    assert!(!dead.is_empty(), "the cascade smashed Destroy leaves");
    let drawn_dead: Vec<usize> = scene.destroy_centres.iter()
        .map(|(_, pl)| *pl).filter(|pl| dead.contains(pl)).collect();
    assert!(!drawn_dead.is_empty(), "drawn Destroy leaves died in the cascade");
    let wrecked_dead: std::collections::HashSet<usize> = scene.destroy_wrecks.iter()
        .map(|(_, pl)| *pl).filter(|pl| dead.contains(pl)).collect();
    assert!(!wrecked_dead.is_empty(),
        "the smashed leaves' wreck draws are live (the game shows the wreck after the smash)");
    // The wreck draw set (the lib.rs feed's own filter shape): the dead leaves' wreck centres
    // ride `destroy_wreck_draw`; their INTACT centres ride `destroy_dead` — the two sets are
    // disjoint (different objects, different glists).
    let wreck_draw: std::collections::HashSet<[u32; 3]> = scene.destroy_wrecks.iter()
        .filter(|(_, pl)| dead.contains(pl)).map(|(c, _)| c.map(f32::to_bits)).collect();
    let intact_dead: std::collections::HashSet<[u32; 3]> = scene.destroy_centres.iter()
        .filter(|(_, pl)| dead.contains(pl)).map(|(c, _)| c.map(f32::to_bits)).collect();
    assert!(wreck_draw.is_disjoint(&intact_dead), "the wreck draws and the dead intact draws are disjoint sets");
    eprintln!("[014] dead destroy leaves {} (drawn {}, wrecked {}), wreck draws {}",
        dead.len(), drawn_dead.len(), wrecked_dead.len(), wreck_draw.len());
}
