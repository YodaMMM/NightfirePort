//! 992-0c-invisible-soldiers: the 0700000c soldiers that are active (they move and shoot) but are
//! not drawn. Two parts:
//!
//! 1. `disc_and_load_census` — the [D] disc census of every drone placement and spawner on the
//!    main map, joined against the port's loaded sim + scene: per drone, its skin hash, whether
//!    the skin resolves (`AnimObjectNew` 0x1be910's substitution + `hashtable_getitem` 0x1ca5b0 —
//!    prior art G-021, docs/systems/drone-skins.md), and whether the scene holds a character for
//!    it (`Sim::char_drone`, the draw list's drone→character link) while it is present.
//! 2. `yard_walk_draws_every_present_drone` — the real-input walk to the playtest's yard-ladder
//!    spot (`--bond 11.64,1.04,-184.13,-2.578,0`): teleport to the documented stand-in east of
//!    the secret door (walkreal_0700000c's route), then REAL legs west to the ladder base. Every
//!    tick asserts: every present, enabled, alive drone has a linked character and that
//!    character is not hidden (`Scene::char_hidden` — the same gate the render walks).
//!
//! Opt-in via NF_ISO.
use nf_game::player::Input;
use nf_game::world::V3;

#[path = "common/walker.rs"]
mod walker;
use walker::{dxz, Walker, Wp};

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

fn view_opts() -> nf_view::scene::ViewOpts {
    nf_view::scene::ViewOpts { reinforcements: true, ..Default::default() }
}

/// One drone's census row: the disc placement joined with the loaded sim + scene.
struct Row {
    placement: usize,
    pos: [f32; 3],
    /// The raw param-0 skin hash [D] (0 when the placement carries none).
    raw_skin: u32,
    /// The sim drone's resolved skin (`captain_setup`'s swap included) — 0xffffffff-ish when none.
    sim_skin: u32,
    /// Spawner group (placement param 4), 0 = none.
    group: u32,
    present: bool,
    enabled: bool,
    /// scene.characters indices linked to this drone via `Sim::char_drone`.
    chars: Vec<usize>,
    /// `Scene::char_hidden` verdicts for those characters (the render's draw gate).
    hidden: Vec<bool>,
    /// The role the scene build gave the characters (Placed / SpawnedAtStart / Reinforcement).
    roles: Vec<String>,
}

