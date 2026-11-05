//! mission-outros: the end-of-mission cut-scenes — the outro's subtitle script starts beside the
//! movie and the state-6 hold carries it, for both reported endings:
//! - 07000004 (Double Cross, the mission's last part): the kill → `Mission_Update` 0x182c60 case 6
//!   → `LevelToEndTo 0x0730004` → case 4's load → the movie branch [M 0x1d6360: the script switch
//!   pairs 0x0730004 with the SP_Create'd subtitle script 0x0600095d; `psiStartBackgroundMovie`
//!   plays the PSS; `psiMovieFinished` is `BGFMVPlaying == 0` [M 0x1082e8]] → phase 2 routes to
//!   the front-end map [M 0x1d3560] with the queued results page [M 0x1d6360:160-163].
//! - 0700004a (Phoenix Fire part 3 — NOT the mission's last part [L the mission table:
//!   mission 0x07000011, part 2; part 3 = 07000013]): its ending is the LEVEL end — the pack
//!   cut-scene's script 0x060006cf fires event 0x12 with the movie code 0x072004a [D
//!   0780004a.bin], which resolves through the briefing to 07000013 [M 0x1d3560:
//!   0x720004a → 0x7100013 → masked] — and the MISSION ends on 0700013 with the outro 0x0730013
//!   (the subtitle script 0x06000968 [D 07000500.bin]) [M 0x182c60 case 6's 0x7000013 arm].
//!   0700004a's own data writes no 0x4a (the objective channel), so the mission cannot complete
//!   there [D — this file re-scans the parsed scene].
//!
//! The slim on-foot image's /MOVIES/*.PSS are 0-byte stubs [D — the ISO tree], and the game
//! itself skips a movie whose open fails (`BGFMVPlaying` stays 0 [M 0x1081a8's strFileOpen arm]);
//! there the port's no-movie hold runs the subtitle track's length [S — the stand-in]. On an
//! image with the real movies the open SUCCEEDS [M 0x1081a8: BGFMVPlaying = 1] and state 6 lasts
//! the PSS's own life — BGFMVPlaying clears only at the decode pump's end of file [M 0x108060;
//! psiMovieFinished = BGFMVPlaying == 0, 0x1082e8] — so the hold runs the movie's real length
//! [D the PSS's audio track: 07300004.PSS = 2 384 128 PCM samples at 48 kHz = 2980 ticks;
//! 07300013.PSS = 2 040 064 = 2550 ticks, golden/pss_movies.tsv], and the tests assert that when
//! the file is present (both lengths measured off the image at run time).

use nf_view::menu::{self, FrontEnd, Outcome, Stats};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// USATxt.dat (the NTSC-U default [D FILES.BIN; FNames 0x244680]) — the subtitle labels' bank.
fn lang(iso_path: &str) -> nf_formats::hud::Lang {
    let img = nf_formats::iso9660::IsoImage::open(iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("USATxt.dat")).expect("USATxt.dat");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    nf_formats::hud::Lang::parse(&data).expect("parse USATxt.dat")
}

/// The host's real per-frame glue + one front-end tick.
fn frame(fe: &mut FrontEnd, slot: &mut Option<nf_view::movie::Player>, iso: &str) {
    nf_view::mission_movie_glue(fe, slot, iso);
    fe.tick();
}

/// The image's PSS length for a movie code, in 60 Hz game ticks: the audio track's duration —
/// the game's master clock [M audioDecSendToIOP 0x107100; main 0x109080's decode pump]. None
/// when the image stubs the file (the slim on-foot ISO's /MOVIES are 0-byte [D]) — the game's
/// own open fails there [M 0x1081a8's strFileOpen arm] and state 6 pops at once [M 0x1082e8].
fn movie_ticks60(iso_path: &str, code: u32) -> Option<u32> {
    let img = nf_formats::iso9660::IsoImage::open(iso_path).expect("open iso");
    let m = nf_view::movie::Movie::load(&img, &format!("{code:08X}")).ok()?;
    let secs = m.audio().expect("the movie has an audio stream").seconds();
    Some((secs * 60.0).round() as u32)
}

