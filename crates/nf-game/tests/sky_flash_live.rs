//! The storm sky's lightning flash on 0700000a / 0700000b [M View_DrawSky 0x1e5ad0:97-157, 161-219]:
//! sky slot 3 holds only the "Thunderhead" flash variant (placement param 1 = 1 [D]), so the slot draws
//! nothing between flashes and the Thunderhead object on a flash tick. A flash runs `1 + flash_len`
//! ticks — the start tick always draws (`iVar11 = iVar2` [M :145]), each following tick re-rolls
//! `Rand_Rand(100) > 50` [M :150-151] — and the next start comes `Rand_Rand(interval)` ticks later.
//! `Sky::flash_mask` is the per-frame record the renderer swaps on. Opt-in: set `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};
use nf_game::drone::Rng;
use nf_game::sky::{Sky, MAX_SLOTS};

fn level(name: &str) -> Option<Level> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Some(Level::parse(&data).unwrap())
}

fn sky_of(l: &Level) -> Sky { Sky::from_placements(&l.maps[l.main_map().unwrap()].placements) }

/// The seed `Drones::new` gives the sky's own stream (`sky_rng`).
const SKY_SEED: u32 = 0x1e_5ad0;

/// Runs `n` frames; returns per frame (flash_mask, slot 3's flash countdown after the frame).
fn run(sky: &mut Sky, n: usize) -> Vec<(u32, i16)> {
    let mut rng = Rng::new(SKY_SEED);
    (0..n).map(|_| { sky.frame_thunder(0, false, &mut rng); (sky.flash_mask, sky.slots[3].flash) }).collect()
}

/// The flash windows: (first tick, last tick) of every run of ticks where slot 3 is inside a flash
/// (the start tick, then the `flash_len` countdown ticks).
fn windows(frames: &[(u32, i16)], flash_len: i16) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for (t, &(_, f)) in frames.iter().enumerate() {
        if f == flash_len { out.push((t, t + flash_len as usize)); }
    }
    out
}

#[test]
fn storm_sky_slot3_is_a_flash_only_thunderhead() {
    for (name, interval, flash_len) in [("0700000a.bin", 1440u16, 5i16), ("0700000b.bin", 1800, 10)] {
        let Some(l) = level(name) else { eprintln!("skip: NF_ISO not set"); return };
        let sky = sky_of(&l);
        let s3 = sky.slots[3];
        assert!(s3.flash_glist != 0 && s3.glist == 0, "{name}: slot 3 = flash variant only [D]: {s3:?}");
        assert_eq!((s3.interval, s3.flash_len, s3.skyid), (interval, flash_len, 0), "{name}: param 2 × 60, param 3, group 0 [D]");
        // The variant the renderer draws comes from the slot's own object: the Thunderhead [D].
        let sl = l.sky_slot_instances();
        let flash: Vec<_> = sl.iter().filter(|s| s.flash).collect();
        assert_eq!(flash.len(), 1, "{name}: one flash placement");
        let o = &l.maps[flash[0].instance.map].objects[flash[0].instance.object];
        assert_eq!((flash[0].slot, o.name.as_str()), (3, "Thunderhead"), "{name}: the flash object [D]");
        // Sister: the still sky objects are not flash variants and no other slot has a timer.
        assert!(sl.iter().filter(|s| !s.flash).all(|s| s.slot != 3), "{name}: slot 3 has no normal variant");
        assert!((0..MAX_SLOTS).filter(|&i| i != 3).all(|i| sky.slots[i].flash_glist == 0 && sky.slots[i].flash_obj == 0));
    }
}

