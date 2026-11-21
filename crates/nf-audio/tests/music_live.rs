//! Opt-in (NF_ISO): 07000005's music (track 15 = PS2/MUSIC/MFX_0/MFX_1) through the stream player.

use nf_audio::{music::Player, Cmd};
use nf_formats::iso9660::IsoImage;
use nf_formats::music::{music_path, track_file, Smf};
use std::sync::Arc;

#[test]
fn castle_music_flows_jumps_and_ends() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap();
    let get = |p: &str| iso.read_file(iso.find(p).unwrap()).unwrap();
    let n = track_file(&get("PS2/MFXINFO.MXI"), 15).unwrap().unwrap();
    assert_eq!(music_path(n, "SMF"), "PS2/MUSIC/MFX_0/MFX_1.SMF");
    let smf = Smf::parse(&get(&music_path(n, "SMF"))).unwrap();
    let ssd = Arc::new(get(&music_path(n, "SSD")));
    assert_eq!((ssd.len(), smf.base_volume), (10_749_696, 99));
    let mut out = Vec::new();
    let mut p = Player::start(smf, ssd, 0, &mut out);
    // The game's level-start volume: 70 · 100 · 99 → 69 → 47 → SPU 7700 / 16383.
    assert!((p.gain(70, 100, 10000) - 7700.0 / 16383.0).abs() < 1e-4);
    let (mut played, mut queued) = (0u64, 0u64);
    let frame = |p: &mut Player, played: &mut u64, queued: &mut u64| {
        *played = (*played + 533).min(*queued);
        let mut out = Vec::new();
        p.update(*played, &mut out);
        for c in out { match c { Cmd::MusicAppend(v) => *queued += v.len() as u64, Cmd::MusicTruncate(at) => *queued = at, _ => {} } }
    };
    for c in &out { if let Cmd::MusicAppend(v) = c { queued += v.len() as u64; } }
    // 30 s: never runs dry (the read-ahead keeps a lap queued), still the start section reported.
    for _ in 0..1800 { frame(&mut p, &mut played, &mut queued); assert!(queued > played, "underrun at {played}"); }
    assert_eq!(p.now_playing, 0);
    // Combat: section 4 is instant — reported at once, the new data 14 336 samples ahead of the playhead.
    p.request(4, false);
    let before = played;
    frame(&mut p, &mut played, &mut queued);
    assert_eq!((p.now_playing, p.last_jump), (4, 4));
    assert!(queued >= before + 14_336);
    for _ in 0..600 { frame(&mut p, &mut played, &mut queued); }
    // Stealth 6 is deferred: cut at the last bar of the first read-ahead buffer after the one being written (≥ one
    // lap, 0.84 s; later when that lap has no bar), then reported.
    p.request(6, false);
    let t0 = played;
    let mut at = None;
    for _ in 0..400 { frame(&mut p, &mut played, &mut queued); if at.is_none() && p.now_playing == 6 { at = Some(played - t0); } }
    let at = at.expect("deferred jump completed");
    eprintln!("deferred jump after {:.2} s", at as f64 / 32000.0);
    assert!((26_880..=128_000).contains(&at), "{at}");
    // End sting 9 (instant, ends with a type-9 marker): the stream stops about its 4.7 s later.
    p.request(9, true);
    let t1 = played;
    let mut stop = None;
    for _ in 0..800 { frame(&mut p, &mut played, &mut queued); if stop.is_none() && p.stopped { stop = Some(played - t1); } }
    let stop = stop.expect("stopped");
    eprintln!("stopped {:.2} s after the end request", stop as f64 / 32000.0);
    // Sister: not before the sting's 4.7 s (+ the 0.448 s lead).
    assert!(stop as f64 / 32000.0 > 4.7 && (stop as f64 / 32000.0) < 6.5, "{stop}");
}

/// 07000005's track (MFX_1) through the player, one 60 Hz frame = 533 samples at 32 kHz.
fn castle() -> Option<(Smf, Arc<Vec<u8>>)> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    let get = |p: &str| iso.read_file(iso.find(p).unwrap()).unwrap();
    let n = track_file(&get("PS2/MFXINFO.MXI"), 15).unwrap().unwrap();
    Some((Smf::parse(&get(&music_path(n, "SMF"))).unwrap(), Arc::new(get(&music_path(n, "SSD")))))
}

fn step(p: &mut Player, played: &mut u64, queued: &mut u64) {
    *played = (*played + 533).min(*queued);
    let mut out = Vec::new();
    p.update(*played, &mut out);
    for c in out { match c { Cmd::MusicAppend(v) => *queued += v.len() as u64, Cmd::MusicTruncate(at) => *queued = at, _ => {} } }
}

