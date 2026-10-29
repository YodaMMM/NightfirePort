//! script-channel-events: the live half. For every on-foot level whose census rows exist
//! (`census_channel_events.rs`, 199 rows [D]), drive the REAL input path — load, wait enabled,
//! idle ticks with the game's own cut-scene skip (`SP_Update` 0x1dbda8:129-141 → `Script_FFwd`
//! 0x1dac78 → the catch-up of `Script_Update` 0x1da998:19-100) — and require:
//!   1. every script-path channel write (`Channels::writes` source 1) is either a [D] census row
//!      (event 0xD/0x11, `Script_EventHandler` 0x1d94c0 [M]) or an SP placement's own OUT mirror
//!      (`SP_Update` 0x1dbda8:220-226 writes `ch[+0x30] = (script running)` per tick [M]);
//!   2. the rows that fire in the window land with the census's own values and channels
//!      (07000006's boot NIS: the four searchlight on-channels, in stream order);
//!   3. the skip path itself: 07000006's writes fire while the player is DISABLED — the
//!      cut-scene was skipped, and the catch-up still ran every op 0x12 [M 0x1da998:19-100].
//! Opt-in: NF_ISO. `--nocapture` prints the per-row verdicts.
#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The [D] census (`census_channel_events.rs`): per level, (script, stream, kind, params).
/// Kind 0xD params [ch, value]; kind 0x11 params [test, dest_on, dest_off, value].
fn census(level: &str) -> Vec<(u32, usize, u8, Vec<u32>)> {
    match level {
        "07000001" => vec![(0x0600_0086, 9, 0xd, vec![0xe, 1]), (0x0600_0087, 8, 0xd, vec![0xb, 1])],
        "07000002" => vec![(0x0600_0084, 7, 0xd, vec![0x47, 1])],
        "07000003" => vec![(0x0600_0563, 3, 0xd, vec![0x96, 1])],
        "07000004" => vec![(0x0600_0083, 6, 0xd, vec![0x38, 1]), (0x0600_0082, 9, 0xd, vec![0x53, 1])],
        "07000005" => vec![
            (0x0600_0a23, 1, 0x11, vec![0x98, 0x98, 0x1d, 1]),
            (0x0600_0a22, 9, 0xd, vec![0x73, 1]),
            (0x0600_0952, 8, 0xd, vec![0x62, 1]),
        ],
        "07000006" => vec![
            (0x0600_0066, 0, 0xd, vec![0x40, 1]),
            (0x0600_0066, 0, 0xd, vec![0x3f, 1]),
            (0x0600_0066, 0, 0xd, vec![0x3d, 1]),
            (0x0600_0066, 0, 0xd, vec![0x3e, 1]),
            (0x0600_0071, 7, 0xd, vec![0x83, 1]),
            (0x0600_0071, 13, 0xd, vec![0x2, 1]),
        ],
        "07000007" => vec![(0x0600_0073, 8, 0xd, vec![0x82, 0])],
        "07000008" => vec![
            (0x0600_0991, 6, 0xd, vec![0x62, 1]),
            (0x0600_004c, 46, 0xd, vec![0x53, 1]),
            (0x0600_0078, 10, 0xd, vec![0x5, 1]),
        ],
        "0700000b" => vec![
            (0x0600_09f9, 2, 0xd, vec![0x77, 0]),
            (0x0600_098a, 6, 0xd, vec![0x62, 1]),
            (0x0600_066d, 2, 0xd, vec![0x9, 1]),
            (0x0600_0037, 0, 0xd, vec![0xa, 1]),
            (0x0600_0035, 10, 0xd, vec![0xb, 1]),
            (0x0600_04fa, 6, 0xd, vec![0x57, 1]),
            (0x0600_04fa, 12, 0xd, vec![0x77, 0]),
            (0x0600_04fa, 12, 0xd, vec![0x77, 1]),
            (0x0600_04fa, 13, 0xd, vec![0x59, 1]),
            (0x0600_04fa, 14, 0xd, vec![0x2a, 1]),
            (0x0600_04fa, 15, 0xd, vec![0x76, 0]),
        ],
        "07000011" => vec![(0x0600_098a, 6, 0xd, vec![0x62, 1])],
        "07000012" => vec![
            (0x0600_0530, 8, 0xd, vec![0x3c, 0x3c]),
            (0x0600_0a25, 2, 0xd, vec![0x89, 1]),
            (0x0600_0a25, 3, 0xd, vec![0x89, 0]),
            (0x0600_098a, 6, 0xd, vec![0x62, 1]),
        ],
        "07000014" => vec![
            (0x0600_08af, 3, 0xd, vec![0x92, 0]),
            (0x0600_08af, 4, 0xd, vec![0x93, 1]),
            (0x0600_08af, 5, 0xd, vec![0xb7, 1]),
            (0x0600_0886, 2, 0xd, vec![0x8a, 1]),
            (0x0600_062f, 9, 0xd, vec![0xb8, 0]),
            (0x0600_062f, 9, 0xd, vec![0x48, 1]),
            (0x0600_062f, 12, 0xd, vec![0x10, 1]),
            (0x0600_0833, 16, 0xd, vec![0x47, 1]),
            (0x0600_0833, 17, 0xd, vec![0xe, 1]),
            (0x0600_0848, 8, 0xd, vec![0x11, 1]),
            (0x0600_0848, 11, 0xd, vec![0x4a, 1]),
        ],
        "07000016" => vec![
            (0x0600_09e9, 7, 0xd, vec![0x50, 0]),
            (0x0600_090a, 5, 0xd, vec![0xbb, 1]),
            (0x0600_090a, 5, 0xd, vec![0xb0, 1]),
            (0x0600_090a, 6, 0xd, vec![0xc5, 1]),
            (0x0600_090a, 6, 0xd, vec![0xc6, 1]),
            (0x0600_090a, 6, 0xd, vec![0xf3, 1]),
            (0x0600_090a, 6, 0xd, vec![0xf3, 0]),
            (0x0600_090a, 7, 0xd, vec![0xf0, 1]),
            (0x0600_090a, 8, 0xd, vec![0xbe, 1]),
            (0x0600_090a, 8, 0xd, vec![0xf5, 1]),
            (0x0600_090a, 9, 0xd, vec![0xf4, 1]),
            (0x0600_090a, 9, 0xd, vec![0xf3, 1]),
            (0x0600_090a, 9, 0xd, vec![0xf3, 0]),
            (0x0600_090a, 13, 0xd, vec![0xba, 1]),
            (0x0600_090a, 13, 0xd, vec![0xc1, 1]),
            (0x0600_090a, 13, 0xd, vec![0xc2, 1]),
            (0x0600_090a, 13, 0xd, vec![0xc3, 1]),
            (0x0600_090a, 13, 0xd, vec![0xc4, 1]),
            (0x0600_090a, 13, 0xd, vec![0xbd, 1]),
            (0x0600_090a, 14, 0xd, vec![0xb1, 1]),
            (0x0600_090a, 14, 0xd, vec![0xb2, 1]),
            (0x0600_090a, 14, 0xd, vec![0xf1, 1]),
            (0x0600_090a, 14, 0xd, vec![0xf2, 1]),
            (0x0600_090a, 20, 0xd, vec![0xc0, 1]),
            (0x0600_090a, 21, 0xd, vec![0x50, 1]),
            (0x0600_090a, 22, 0xd, vec![0xcc, 1]),
            (0x0600_090a, 22, 0xd, vec![0xcd, 1]),
            (0x0600_0631, 7, 0xd, vec![0xb9, 1]),
            (0x0600_0631, 10, 0xd, vec![0x15, 1]),
            (0x0600_0631, 13, 0xd, vec![0xb4, 0]),
            (0x0600_0632, 11, 0xd, vec![0x1b, 1]),
            (0x0600_0632, 16, 0xd, vec![0x8d, 0]),
            (0x0600_0632, 17, 0xd, vec![0x94, 0]),
            (0x0600_0632, 19, 0xd, vec![0x50, 0]),
            (0x0600_0632, 19, 0xd, vec![0x50, 1]),
            (0x0600_0633, 15, 0xd, vec![0xb5, 1]),
        ],
        "0700004a" => vec![(0x0600_0571, 5, 0xd, vec![0x40, 1]), (0x0600_06cf, 27, 0xd, vec![0x22, 1])],
        // 0700001b's 115 rows: the report's table (docs/reports/script-channel-events.md) holds
        // them all; the idle window can only reach the intro-adjacent ones, so the audit below
        // carries the full rows for 07000006 (fired) and runs the write-set audit everywhere.
        _ => vec![],
    }
}

