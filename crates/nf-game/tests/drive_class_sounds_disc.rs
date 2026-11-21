//! Disc test for `nf_game::driving::class_sounds` and the ambience band (NF_ISO, full image): on
//! every track, which of the boat / snowmobile / turret / tank keys resolve in the bank slot the
//! game reads them from, the `ESetCreakLevel` events (level, interval), and the `Envi` ambience
//! regions (stream name, environment id, band) with the archive each ambience stream is in.
//! The ambience level of the band at every road-node height must lie in 0..1. Golden:
//! `drive_class_sounds.tsv` (one line per track). Skips cleanly without NF_ISO.

use std::collections::BTreeSet;

use nf_formats::crp::Crp;
use nf_formats::easnd::{self, SfxTable};
use nf_formats::iso9660::IsoImage;
use nf_formats::trackstart::map_script;
use nf_game::driving::class_sounds as cs;
use nf_game::driving::road::RoadNet;
use nf_game::driving::stream_events::{ambience_level, stream_archive, stream_entry, StreamTarget};
use nf_game::driving::audio;

const VIVS: [&str; 8] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

fn iso() -> Option<IsoImage> {
    let iso = IsoImage::open(std::env::var_os("NF_ISO")?).ok()?;
    let e = iso.find("/DRIVING/MIS01.VIV")?;
    (iso.read_bytes(e.lba as u64 * 2048, 4).ok()? == b"BIGF").then_some(iso)
}

fn viv_files(iso: &IsoImage, viv: &str) -> Vec<(String, Vec<u8>)> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV")).unwrap();
    let g = nf_formats::disc::read_bigf(iso, e).unwrap();
    g.entries
        .iter()
        .filter(|x| {
            let n = x.name.to_ascii_lowercase();
            n.contains("data\\audio\\") || n.contains("data\\track\\")
        })
        .map(|x| {
            let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).unwrap();
            let b = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
            (x.name.replace('\\', "/").to_ascii_lowercase(), b)
        })
        .collect()
}

fn archive(iso: &IsoImage, path: &str) -> Option<BTreeSet<String>> {
    let p = format!("/{}", path.replace('\\', "/").to_ascii_uppercase());
    let e = iso.find(&p)?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    Some(g.entries.iter().map(|x| x.name.replace('/', "\\").to_ascii_lowercase()).collect())
}

/// (class, slot, key) for every key the class steps look up [M, slots in class_sounds.rs].
fn class_keys() -> Vec<(&'static str, usize, &'static str)> {
    let mut k = Vec::new();
    for s in cs::BOAT_VOICES.iter().chain(&cs::BOAT_RUDDER_STOPS).chain(&cs::BOAT_SERVO_ON).chain(&cs::BOAT_SERVO_OFF) { k.push(("boat", 9, *s)); }
    for s in cs::BOAT_CREAKS { k.push(("boat", 0, s)); }
    for s in cs::SLED_ROAD { k.push(("sled", 0, s)); }
    for s in cs::SLED_ENGINE.iter().chain(&cs::SLED_REVS).chain([&cs::SLED_FREEWHEEL]) { k.push(("sled", 9, *s)); }
    for (_, s) in cs::SLED_LANDINGS { k.push(("sled", 1, s)); }
    k.push(("sled", 4, cs::WIND_VOICE));
    for s in cs::TURRET_LR.iter().chain(&cs::TURRET_UD) { k.push(("turret", 0, *s)); }
    for s in cs::TURRET_SERVO.iter().chain([&cs::TURRET_KACHUNG]) { k.push(("turret", 4, *s)); }
    for s in audio::ENGINE_VOICES.iter().chain([&audio::ENGINE_VACUUM]) { k.push(("tank", 9, *s)); }
    k.push(("tank", 0, "SFX_TankTread"));
    k
}