/// The Double Cross outro: the track's disc data + the script start + the hold + the resolve.
#[test]
fn the_double_cross_outro_runs_its_subtitle_script() {
    let iso_v = iso();
    // The track off the disc [D 07000500.bin 0600095d — the script 0x1d6360's switch names for
    // 0x0730004]: length 2695, the first entry at tick 249 (show 403).
    let tr = menu::subtitle_track(&iso_v, 0x0730_0004).expect("the 0x0730004 subtitle track exists");
    assert_eq!(menu::outro_script_for(0x0730_0004), 0x0600_095d, "the switch arm [M 0x1d6360]");
    assert_eq!(tr.length, 2695, "the script header's length [D 07000500.bin]");
    let first = *tr.entries.first().expect("the track has entries");
    assert_eq!((first.tick, first.show), (249, 403), "the first Text's stamp [D]");
    assert_eq!(first.label, 0x0600_0061, "the first Text's label [D]");
    assert_eq!(tr.entries.len(), 8, "the track's Text entries [D 0600095d — the last Text has no following Marker]");
    let lg = lang(&iso_v);
    let text = lg.bind(first.label).expect("the subtitle label binds in USATxt.dat [D]");
    assert!(!text.is_empty(), "the label's string is non-empty");
    eprintln!("[mo] 0600095d: {} entries, first \"{}\" at t{}", tr.entries.len(), text, first.tick);

    // The host-shaped end sequence: mission_ended → the outro code wraps to a movie; the glue
    // loads the track and starts the PSS (or, with the file a stub, arms the no-movie hold).
    let mut fe = FrontEnd::new();
    fe.flow.current_level = 0x0700_0004;
    fe.flow.stack = vec![menu::ST_RUNNER]; // the host's demo_results boots the runner state
    fe.mission_ended(Outcome::Complete, Stats::default());
    assert_eq!(fe.requested_level.take(), Some(0x0730_0004), "LevelToEndTo [M 0x182c60:89-90]");
    assert!(fe.flow.movie_pending, "the 0x73 code wraps to a movie [M 0x1d6360]");
    let mut slot: Option<nf_view::movie::Player> = None;
    let mut movie_ticks: u32 = 0;
    let mut saw_first_subtitle = false;
    let mut resolved = false;
    for _ in 0..6000u32 {
        frame(&mut fe, &mut slot, &iso_v);
        if fe.flow.get_state() == menu::ST_MOVIE {
            movie_ticks += 1;
            if fe.flow.outro_sub_active.map_or(false, |(l, _)| l == first.label) { saw_first_subtitle = true; }
        }
        if fe.flow.phase2_resolved.is_some() { resolved = true; break; }
    }
    assert!(resolved, "the outro resolved (the state-6 hold ran its course)");
    // The hold is the movie's own life: BGFMVPlaying = 1 on the successful open [M 0x1081a8] and
    // 0 only at the decode pump's end of file [M 0x108060], so state 6 lasts the PSS's real
    // duration [D — 07300004.PSS: 2 384 128 PCM samples at 48 kHz = 49.67 s = 2980 ticks,
    // golden/pss_movies.tsv] with the subtitle script running beside it [M 0x1d6360's SP_Create].
    // On a stub image the open fails and the game pops at once [M 0x1081a8] — the port's
    // no-movie hold then runs the track's length [S], which still outlasts the entries.
    match movie_ticks60(&iso_v, 0x0730_0004) {
        Some(pss) => assert!(movie_ticks.abs_diff(pss) <= 2,
            "the hold ran the movie's real length [M 0x1081a8 + 0x108060 + 0x1082e8; \
             D the PSS's audio track], hold {movie_ticks} vs PSS {pss}"),
        None => eprintln!("[mo] 07300004.PSS is a stub on this image — the [S] track-length stand-in held"),
    }
    assert!(movie_ticks >= 2690,
        "the outro stayed up through the track's entries [D the track's 9th entry completes at \
         t2690; the PSS runs 2980 ticks [D], the stand-in hold runs the track's 2695 [S]], \
         got {movie_ticks}");
    assert!(saw_first_subtitle, "the subtitle script's first Text showed [M 0x1d6360's SP_Create]");
    assert_eq!(fe.flow.phase2_resolved.take(), Some(menu::FRONTEND_LEVEL),
        "the outro routes to the front-end map [M 0x1d3560: 0x7300004 → 0x7000048]");
    assert!(fe.flow.load_tick(), "phase 3 done");
    assert!(fe.reopen_queued_page(), "the load tail opens the results [M 0x1d6360:160-163]");
    assert_eq!(fe.page, menu::Page::Results, "the results page, after the outro");
}