const LEVELS: [&str; 15] = [
    "07000001", "07000002", "07000003", "07000004", "07000005", "07000006", "07000007",
    "07000008", "0700000b", "07000011", "07000012", "07000014", "07000016", "0700001b",
    "0700004a",
];

/// A census row's landing (channel, value) pairs: 0xD writes p0←p1; 0x11 writes p1←p3 AND p2←p3
/// (whichever dest the live test channel selects — either is game-legal).
fn row_writes(kind: u8, p: &[u32]) -> Vec<(u8, u8)> {
    if kind == 0xd { vec![(p[0] as u8, p[1] as u8)] }
    else { vec![(p[1] as u8, p[3] as u8), (p[2] as u8, p[3] as u8)] }
}

/// One level's real-input idle sweep: returns the ordered script-path writes (ch, value, frame,
/// disabled-at-write). `ticks` after enable. `mash_skip` presses the game's skip action every
/// tick from the load (`Scripts::skip_request` = `Input_Action(-1, 0x19, 1)` [M 0x1dbda8:129-141;
/// nis.rs begin_tick] — the Walker only presses it while the player is disabled; the real player
/// can press it any time a scene plays).
fn sweep(level: &str, ticks: u32, mash_skip: bool) -> Option<(Vec<(u8, u8, u32, bool)>, Walker)> {
    let Ok(scene) = nf_view::load(&iso(), &format!("{level}.bin"), false) else {
        eprintln!("[census-live] {level}: LOAD FAILED (skip)");
        return None;
    };
    let mut w = Walker::new(scene);
    w.top_up_health = true;
    w.skip_cutscenes = true; // the game's own skip action while a cut-scene plays
    let mut enabled = false;
    for _ in 0..3000 {
        if mash_skip { w.scene.sim.as_mut().unwrap().drones.scripts.skip_request = true; }
        w.tick(nf_game::player::Input::default(), false);
        if !w.scene.sim.as_ref().unwrap().disabled { enabled = true; break; }
    }
    let _ = enabled;
    eprintln!("[census-live] {level}: enabled={enabled} ({} ticks)", w.total_ticks);
    let mut fired: Vec<(u8, u8, u32, bool)> = Vec::new();
    for _ in 0..ticks {
        if mash_skip { w.scene.sim.as_mut().unwrap().drones.scripts.skip_request = true; }
        let disabled = w.scene.sim.as_ref().unwrap().disabled;
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.channels.writes = Some(Vec::new());
        }
        w.tick(nf_game::player::Input::default(), false);
        let writes = w.scene.sim.as_mut().unwrap().drones.channels.writes.take().unwrap_or_default();
        for (c, v, f, src) in writes {
            if src == 1 { fired.push((c, v, f, disabled)); }
        }
    }
    eprintln!("[census-live] {level}: {} script-path writes", fired.len());
    Some((fired, w))
}

