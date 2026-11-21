//! Disc test for `nf_game::driving::audio` against the driving archives (NF_ISO, full image):
//! per track, the bank list `track_banks` picks is all in the archive, the mix tree built from the
//! track's mix `.ini` gives every linked group a volume, and the sound keys the ported logic uses
//! resolve (or not — pinned) in the banks the game looks them up in.

use nf_formats::easnd::{self, SfxTable};
use nf_formats::iso9660::IsoImage;
use nf_game::driving::audio::{self, Mixer};

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

#[test]
fn driving_audio_logic_against_the_disc() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO with /DRIVING/ data not available"); return };
    let mut report = String::new();
    for viv in VIVS {
        let files = audio_files(&iso, viv);
        let get = |n: &str| files.iter().find(|f| f.0 == n).map(|f| f.1.as_slice());
        let track = files.iter().find_map(|f| f.0.strip_prefix("data/track/").and_then(|s| s.strip_suffix(".crp"))).unwrap().to_string();
        let secs = easnd::parse_banks_ini(std::str::from_utf8(get("data/audio/banks.ini").unwrap()).unwrap());
        let banks = audio::track_banks(&secs, &track);
        let mut tables: Vec<(usize, SfxTable)> = Vec::new();
        for (slot, path) in &banks {
            let bnk = format!("data/audio/{}", path.to_ascii_lowercase());
            assert!(get(&bnk).is_some(), "{viv}: slot {slot} {path}");
            let h = bnk.replace(".bnk", ".h");
            tables.push((*slot, SfxTable::from_header(std::str::from_utf8(get(&h).unwrap()).unwrap())));
        }
        let slot = |s: usize| tables.iter().find(|t| t.0 == s).map(|t| &t.1);
        // the mix tree from the track's ini (RACE: default.ini)
        let ini = get(&format!("data/audio/{track}.ini")).or_else(|| get("data/audio/default.ini")).unwrap();
        let mut m = Mixer::driving(&easnd::parse_mix_ini(ini), 100, 100);
        let silent: Vec<&str> = audio::MIX_PARENTS.iter().map(|p| p.0).filter(|g| m.volume_of(g) == 0.0).collect();
        // engine voices: slot 9 (Engine); scrapes: slot 1 (Collisions); skids + road noise: slot 0 [M]
        let engine: Vec<&str> = audio::ENGINE_VOICES.iter().copied().chain([audio::ENGINE_VACUUM])
            .filter(|k| slot(audio::ENGINE_SLOT).is_none_or(|t| t.get(k).is_none())).collect();
        let scrapes: Vec<&str> = audio::SCRAPE_VOICES.iter().copied()
            .filter(|k| slot(audio::SCRAPE_SLOT).is_none_or(|t| t.get(k).is_none())).collect();
        let mut generic: Vec<&str> = Vec::new();
        for (f, r, _, _) in audio::SKID_SETS { generic.push(f); generic.push(r); }
        for s in [1, 2, 4, 8, 9, 10] { generic.push(audio::road_noise(s).unwrap()); }
        generic.sort();
        generic.dedup();
        let generic_missing: Vec<&str> = generic.into_iter().filter(|k| slot(audio::VEHICLE_SLOT).is_none_or(|t| t.get(k).is_none())).collect();
        let bolt = slot(4).and_then(|t| t.get("SFX_sniperbolt"));
        report += &format!("{viv}\t{track}\tslots {:?}\tsilent groups {silent:?}\tengine missing {engine:?}\tscrapes missing {scrapes:?}\tgeneric missing {generic_missing:?}\tsniperbolt {bolt:?}\n",
            banks.iter().map(|b| b.0).collect::<Vec<_>>());
    }
    eprint!("{report}");
    // [D measured over the full image by this test]; rows per archive
    let expected = include_str!("drive_audio_logic.tsv");
    for (a, b) in expected.lines().zip(report.lines()) { assert_eq!(a, b); }
    assert_eq!(expected.lines().count(), report.lines().count());
}
