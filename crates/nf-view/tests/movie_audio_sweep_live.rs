//! movie-audio-sweep: every movie on the disc, headless and silent. For each PSS the audio layer's
//! own sample counts are read (no speakers): the decoded PCM against the `SShd`/`SSbd` header and
//! the video's picture count (A/V length drift at the end), interior silence runs (gaps) and the
//! tail (early stop), and then the device side (`PcmCursor`, the cpal callback's body) at the
//! common device rates: every source frame reaches the device, the end writes silence instead of
//! wrapping, and the stereo pair lands on the device's front pair at any channel count.
//! Needs a retail image with movie data (`NF_ISO`); prints one table row per movie.

use nf_view::movie::{Movie, PcmCursor, USING_MOVIE};
use std::sync::Arc;

fn iso() -> Option<nf_formats::iso9660::IsoImage> {
    let p = std::env::var("NF_ISO").ok()?;
    nf_formats::iso9660::IsoImage::open(&p).ok()
}

/// The golden per-movie counts [D crates/nf-formats/golden/pss_movies.tsv]: name → (pictures, audio_samples).
fn golden() -> std::collections::HashMap<String, (usize, usize)> {
    let t = include_str!("../../nf-formats/golden/pss_movies.tsv");
    let mut lines = t.lines();
    let head: Vec<&str> = lines.next().unwrap().split('\t').collect();
    let col = |n: &str| head.iter().position(|h| *h == n).unwrap();
    let (pc, ac) = (col("pictures"), col("audio_samples"));
    lines.map(|l| { let f: Vec<&str> = l.split('\t').collect(); (f[0].to_string(), (f[pc].parse().unwrap(), f[ac].parse().unwrap())) }).collect()
}

/// Longest run of exact digital silence (both channels 0) strictly inside the track, in samples,
/// plus the leading and trailing silence.
fn silence(pcm: &[[i16; 2]]) -> (usize, usize, usize) {
    let first = pcm.iter().position(|s| *s != [0, 0]).unwrap_or(pcm.len());
    let last = pcm.iter().rposition(|s| *s != [0, 0]).map_or(0, |i| i + 1);
    let (mut run, mut best) = (0usize, 0usize);
    for s in &pcm[first..last.max(first)] {
        if *s == [0, 0] { run += 1; best = best.max(run); } else { run = 0; }
    }
    (best, first, pcm.len() - last.max(first))
}

#[test]
fn every_movie_audio_matches_its_header_and_its_video() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let gold = golden();
    let mut rows = 0;
    eprintln!("movie     kind rate  ch il   samples  audio_s  pictures video_s  drift_ms  gap_ms  head_ms tail_ms");
    for code in USING_MOVIE {
        let name = format!("{code:08X}");
        let Ok(m) = Movie::load(&iso, &name) else { eprintln!("{name}  (no file on the disc)"); continue };
        let (pics, ..) = nf_formats::mpeg2::picture_counts(&m.demuxed.video);
        let video_s = pics as f64 / 30.0; // frame_rate_code 5 = 30 Hz on every movie [D]
        assert_eq!(m.frame_rate_code, 5, "{name}: 30 fps");
        let g = gold[&format!("{name}.PSS")];
        assert_eq!(pics, g.0, "{name}: picture count vs the golden table [D]");
        let Ok(a) = m.audio() else {
            assert_eq!(g.1, 0, "{name}: the golden table lists audio");
            eprintln!("{name}  -    (no audio stream)          {pics:>8} {video_s:>7.3}");
            rows += 1;
            continue;
        };
        let h = a.header;
        assert_eq!((h.kind, h.rate, h.channels, h.interleave), (1, 48000, 2, 512), "{name}: PCM 48 kHz stereo, 512-byte blocks [D]");
        // The demuxed body is exactly the header's SSbd size: nothing lost or padded by the demux.
        assert_eq!(a.body.len(), h.body_bytes as usize, "{name}: demuxed body vs SSbd size");
        assert_eq!(a.body.len() % (h.interleave as usize * 2), 0, "{name}: whole L/R blocks, no partial tail");
        let pcm = a.decode();
        assert_eq!(pcm.len(), a.samples(), "{name}: decoded vs samples()");
        assert_eq!(pcm.len(), h.body_bytes as usize / 4, "{name}: decoded vs header body / 4");
        assert_eq!(pcm.len(), g.1, "{name}: decoded vs the golden table [D]");
        // The last block decodes (an early stop would leave it zero while the body is not).
        let tail_body = &a.body[a.body.len() - 1024..];
        let tail_pcm = &pcm[pcm.len() - 256..];
        assert_eq!(tail_body.iter().any(|b| *b != 0), tail_pcm.iter().any(|s| *s != [0, 0]), "{name}: the last block decodes");
        let audio_s = a.seconds();
        let drift_ms = (video_s - audio_s) * 1000.0;
        let (gap, head, tail) = silence(&pcm);
        let ms = |n: usize| n as f64 / 48.0;
        eprintln!("{name}  {}   {} {}  {} {:>8} {audio_s:>8.3} {pics:>8} {video_s:>7.3} {drift_ms:>8.1} {:>7.1} {:>7.1} {:>7.1}",
            h.kind, h.rate, h.channels, h.interleave, pcm.len(), ms(gap), ms(head), ms(tail));
        // A/V at the end: the hold is the audio's length, so the picture may run past the audio
        // by at most one picture (it would be cut). Audio longer than the picture only holds the
        // last picture; the disc has two such tracks, 0730000D (101 ms) and 07300017 (72 ms) [D
        // the table above], within a few pictures.
        assert!(drift_ms < 1000.0 / 30.0, "{name}: video {video_s:.3} s runs past the audio {audio_s:.3} s");
        assert!(drift_ms > -110.0, "{name}: audio {audio_s:.3} s runs well past the video {video_s:.3} s");
        // The hold (state 6) is the audio's length in 60 Hz ticks: it covers the whole track.
        let ticks = (audio_s * 60.0).round();
        assert!((ticks / 60.0 - audio_s).abs() <= 1.0 / 120.0, "{name}: hold {ticks} ticks vs {audio_s:.3} s");
        rows += 1;
    }
    assert!(rows >= 22, "only {rows} movies swept");
}