fn load_rows(iso: &str) -> (Walker, Vec<Row>, Vec<(usize, [f32; 3], Vec<(u32, u32)>)>) {
    let scene = nf_view::load(iso, "0700000c.bin", false).expect("load 0700000c");
    // The [D] disc table: the main map's 0x0F drone and 0xF1 spawner placements.
    let mut disc_drones: Vec<(usize, [f32; 3], u32, u32)> = Vec::new();
    let mut disc_spawners: Vec<(usize, [f32; 3], Vec<(u32, u32)>)> = Vec::new();
    {
        let level = nf_formats::level::Level::parse(
            &{
                let img = nf_formats::iso9660::IsoImage::open(iso).unwrap();
                let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
                let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).unwrap();
                img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap()
            }).unwrap();
        let main = level.main_map().unwrap();
        for (i, p) in level.maps[main].placements.iter().enumerate() {
            let kind = p.flags & 0xff;
            if kind != 0x0f && kind != 0xf1 { continue; }
            let skin = p.params.iter().find(|(k, _)| *k == 0).map(|&(_, v)| v).unwrap_or(0);
            let group = p.params.iter().find(|(k, _)| *k == 4).map(|&(_, v)| v).unwrap_or(0);
            if kind == 0x0f { disc_drones.push((i, p.translation, skin, group)); }
            else { disc_spawners.push((i, p.translation, p.params.clone())); }
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    // The unlinked characters: char_drone[ci] == None — where did the build put them?
    let unlinked: Vec<(usize, u32, [f32; 3], String)> = sim.char_drone.iter().enumerate()
        .filter(|(_, l)| l.is_none())
        .filter_map(|(ci, _)| scene.characters.get(ci).map(|c| (ci, c.skin.hashcode,
            [c.matrix[3][0], c.matrix[3][1], c.matrix[3][2]], format!("{:?}", c.role))))
        .collect();
    eprintln!("=== unlinked characters: {} ===", unlinked.len());
    for (ci, sk, t, role) in &unlinked {
        eprintln!("char {ci} skin {sk:08x} at ({:.2},{:.2},{:.2}) role {role}", t[0], t[1], t[2]);
    }
    // The probe: why do the group-1 drones (#261/#263/#310) have no characters?
    {
        let img = nf_formats::iso9660::IsoImage::open(iso).unwrap();
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = nf_formats::level::Level::parse(&data).unwrap();
        let elf = img.read_file(img.find("/ACTION.ELF").unwrap()).unwrap();
        let props = nf_formats::drone::PropertyTables::from_action_elf(&elf).unwrap();
        let (chars_i, missing) = level.character_instances();
        eprintln!("probe: character_instances {} instances, {} missing-skin placements", chars_i.len(), missing);
        for pi in [261usize, 263, 310, 254, 461, 462, 463] {
            let p = &level.maps[level.main_map().unwrap()].placements[pi];
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
            let raw = param(0);
            let swap = nf_game::drone::captain_skin(raw, &param, &props, 0x0700_000c, 1);
            let kind = nf_formats::drone::drone_setup(raw, param(9), 0x0700_000c).kind;
            let swapped_in_skins = swap.map(|s| level.skins.contains_key(&s));
            let mains = swap.and_then(|s| level.skins.get(&s)).map(|sk| sk.main.iter().flatten().count());
            let raw_mains = level.skins.get(&raw).map(|sk| sk.main.iter().flatten().count());
            eprintln!("probe placement #{pi}: t {:?} raw {raw:08x} kind {kind} captain_skin {swap:08x?} swapped-in-skins {swapped_in_skins:?} swapped-mains {mains:?} raw-mains {raw_mains:?} params {:?}",
                p.translation, p.params);
            let bits = p.translation.map(f32::to_bits);
            let n = chars_i.iter().filter(|(_, i)| i.placement.translation.map(f32::to_bits) == bits).count();
            eprintln!("probe placement #{pi}: character_instances at its translation: {n}");
        }
    }
    // Reverse the char_drone link: drone index → its characters.
    let mut by_drone: std::collections::HashMap<usize, Vec<usize>> = Default::default();
    for (ci, link) in sim.char_drone.iter().enumerate() {
        if let Some(di) = link { by_drone.entry(*di).or_default().push(ci); }
    }
    let opts = view_opts();
    eprintln!("=== sim.drones.list: {} drones ===", sim.drones.list.len());
    for (di, d) in sim.drones.list.iter().enumerate() {
        eprintln!("sim drone {di}: placement {} t ({:.2},{:.2},{:.2}) skin {:08x} grp {} present {} enabled {} chars {:?} role {:?}",
            d.placement, d.pos[0], d.pos[1], d.pos[2], d.skin, d.group, d.present, d.enabled,
            by_drone.get(&di).cloned().unwrap_or_default(),
            by_drone.get(&di).and_then(|cs| cs.first().map(|&ci| format!("{:?}", scene.characters[ci].role))));
    }
    let rows = disc_drones.iter().map(|&(pi, t, raw_skin, group)| {
        let di = sim.drones.list.iter().position(|d| d.placement == pi);
        match di {
            Some(di) => {
                let d = &sim.drones.list[di];
                let chars = by_drone.get(&di).cloned().unwrap_or_default();
                let hidden = chars.iter().map(|&ci| scene.char_hidden(ci, opts)).collect();
                let roles = chars.iter().map(|&ci| format!("{:?}", scene.characters[ci].role)).collect();
                Row { placement: pi, pos: d.pos, raw_skin, sim_skin: d.skin, group, present: d.present, enabled: d.enabled, chars, hidden, roles }
            }
            None => Row { placement: pi, pos: t, raw_skin, sim_skin: 0, group, present: false, enabled: false, chars: vec![], hidden: vec![], roles: vec![] },
        }
    }).collect();
    (Walker::new(scene), rows, disc_spawners)
}

/// The disc + load census (numbers and ids only — run with --nocapture for the tables).
#[test]
fn disc_and_load_census() {
    let iso = iso();
    let (_w, rows, spawners) = load_rows(&iso);
    eprintln!("=== [D] 0700000c main map: {} drone placements, {} spawners ===", rows.len(), spawners.len());
    for &(i, t, ref p) in &spawners {
        eprintln!("spawner #{i} at ({:.2},{:.2},{:.2}) params {:?}", t[0], t[1], t[2], p);
    }
    for r in &rows {
        eprintln!("drone #{:<4} t ({:.2},{:.2},{:.2}) raw {:08x} sim {:08x} grp {} present {} enabled {} chars {:?} hidden {:?} roles {:?}",
            r.placement, r.pos[0], r.pos[1], r.pos[2], r.raw_skin, r.sim_skin, r.group, r.present, r.enabled, r.chars, r.hidden, r.roles);
    }
    // Every drone placement creates a sim drone (the load difficulty admits them all on 0c —
    // the guards-0c census counted 40).
    assert_eq!(rows.len(), 40, "the disc's 40 drone placements [D census, docs/levels/0700000c.md §4]");
    // Every drone's skin resolves: the sim's skins table has it with a skeleton, and the scene
    // built at least one character with that hash (AnimObjectNew 0x1be910:44-46's get + the
    // Control_CreateObjEx loop :154-172 — a miss renders skinless, G-021).
    for r in &rows {
        let scene_chars = _w.scene.characters.iter().filter(|c| c.skin.hashcode == r.sim_skin).count();
        assert!(scene_chars > 0, "drone #{} skin {:08x} built no character — the skinless render (G-021)", r.placement, r.sim_skin);
        assert!(!r.chars.is_empty() || !r.present,
            "drone #{} present at load with NO linked character — the draw list never holds it", r.placement);
    }
    // At load (Bond on the start walkway) the present drones' characters are not hidden.
    for r in &rows {
        if !(r.present && r.enabled) { continue; }
        for (k, &h) in r.hidden.iter().enumerate() {
            assert!(!h, "drone #{} char {:?} hidden at load (present+enabled)", r.placement, r.chars[k]);
        }
    }
}

/// The real-path walk to the yard-ladder spot, asserting on every tick that every present,
/// enabled, alive drone is drawn (its character linked and not hidden).
#[test]
fn yard_walk_draws_every_present_drone() {
    let iso = iso();
    let (mut w, _rows, _sp) = load_rows(&iso);
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    // The documented reach stand-in (walkreal_0700000c: the port's unported 0xE2 winches cross
    // the band gap; the yard east stand-in is the secret-door exit). From here the legs are the
    // REAL walk (no bond.pos writes) — the interact tests' pattern.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [56.0, 1.5, -183.5];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..200 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.tick(Input::default(), false);
    }
    let opts = view_opts();
    let mut violations: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<usize> = Default::default();
    let mut check = |w: &mut Walker, violations: &mut Vec<String>, seen: &mut std::collections::HashSet<usize>| {
        let scene = &w.scene;
        let sim = scene.sim.as_ref().unwrap();
        for (di, d) in sim.drones.list.iter().enumerate() {
            let active = d.present && d.enabled && !d.corpse && d.health > 0.0;
            if !active { continue; }
            seen.insert(di);
            let link = sim.char_drone.iter().position(|l| *l == Some(di));
            let Some(ci) = link else {
                violations.push(format!("t{} drone #{di} skin {:08x} ACTIVE at ({:.2},{:.2},{:.2}) state {:#x} — NO character linked (draw list never holds it)",
                    w.total_ticks, d.skin, d.pos[0], d.pos[1], d.pos[2], d.state));
                continue;
            };
            if scene.char_hidden(ci, opts) {
                violations.push(format!("t{} drone #{di} skin {:08x} ACTIVE at ({:.2},{:.2},{:.2}) state {:#x} — character {ci} HIDDEN",
                    w.total_ticks, d.skin, d.pos[0], d.pos[1], d.pos[2], d.state));
            }
        }
    };
    // The yard legs, real inputs (walkreal_0700000c's route): south → west → the ladder base.
    let legs: [(V3, &str); 4] = [
        ([56.0, 1.5, -183.5], "the yard south"),
        ([40.0, 1.5, -184.5], "the yard west"),
        ([20.0, 1.4, -184.5], "the yard west 2"),
        ([11.6, 1.2, -186.6], "the ladder base"),
    ];
    for (at, label) in legs {
        w.walk_to(Wp::new(at[0], at[1], at[2]).r(1.5), label);
        check(&mut w, &mut violations, &mut seen);
    }
    // Hold at the yard-ladder spot (the playtest spot --bond 11.64,1.04,-184.13,-2.578,0) for
    // 10 s of ticks: the spawners' releases and the patrols all run.
    for _ in 0..600 { w.tick(Input::default(), false); check(&mut w, &mut violations, &mut seen); }
    eprintln!("=== yard walk: {} distinct drones went active; {} violations ===", seen.len(), violations.len());
    for &di in seen.iter() {
        let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
        eprintln!("active drone {di}: placement {} skin {:08x} grp {} at ({:.2},{:.2},{:.2})",
            d.placement, d.skin, d.group, d.pos[0], d.pos[1], d.pos[2]);
    }
    for v in violations.iter().take(40) { eprintln!("{v}"); }
    assert!(violations.is_empty(), "{} active-drone draw violations (see stderr)", violations.len());
    // The three placements that shipped bit-identical twins (#263/#461, #261/#462, #310/#463
    // [D]) — the invisible soldiers: each pair's group-1 member is released by spawner #264
    // (immediate) and patrols from tick 0, so the walk must have seen them active AND drawn.
    let trio: Vec<usize> = seen.iter().copied().filter(|&di| {
        let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
        matches!(d.placement, 261 | 263 | 310)
    }).collect();
    assert_eq!(trio.len(), 3, "the three group-1 wave guards (placements 261/263/310 [D]) must all go active on the walk — seen {:?}", trio);
    // Plus the yard's own guards (the patrol circle and the yard spawners' releases).
    assert!(seen.len() >= 5, "the yard walk must pass active guards ({} seen)", seen.len());
}
