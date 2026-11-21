//! Disc test for `nf_formats::easnd` (docs/formats/driving-audio.md): every `.bnk` of the 8 driving
//! archives parses and every patch decodes to exactly its sample count from inside its memory class's
//! block; every `.asf` stream of the `.MUS`/`*EN.SPE` archives walks `SCHl … SCEl` with the `SCDl`
//! sample sum equal to the header's count and decodes; every `.h` id names a filled slot of its bank;
//! the archive's own `banks.ini` section names banks that are in the archive; every mix `.ini` parses.
//! Goldens: `golden/driving_audio_banks.tsv`, `golden/driving_audio_streams.tsv` (numbers + a SHA-1 digest of
//! the decoded PCM; `NF_BLESS=1` rewrites them).
//!
//! Opt-in via NF_ISO (the full image; a slim image without the `/DRIVING/` extents skips).

use nf_formats::easnd::{self, Bank, Stream};
use nf_formats::golden::diff_tsv;
use nf_formats::iso9660::IsoImage;

fn iso() -> Option<IsoImage> {
    let p = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(p).ok()?;
    let e = iso.find("/DRIVING/MIS01.VIV")?;
    let head = iso.read_bytes(e.lba as u64 * 2048, 4).ok()?;
    if head != b"BIGF" { eprintln!("skip: no /DRIVING/ extents in this image"); return None; }
    Some(iso)
}

/// Every wanted entry of one archive: (name with `/` separators, lower case; bytes after RefPack).
fn entries(iso: &IsoImage, path: &str, want: impl Fn(&str) -> bool) -> Vec<(String, Vec<u8>)> {
    let e = iso.find(path).unwrap_or_else(|| panic!("{path}"));
    let g = nf_formats::disc::read_bigf(iso, e).unwrap();
    g.entries.iter().filter(|x| want(&x.name.to_ascii_lowercase())).map(|x| {
        let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).unwrap();
        let b = if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw };
        (x.name.replace('\\', "/").to_ascii_lowercase(), b)
    }).collect()
}

fn pcm_digest(chans: &[&[i16]]) -> String {
    let mut b = Vec::new();
    for c in chans { for s in *c { b.extend_from_slice(&s.to_le_bytes()); } }
    // the first 8 bytes of the SHA-1 as a decimal u64 (decimal keeps the golden free of letter runs)
    let h = nf_formats::sha1::sha1(&b);
    u64::from_be_bytes(h[..8].try_into().unwrap()).to_string()
}

fn check_golden(file: &str, golden: &str, actual: &str) {
    if std::env::var_os("NF_BLESS").is_some() {
        std::fs::write(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden").join(file), actual).unwrap();
        return;
    }
    let d = diff_tsv(golden, actual);
    assert!(d.is_empty(), "{file}: {} differences, first: {:?}", d.len(), &d[..d.len().min(5)]);
}

const VIVS: [&str; 8] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];
const STREAM_ARCHIVES: [&str; 14] = ["MIS01.MUS", "MIS11.MUS", "MIS13A.MUS", "MIS13B.MUS", "MIS13C.MUS", "MIS3.MUS", "MIS4.MUS",
    "MIS01EN.SPE", "MIS11EN.SPE", "MIS13AEN.SPE", "MIS13BEN.SPE", "MIS13CEN.SPE", "MIS3EN.SPE", "MIS4EN.SPE"];

