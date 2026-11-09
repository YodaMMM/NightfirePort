//! The disc census of the 0700000b scenes' created objects (task level-0b-scene-entity-flags):
//! every kind-7 script in `0700000b.bin` and in its loadable packs (the kind-0xC records, the
//! same 078/079/07A rule `script_break_census` uses), every stream, every `Script_CreateEntity`
//! 0x1d8968 command decoded (all obj_types — the typed arms attach the hash's glist through their
//! creators, the fallthrough through `hashtable_set_object_to_entity_gfx` → `Control_SetGList`
//! [M 0x1d8968:99,170-173; 0x1ca6d8:38-40]), resolved through the level's hash table to the
//! object's glist flags word (celglist+0x44).
//!
//! Why the word matters: `Control_SetGList` 0x134960 copies glist+0x44 into obj+0xF0 and, when the
//! word carries 0x40000, clears obj+0xF8's bit 0x10 (the display-mask byte init 0x1F,
//! control_init_object 0x134630:11) [M 0x134960:11-16]. `View_AddObjects` 0x1e62d8:30 skips an
//! object when `(obj+0xF8 & object_display_mask) == 0`, and `View_CaptureScene` 0x1e6df0:15 sets
//! `object_display_mask = 1 << (viewer+0x28 & 0x1f)` per capture pass; `View_AddCels` 0x1e6ba0:57
//! hides a STATIC placement whose flags word carries 0x40000 only when `object_display_mask ==
//! 0x10`. Bit 0x10 is therefore the scene-camera pass: a 0x40000 glist flag hides the object from
//! the scene camera only — the normal game views keep drawing it.
//!
//! The pin: which of the 0b scenes' created objects (the lift cab 0x02000367, its glass
//! 0x02000421, the NIS door 0x02000668, the rest) carry 0x40000 on the disc. The port's scene
//! build (nf-view scene.rs `Cmd::Entity` arm) resolves the hash and draws the mesh with no
//! 0x40000 check at all — the draw change (collapse those slots while a script camera renders) is still open.
//! Opt-in via NF_ISO; without it the test prints "skip".

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level, levelbin, nis};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn the_0b_scene_entity_glist_census() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}"));
    let (files, bin) = disc::files_bin_entries(&iso).unwrap();
    let read = |name: &str| -> Option<Vec<u8>> {
        let f = files.iter().find(|f| f.name.eq_ignore_ascii_case(name))?;
        iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64).ok()
    };

    let data = read("0700000b.bin").expect("0700000b.bin on the disc");
    let lvl = level::Level::parse(&data).expect("0700000b parses as a level");

    // hashcode → (archive tag, glist flags word celglist+0x44, object name), level bin first-wins
    // then the packs (`level.by_hash.entry().or_insert` is the port's own merge law [M lib.rs:1276-1278]).
    let mut words: BTreeMap<u32, (String, u32, String)> = BTreeMap::new();
    for m in &lvl.maps {
        for o in &m.objects {
            words.entry(o.hashcode).or_insert(("0700000b".into(), o.flags, o.name.clone()));
        }
    }
    // The loadable packs the level streams (kind-0xC records, flag 1..=3 → 078/079/07A) [M
    // LoadableLoad 0x1d69e8; the same decode script_break_census.rs uses].
    let lb = levelbin::LevelBin::parse(&data).unwrap();
    let mut pack_names: Vec<String> = Vec::new();
    for e in &lb.entries {
        if e.kind != 0xC { continue; }
        let r = lb.data(&data, e);
        if r.len() < 5 { continue; }
        let flag = r[4];
        if flag == 0 || flag > 3 { continue; }
        let name = format!("{:08x}.bin", (0x0700_000bu32 & 0xff0f_ffff) | ((flag as u32 + 7) << 20));
        if !pack_names.contains(&name) { pack_names.push(name); }
    }
    // Every kind-7 script of the level bin and each present pack, with its hash.
    let mut scripts: BTreeMap<u32, nis::Nis> = lvl.nis.iter().map(|(&h, n)| (h, n.clone())).collect();
    for name in &pack_names {
        let Some(pd) = read(name) else { eprintln!("[census] pack {name} not on the disc"); continue };
        let Ok(plvl) = level::Level::parse(&pd) else { eprintln!("[census] pack {name} does not parse"); continue };
        for m in &plvl.maps {
            for o in &m.objects {
                words.entry(o.hashcode).or_insert((name.clone(), o.flags, o.name.clone()));
            }
        }
        for (&h, n) in &plvl.nis {
            scripts.insert(h, n.clone());
        }
        eprintln!("[census] pack {name}: {} scripts", plvl.nis.len());
    }

    // Walk every script's streams (including sub-scripts, recursively): every Entity command.
    // `seen_scripts` stops sub-script cycles; `seen_ents` dedupes the printout but the
    // creators list keeps one row per (script, stream, hash).
    let mut creators: Vec<(u32, u8, u32, u8, Option<u8>, Vec<u32>)> = Vec::new(); // (script, stream, hash, obj_type, parent, params)
    let mut seen_scripts: BTreeSet<u32> = BTreeSet::new();
    let mut subs_of: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    let mut queue: Vec<u32> = scripts.keys().copied().collect();
    while let Some(h) = queue.pop() {
        if !seen_scripts.insert(h) { continue; }
        let Some(n) = scripts.get(&h) else { continue };
        for (si, st) in n.streams.iter().enumerate() {
            for cmd in st.decode() {
                match cmd {
                    nis::Cmd::Entity { hash, parent, obj_type, params, .. } => {
                        creators.push((h, si as u8, hash, obj_type, parent, params));
                    }
                    nis::Cmd::SubScript { hash, .. } => {
                        subs_of.entry(h).or_default().insert(hash);
                        if !seen_scripts.contains(&hash) { queue.push(hash); }
                    }
                    _ => {}
                }
            }
        }
    }
    for (from, tos) in &subs_of {
        eprintln!("[census] sub-scripts of {from:08x}: {}", tos.iter().map(|t| format!("{t:08x}")).collect::<Vec<_>>().join(" "));
    }
    for h in &seen_scripts {
        let src = if lvl.nis.contains_key(h) { "0700000b" } else { pack_names.iter().find(|p| {
            read(p).and_then(|pd| level::Level::parse(&pd).ok()).is_some_and(|pl| pl.nis.contains_key(h))
        }).map(|s| s.as_str()).unwrap_or("?") };
        eprintln!("[census] script {h:08x} in {src}");
    }

    // The census print: every created hash, its creating scripts, obj_type, and glist word.
    let mut per_hash: BTreeMap<u32, Vec<(u32, u8, u8, Option<u8>)>> = BTreeMap::new();
    for &(sh, si, hash, ot, parent, _) in &creators {
        per_hash.entry(hash).or_default().push((sh, si, ot, parent));
    }
    eprintln!("[census] 0700000b scenes: {} scripts walked, {} Entity commands, {} distinct hashes",
        seen_scripts.len(), creators.len(), per_hash.len());
    let mut flagged: Vec<(u32, u32, String)> = Vec::new();
    for (hash, rows) in &per_hash {
        let Some((_src, flags, name)) = words.get(hash) else {
            eprintln!("[census] hash {hash:#09x} — NOT in any 0b archive hash table ({} creators)", rows.len());
            continue;
        };
        let scripts_of = rows.iter().map(|(sh, si, ot, _)| format!("{sh:08x}/s{si}/t{ot:#x}")).collect::<Vec<_>>().join(" ");
        let scene_hidden = flags & 0x40000 != 0;
        let drawn_by_port = !matches!(rows[0].2, 0xea | 0xeb | 0xf5 | 0xf2);
        eprintln!("[census] hash {hash:#09x} '{name}' glist+0x44 {flags:#x} {} {} [{}]",
            if scene_hidden { "SCENE-HIDDEN(0x40000)" } else { "scene-visible" },
            if drawn_by_port { "port-draws" } else { "port-skips(trigger/emitter)" },
            scripts_of);
        if scene_hidden { flagged.push((*hash, *flags, name.clone())); }
    }
    eprintln!("[census] scene-hidden set ({}): {:?}", flagged.len(),
        flagged.iter().map(|(h, f, n)| format!("{h:#09x} {f:#x} {n}")).collect::<Vec<_>>());

    // The named objects the task calls out, pinned to their disc words.
    let lift = per_hash.get(&0x0200_0367).expect("the lift cab 0x02000367 is a 0b scene creation [D task; walkreal finds it live]");
    let glass = per_hash.get(&0x0200_0421).expect("the lift glass 0x02000421 is a 0b scene creation [D task; walkreal pins it as stream 1's parented entity]");
    let &(lscript, lstream, ltype, _) = lift.iter().next().unwrap();
    let &(gscript, gstream, gtype, _) = glass.iter().next().unwrap();
    let (lflags, gflags) = (words[&0x0200_0367].1, words[&0x0200_0421].1);
    eprintln!("[census] lift cab: script {lscript:08x} stream {lstream} type {ltype:#04x} glist {lflags:#x}");
    eprintln!("[census] lift glass: script {gscript:08x} stream {gstream} type {gtype:#04x} glist {gflags:#x}");
    assert_ne!(lflags & 0x40000, 0, "the lift cab's glist word {lflags:#x} carries the scene-hide bit [D]");
    assert_ne!(gflags & 0x40000, 0, "the lift glass's glist word {gflags:#x} carries the scene-hide bit [D]");
    if let Some(&(dscript, dstream, dtype, _)) = per_hash.get(&0x0200_0668).and_then(|v| v.first()) {
        let dflags = words[&0x0200_0668].1;
        eprintln!("[census] NIS door 0x02000668: script {dscript:08x} stream {dstream} type {dtype:#04x} glist {dflags:#x}");
    } else {
        eprintln!("[census] NIS door 0x02000668: not created by any 0b scene script");
    }

    // The static twins: placements (by hash) of the scene-hidden objects, with their placement
    // flags words — `View_AddCels` 0x1e6ba0:57 hides a STATIC record whose +0x94 word carries
    // 0x40000 from the 0x10 pass too, so a static twin changes what the scene pass should show.
    for (mi, m) in lvl.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            if matches!(p.hashcode, 0x0200_0367 | 0x0200_0421 | 0x0200_0668) {
                eprintln!("[census] static placement map {mi} #{pi} hash {:#x} flags {:#x} at ({:.2},{:.2},{:.2})",
                    p.hashcode, p.flags, p.translation[0], p.translation[1], p.translation[2]);
            }
        }
    }
    // The scene-hidden set is EXACTLY the lift pair — the pin a future draw change must refute.
    let hidden: BTreeSet<u32> = flagged.iter().map(|(h, _, _)| *h).collect();
    let expected: BTreeSet<u32> = [0x0200_0367, 0x0200_0421].into_iter().collect();
    assert_eq!(hidden, expected, "the 0b scenes' 0x40000 glist words are exactly the lift cab and its glass [D disc census]");
}