/// Runs a cursor to its end in `buf`-frame device buffers; returns (frames written before done, output).
fn play(src: &Arc<Vec<i16>>, dev_rate: f64, ch: usize, buf: usize) -> (usize, Vec<f32>) {
    let mut c = PcmCursor::new(src.clone(), 48000.0, dev_rate, false, 1.0);
    let mut out = Vec::new();
    let mut d = vec![0f32; buf * ch];
    let mut written = 0;
    while !c.done() {
        d.fill(f32::NAN);
        let before = c.pos();
        c.fill(&mut d, ch);
        // Frames that advanced the cursor carried track audio; the rest of the last buffer is silence.
        let step = 48000.0 / dev_rate;
        written += ((c.pos() - before) / step).round() as usize;
        out.extend_from_slice(&d);
    }
    // After the end: silence, never a wrap back to the start.
    c.fill(&mut d, ch);
    assert!(d.iter().all(|v| *v == 0.0), "the track's end writes silence");
    assert!(out.iter().all(|v| v.is_finite()), "every device sample is written");
    (written, out)
}

#[test]
fn every_movie_track_reaches_the_device_whole() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    eprintln!("movie     src_frames  dev48k_frames  dev44k1_frames  expect44k1  end_ms");
    for code in USING_MOVIE {
        let name = format!("{code:08X}");
        let Ok(m) = Movie::load(&iso, &name) else { continue };
        let Ok(a) = m.audio() else { continue };
        let pcm = a.decode();
        let src: Arc<Vec<i16>> = Arc::new(pcm.iter().flat_map(|[l, r]| [*l, *r]).collect());
        let n = pcm.len();
        // 48 kHz device: one device frame per source frame, sample-exact (the interpolation's last
        // base frame is n - 2, so n - 1 frames go out).
        let (w48, out48) = play(&src, 48000.0, 2, 480);
        assert_eq!(w48, n - 1, "{name}: 48 kHz device frames");
        for k in [0, n / 3, n - 2] {
            assert_eq!(out48[2 * k], pcm[k][0] as f32 / 32768.0, "{name}: L at {k}");
            assert_eq!(out48[2 * k + 1], pcm[k][1] as f32 / 32768.0, "{name}: R at {k}");
        }
        // 44.1 kHz device: the track's length in device time is unchanged (no drift).
        let (w44, _) = play(&src, 44100.0, 2, 441);
        let expect = ((n - 1) as f64 * 44100.0 / 48000.0).ceil() as usize;
        assert!(w44.abs_diff(expect) <= 1, "{name}: 44.1 kHz frames {w44} vs {expect}");
        let end_ms = (w44 as f64 / 44100.0 - n as f64 / 48000.0) * 1000.0;
        assert!(end_ms.abs() < 0.1, "{name}: device length off by {end_ms:.3} ms");
        eprintln!("{name}  {n:>10} {w48:>14} {w44:>15} {expect:>11} {end_ms:>7.3}");
    }
}

/// The stereo pair goes to the device's FRONT pair on any channel count, and a mono device gets
/// both channels mixed — the game mixer's own mapping (nf-audio's music path). The old movie
/// callback wrote L to every even channel and R to every odd one: on a 5.1 device the centre
/// speaker carried the left channel and the LFE the right, and a mono device dropped the right
/// channel entirely.
#[test]
fn the_movie_pair_lands_on_the_front_speakers() {
    let src = Arc::new(vec![8192i16, -16384, 8192, -16384, 8192, -16384, 8192, -16384]);
    let (l, r) = (0.25f32, -0.5f32);
    let mut c = PcmCursor::new(src.clone(), 48000.0, 48000.0, false, 1.0);
    let mut d = vec![9.0f32; 6];
    c.fill(&mut d, 6);
    assert_eq!(d, [l, r, 0.0, 0.0, 0.0, 0.0], "5.1: front L/R only");
    let mut c = PcmCursor::new(src.clone(), 48000.0, 48000.0, false, 1.0);
    let mut d = vec![9.0f32; 8];
    c.fill(&mut d, 8);
    assert_eq!(d, [l, r, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], "7.1: front L/R only");
    let mut c = PcmCursor::new(src.clone(), 48000.0, 48000.0, false, 1.0);
    let mut d = vec![9.0f32; 2];
    c.fill(&mut d, 1);
    assert_eq!(d, [(l + r) * 0.5; 2], "mono: L and R mixed");
    let mut c = PcmCursor::new(src, 48000.0, 48000.0, false, 1.0);
    let mut d = vec![9.0f32; 4];
    c.fill(&mut d, 2);
    assert_eq!(d, [l, r, l, r], "stereo: unchanged");
}