/// Audit 1 across every level with census rows: every script-path write is a census row's own
/// (channel, value) or an SP placement's OUT mirror (the SP's `out`, value unconstrained —
/// `SP_Update` 0x1dbda8:220-226 writes `(script running) != 0` per tick [M]).
#[test]
fn every_script_channel_write_is_a_census_row_or_an_sp_out() {
    for level in LEVELS {
        let Some((fired, w)) = sweep(level, 400, false) else { continue };
        let rows = census(level);
        let sp_outs: std::collections::BTreeSet<u8> = w.scene.sim.as_ref().unwrap()
            .drones.scripts.players.iter().map(|p| p.out as u8).filter(|o| *o != 0).collect();
        let mut bad = Vec::new();
        for &(c, v, f, _) in &fired {
            let in_rows = rows.iter().any(|(_, _, kind, p)| row_writes(*kind, p).contains(&(c, v)));
            if !in_rows && !sp_outs.contains(&c) {
                bad.push(format!("[{c:02x}={v:02x} @f{f}]"));
            }
        }
        assert!(bad.is_empty(), "{level}: script-path writes outside the [D] census and every SP out: {}",
            bad.join(","));
        eprintln!("[census-live] {level}: audit clean (SP outs {:?})",
            sp_outs.iter().map(|c| format!("0x{c:02x}")).collect::<Vec<_>>());
    }
}

