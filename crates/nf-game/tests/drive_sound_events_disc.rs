//! Disc test for `nf_game::driving::sound_events` (NF_ISO, full image): every sound key the ported
//! events name resolves in the bank slot the game looks it up in, on at least one track, and the
//! per-track gaps are pinned (golden `drive_sound_events.tsv`). The keys of the vehicle classes whose
//! update is not ported are located (which slots hold them) and pinned too.

use nf_formats::easnd::{self, SfxTable};
use nf_formats::iso9660::IsoImage;
use nf_game::driving::{audio, sound_events as ev};

fn iso() -> Option<IsoImage> {
    let iso = IsoImage::open(std::env::var_os("NF_ISO")?).ok()?;
    let e = iso.find("/DRIVING/MIS01.VIV")?;
    (iso.read_bytes(e.lba as u64 * 2048, 4).ok()? == b"BIGF").then_some(iso)
}

fn audio_files(iso: &IsoImage, viv: &str) -> Vec<(String, Vec<u8>)> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV")).unwrap();
    let g = nf_formats::disc::read_bigf(iso, e).unwrap();
    g.entries.iter().filter(|x| { let n = x.name.to_ascii_lowercase(); n.contains("data\\audio\\") || n.contains("data\\track\\") }).map(|x| {
        let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).unwrap();
        let b = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
        (x.name.replace('\\', "/").to_ascii_lowercase(), b)
    }).collect()
}

const VIVS: [&str; 8] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

/// (slot, key) for every key the ported events look up [M, slots as in sound_events.rs].
fn traced_keys() -> Vec<(usize, &'static str)> {
    let mut k: Vec<(usize, &'static str)> = Vec::new();
    for (_, s) in ev::IMPACTS_A.iter().chain(ev::IMPACTS_B.iter()).chain(ev::IMPACTS_C.iter()) { k.push((1, s)); }
    for s in ["SFX_TiresHeavy", "SFX_TiresWood", "SFX_TiresSnow", "SFX_Light4", "SFX_Med3"] { k.push((1, s)); }
    for s in ev::METAL_GATES { k.push((2, s)); }
    k.push((0, "SFX_TrafficRN"));
    k.push((0, "SFX_tanktreads"));
    k.push((0, "SFX_CarAlarm"));
    for w in [3, 0xb, 0xc, 0xf, 0x12] { let o = ev::car_weapon_fire(w).unwrap(); k.push((o.slot, o.sfx)); }
    k.push((ev::DRY_FIRE.slot, ev::DRY_FIRE.sfx));
    k.sort();
    k.dedup();
    k
}

#[test]
fn driving_sound_event_keys_against_the_disc() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO with /DRIVING/ data not available"); return };
    let keys = traced_keys();
    let mut found = vec![false; keys.len()];
    let mut report = String::new();
    for viv in VIVS {
        let files = audio_files(&iso, viv);
        let get = |n: &str| files.iter().find(|f| f.0 == n).map(|f| f.1.as_slice());
        let track = files.iter().find_map(|f| f.0.strip_prefix("data/track/").and_then(|s| s.strip_suffix(".crp"))).unwrap().to_string();
        let secs = easnd::parse_banks_ini(std::str::from_utf8(get("data/audio/banks.ini").unwrap()).unwrap());
        let mut tables: Vec<(usize, SfxTable)> = Vec::new();
        for (slot, path) in audio::track_banks(&secs, &track) {
            let h = format!("data/audio/{}", path.to_ascii_lowercase()).replace(".bnk", ".h");
            tables.push((slot, SfxTable::from_header(std::str::from_utf8(get(&h).unwrap()).unwrap())));
        }
        let slot = |s: usize| tables.iter().find(|t| t.0 == s).map(|t| &t.1);
        let mut missing = Vec::new();
        for (i, (s, k)) in keys.iter().enumerate() {
            if slot(*s).and_then(|t| t.get(k)).is_some() { found[i] = true } else { missing.push(format!("{s}:{k}")) }
        }
        // the unported classes' ctor keys: which slots hold them on this track
        let mut located = Vec::new();
        for (_, class, ks) in ev::CLASS_VOICES.iter().copied().chain([(0x2debc8, "alarm", &["SFX_CarAlarm"][..])]) {
            for k in ks {
                let at: Vec<usize> = tables.iter().filter(|t| t.1.get(k).is_some()).map(|t| t.0).collect();
                if !at.is_empty() { located.push(format!("{class}:{k}@{at:?}")); }
            }
        }
        // traffic load layers present in slot 0 (`SFX_<car>LdEn`)
        let mut traffic: Vec<String> = slot(0).map(|t| t.0.iter().map(|e| e.0.clone()).filter(|n| n.to_ascii_lowercase().ends_with("ldEn".to_ascii_lowercase().as_str())).collect()).unwrap_or_default();
        traffic.sort();
        report += &format!("{viv}\t{track}\tmissing {missing:?}\tclass keys {located:?}\ttraffic LdEn {traffic:?}\n");
    }
    eprint!("{report}");
    let never: Vec<String> = keys.iter().zip(&found).filter(|(_, f)| !**f).map(|((s, k), _)| format!("{s}:{k}")).collect();
    // [D] every traced key is in its slot's bank on some track, except the pinned ones
    assert_eq!(never, include_str!("drive_sound_events_never.txt").lines().map(String::from).collect::<Vec<_>>(), "keys on no track");
    let expected = include_str!("drive_sound_events.tsv");
    for (a, b) in expected.lines().zip(report.lines()) { assert_eq!(a, b); }
    assert_eq!(expected.lines().count(), report.lines().count());
}
