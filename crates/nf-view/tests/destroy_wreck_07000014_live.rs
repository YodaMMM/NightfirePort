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
//! census]. BUT the swap itself is conditional: `Destroy_Create` 0x12c318:57-59 ZEROES tag[0]
//! when the leaf's own glist flags carry 0x10 (obj+0xF0 = glist+0x44 [M Control_SetGList
//! 0x134960]) — `Destroy_Smash` then takes its `*piVar1 == 0` arm and skips the swap, so the
//! intact glist stays attached and the leaf keeps its intact draw. 07000014's server racks are
//! that case [D obj#273 DESTROY_BoxSatellite flags 0x30]: after the cascade every rack still
//! shows its model at its spot. Opt-in via NF_ISO.

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
    // The satellite box's own glist carries the no-draw bit [D flags 0x30] — Destroy_Create
    // 0x12c318:57-59 zeroes tag[0] for it, so the parsemap fallback (the next object's record —
    // 3RD_Person_Icon here [D object-table adjacency]) is NEVER swapped in: the smash skips the
    // SetGList [M 0x12bb60's `*piVar1 == 0` arm] and the intact glist stands.
    assert_eq!(level.maps[m].objects[sat].flags & 0x10, 0x10,
        "DESTROY_BoxSatellite's own glist flags carry the no-draw bit 0x10 [D]");
    assert_eq!(sat + 1, icon, "the satellite box's parsemap fallback record is 3RD_Person_Icon [D]");
    assert!(!drawable(&level, m, icon), "3RD_Person_Icon's glist is undrawable (flags {:#x}) [D]",
        level.maps[m].objects[icon].flags);
    // The port's bake: every DRAWN leaf with a drawable destroyed glist has a wreck entry, and
    // no satellite leaf got one (the creator zeroes its swap — the fallback never swaps in).
    let scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    assert!(!scene.destroy_centres.is_empty(), "07000014 draws Destroy leaves [D]");
    let drawn: std::collections::HashSet<usize> = scene.destroy_centres.iter().map(|(_, pl)| *pl).collect();
    let wrecked: std::collections::HashSet<usize> = scene.destroy_wrecks.iter().map(|(_, pl)| *pl).collect();
    assert!(!wrecked.is_empty(), "the drawn barrel/generator leaves bake their wrecks");
    assert!(wrecked.is_subset(&drawn), "wreck entries exist only for drawn leaves");
    let sat_placements: Vec<usize> = level.maps[m].placements.iter().enumerate()
        .filter(|(_, p)| p.flags & 0xff == 0xe1 && !p.by_hash() && p.index as usize == sat)
        .map(|(i, _)| i).collect();
    assert!(!sat_placements.is_empty(), "the disc places the satellite box [D]");
    for i in &sat_placements { assert!(!wrecked.contains(i), "no wreck baked for the satellite leaf #{i} (the swap is zeroed)"); }
    // The no-swap case's draw side: the satellite leaves are NOT in the per-frame dead gate —
    // their intact draws survive the smash (the game's racks never disappear) [M 0x12c318:57-59
    // + 0x12bb60's skip]. The swap-case leaves (barrels, generators — flags without 0x10 [D]) stay
    // in the gate.
    for i in &sat_placements {
        assert!(!drawn.contains(i), "the satellite leaf #{i} keeps its intact draw (no-swap case)");
    }
    assert!(sat_placements.iter().all(|i| !drawn.contains(i)) && !drawn.is_empty(),
        "the swap-case leaves ride the dead gate, the no-swap leaves do not");
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
    for _ in 0..1500u32 {
        if scene.sim.as_ref().unwrap().drones.channels.on(0x85) { ch85 = true; }
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.health.health = 100.0;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        if ch85 { break; }
    }
    assert!(ch85, "the cascade reaches box #729's death 0x85 [D the kill→death pairs]");
    let sim = scene.sim.as_ref().unwrap();
    // The smash landed: some Destroy leaves are dead, and every gated dead leaf (the swap case —
    // flags without 0x10) is wrecked or vanished, never intact-drawn. The no-swap leaves (the
    // racks — flags 0x30 [D]) are not in the gate at all: their intact draws stand.
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
    // The no-swap leaves that died in the same cascade keep their intact draws: not in
    // destroy_centres means the renderer's dead gate never touches them [M 0x12c318:57-59 +
    // 0x12bb60's skip].
    let noswap_dead: Vec<usize> = dead.iter().copied()
        .filter(|pl| !scene.destroy_centres.iter().any(|(_, c)| c == pl)).collect();
    assert!(!noswap_dead.is_empty(),
        "cascade-killed no-swap leaves (the racks) keep their intact draws");
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