#[test]
fn flash_mask_follows_the_decompile_schedule() {
    for (name, flash_len) in [("0700000a.bin", 5i16), ("0700000b.bin", 10)] {
        let Some(l) = level(name) else { eprintln!("skip: NF_ISO not set"); return };
        let mut sky = sky_of(&l);
        let frames = run(&mut sky, 20_000);
        // Only slot 3 ever flashes.
        assert!(frames.iter().all(|&(m, _)| m & !(1 << 3) == 0), "{name}: only slot 3's bit");
        let w = windows(&frames, flash_len);
        eprintln!("{name}: flash windows {:?}", &w[..w.len().min(12)]);
        for (t, &(m, _)) in frames.iter().enumerate() {
            let inside = w.iter().any(|&(a, b)| (a..=b).contains(&t));
            if !inside { assert_eq!(m, 0, "{name} tick {t}: no flash draw outside a flash window"); }
        }
        // Every window's start tick draws (the `iVar11 = iVar2` start [M :145]); the countdown ticks
        // draw about half the time (`Rand_Rand(100) > 50` → 49 of 100).
        assert!(w.len() >= 10, "{name}: several flashes in 20000 ticks: {}", w.len());
        assert!(w.iter().all(|&(a, _)| frames[a].0 == 1 << 3), "{name}: the start tick always draws");
        let (mut drawn, mut rolls) = (0usize, 0usize);
        for &(a, b) in &w { for t in a + 1..=b.min(frames.len() - 1) { rolls += 1; drawn += (frames[t].0 != 0) as usize; } }
        let frac = drawn as f32 / rolls as f32;
        assert!((0.35..0.65).contains(&frac), "{name}: flicker ratio {frac} (49 %)");
        // The gap between starts never exceeds the interval + the flash (Rand_Rand(interval) < interval),
        // and the first flash comes on the first frame (a fresh 0 dark timer wraps [M :120-121]).
        assert_eq!(w[0].0, 0, "{name}: a fresh slot flashes at once");
        let interval = sky.slots[3].interval as usize;
        for p in w.windows(2) { assert!(p[1].0 - p[0].0 <= interval + flash_len as usize + 1, "{name}: gap {:?}", p); }
        // Sister: a paused frame draws the normal pair — the mask clears, the timers do not run.
        let before = sky.slots[3];
        let mut r = Rng::new(1);
        sky.flash_mask = 1 << 3;
        sky.frame_thunder(0, true, &mut r);
        assert_eq!((sky.flash_mask, sky.slots[3]), (0, before), "{name}: paused → normal variant, timers frozen");
    }
}

/// The schedule the renders were taken at: the sim's sky stream from `Drones::new`. Tick numbers are
/// 0-based frame indices of `Sky::frame_thunder` (`--frame N` renders index N − 1). Pinned so the
/// mid-flash / between-flash render frames stay meaningful.
#[test]
fn render_schedule() {
    for (name, want) in [
        ("0700000a.bin", &[0usize, 2, 3, 4, 5, 779, 2005, 2007, 2008, 2630, 2631, 2635, 3656, 3657, 3770, 3771, 3772, 3773, 3775][..]),
        ("0700000b.bin", &[0, 2, 3, 4, 5, 6, 977, 979, 980, 982, 983, 986, 987, 1148, 1151, 1154, 1418, 1419, 1420, 1421,
            1423, 1425, 1426, 1427, 1428, 2225, 2228, 2230, 2231, 2233, 3112, 3114, 3115, 3117, 3118, 3119, 3122][..]),
    ] {
        let Some(l) = level(name) else { eprintln!("skip: NF_ISO not set"); return };
        let mut sky = sky_of(&l);
        let frames = run(&mut sky, 4000);
        let lit: Vec<usize> = frames.iter().enumerate().filter(|(_, f)| f.0 != 0).map(|(t, _)| t).collect();
        assert_eq!(lit, want, "{name}: the lit ticks of the sky stream");
        // Sister: another seed gives another schedule (the pin depends on the stream).
        let mut sky2 = sky_of(&l);
        let mut r = Rng::new(SKY_SEED ^ 1);
        let lit2: Vec<usize> = (0..4000).filter(|_| { sky2.frame_thunder(0, false, &mut r); sky2.flash_mask != 0 }).collect();
        assert_ne!(lit2, want, "{name}: the schedule follows the stream");
    }
}

/// What the flash draws [D + M]: the Thunderhead's PS2_GFX block is 32 bytes whose header word (the
/// footer offset) is 0x10 — `psiCreateEntityGfx` 0x1171b0:23-27 zeroes such a glist (box count 0, box
/// array 0) and `psiDrawObjectMatrix` 0x124708 then returns before `RecurseAndDrawBoxes`. So the slot's
/// flash variant puts no pixels on screen on either storm level; the swap is exact and the frames
/// are unchanged. Sister: the slots' normal sky objects carry geometry.
#[test]
fn thunderhead_flash_glist_is_empty() {
    for name in ["0700000a.bin", "0700000b.bin"] {
        let Some(l) = level(name) else { eprintln!("skip: NF_ISO not set"); return };
        let sl = l.sky_slot_instances();
        for s in &sl {
            let o = &l.maps[s.instance.map].objects[s.instance.object];
            if s.flash {
                assert_eq!((o.name.as_str(), o.gfx.boxes.len(), o.gfx.leaves.len()), ("Thunderhead", 0, 0), "{name}: empty flash glist [D]");
            } else {
                assert!(!o.gfx.leaves.is_empty(), "{name}: {} draws geometry", o.name);
            }
        }
    }
}