/// The Phoenix Fire (Tower-2) mission outro on 07000013 — the same shape, the 0x06000968 track.
#[test]
fn the_phoenix_fire_outro_runs_its_subtitle_script() {
    let iso_v = iso();
    assert_eq!(menu::outro_for(0x0700_0013), 0x0730_0013, "the mission's last part routes to its outro [M 0x182c60:107-110]");
    assert_eq!(menu::outro_script_for(0x0730_0013), 0x0600_0968, "the switch arm [M 0x1d6360]");
    let tr = menu::subtitle_track(&iso_v, 0x0730_0013).expect("the 0x0730013 subtitle track exists");
    assert_eq!(tr.length, 2292, "the script header's length [D 07000500.bin]");
    let first = *tr.entries.first().expect("the track has entries");
    assert_eq!((first.tick, first.show), (468, 338), "the first Text's stamp [D]");
    assert_eq!(first.label, 0x0600_00f1, "the first Text's label [D]");
    assert_eq!(tr.entries.len(), 4, "the track's Text entries [D 06000968]");
    let lg = lang(&iso_v);
    let text = lg.bind(first.label).expect("the subtitle label binds [D]");
    assert!(!text.is_empty());
    eprintln!("[mo] 06000968: {} entries, first \"{}\" at t{}", tr.entries.len(), text, first.tick);

    let mut fe = FrontEnd::new();
    fe.flow.current_level = 0x0700_0013;
    fe.flow.stack = vec![menu::ST_RUNNER];
    fe.mission_ended(Outcome::Complete, Stats::default());
    assert_eq!(fe.requested_level.take(), Some(0x0730_0013), "LevelToEndTo [M 0x182c60:109-110]");
    let mut slot: Option<nf_view::movie::Player> = None;
    let mut movie_ticks: u32 = 0;
    let mut saw_first_subtitle = false;
    let mut resolved = false;
    for _ in 0..6000u32 {
        frame(&mut fe, &mut slot, &iso_v);
        if fe.flow.get_state() == menu::ST_MOVIE {
            movie_ticks += 1;
            if fe.flow.outro_sub_active.map_or(false, |(l, _)| l == first.label) { saw_first_subtitle = true; }
        }
        if fe.flow.phase2_resolved.is_some() { resolved = true; break; }
    }
    assert!(resolved, "the outro resolved");
    // The hold is the movie's own life [M 0x1081a8 + 0x108060 + 0x1082e8]: the PSS's real
    // duration [D — 07300013.PSS: 2 040 064 PCM samples at 48 kHz = 42.5 s = 2550 ticks,
    // golden/pss_movies.tsv]; the stub image takes the [S] track-length stand-in instead.
    match movie_ticks60(&iso_v, 0x0730_0013) {
        Some(pss) => assert!(movie_ticks.abs_diff(pss) <= 2,
            "the hold ran the movie's real length [M 0x1081a8 + 0x108060 + 0x1082e8; \
             D the PSS's audio track], hold {movie_ticks} vs PSS {pss}"),
        None => eprintln!("[mo] 07300013.PSS is a stub on this image — the [S] track-length stand-in held"),
    }
    assert!(movie_ticks >= 2280,
        "the outro stayed up through the track's entries [D the track's last full entry completes \
         at t2286; the PSS runs 2550 ticks [D], the stand-in hold runs the track's 2292 [S]], \
         got {movie_ticks}");
    assert!(saw_first_subtitle, "the subtitle script's first Text showed [M 0x1d6360's SP_Create]");
    assert_eq!(fe.flow.phase2_resolved.take(), Some(menu::FRONTEND_LEVEL),
        "the outro routes to the front-end map [M 0x1d3560]");
    assert!(fe.flow.load_tick(), "phase 3 done");
    assert!(fe.reopen_queued_page(), "the results [M 0x1d6360:160-163]");
    assert_eq!(fe.page, menu::Page::Results);
}