#[test]
fn class_sound_keys_creaks_and_ambience_bands_against_the_disc() {
    let Some(iso) = iso() else {
        eprintln!("skip: NF_ISO with /DRIVING/ data not available");
        return;
    };
    let keys = class_keys();
    let mut report = String::new();
    let mut nodes_checked = 0usize;
    for viv in VIVS {
        let files = viv_files(&iso, viv);
        let get = |n: &str| files.iter().find(|f| f.0 == n).map(|f| f.1.as_slice());
        let track = files.iter().find_map(|f| f.0.strip_prefix("data/track/").and_then(|s| s.strip_suffix(".crp"))).unwrap().to_string();
        let crp_bytes = files.iter().find(|f| f.0.starts_with("data/track/") && f.0.ends_with(".crp")).unwrap().1.clone();
        let crp = Crp::parse(&crp_bytes).unwrap();

        // bank slots
        let secs = easnd::parse_banks_ini(std::str::from_utf8(get("data/audio/banks.ini").unwrap()).unwrap());
        let mut tables: Vec<(usize, SfxTable)> = Vec::new();
        for (slot, path) in audio::track_banks(&secs, &track) {
            let h = format!("data/audio/{}", path.to_ascii_lowercase()).replace(".bnk", ".h");
            tables.push((slot, SfxTable::from_header(std::str::from_utf8(get(&h).unwrap()).unwrap())));
        }
        let slot = |s: usize| tables.iter().find(|t| t.0 == s).map(|t| &t.1);
        let mut present: Vec<String> = Vec::new();
        for class in ["boat", "sled", "turret", "tank"] {
            let ks: Vec<_> = keys.iter().filter(|k| k.0 == class).collect();
            let hit: Vec<&str> = ks.iter().filter(|k| slot(k.1).is_some_and(|t| t.get(k.2).is_some())).map(|k| k.2).collect();
            if hit.len() == ks.len() {
                present.push(format!("{class}:all"));
            } else if !hit.is_empty() {
                present.push(format!("{class}:{hit:?}"));
            }
        }

        // ESetCreakLevel (action 0xff): word 1 = level (s16), word 2 = interval
        let m = map_script(&crp).unwrap();
        let mut creaks = BTreeSet::new();
        for acts in m.triggers.iter().map(|t| &t.actions).chain(m.rules.iter().map(|r| &r.actions)) {
            for a in acts.iter().filter(|a| a.id == 0xff) {
                let (level, interval) = (a.word(1) as i16, a.word(2) as i32);
                assert!((0..=4).contains(&level), "{viv}: creak level {level}");
                creaks.insert((level, interval));
            }
        }

        // the Envi regions and the archive their ambience streams are in
        let regions = crp.env_regions().unwrap().unwrap_or_default();
        let speech = audio::speech_archive(&track, 0, |p| archive(&iso, p).is_some());
        let mus = stream_archive(&StreamTarget::Channel("music".into()), &track, &speech).unwrap();
        let (spe_set, mus_set) = (archive(&iso, &speech).unwrap_or_default(), archive(&iso, &mus).unwrap_or_default());
        let mut env = Vec::new();
        for r in &regions {
            let e = stream_entry(&r.name);
            let at = match (spe_set.contains(&e), mus_set.contains(&e)) {
                (true, true) => "spe+mus",
                (true, false) => "spe",
                (false, true) => "mus",
                _ => "none",
            };
            // [D] every region's ambience stream is in an archive a stream channel reads
            assert_ne!(at, "none", "{viv}: ambience {}", r.name);
            env.push(format!("{}#{}@{}/{}:{at}", r.name, r.env, r.top, r.depth));
        }

        // the band's level at every road-node height stays in 0..1
        let road = RoadNet::parse_crp(&crp).unwrap();
        let mut ymin = f32::MAX;
        for n in 0..road.nodes.len() {
            let y = road.node(n).unwrap().pos()[1];
            ymin = ymin.min(y);
            for r in &regions {
                let v = ambience_level(y, r.top, r.depth);
                assert!((0.0..=1.0).contains(&v), "{viv}: {} at y {y}: {v}", r.name);
            }
            nodes_checked += 1;
        }

        // [D] the boat's keys all resolve on the underwater track, the snowmobile's on MIS3
        if viv == "MIS11" { assert!(present.iter().any(|p| p == "boat:all"), "{present:?}"); }
        if viv == "MIS3" { assert!(present.iter().any(|p| p == "sled:all"), "{present:?}"); }
        let ymin = if road.nodes.is_empty() { "none".to_string() } else { format!("{ymin:.0}") };
        report += &format!("{viv}\t{track}\tkeys {present:?}\tcreaks {creaks:?}\troad ymin {ymin}\tenvi {env:?}\n");
    }
    eprint!("{report}");
    if std::env::var_os("NF_BLESS").is_some() {
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drive_class_sounds.tsv"), &report).unwrap();
    }
    assert!(nodes_checked > 1000, "{nodes_checked}");
    let expected = include_str!("drive_class_sounds.tsv");
    for (a, b) in expected.lines().zip(report.lines()) {
        assert_eq!(a, b);
    }
    assert_eq!(expected.lines().count(), report.lines().count());
}
