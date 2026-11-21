//! Disc test for `nf_game::driving::stream_events` and `vehicle_sounds` (NF_ISO, full image): every
//! stream a mission-script `EStreamEvent` names is in the archive its channel reads, the stinger
//! is in both archives, every `EPlaySound` / `EPlaySoundPos` key is in the bank it names, the mix
//! events' groups and the duck volumes per track, and where the weapon / flight / explosion /
//! vehicle-class keys resolve. Golden: `drive_streams.tsv` (one line per track).
//! Skips cleanly when NF_ISO is unset or has no `/DRIVING/` data.

use std::collections::BTreeSet;

use nf_formats::crp::Crp;
use nf_formats::easnd::{self, SfxTable};
use nf_formats::iso9660::IsoImage;
use nf_formats::trackstart::map_script;
use nf_game::driving::audio;
use nf_game::driving::stream_events::{script_sound, stream_archive, stream_entry, ScriptSound, StreamTarget, STINGER};
use nf_game::driving::vehicle_sounds::{self as vs, Shooter};
use nf_game::driving::weapons::WEAPONS;

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

/// The entry names of an archive on the disc (`driving\mis01.mus` → `/DRIVING/MIS01.MUS`).
fn archive(iso: &IsoImage, path: &str) -> Option<BTreeSet<String>> {
    let p = format!("/{}", path.replace('\\', "/").to_ascii_uppercase());
    let e = iso.find(&p)?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    Some(g.entries.iter().map(|x| x.name.replace('/', "\\").to_ascii_lowercase()).collect())
}

fn norm(p: &str) -> String { p.replace('\\', "/").to_ascii_lowercase() }