#[test]
fn every_driving_bank_decodes() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO with /DRIVING/ data not available"); return };
    let (mut banks, mut slots, mut patches, mut samples) = (0usize, 0usize, 0usize, 0u64);
    let mut codecs = std::collections::BTreeMap::<i32, usize>::new();
    let mut rows = String::from("bank\tslots\tfilled\tpatches\tsamples\tpcm_digest\n");
    for viv in VIVS {
        let files = entries(&iso, &format!("/DRIVING/{viv}.VIV"), |n| n.contains("data\\audio\\") || n.contains("data\\track\\"));
        for (name, data) in &files {
            if !name.ends_with(".bnk") { continue; }
            let b = Bank::parse(data).unwrap_or_else(|e| panic!("{viv} {name}: {e}"));
            banks += 1;
            // the two memory-class blocks tile the file exactly from the data start [D]
            assert_eq!(b.data_start as usize + b.size_mem100 as usize + b.size_mem8 as usize, data.len(), "{viv} {name}");
            let (mut np, mut ns) = (0usize, 0u64);
            let mut all: Vec<Vec<i16>> = Vec::new();
            for (i, s) in b.slots.iter().enumerate() {
                let Some(ps) = s else { continue };
                slots += 1;
                for p in ps {
                    patches += 1;
                    np += 1;
                    *codecs.entry(p.codec).or_default() += 1;
                    assert_eq!(p.channels, 1, "{viv} {name} #{i}");
                    let r = b.mem_range(p.mem).unwrap_or_else(|| panic!("{viv} {name} #{i}: mem {:#x}", p.mem));
                    let len = Bank::patch_bytes(p).unwrap_or_else(|| panic!("{viv} {name} #{i}: codec {:#x}", p.codec));
                    let off = p.data_offset as usize;
                    assert!(off >= r.start && off + len <= r.end, "{viv} {name} #{i}: {off:#x}+{len:#x} outside {r:?}");
                    // VAG lives in class 8, EA-XA in class 0x100 [D]
                    assert_eq!(p.mem, if p.codec == easnd::CODEC_VAG { 8 } else { 0x100 }, "{viv} {name} #{i}");
                    if p.codec == easnd::CODEC_EAXA {
                        for f in data[off + easnd::XA_SAMPLE_HEADER..off + len].chunks(easnd::XA_FRAME_BYTES) {
                            assert!(easnd::xa_frame_ok(f[0]), "{viv} {name} #{i}");
                        }
                        // the 16-byte sample header repeats the PT header [D]
                        assert_eq!(Bank::xa_sample_header(data, p), Some((0x0a, p.loop_start, p.loop_end, p.samples)), "{viv} {name} #{i}");
                    }
                    if p.loop_end >= 0 {
                        assert!(p.loop_start >= 0 && p.loop_start <= p.loop_end && p.loop_end < p.samples, "{viv} {name} #{i} loop");
                    }
                    let pcm = Bank::decode(data, p).unwrap();
                    assert_eq!(pcm.len(), p.samples as usize, "{viv} {name} #{i}");
                    ns += pcm.len() as u64;
                    all.push(pcm);
                }
            }
            samples += ns;
            let refs: Vec<&[i16]> = all.iter().map(|v| v.as_slice()).collect();
            rows += &format!("{viv}/{name}\t{}\t{}\t{np}\t{ns}\t{}\n", b.slots.len(), b.slots.iter().flatten().count(), pcm_digest(&refs));
        }
        // each .h names filled slots of its sibling .bnk
        for (name, data) in &files {
            let Some(stem) = name.strip_suffix(".h") else { continue };
            let bnk = files.iter().find(|f| f.0 == format!("{stem}.bnk")).unwrap_or_else(|| panic!("{viv}: {name} has no bank"));
            let b = Bank::parse(&bnk.1).unwrap();
            let ids = easnd::parse_sfx_header(std::str::from_utf8(data).unwrap());
            assert!(!ids.is_empty(), "{viv} {name}");
            for (sym, id) in ids {
                assert!(b.slots.get(id as usize).is_some_and(|s| s.is_some()), "{viv} {name}: {sym} = {id} is an empty slot");
            }
        }
        // the archive's own track section of banks.ini names banks that are all in the archive
        let track = files.iter().find_map(|f| f.0.strip_prefix("data/track/").and_then(|s| s.strip_suffix(".crp"))).unwrap().to_string();
        let ini = files.iter().find(|f| f.0.ends_with("banks.ini")).unwrap();
        let secs = easnd::parse_banks_ini(std::str::from_utf8(&ini.1).unwrap());
        let (_, keys) = secs.iter().find(|s| s.0 == track).unwrap_or_else(|| panic!("{viv}: no [{track}] in banks.ini"));
        assert!(!keys.is_empty());
        for (_, path) in keys {
            let full = format!("data/audio/{}", path.to_ascii_lowercase());
            assert!(files.iter().any(|f| f.0 == full), "{viv} [{track}]: {path} not in the archive");
        }
        // the per-track mix .ini, else the data/audio/default.ini fallback (FUN_002f3058 [M]) — RACE
        // carries only the fallback [D]
        let own = files.iter().find(|f| f.0 == format!("data/audio/{track}.ini"));
        assert_eq!(own.is_none(), viv == "RACE", "{viv}");
        let mix = own.or_else(|| files.iter().find(|f| f.0 == "data/audio/default.ini")).unwrap_or_else(|| panic!("{viv}: no mix ini"));
        assert!(easnd::parse_mix_ini(&mix.1).len() > 40, "{viv} {track}.ini");
    }
    eprintln!("banks {banks} slots {slots} patches {patches} samples {samples} codecs {codecs:?}");
    // [D measured over the full image by this test]
    assert_eq!((banks, slots, patches, samples), (56, 1379, 1468, 30_331_875));
    assert_eq!(codecs.into_iter().collect::<Vec<_>>(), vec![(easnd::CODEC_VAG, 1219), (easnd::CODEC_EAXA, 249)]);
    check_golden("driving_audio_banks.tsv", include_str!("../golden/driving_audio_banks.tsv"), &rows);
}

