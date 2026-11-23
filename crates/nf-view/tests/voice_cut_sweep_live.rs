//! voice-cut-sweep: every spoken line a level starts plays to its full length.
//!
//! Each on-foot level boots fresh (its intro cut-scene plays, not skipped), then Bond stands idle
//! for a long window with the walker's health top-up, so the guards that find him talk and fight.
//! The headless driver's line ledger (`Driver::line_ledger`) logs every item that keyed on: its
//! first sample's length on the frame clock (the decoded VAG / stream length at the voice's pitch)
//! and how many updates it ran before the kill pass took it, with the kill's reason.
//!
//! A spoken line is an id with a subtitle row (`Snd2Lbl`), a drone voice-table id
//! (`Drone_SfxInfo`), or a streamed effect. Opt-in via `NF_ISO`; `NF_SWEEP_LEVELS` (comma list of
//! hex ids) and `NF_SWEEP_TICKS` narrow or lengthen a run.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

const LEVELS: [u32; 21] = [
    0x0700_0001, 0x0700_0002, 0x0700_0003, 0x0700_0004, 0x0700_0005, 0x0700_0006, 0x0700_0007,
    0x0700_0008, 0x0700_0009, 0x0700_000a, 0x0700_000b, 0x0700_000c, 0x0700_000d, 0x0700_0011,
    0x0700_0012, 0x0700_0013, 0x0700_0014, 0x0700_0015, 0x0700_0016, 0x0700_001b, 0x0700_004a,
];

struct Sweep { seen: usize, cut: Vec<String>, stolen: Vec<String>, dropped: Vec<String> }

fn sweep_level(iso: &str, level: u32, ticks: u32) -> Sweep {
    let name = format!("{level:08x}.bin");
    let mut w = Walker::new(nf_view::load(iso, &name, false).unwrap_or_else(|e| panic!("load {name}: {e}")));
    w.skip_cutscenes = false;
    let spoken: std::collections::HashSet<u32> = {
        let sim = w.scene.sim.as_ref().unwrap();
        let mut s: std::collections::HashSet<u32> = sim.drones.snd2lbl.iter().map(|&(_, id)| id).collect();
        for v in sim.voices.iter().flatten() { if v.sfx != 0 { s.insert(v.sfx); } }
        s
    };
    if std::env::var_os("NF_SWEEP_SPOKEN").is_some() {
        let mut ids: Vec<u32> = spoken.iter().copied().collect();
        ids.sort();
        eprintln!("[spoken] {}", ids.iter().map(|i| format!("{i:#x}")).collect::<Vec<_>>().join(","));
    }
    w.scene.sim.as_mut().unwrap().headless_sound.as_mut().expect("headless sound").0.driver.line_ledger = Some(Vec::new());
    for _ in 0..ticks { w.tick(Default::default(), false); }
    let drv = &w.scene.sim.as_ref().unwrap().headless_sound.as_ref().unwrap().0.driver;
    let mut rows = drv.line_ledger.clone().unwrap_or_default();
    rows.extend(drv.live_line_recs());
    let mut out = Sweep { seen: 0, cut: Vec::new(), stolen: Vec::new(), dropped: Vec::new() };
    for r in rows.iter().filter(|r| !r.looped && (r.streamed || spoken.contains(&r.id))) {
        if !r.keyed {
            // A start the driver turned down (distance/budget reject, no free voice or stream slot,
            // a precache that expired): logged, not counted as a cut.
            let line = format!("{level:08x} id {:#x} handle {} f{} never keyed: {}", r.id, r.handle, r.end_frame, r.why);
            eprintln!("[sweep] DROP {line}");
            out.dropped.push(line);
            continue;
        }
        out.seen += 1;
        let alive = r.why.is_empty();
        // The frame clock ends a voice in the frame its last sample plays; the kill pass reads the end
        // flag one update later (+2 frames of slack).
        let short = !alive && !r.multi && r.played_frames + 2 < r.expect_frames;
        let line = format!("{level:08x} id {:#x} handle {} f{}-f{} played {} of {} why {} live {} stream {}",
            r.id, r.handle, r.start_frame, r.end_frame, r.played_frames, r.expect_frames, if alive { "alive" } else { r.why }, r.voice_live, r.streamed);
        // `SFXSetup` 0x4220's own steals: the voice budget takes the lowest-priority live item
        // (head-first on ties, streams and tracking-3 exempt) and the per-id limit the oldest
        // same-id item [M SFX.IRX 0x4220:63-111]. Those cuts are the game's; any other short end
        // (an early end flag, an EE stop, an unknown kill) is a port cut.
        let stolen = short && matches!(r.why, "budget-steal" | "max-voices-oldest");
        eprintln!("[sweep] {}{line}", if stolen { "STOLEN " } else if short { "CUT " } else { "" });
        if stolen { out.stolen.push(line); } else if short { out.cut.push(line); }
    }
    out
}

#[test]
fn every_spoken_line_plays_to_its_end() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let ticks: u32 = std::env::var("NF_SWEEP_TICKS").ok().and_then(|s| s.parse().ok()).unwrap_or(5400);
    let levels: Vec<u32> = match std::env::var("NF_SWEEP_LEVELS") {
        Ok(s) => s.split(',').filter_map(|x| u32::from_str_radix(x.trim().trim_start_matches("0x"), 16).ok()).collect(),
        Err(_) => LEVELS.to_vec(),
    };
    let mut all_cut = Vec::new();
    for level in levels {
        let s = sweep_level(&iso, level, ticks);
        eprintln!("[sweep] {level:08x}: {} spoken lines seen, {} cut, {} taken by the driver's steal rules, {} starts turned down",
            s.seen, s.cut.len(), s.stolen.len(), s.dropped.len());
        all_cut.extend(s.cut);
    }
    assert!(all_cut.is_empty(), "spoken lines cut short:\n{}", all_cut.join("\n"));
}

/// Probe (ignored): every level's sound triggers (the OFF-less ones ride ch 0 writes).
#[test]
#[ignore]
fn probe_sound_triggers() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    for level in LEVELS {
        let w = Walker::new(nf_view::load(&iso, &format!("{level:08x}.bin"), false).unwrap());
        for t in &w.scene.sim.as_ref().unwrap().drones.sound_triggers.list {
            eprintln!("[trig] {level:08x} id {:#x} on {:#x} off {:#x} one_shot {} 2d {} global {} keep {}", t.id, t.on, t.off, t.one_shot, t.is2d, t.global, t.keep_on);
        }
    }
}