#[test]
fn driving_streams_and_sound_keys_against_the_disc() {
    let Some(iso) = iso() else {
        eprintln!("skip: NF_ISO with /DRIVING/ data not available");
        return;
    };
    let mut report = String::new();
    let mut total_streams = 0usize;
    for viv in VIVS {
        let files = viv_files(&iso, viv);
        let get = |n: &str| files.iter().find(|f| f.0 == n).map(|f| f.1.as_slice());
        let track = files.iter().find_map(|f| f.0.strip_prefix("data/track/").and_then(|s| s.strip_suffix(".crp"))).unwrap().to_string();
        let crp_bytes = files.iter().find(|f| f.0.starts_with("data/track/") && f.0.ends_with(".crp")).unwrap().1.clone();
        let crp = Crp::parse(&crp_bytes).unwrap();
        let m = map_script(&crp).unwrap();
        let speech = audio::speech_archive(&track, 0, |p| archive(&iso, p).is_some());
        let mus = stream_archive(&StreamTarget::Channel("music".into()), &track, &speech).unwrap();
        let spe_set = archive(&iso, &speech).unwrap_or_default();
        let mus_set = archive(&iso, &mus).unwrap_or_default();

        // banks by name and slot
        let secs = easnd::parse_banks_ini(std::str::from_utf8(get("data/audio/banks.ini").unwrap()).unwrap());
        let banks = audio::track_banks(&secs, &track);
        let table = |path: &str| -> Option<SfxTable> {
            let h = format!("data/audio/{}", norm(path)).replace(".bnk", ".h");
            get(&h).map(|b| SfxTable::from_header(std::str::from_utf8(b).unwrap()))
        };
        let slot = |s: usize| banks.iter().find(|b| b.0 == s).and_then(|b| table(&b.1));
        // the mix ini (RACE carries only default.ini [D])
        let ini = files.iter().find(|f| f.0 == format!("data/audio/{track}.ini")).or_else(|| files.iter().find(|f| f.0 == "data/audio/default.ini")).unwrap();
        let mix = easnd::parse_mix_ini(&ini.1);
        let ini_vol = |g: &str| mix.iter().find(|l| l.name.eq_ignore_ascii_case(g)).map(|l| l.volume);

        let (mut streams, mut missing_streams) = (BTreeSet::new(), BTreeSet::new());
        let (mut sounds, mut missing_sounds) = (0usize, BTreeSet::new());
        let mut mix_groups = BTreeSet::new();
        let lists = m.triggers.iter().map(|t| &t.actions).chain(m.rules.iter().map(|r| &r.actions));
        for acts in lists {
            for a in acts {
                let Some(ev) = script_sound(a, &m) else { continue };
                match ev {
                    ScriptSound::Stream(r) => {
                        let set = match &r.target {
                            StreamTarget::Speech | StreamTarget::Nis => &spe_set,
                            StreamTarget::Channel(c) if c.eq_ignore_ascii_case("music") => &mus_set,
                            StreamTarget::Channel(c) => panic!("{viv}: stream channel {c}"),
                        };
                        let e = stream_entry(&r.name);
                        let tag = format!("{}:{}", if set as *const _ == &mus_set as *const _ { "mus" } else { "spe" }, e);
                        if !set.contains(&e) { missing_streams.insert(tag.clone()); }
                        streams.insert(tag);
                    }
                    ScriptSound::PlaySound { bank, sfx } | ScriptSound::PlaySoundPos { bank, sfx, .. } => {
                        sounds += 1;
                        let ok = banks.iter().any(|b| norm(&b.1) == norm(&bank)) && table(&bank).is_some_and(|t| t.get(&sfx).is_some());
                        if !ok { missing_sounds.insert(format!("{bank}:{sfx}")); }
                    }
                    ScriptSound::SetMix { group, .. } | ScriptSound::RevertMix { group, .. } => {
                        mix_groups.insert(format!("{group}{}", if ini_vol(&group).is_some() { "" } else { "(no ini)" }));
                    }
                    _ => {}
                }
            }
        }
        total_streams += streams.len();
        let stinger = (spe_set.contains(&stream_entry(STINGER)), mus_set.contains(&stream_entry(STINGER)));

        // weapon / flight / explosion / class keys: which are in their slot's bank on this track
        let mut keys: Vec<(usize, String)> = Vec::new();
        let mut c = vs::FireCounters::default();
        for w in 0..31 {
            for sh in [Shooter::Vehicle { kind1: true, player: true, flying: false }, Shooter::Vehicle { kind1: true, player: false, flying: false }, Shooter::Vehicle { kind1: true, player: false, flying: true }, Shooter::Element { bit1: false }] {
                if let Some(s) = vs::fire_sound(w, WEAPONS[w].sfx_stem, sh, false, &mut c) { keys.push((s.slot, s.sfx)); }
            }
            for p in [true, false] {
                if let Some(s) = vs::transit_sound(w, p, &track) { keys.push((s.slot, s.sfx)); }
            }
        }
        keys.push((4, vs::emp_detonate().sfx));
        keys.push((vs::LOCK_WARNING.0, vs::LOCK_WARNING.1.into()));
        for k in vs::EXPLOSIONS.iter().chain(["SFX_DistExp"].iter()) { keys.push((4, k.to_string())); }
        for k in vs::HELI_VOICES.iter().chain(vs::ULTRALIGHT_VOICES.iter()).chain(["SFX_TankTread"].iter()) { keys.push((0, k.to_string())); }
        keys.sort();
        keys.dedup();
        let present: Vec<String> = keys.iter().filter(|(s, k)| slot(*s).is_some_and(|t| t.get(k).is_some())).map(|(s, k)| format!("{s}:{k}")).collect();

        report += &format!(
            "{viv}\t{track}\t{speech}\t{mus}\tstreams {}\tmissing {:?}\tstinger spe/mus {:?}\tsounds {sounds} missing {:?}\tmix {:?}\tduck fx/music {:?}/{:?}\tkeys {:?}\n",
            streams.len(),
            missing_streams,
            stinger,
            missing_sounds,
            mix_groups,
            ini_vol("0:Fade Effects"),
            ini_vol("0:Fade Music"),
            present,
        );
    }
    eprint!("{report}");
    if std::env::var_os("NF_BLESS").is_some() {
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/drive_streams.tsv"), &report).unwrap();
    }
    assert!(total_streams > 100, "{total_streams}");
    let expected = include_str!("drive_streams.tsv");
    for (a, b) in expected.lines().zip(report.lines()) {
        assert_eq!(a, b);
    }
    assert_eq!(expected.lines().count(), report.lines().count());
}