#[test]
fn every_driving_stream_decodes() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO with /DRIVING/ data not available"); return };
    let (mut streams, mut samples, mut stereo) = (0usize, 0u64, 0usize);
    let mut ids = std::collections::BTreeMap::<String, usize>::new();
    let (mut jump_carry, mut jump_reset, mut jump_prefix) = (0u64, 0u64, 0u64);
    let mut rows = String::from("stream\tchannels\trate\tsamples\tblocks\tpcm_digest\n");
    for a in STREAM_ARCHIVES {
        for (name, data) in entries(&iso, &format!("/DRIVING/{a}"), |n| n.ends_with(".asf")) {
            let s = Stream::parse(&data).unwrap_or_else(|e| panic!("{a} {name}: {e}"));
            streams += 1;
            for c in &s.chunks { *ids.entry(String::from_utf8_lossy(&c.id).into_owned()).or_default() += 1; }
            assert_eq!(&s.chunks.last().unwrap().id, b"SCEl", "{a} {name}");
            assert_eq!(s.header.codec, easnd::CODEC_EAXA, "{a} {name}");
            assert_eq!(s.header.rate, 48000, "{a} {name}");
            if s.header.channels == 2 { stereo += 1; }
            let n = s.block_samples(&data).unwrap();
            assert_eq!(n as i32, s.header.samples, "{a} {name}");
            let pcm = s.decode(&data).unwrap();
            for ch in &pcm { assert_eq!(ch.len(), n, "{a} {name}"); }
            samples += n as u64 * pcm.len() as u64;
            // every block's frame grid (after the 4-byte prefix) holds valid coefficient indices, and the
            // continuity at block joins for: history carried (the port), reset to 0, or loaded from the
            // prefix as two LE i16 (h1, h2) [S probe — the IOP stream feed is not traced]
            let mut at = 0usize;
            let mut blocks = 0usize;
            for c in s.chunks.iter().filter(|c| &c.id == b"SCDl") {
                blocks += 1;
                let (bn, offs) = s.block(&data, c).unwrap();
                for &o in &offs {
                    for f in data[o + easnd::XA_BLOCK_PREFIX..].chunks(easnd::XA_FRAME_BYTES).take(bn.div_ceil(28)) {
                        assert!(easnd::xa_frame_ok(f[0]), "{a} {name}");
                    }
                }
                if at > 0 {
                    for (i, &o) in offs.iter().enumerate() {
                        let body = &data[o + easnd::XA_BLOCK_PREFIX..c.at + c.size];
                        let prev = pcm[i][at - 1] as i64;
                        jump_carry += (pcm[i][at] as i64 - prev).unsigned_abs();
                        let f0 = easnd::decode_xa(body, 1, &mut easnd::XaHistory::default()).unwrap()[0] as i64;
                        jump_reset += (f0 - prev).unsigned_abs();
                        let le = |k: usize| i16::from_le_bytes([data[o + k], data[o + k + 1]]) as i32;
                        let fp = easnd::decode_xa(body, 1, &mut easnd::XaHistory { h1: le(0), h2: le(2) }).unwrap()[0] as i64;
                        jump_prefix += (fp - prev).unsigned_abs();
                    }
                }
                at += bn;
            }
            let refs: Vec<&[i16]> = pcm.iter().map(|v| v.as_slice()).collect();
            rows += &format!("{a}/{name}\t{}\t{}\t{n}\t{blocks}\t{}\n", s.header.channels, s.header.rate, pcm_digest(&refs));
        }
    }
    eprintln!("streams {streams} stereo {stereo} samples {samples} ids {ids:?} block-join |step| sums: carried {jump_carry} zeroed {jump_reset} prefix-loaded {jump_prefix}");
    // [D measured over the full image by this test]
    assert_eq!((streams, stereo, samples), (236, 118, 495_747_423));
    assert_eq!(ids.get("SCDl"), Some(&80326));
    // carrying the history joins blocks far more smoothly than zeroing it [D]
    assert!(jump_carry * 4 < jump_reset, "carried {jump_carry} vs zeroed {jump_reset}");
    check_golden("driving_audio_streams.tsv", include_str!("../golden/driving_audio_streams.tsv"), &rows);
}