/// The 0700004a ending is the LEVEL end only: the pack cut-scene's script hands the 0x072004a
/// movie code, which chains to 07000013 — where the MISSION ends with its own outro. 4a's own
/// data writes no 0x4a, so the mission layer cannot complete there.
#[test]
fn the_4a_level_end_chains_to_13_and_the_mission_stays_open() {
    let iso_v = iso();
    // The chain [M 0x1d3560]: the pack cut-scene's movie code → the 13 briefing → masked to the
    // level. (The wrap switch also wraps a plain 0x070004a exit to the 0x071004a briefing
    // [M 0x1d3800].)
    assert_eq!(nf_game::triggers::resolve_exit_code(0x0720_004a), 0x0700_0013,
        "4a's pack cut-scene chains to 07000013 [M 0x1d3560: 0x720004a → 0x7100013 → masked]");
    assert_eq!(nf_game::triggers::resolve_exit_code(0x0710_004a), 0x0700_004a);
    // The mission's end lives on 0700013, not here [L the mission table: 0x0700004a is part 2 of
    // mission 0x07000011; part 3 = 0x07000013 — and 0x0700013 is in Mission_Update case 6's
    // compiled switch [M 0x182c60:107-110] while a mid-mission level never routes to an outro
    // (the default arm → the front-end map [M :93-95]).
    assert_eq!(menu::outro_for(0x0700_004a), menu::FRONTEND_LEVEL,
        "a mid-mission level's completion routes to the front end, no outro [M 0x182c60:93-95]");
    assert_eq!(menu::outro_for(0x0700_0013), 0x0730_0013, "the last part's outro [M 0x182c60:107-110]");

    // The parsed 0700004a scene: the pack cut-scene's end hands the movie code through the
    // out-channel chain, and nothing in the level's own data writes ch 0x4a (the "Gain access to
    // lobby" objective's condition channel [L the mission table 0x2a41e8]) — the mission cannot
    // complete on 4a.
    let scene = nf_view::load(&iso_v, "0700004a.bin", false).expect("load 0700004a");
    let sim = scene.sim.as_ref().expect("the level has a sim");
    // 1. The pack cut-scene's script 0x060006cf [D 0780004a.bin] ends writing ch 0x22 (stream 27's
    //    Event kind 0xD params [34, 1]).
    let nis = sim.drones.scripts.lib.get(&0x0600_06cf).expect("the pack cut-scene's script is in the lib [D 0780004a.bin]");
    let s27 = &nis.streams[27];
    let wrote_22 = s27.decode().iter().any(|c| matches!(c,
        nf_formats::nis::Cmd::Event { kind: 0x0d, params } if params.first() == Some(&(34u32)) && params.get(1) == Some(&(1u32))));
    assert!(wrote_22, "the pack cut-scene's end writes ch 0x22 [D 060006cf s27]");
    // 2. A ScriptPlayer starts on ch 0x22 (out 34) [D the placement table] and the lib's script
    //    0x060006d3 fires the level transition with the movie code [D].
    assert!(sim.drones.scripts.players.iter().any(|p| p.out == 34), "an SP starts on ch 0x22 [D]");
    let d3 = sim.drones.scripts.lib.get(&0x0600_06d3).expect("060006d3 in the lib [D]");
    let exit_event = d3.streams.iter().any(|st| st.decode().iter().any(|c| matches!(c,
        nf_formats::nis::Cmd::Event { kind: 0x12, params } if params.contains(&(0x0720_004a as u32)))));
    assert!(exit_event, "060006d3 fires event 0x12 with 0x0720004a [D] — the movie code 0x1d3560 resolves to 07000013");
    // 3. No ch-0x4a writer in the level's own data.
    for t in &sim.drones.triggers {
        assert!(t.out != 0x4a, "no 0700004a trigger writes ch 0x4a [D]");
    }
    for (_, script) in &sim.drones.scripts.lib {
        for st in &script.streams {
            for c in st.decode() {
                if let nf_formats::nis::Cmd::Event { kind: 0x0d, params } = c {
                    assert!(params.first() != Some(&(0x4a as u32)),
                        "no 0700004a script writes ch 0x4a [D — the objective folds on 0700013's spawn TouchOnce #153]");
                }
            }
        }
    }
}
