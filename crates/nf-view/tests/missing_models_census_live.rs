//! missing-models-census: for every on-foot story level, load through `nf_view::load` and count
//! every model lane's misses (task missing-models-census).
//!
//! The census columns, all through the real load path (the game's own loader order):
//! - `unresolved` — placements whose by-hash/index target does not resolve to an object
//!   [M parsemap 0x1d1998/0x1d04f0 resolve placements through the same two arms the parser walks];
//! - `fallback` — scene materials whose texture key does not resolve (the GPU draws the 1×1 white
//!   fallback; `NF_TEX_TRACE=1` prints them at load) — the same condition models_sweep pins;
//! - `no_anim` — the level's 07F animation archive missing from FILES.BIN (the loader's
//!   `read(&anim_name)` silently skips it, leaving every sequence table empty);
//! - `skinless_script` — script-object `Cmd::Anim` skins the loaded skin table does not hold
//!   (Scene::build silently skips the character — the script entity renders nothing);
//! - `skinless_drones` — sim drones whose skin/skeleton/character does not resolve
//!   (the drone-skin census's resolved half, counted per level here);
//! - `entity_miss` — script `Cmd::Entity` hashes not in the level's by-hash table (the spawned
//!   script entity would have no mesh);
//! - `pickup_miss` — 0xf0 pickup placements whose resolved object's hashcode the pickup-model
//!   table lacks, or whose object does not resolve: Pickup creation returns None and the pickup
//!   is silently dropped [M 0x1d04f0 case 0xf0 → Pickup_Create].
//!
//! Zero is the goal for every column; a non-zero pinned number needs the game's own evidence
//! (e.g. 07000014's 0x050000b1 skinless drones are the shipped game's misses too — G-021).
//! Numbers/ids only; ISO-gated. Re-landed by missing-models-census-2: the CREATED list corrected to
//! Script_CreateEntity 0x1d8968's dispatch (raw 0x3e has no arm), the 07000001 skin pin re-analysed
//! against master's g-021 merge (docs/systems/missing-models-census.md).

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The on-foot story levels: `mission::LEVELS` (the game's level table [M]) filtered to the
/// archives FILES.BIN carries (0700000e/0f/10 have no archive — the game never loads them).
const ON_FOOT: [u32; 21] = [
    0x0700_0001, 0x0700_0002, 0x0700_0003, 0x0700_0004, 0x0700_0005, 0x0700_0006, 0x0700_0007,
    0x0700_0008, 0x0700_0009, 0x0700_000a, 0x0700_000b, 0x0700_000c, 0x0700_000d, 0x0700_0011,
    0x0700_0012, 0x0700_0013, 0x0700_0014, 0x0700_0015, 0x0700_0016, 0x0700_001b, 0x0700_004a,
];

struct Row {
    name: String,
    unresolved: usize,
    fallback: usize,
    no_anim: usize,
    skinless_script: usize,
    skinless_drones: usize,
    entity_miss: usize,
    pickup_miss: usize,
    pickups: usize,
    /// The missing (hash, obj_type) pairs behind entity_miss and the missing skins behind
    /// skinless_script (diagnostics; the pin is the counts).
    entity_miss_hashes: Vec<(u32, u8)>,
    anim_miss_hashes: Vec<u32>,
}

fn census(iso_path: &str, name: &str) -> Row {
    let img = IsoImage::open(iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("{name}: not in FILES.BIN"));
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read level");
    let level = Level::parse(&data).unwrap_or_else(|e| panic!("{name}: parse failed: {e}"));

    // -- unresolved placements (parse level, main map: the placement walk the game does) --
    let mut unresolved = 0usize;
    let main = level.main_map();
    if let Some(m) = main {
        for p in &level.maps[m].placements {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
            if target.is_none() { unresolved += 1; }
        }
    }

    // -- the 07F animation archive (the loader skips its absence silently) --
    let anim_name = format!("{}F{}", &name[..2], &name[3..]);
    let no_anim = usize::from(!entries.iter().any(|e| e.name.eq_ignore_ascii_case(&anim_name)));

    // -- script streams: skinless script objects and unresolvable entity spawns --
    // `Script_CreateEntity` 0x1d8968's dispatch creates only these types (0x22 aliases to the
    // 0x2d Ladder_Create arm); a type with no arm — e.g. the ty-0x31 mesh mentions on 01/04/08/0a/
    // 0c/14 — creates nothing in the game, so a hash it cannot resolve is not a model miss.
    // Raw types with an arm in the dispatch: 0x22 jumps to the 0x2d Ladder_Create arm, and 0xf5
    // creates as type 0x3e — but a RAW 0x3e mention has NO arm (it is only the target of 0xf5), so
    // unlike the first census pass (which listed 0x3e) a raw 0x3e mention must not count [M 0x1d8968].
    const CREATED: &[u8] = &[0x20, 0x22, 0x29, 0xe1, 0xea, 0xeb, 0xf0, 0xf2, 0xf5];
    let mut skinless_script = 0usize;
    let mut entity_miss = 0usize;
    let mut entity_miss_hashes: Vec<(u32, u8)> = Vec::new();
    let mut anim_miss_hashes: Vec<u32> = Vec::new();
    let mut script_hashes: Vec<u32> = level.nis.keys().copied().collect();
    script_hashes.sort_unstable();
    let mut queued: std::collections::HashSet<u32> = script_hashes.iter().copied().collect();
    while let Some(h) = script_hashes.pop() {
        let Some(nis) = level.nis.get(&h) else { continue };
        for st in &nis.streams {
            for cmd in st.decode() {
                match cmd {
                    nf_formats::nis::Cmd::Anim { skin: sh, .. } => {
                        if sh >> 24 == 5 && sh != 0 && !level.skins.contains_key(&sh) {
                            skinless_script += 1;
                            anim_miss_hashes.push(sh);
                        }
                    }
                    nf_formats::nis::Cmd::Entity { hash: eh, obj_type: ty, .. } => {
                        if CREATED.contains(&ty) && eh != 0 && !level.by_hash.contains_key(&eh) {
                            entity_miss += 1;
                            entity_miss_hashes.push((eh, ty));
                        }
                    }
                    nf_formats::nis::Cmd::SubScript { hash: sub, .. } => {
                        if sub != 0 && queued.insert(sub) { script_hashes.push(sub); }
                    }
                    _ => {}
                }
            }
        }
    }

    // -- the real load --
    let scene = nf_view::load(iso_path, name, false).unwrap_or_else(|e| panic!("{name}: load failed: {e}"));

    // The load-time session merge (g-021, lib.rs `wanted` + `session_container`) covers drone
    // placements (0x0f param 0) and 0xd9 ScriptPlayer placements' DIRECT `Cmd::Anim` skins. A script
    // skin no 0xd9 placement reaches is NOT merged — 07000001's 0x05000037 ask sits in script
    // 0x0600005b stream 7 [D probe: 01's 0xd9 scripts are 0x06000016/86/87; 0x0600005b is not
    // reachable from them], so the sim still lacks it post-load. Diagnostic (not a pin: the lane is
    // an OPEN gap — closing it is a loader change needing render checks).
    if let Some(sim) = scene.sim.as_ref() {
        let unmerged: Vec<String> = anim_miss_hashes.iter().filter(|sh| !sim.skins.contains_key(sh)).map(|sh| format!("{sh:#010x}")).collect();
        if !unmerged.is_empty() { eprintln!("census {name}: raw-missing script skins NOT merged at load: {}", unmerged.join(" ")); }
    }

    // -- materials with no texture (the white fallback condition) --
    let tex_keys: std::collections::HashSet<_> = scene.textures.iter().map(|t| t.key).collect();
    let fallback = scene.materials.iter().filter(|m| m.key.map_or(true, |k| !tex_keys.contains(&k))).count();

    // -- drones / pickups through the sim --
    let mut skinless_drones = 0usize;
    let mut pickup_miss = 0usize;
    let mut pickups = 0usize;
    if let Some(sim) = scene.sim.as_ref() {
        for dr in &sim.drones.list {
            let ok = sim.skins.get(&dr.skin).map(|sk| sim.skeletons.get(&sk.skeleton).map(|s| s.bones > 0).unwrap_or(false)).unwrap_or(false)
                && scene.characters.iter().any(|c| c.skin.hashcode == dr.skin);
            if !ok { skinless_drones += 1; }
        }
        pickups = sim.drones.pickups.len();
    }
    // The pickups that failed to create: count the 0xf0 placements whose object resolves and whose
    // hashcode the pickup-model table holds (a create failure = the drop lane).
    if let Some(m) = main {
        let models = nf_game::pickup::models(&level);
        for p in &level.maps[m].placements {
            if p.flags & 0xff != 0xf0 { continue; }
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { pickup_miss += 1; continue };
            let hc = level.maps[om].objects[oi].hashcode;
            if !models.contains_key(&hc) { pickup_miss += 1; }
        }
    }

    Row { name: name.to_string(), unresolved, fallback, no_anim, skinless_script,
        skinless_drones, entity_miss, pickup_miss, pickups,
        entity_miss_hashes, anim_miss_hashes }
}

/// The census walk. `--nocapture` prints the per-level table; the pin below holds the numbers.
/// The mission-order session model (`mission::LEVELS` — the game keeps every earlier archive's
/// skins in its hashtable, nothing frees them [M G-021's table walk]) backs the anim-miss
/// cross-reference: a skin an EARLIER mission archive carries is one the game resolves and the
/// port must merge; a skin on NO earlier archive is the game's own miss.
#[test]
fn on_foot_levels_census() {
    let Some(iso_path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut rows = Vec::new();
    for id in ON_FOOT {
        let name = format!("{id:08x}.bin");
        rows.push(census(&iso_path, &name));
    }
    // The disc scan behind the anim-miss classification: which ON_FOOT archives carry the census's
    // missing skins (0x05000037 today) — the game's session table is the union of every earlier
    // mission archive's skins [M G-021's table walk], so a carrier earlier than the level means
    // the game resolves it (a port merge gap); no carrier means the game misses it too.
    if rows.iter().any(|r| !r.anim_miss_hashes.is_empty()) {
        let mut carriers = String::new();
        for id in ON_FOOT {
            let name = format!("{id:08x}.bin");
            let img = IsoImage::open(&iso_path).expect("open iso");
            let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
            let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)).expect("level");
            let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
            let level = Level::parse(&data).expect("parse");
            if level.skins.contains_key(&0x0500_0037) { carriers.push_str(&format!(" {name}")); }
        }
        eprintln!("0x05000037 carriers:{}", if carriers.is_empty() { " (none)" } else { &carriers });
    }
    eprintln!("{:<14} {:>10} {:>8} {:>7} {:>14} {:>15} {:>11} {:>11} {:>7}",
        "level", "unresolved", "fallback", "no_anim", "skinless_script", "skinless_drones", "entity_miss", "pickup_miss", "pickups");
    for r in &rows {
        eprintln!("{:<14} {:>10} {:>8} {:>7} {:>14} {:>15} {:>11} {:>11} {:>7}",
            r.name, r.unresolved, r.fallback, r.no_anim, r.skinless_script, r.skinless_drones, r.entity_miss, r.pickup_miss, r.pickups);
        if !r.entity_miss_hashes.is_empty() {
            eprintln!("    entity_miss: {}", r.entity_miss_hashes.iter().map(|(h, t)| format!("{h:#010x}/ty{t}")).collect::<Vec<_>>().join(", "));
        }
        if !r.anim_miss_hashes.is_empty() {
            eprintln!("    anim_miss: {}", r.anim_miss_hashes.iter().map(|h| format!("{h:#010x}")).collect::<Vec<_>>().join(", "));
        }
    }
    for r in &rows {
        assert_eq!(r.unresolved, 0, "{}: every placement must resolve [M parsemap 0x1d1998/0x1d04f0]", r.name);
        assert_eq!(r.no_anim, 0, "{}: the 07F animation archive must exist (the loader skips its absence silently)", r.name);
        // The white-fallback pin (the U-017 fix): every material resolves its texture. The pre-fix
        // tree had 70 on 0700000d (6 short textures dropped out of a session-merged map's table).
        assert_eq!(r.fallback, 0, "{}: no material may draw the 1×1 white fallback", r.name);
        // Script entities the game creates resolve their meshes (the census counts only
        // Script_CreateEntity 0x1d8968's created types).
        assert_eq!(r.entity_miss, 0, "{}: every created script entity resolves its mesh", r.name);
        assert_eq!(r.pickup_miss, 0, "{}: every 0xf0 placement resolves its pickup model", r.name);
        match r.name.as_str() {
            // G-021: the game itself renders these skinless — 0x05000090/0x050000b1 ride archives
            // the fresh session has not loaded when 14 boots; the census counts the sim's drones.
            "07000014.bin" => assert_eq!(r.skinless_drones, 5, "07000014: the 0x050000b1 non-captain drones are the shipped game's own skinless renders [D drone-skins-g021]"),
            _ => assert_eq!(r.skinless_drones, 0, "{}: every drone resolves its skin", r.name),
        }
        // The RAW-archive pin (disc content): 07000001's script Anim (0x0600005b stream 7) asks for
        // skin 0x05000037, which its own archive lacks and 07000005/06 carry [D census carrier scan].
        // The game's skin table holds every earlier mission archive's skins (nothing frees them
        // [M 0x1ca5b0/0x1ca520, G-021]), so the game resolves it. The port's session-skins merge
        // (lib.rs g-021) covers drone placements and 0xd9 placements' direct Anim skins only — this
        // lane stays open (see the census() diagnostic), still the one known remaining gap.
        if r.name == "07000001.bin" {
            assert_eq!(r.skinless_script, 1, "07000001: the raw archive lacks exactly one script skin (0x05000037, script 0x0600005b stream 7) — the game resolves it from the 05/06 session skins; the port merge lane for it is still open");
        } else {
            assert_eq!(r.skinless_script, 0, "{}: no script object may request a skin the tables lack", r.name);
        }
    }
    // Sisters (each can fail): the fallback pin above re-breaks if the parser drops a short
    // texture again (the pre-fix tree's exact symptom); nf-formats'
    // `short_texture_stays_at_its_index_with_the_overread_bytes` asserts the kept-texture
    // behaviour itself and fails if the drop returns.
}