fn started(smf: Smf, ssd: Arc<Vec<u8>>) -> (Player, u64, u64) {
    let mut out = Vec::new();
    let p = Player::start(smf, ssd, 0, &mut out);
    let queued = out.iter().map(|c| if let Cmd::MusicAppend(v) = c { v.len() as u64 } else { 0 }).sum();
    (p, 0, queued)
}

/// `HasJustJumped` starts clear: `SFXInitialiseStreamUpdate` 0x92dc:46 zeroes the stream, and only
/// `ES_JumpRequestComplete` 0x9cc8 sets it [M]. So a deferred request right after the music starts
/// is taken at once: it cuts the first read-ahead buffer after the one being written at its last
/// jump marker (section 0's bar at 0x2cf25, buffer 5 [D MFX_1.SMF marker 1]) in that frame. The old
/// player refused deferred requests for a ring lap (26 880 samples, about 50 frames) after the start.
#[test]
fn a_deferred_request_at_the_music_start_is_taken_at_once() {
    let Some((smf, ssd)) = castle() else { eprintln!("skip: NF_ISO not set"); return };
    assert_eq!((smf.starts[1].instant, smf.markers[1].pos, smf.markers[1].mtype), (false, 0x2cf25, 0), "[D MFX_1.SMF]");
    let (mut p, mut played, mut queued) = started(smf, ssd);
    p.request(1, false);
    step(&mut p, &mut played, &mut queued);
    assert_eq!(p.last_jump, 1, "the cut is planned in the request's frame (played {played})");
    assert_eq!(p.now_playing, 0, "and reported only when the playhead reaches it");
    // Sister: the jump completes at the cut, about 4.2 s in (buffer 5 of a 0.84 s ring).
    let mut at = None;
    for _ in 0..600 { step(&mut p, &mut played, &mut queued); if at.is_none() && p.now_playing == 1 { at = Some(played); } }
    let at = at.expect("the jump completes");
    assert!((5 * 26_880..6 * 26_880 + 533).contains(&at), "{at}");
}

/// After a jump completes, a deferred request waits for the can-jump marker: the end of the buffer
/// holding the new section's second jump marker (`UpdateMusicMarkers` 0xb560:44-53 arms it on the
/// marker after the jump read's first; `SFXUpdateStreams` 0xa1a0:84-90 clears `HasJustJumped` when
/// the playhead passes it) [M]. Section 1 opens with its start marker (type 10 at 0x5e8db) and its
/// first bar at 0x68792, 35 616 samples later [D MFX_1.SMF markers 3, 4]; the old player let the
/// request through one ring lap (26 880 samples) after the cut, cutting section 1 before its first
/// bar had played.
#[test]
fn a_deferred_request_after_a_jump_waits_for_the_new_sections_second_marker() {
    let Some((smf, ssd)) = castle() else { eprintln!("skip: NF_ISO not set"); return };
    let (m3, m4) = (smf.markers[3], smf.markers[4]);
    assert_eq!((m3.name, m3.pos, m3.mtype, m4.name, m4.pos, m4.mtype), (1, 0x5e8db, 10, 1, 0x68792, 0), "[D MFX_1.SMF]");
    assert!(!smf.starts[3].instant, "section 3 is deferred [D]");
    let second = nf_audio::music::samples((m4.pos & !0xff) - (smf.starts[1].pos & !0xff));
    assert_eq!(second, 35_616);
    let (mut p, mut played, mut queued) = started(smf, ssd);
    p.request(1, false);
    let mut done = None;
    for _ in 0..600 { step(&mut p, &mut played, &mut queued); if p.now_playing == 1 { done = Some(played); break; } }
    let done = done.expect("the jump to section 1 completes");
    // The cut lies within the frame before `done`.
    p.request(3, false);
    let mut taken = None;
    for _ in 0..600 { step(&mut p, &mut played, &mut queued); if p.last_jump == 3 { taken = Some(played); break; } }
    let taken = taken.expect("the request is taken once the can-jump marker passes");
    let after = taken - done;
    eprintln!("section 3 taken {after} samples after section 1 began");
    assert!(after + 533 >= second, "not before section 1's first bar has played ({after} < {second})");
    assert!(after <= second + 26_880 + 533, "by the end of the buffer holding that bar ({after})");
    // Sister: one frame before, the request was still waiting (no cut planned).
    assert_ne!(p.now_playing, 3);
}