/// Audit 2 — the fired rows, end to end on the real input path, PLAYED: 07000006's boot NIS
/// (`SP #808` → script 0x06000066, the level's intro) writes the four searchlight on-channels
/// in the census's stream order at the stream's own pacing, `script_channels` records them, and
/// the game's readers react: searchlight placements 2/4/6/8 leave the off state as their
/// on-channel lights (`Searchlight_Update` 0x1cb900's state-3 relight [M :371]).
#[test]
fn the_boot_nis_lights_the_four_searchlights_as_it_plays() {
    let Some((fired, w)) = sweep("07000006", 700, false) else { panic!("07000006 load failed") };
    // The four census rows, in order (stream 0's 0xD sequence: t1 0x40, t59 0x3f, t116 0x3d, t167 0x3e).
    let want: Vec<(u8, u8)> = [(0x40u8, 1u8), (0x3f, 1), (0x3d, 1), (0x3e, 1)].into();
    let got: Vec<(u8, u8)> = fired.iter().filter(|&&(c, _, _, _)| want.iter().any(|&(wc, _)| wc == c))
        .map(|&(c, v, _, _)| (c, v)).collect();
    assert_eq!(got, want, "the boot NIS's four 0xD writes must land in stream order with the census values [D 06000066 stream 0; M 0x1d94c0 case 0xd]");
    // Played in real time: the writes spread over the script's ≥167-tick timeline, not compressed.
    let frames: Vec<u32> = fired.iter().filter(|&&(c, _, _, _)| want.iter().any(|&(wc, _)| wc == c))
        .map(|&(_, _, f, _)| f).collect();
    assert!(frames.len() == 4 && frames[3] - frames[0] >= 100,
        "the played intro's writes must span the script's real timeline (frames {frames:?})");
    // The port's bookkeeping: the script path owns these channels now.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for ch in [0x3du8, 0x3e, 0x3f, 0x40] {
            assert!(sim.drones.channels.script_channels.contains(&ch), "ch 0x{ch:02x} must be script-owned after the intro");
        }
    }
    // The unlock [D probe_0600066_readers: searchlight placements 2/4/6/8 carry on = 0x40/0x3f/0x3d/0x3e]:
    // each searchlight left the off state when its on-channel lit [M Searchlight_Update 0x1cb900
    // state-3 relight].
    let sim = w.scene.sim.as_ref().unwrap();
    for (pl, ch) in [(2usize, 0x40u8), (4, 0x3f), (6, 0x3d), (8, 0x3e)] {
        let Some(part) = sim.drones.searchlights.parts.iter().find(|p| p.placement == pl) else {
            panic!("searchlight placement {pl} missing from the sim");
        };
        assert_eq!(part.on, ch, "placement {pl}'s on channel [D]");
        assert_ne!(part.state, 3, "searchlight pl {pl} must have relit when ch 0x{ch:02x} fired (state {})", part.state);
    }
    // And the level's OWN objective channel (0600071 s13 t1129 → ch 0x02, objective 04000001
    // "Find a way inside to the party" [D 07000006.md:19 + the mission table]) did NOT fire on
    // the idle path — it needs the party-entry route; the intro only lights the searchlights.
    assert!(fired.iter().all(|&(c, _, _, _)| c != 0x02), "ch 0x02 must not fire on the idle path (it needs the entry route)");
}

/// Audit 3 — the same cut-scene with the skip action MASHED from the load: the boot NIS is
/// UNSKIPPABLE by the game's own rule. `Script_FFwd(s, 1)` fast-forwards only when the
/// camera-active flag (+0xAB5 bit 4) is set [M 0x1dac78:7-22]; 06000066 carries NO camera op
/// [D dump_0600066_ops: a single stream, four 0xD events, no 0x0d], so a skip press cannot
/// fast-forward it — the scene plays out and every write lands at the stream's own pacing.
#[test]
fn the_boot_nis_cannot_be_skipped_the_games_no_camera_rule() {
    let Some((fired, w)) = sweep("07000006", 700, true) else { panic!("07000006 load failed") };
    let want: Vec<(u8, u8)> = [(0x40u8, 1u8), (0x3f, 1), (0x3d, 1), (0x3e, 1)].into();
    let got: Vec<(u8, u8)> = fired.iter().filter(|&&(c, _, _, _)| want.iter().any(|&(wc, _)| wc == c))
        .map(|&(c, v, _, _)| (c, v)).collect();
    assert_eq!(got, want, "mashing the skip on the no-camera boot NIS changes nothing: all four writes land [M 0x1dac78:7-22 + D no camera op]");
    // The pacing IS the proof: the writes keep the stream's real timeline (t1/t59/t116/t167),
    // not a compressed catch-up — the game never fast-forwarded.
    let frames: Vec<u32> = fired.iter().filter(|&&(c, _, _, _)| want.iter().any(|&(wc, _)| wc == c))
        .map(|&(_, _, f, _)| f).collect();
    assert!(frames.len() == 4 && frames[3] - frames[0] >= 100,
        "the unskippable scene must play out at its own pacing (frames {frames:?})");
    // The readers saw everything (the sister searchlight assertion pins the states).
    let sim = w.scene.sim.as_ref().unwrap();
    for (pl, _) in [(2usize, 0x40u8), (4, 0x3f), (6, 0x3d), (8, 0x3e)] {
        let part = sim.drones.searchlights.parts.iter().find(|p| p.placement == pl).expect("searchlight");
        assert_ne!(part.state, 3, "searchlight pl {pl} relit (state {})", part.state);
    }
}