/// Diagnostic: the fallback materials' texture keys on one level (NF_CENSUS_LEVEL=0700000d).
/// Prints which keys fail and which map-texture hashcodes the level holds; the port's fix
/// lives where the game's loader resolves them.
#[test]
#[ignore = "diagnostic; run with --ignored"]
fn fallback_materials_diagnostic() {
    let Some(iso_path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let name = format!("{}.bin", std::env::var("NF_CENSUS_LEVEL").unwrap_or_else(|_| "0700000d".into()));
    let img = IsoImage::open(&iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)).expect("level");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse");
    for (mi, m) in level.maps.iter().enumerate() {
        let mut hs: Vec<u32> = m.textures.iter().map(|t| t.hashcode).collect();
        hs.sort_unstable();
        eprintln!("map {mi} \"{}\" kind {} objects {} textures {} hashcodes {:?}", m.name.trim(), m.kind, m.objects.len(), m.textures.len(), &hs[..hs.len().min(24)]);
    }
    let scene = nf_view::load(&iso_path, &name, false).expect("load");
    let tex_keys: std::collections::HashSet<_> = scene.textures.iter().map(|t| t.key).collect();
    let mut by_key: std::collections::BTreeMap<(usize, u32), usize> = Default::default();
    let mut key_none = 0usize;
    for m in &scene.materials {
        match m.key {
            None => { key_none += 1; }
            Some(k) => { if !tex_keys.contains(&k) { *by_key.entry(k).or_default() += 1; } }
        }
    }
    eprintln!("fallback materials: {} distinct keys, {} no-key materials", by_key.len(), key_none);
    for ((mi, h), n) in &by_key { eprintln!("  key map{mi} {h:#010x}: {n} materials"); }
}