/// The post-cascade door spam pin (level-14-cascade-servers-3): the cascade → 0x85 unlocks
/// doors 45/46 (Door_Slide_11A/B, ch_lock 0x85 — captured BEFORE the unlock, which clears
/// ch_lock to 0x0) and the waiting guards (the group-9/13 clones) walk the route through the
/// pair. The game allows at most one OpenDoor enter+exit per door of the pair, then
/// combat-move: `NDrone2_ReachedDestNode` 0x153358:151-155 — a re-arrival at a door node whose
/// door already stands in +0xBA8 returns immediately (`if (iVar11 == *(int *)(iVar2 + 0xba8))
/// return 1;`), no re-bind, no 0x81 re-entry, no +0x5A6 clobber; +0xBA8 stands until the tail
/// clears it on the door-open arrival [M 0x153358:170-176]; `DSTATE_OpenDoor` 0x1681f8 case 3
/// exits on the live DoorIsOpen. The unfixed port re-bound and re-entered 0x81 on every
/// arrival — 96 OpenDoor ticks in a 1200-tick window [L the level-14-cascade-servers-2 run].
#[test]
fn no_guard_spams_the_opendoor_after_the_cascade() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    // The doors and the guard templates, before anything moves: the unlock clears ch_lock
    // [L the level-14-cascade-servers-2 session], so the indices are hoisted out of the wait.
    let (doors, clones) = {
        let sim = scene.sim.as_ref().unwrap();
        let doors: Vec<usize> = sim.drones.doors.list.iter().enumerate()
            .filter(|(_, d)| d.ch_lock == 0x85).map(|(i, _)| i).collect();
        let clones: Vec<usize> = sim.drones.list.iter().enumerate()
            .filter(|(_, x)| x.group == 9 || x.group == 13).map(|(i, _)| i).collect();
        assert!(doors.len() == 2, "the ch_lock-0x85 door pair exists [D Door_Slide_11A/B], got {doors:?}");
        assert!(!clones.is_empty(), "the door-waiting guard templates exist (groups 9/13) [L the probe14 census]");
        (doors, clones)
    };
    // Bond to the hall west end; the guards' real wait at the locked door (the probe's phase 1).
    scene.sim.as_mut().unwrap().drones.channels.set(0x87, 1, 1);
    scene.sim.as_mut().unwrap().drones.channels.set(0x88, 1, 1);
    scene.sim.as_mut().unwrap().bond.pos = [-87.5, -1.7, 33.4];
    for _ in 0..2700 {
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    // The REAL E press on the console plate [M Player_Activate 0x1a5170's class-0x1D arm →
    // Trigger_Activate 0x1b2318 mask 0x4] — the same shape as the cascade-swap test above.
    let t_centre = {
        let sim = scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter()
            .find(|t| t.out == 0xa0 && t.mask & 4 != 0 && t.kind == nf_game::triggers::kind::TOUCH)
            .expect("the 0xA0 E-use touch exists [D placement 399]");
        [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
    };
    scene.sim.as_mut().unwrap().bond.pos = [-60.3, 0.9, 38.4];
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
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, true);
        if scene.sim.as_ref().unwrap().drones.channels.on(0xa0) { fired = true; break; }
    }
    assert!(fired, "the E press fires the mask-4 touch → 0xA0 [M 0x1b2318:23-33]");
    let mut ch85 = false;
    for _ in 0..1500u32 {
        if scene.sim.as_ref().unwrap().drones.channels.on(0x85) { ch85 = true; }
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        if ch85 { break; }
    }
    assert!(ch85, "the cascade reaches 0x85 [D the kill→death pairs]");
    // The unlock opens the pair; the guards walk the route. 1500 ticks (the probe's measured
    // crossings land at 60/1055/1468 after E [L probe14_guards]): count every tick a guard
    // stands in OpenDoor (0x81) — split by the door being worked — and every guard that
    // combat-moves west of the hall's mid.
    let mut opendoor_ticks = 0u32;
    let mut pair_ticks = 0u32;
    let mut open_door_ticks = 0u32;
    let mut crossed = 0usize;
    let mut worked: std::collections::HashMap<usize, u32> = std::collections::HashMap::new();
    for _ in 0..1500u32 {
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for &c in &clones {
            let g = &sim.drones.list[c];
            if g.present && g.state == 0x81 {
                opendoor_ticks += 1;
                if let Some(i) = g.hit_door {
                    if sim.drones.doors.list[i].state == 4 { open_door_ticks += 1; }
                    if doors.contains(&i) { pair_ticks += 1; }
                    *worked.entry(i).or_insert(0u32) += 1;
                }
            }
            if g.present && g.pos[0] < -52.4 { crossed += 1; }
        }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        for &d in &doors {
            assert!(sim.drones.doors.list[d].pos01 > 0.99,
                "door #{d} stands open after the unlock [L the -2 run's pos01 1.0]");
        }
    }
    assert!(crossed >= 1, "at least one guard combat-moves west past x −52.4 [L probe14_guards' crossings 60/1055/1468]");
    // The game's OpenDoor never WORKS an open door: the ENTER checks DoorIsOpen first and
    // returns to +0x5A6 at once [M 0x1681f8 case 1 → LAB_168400; DoorIsOpen = bound+0xf4 == 4,
    // 0x14aaa0], and the UPDATE exits on it [M 0x1681f8 case 3] — a guard standing in 0x81 at
    // an OPEN (state 4) door is the port's spam signature (108 ticks of it before the fix [L
    // this session's runs]). The residual at-open ticks are the exit boundary of a legit open:
    // the door flips to OPEN (state 4) one tick before the case-3 UPDATE's DoorIsOpen read
    // fires the exit [M 0x1681f8 case 3] — a couple of boundary ticks per legit open, never a
    // cycle (the unfixed tree logged 108 [L this session's reproduce]).
    // (2026-11-03 level-14-cascade-servers-4 re-read: the -3 tree's ENTER ran before the port's
    // door-node resolution could land, so its DoorIsOpen read was the stale pre-bind latch —
    // true — and EVERY fresh binding exited instantly, legit re-opens of re-closed doors
    // included. The fix defers the ENTER to the resolution (the game's order: ReachedDestNode
    // binds +0xBA8, THEN the ENTER reads it [M 0x153358:149-163 → 0x1681f8 case 1]), so a fresh
    // bind of a not-yet-open door WORKS it once [M 0x1681f8 case 1's DoorIsOpen-false branch]:
    // one ~31-tick working cycle per re-closed route door (doors 3/23/39, one each [L this
    // run]), the pair's 45 worked once during its own opening phase (12 ticks, the door hit
    // state 4 mid-cycle [L this run]).)
    let mut wv: Vec<_> = worked.iter().collect(); wv.sort(); eprintln!("[014] OpenDoor ticks {opendoor_ticks} (pair {pair_ticks}, at-open-door {open_door_ticks}) in 1500, west {crossed}, per-door {:?}", wv);
    assert!(open_door_ticks <= 4,
        "OpenDoor at an open door must be the exit boundary of a legit open only [M 0x1681f8 case 1 + case 3]: {open_door_ticks} ticks");
    // No door is CYCLED: one fresh bind of a not-yet-open door costs at most one working cycle
    // (the anim + the door reaching 4, ~31 ticks [L this run's 31/31/31]) + the boundary — a
    // second cycle at the same door is the spam the no-op key kills [M 0x153358:151-155].
    for (&di, &ticks) in &worked {
        assert!(ticks <= 40,
            "door #{di} was worked {ticks} ticks in the 1500-tick window — more than one working cycle [M 0x153358:151-155 + 0x1681f8 case 1]");
    }
    // The cascade pair (45/46) likewise: re-binds after the >2.0u heal [M 0x1490f0:233-244]
    // re-enter 0x81, but a still-opening door costs only the one partial cycle (12 [L this
    // run]), never a loop.
    assert!(pair_ticks <= 16,
        "the cascade door pair (45/46) must not be re-worked in a cycle [M 0x153358:151-155]: {pair_ticks} OpenDoor ticks at the pair in the 1500-tick window");
    // The total stays at the one-open-per-door scale: the window's fresh re-binds (the heal's
    // clears) plus the legit opens — the observed shape is one cycle per worked door (105 [L
    // this run]); the unfixed tree's 108-tick single-door spam stands far above it [L].
    assert!(opendoor_ticks <= 128,
        "the guards' total OpenDoor occupancy must stay at the one-open-per-door scale: {opendoor_ticks} ticks in the 1500-tick window");
}
