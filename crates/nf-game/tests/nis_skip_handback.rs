//! The skip hand-back (level-02-skip-handback): where Bond comes back when a cut-scene is
//! SKIPPED, driven through the port's real skip input (`Scripts::skip_request` — the viewer's
//! `Input_Action(-1, 0x19, 1)` latch → `begin_tick` → `SP_Update` state 2 → `Script_FFwd(cur, 1)`
//! [M 0x1dbda8:129-141, 0x1dac78]), for every SP with a kind-1/kind-2 hand-back in 07000002/03/14.
//!
//! The game's skip mechanics, from the decompile:
//! - `Script_FFwd(si, 1)` arms a fade of `min(length − clock, 45)` ticks while the script keeps
//!   playing at 1× [M 0x1dac78:48-57]; later presses only clear play bit 4 [M :86-88].
//! - `Script_Update`'s do-while [M 0x1da998:19-100] ticks the script normally while the fade timer
//!   is within its length; once `+0xAB4 < +0xA4` it sets `+0xAB5` bit 5 and SPINS the streams and
//!   clocks until the script stops — the catch-up. Events (op 0x12) still fire at their scripted
//!   ticks: the clocks advance +1 per pass, never skipping [M Script_Run 0x1d8548].
//! - `Script_Interp` case 2 (kind-2 anim) and case 4 (camera) RETURN while `+0xAB5 & 0x20`
//!   [M 0x1d9a08:146, 205] — the anim object's `+0x90` matrix freezes at the last pre-freeze pose.
//!   Case 1 (kind-1 entity spline) has NO freeze gate [M 0x1d9a08:91-139] — entities keep keying
//!   through the catch-up.
//! - Event 6 snapshots the prev stream's object matrix when it FIRES [M 0x1d94c0:73-84, 125-134];
//!   `Script_Free` re-enables the player there [M 0x1da880:44-48].
//!
//! So the [M] prediction for a skip armed at script clock K: the anim object's last written pose is
//! its clamped local frame at clock K + fade_len [M 0x1d9a08 case 2 clamp (0x90 write, +0xB0 length)];
//! an event 6 firing after that freeze snapshots the frozen pose (not the anim's end pose unless the
//! anim had already clamped to its last frame by the freeze tick); an event 6 firing at or before the
//! freeze snapshots the same pose the play path does.
//!
//! Run: `NF_ISO=… cargo test -p nf-game --test nis_skip_handback -- --nocapture`

use nf_formats::anim::Mat4;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis::Cmd;
use nf_formats::{disc, nis::Nis};
use nf_game::nis::{Request, Scripts, World};
use nf_game::triggers::Channels;
use std::collections::HashMap;
use std::sync::Arc;

/// The anim data the kind-2 rows resolve through (the census's harness: the level's own skins,
/// skeletons and anim scripts plus its `07F` sequence pack and the 078/079/07A cut-scene packs).
struct AnimWorld {
    scripts: HashMap<u32, nf_formats::anim::Script>,
    sequences: HashMap<u32, nf_formats::anim::Sequence>,
    skins: HashMap<u32, nf_formats::character::Skin>,
    skeletons: HashMap<u8, nf_formats::character::Skeleton>,
}
impl Clone for AnimWorld {
    fn clone(&self) -> Self {
        AnimWorld {
            scripts: self.scripts.clone(), sequences: self.sequences.clone(),
            skins: self.skins.clone(), skeletons: self.skeletons.clone(),
        }
    }
}
impl Default for AnimWorld {
    fn default() -> Self {
        AnimWorld { scripts: Default::default(), sequences: Default::default(), skins: Default::default(), skeletons: Default::default() }
    }
}
impl AnimWorld {
    fn for_level(iso: &IsoImage, level: &Level, name: &str, data: &[u8]) -> AnimWorld {
        let lb = LevelBin::parse(data).ok();
        let mut skeletons: HashMap<u8, nf_formats::character::Skeleton> = lb.as_ref().map(|lb| lb.entries.iter().filter(|e| e.kind == 6)
            .filter_map(|e| nf_formats::character::parse_skeleton(lb.data(data, e)).ok())
            .map(|s| (s.id as u8, s)).collect()).unwrap_or_default();
        let mut scripts: HashMap<u32, nf_formats::anim::Script> = lb.as_ref().map(|lb| lb.entries.iter().filter(|e| e.kind == 5)
            .filter_map(|e| nf_formats::anim::Script::parse(lb.data(data, e)).ok())
            .map(|x| (x.hashcode, x)).collect()).unwrap_or_default();
        let mut sequences = disc::level_sequences(iso, name).unwrap_or_default();
        let mut skins = level.skins.clone();
        let (entries, bin) = disc::files_bin_entries(iso).expect("FILES.BIN");
        let nn = &name[3..8];
        for pfx in ["078", "079", "07A"] {
            let pack = format!("{pfx}{nn}.bin");
            let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&pack)) else { continue };
            let Ok(pd) = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64) else { continue };
            let Ok(plb) = LevelBin::parse(&pd) else { continue };
            for e in plb.entries.iter().filter(|e| e.kind == 4 && e.length >= 0x40) {
                if let Ok(s) = nf_formats::anim::Sequence::parse(plb.data(&pd, e)) { sequences.entry(s.hashcode).or_insert(s); }
            }
            for e in plb.entries.iter().filter(|e| e.kind == 5) {
                if let Ok(s) = nf_formats::anim::Script::parse(plb.data(&pd, e)) { scripts.entry(s.hashcode).or_insert(s); }
            }
            for e in plb.entries.iter().filter(|e| e.kind == 6) {
                if let Ok(s) = nf_formats::character::parse_skeleton(plb.data(&pd, e)) { skeletons.entry(s.id as u8).or_insert(s); }
            }
            for e in plb.entries.iter().filter(|e| e.kind == 3) {
                if let Ok(s) = nf_formats::character::parse_skin(plb.data(&pd, e), |id| skeletons.get(&id).map(|k| k.bones)) {
                    skins.entry(s.hashcode).or_insert(s);
                }
            }
        }
        AnimWorld { scripts, sequences, skins, skeletons }
    }
}
impl World for AnimWorld {
    fn anim_length(&self, script: u32) -> Option<u16> { self.scripts.get(&script).map(|s| s.length) }
    fn discrete0(&self, _skin: u32, _script: u32, _frame: f32) -> Option<Mat4> { None }
    /// The anim object's own world matrix (obj+0x90): the posed root at `frame` — the same rule the
    /// runtime's `Data` implements [M 0x1befd8:290 → 0x134210].
    fn anim_obj_matrix(&self, skin: u32, script: u32, frame: f32) -> Option<Mat4> {
        let sk = self.skins.get(&skin)?;
        let skel = self.skeletons.get(&sk.skeleton)?;
        let pose = nf_game::anim::script_pose_together(&self.scripts, &self.sequences, skel, sk, script, frame)?;
        nf_formats::anim::world_pose(&pose, sk).first().copied()
    }
}

fn iso_path() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Every archive's kind-7 NIS scripts (the census's lib).
fn script_lib(iso: &IsoImage) -> HashMap<u32, Arc<Nis>> {
    let (entries, bin) = disc::files_bin_entries(iso).expect("FILES.BIN");
    let mut lib = HashMap::new();
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = iso
            .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
            .unwrap_or_default();
        let Ok(lb) = LevelBin::parse(&data) else { continue };
        for e in lb.entries.iter().filter(|e| e.kind == 7) {
            let Ok(h) = u32::from_str_radix(&e.name, 16) else { continue };
            if lib.contains_key(&h) { continue; }
            if let Ok(n) = Nis::parse(h, lb.data(&data, e)) {
                lib.insert(h, Arc::new(n));
            }
        }
    }
    lib
}

/// The event-6 fires of one script: (stream, fire clock) in command order — the clock a stream
/// holds at when the op runs (the preceding Wait/End values) [M Script_Run 0x1d8548's wait gate].
fn event6_fires(nis: &Nis) -> Vec<(usize, f32)> {
    let mut out = Vec::new();
    for (si, st) in nis.streams.iter().enumerate() {
        let mut clock = 0.0f32;
        for cmd in st.decode() {
            match &cmd {
                Cmd::Wait(t) | Cmd::End(t) => clock = clock.max(*t as f32),
                Cmd::Event { kind: 6, .. } => out.push((si, clock)),
                _ => {}
            }
        }
    }
    out
}

/// The prev-stream anim binding of the FIRST event-6 fire: (anim script, skin, start frame,
/// start clock) from the stream's `Anim` op [M Script_AnimStart 0x1d8f40:88-92 — stream+0x00 =
/// the clock, stream+0x04 = the anim's clamped start frame].
fn anim_prev_stream(nis: &Nis, e6_stream: usize) -> Option<(u32, u32, u32, f32)> {
    let st = nis.streams.get(e6_stream.wrapping_sub(1))?;
    let mut clock = 0.0f32;
    let mut anim = None;
    let mut killed_at = f32::INFINITY;
    for cmd in st.decode() {
        match &cmd {
            Cmd::Wait(t) | Cmd::End(t) => clock = clock.max(*t as f32),
            Cmd::Anim { script, skin, start } => { anim = Some((*script, *skin, *start, clock)); killed_at = f32::INFINITY; }
            // An AnimEnd after the (last) Anim op kills the stream: the kind byte drops to 0
            // [M Script_KillStream 0x1d9860:107] — the event-6 gate then fails and the game
            // falls back, so such a row is NOT an anim snapshot row when the kill precedes
            // the fire. An Anim op after a kill re-binds (the reset above).
            Cmd::AnimEnd => killed_at = killed_at.min(clock),
            _ => {}
        }
    }
    // The kill beats the fire when its wait is at or before the event-6 tick (the streams'
    // clocks advance together [M Script_Update 0x1da998:36-43]).
    anim.filter(|(_, _, _, start_clock)| *start_clock < killed_at)
}

/// Drive one script through the real SP path: `spawn_once_scaled` (the placement's own base and
/// scale), then per tick `begin_tick` (which consumes `skip_request` — the viewer's latch) +
/// `update_player`. `skip_from` latches the skip from that tick on (the game's held
/// `Input_Action(-1, 0x19, 1)` [M 0x1dbda8:129-141]). When the script stops, the SP_Delete free
/// runs (the game's state-5 arm [M 0x1dbda8 case 5]; spawn_once's stay byte parks in state 3).
/// Returns (hand-back xyz, the script clock when the skip armed, the armed fade length).
fn drive(
    lib: &HashMap<u32, Arc<Nis>>, placements: &[nf_formats::placement::Placement], pi: usize,
    hash: u32, base: Mat4, scale: f32, skip_from: Option<usize>, w: &AnimWorld,
) -> (Option<[f32; 3]>, f32, u8) {
    let mut scripts = Scripts::new(lib.clone(), placements);
    scripts.spawn_once_scaled(pi, hash, base, scale);
    let idx = scripts.players.len() - 1;
    let mut ch = Channels::default();
    let gate = nf_game::nis::SpStart::default();
    let mut k_clock = 0.0f32;
    let mut fade_len = 0u8;
    for tick in 0..9000usize {
        let skip = skip_from.map(|k| tick >= k).unwrap_or(false);
        scripts.skip_request = skip;
        scripts.begin_tick(&mut ch, w, tick as u32);
        scripts.update_player(idx, &mut ch, w, tick as u32, skip, gate.clone());
        if skip {
            if let Some(s) = scripts.players[idx].info[0].as_ref() {
                // The arm tick: `Script_FFwd` ran this tick (after `Script_Update` advanced the
                // clock) — the clock NOW is the game's `+0xA0` at `+0xAB4 = min(len − clock, 45)`
                // [M 0x1dac78:48-54].
                if s.fade_clock != 0.0 && k_clock == 0.0 && s.fade_len != 0 {
                    k_clock = s.clock;
                    fade_len = s.fade_len;
                }
            }
        }
        let stopped = scripts.players[idx]
            .info[0].as_ref().map(|s| !s.playing()).unwrap_or(true);
        if stopped && tick > 2 {
            // The game's SP_Delete (state 5) frees the scripts → the event-6 `PlayerEnable`
            // [M Script_Free 0x1da880:44-48]. `spawn_once`'s stay byte parks in state 3, so the
            // free is delivered here exactly as the state-5 arm runs it.
            let p = &mut scripts.players[idx];
            let mut out: Vec<Request> = Vec::new();
            for s in p.info.iter_mut().flatten() { s.free(&mut p.subs, &mut scripts.objects, &mut out); }
            p.info = [None, None];
            for r in out {
                if let Request::PlayerEnable { at: Some(m), .. } = r {
                    return (Some([m[3][0], m[3][1], m[3][2]]), k_clock, fade_len);
                }
            }
            return (None, k_clock, fade_len);
        }
    }
    (None, k_clock, fade_len)
}

fn close(a: Option<[f32; 3]>, b: Option<[f32; 3]>, eps: f32) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => (x[0] - y[0]).abs() < eps && (x[1] - y[1]).abs() < eps && (x[2] - y[2]).abs() < eps,
        (None, None) => true,
        _ => false,
    }
}
fn fmt(p: Option<[f32; 3]>) -> String {
    p.map(|x| format!("({:.3},{:.3},{:.3})", x[0], x[1], x[2])).unwrap_or("fallback".into())
}

/// The three levels' every event-6 SP script: the play hand-back vs the skip hand-back, each
/// checked against the [M] freeze rule (header).
#[test]
fn skip_handback_matches_the_games_freeze_rule() {
    let Some(path) = iso_path() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap();
    let lib = script_lib(&iso);
    let (entries, bin) = disc::files_bin_entries(&iso).expect("FILES.BIN");

    for name in ["07000002.bin", "07000003.bin", "07000014.bin"] {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)) else {
            panic!("{name}: not on the disc");
        };
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let mi = level.main_map().expect("main map");
        let placements = &level.maps[mi].placements;
        let mut anim: Option<AnimWorld> = None;
        eprintln!("== {name}");

        for (pi, p) in placements.iter().enumerate() {
            if p.flags & 0xff != 0xd9 { continue; }
            let g = |k: u32| p.params.iter().find(|x| x.0 == k).map_or(0, |x| x.1);
            for h in [g(0), g(1)] {
                if h == 0 { continue; }
                let Some(nis) = lib.get(&h) else { continue };
                let fires = event6_fires(nis);
                if fires.is_empty() { continue; }
                // The LAST event-6 fire decides the hand-back (each fire overwrites +0x40
                // [M 0x1d94c0 case 6]) — last in EXECUTION order: by fire tick, then stream
                // index (the streams run 0..n within one tick [M Script_Update 0x1da998:26-32]).
                let mut fires = fires;
                fires.sort_by_key(|&(si, t)| (t.to_bits(), si));
                let (e6_stream, e6_tick) = *fires.last().unwrap();
                let prev_kind = nis.streams.get(e6_stream.wrapping_sub(1))
                    .map(|st| st.decode().iter().any(|c| matches!(c, Cmd::Anim { .. })))
                    .unwrap_or(false);
                if prev_kind && anim.is_none() { anim = Some(AnimWorld::for_level(&iso, &level, name, &data)); }
                let empty = AnimWorld::default();
                let w = anim.as_ref().unwrap_or(&empty);
                let base = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
                let scale = if g(10) == 0 { 1.0 } else { g(10) as f32 * 0.01 };

                let (hb_play, _, _) = drive(&lib, placements, pi, h, base, scale, None, w);
                // The real skip: the latch held from the third tick (the camera op has run by
                // then on every camera script [D the census's cam@ ticks]).
                let (hb_skip, k_clock, fade_len) = drive(&lib, placements, pi, h, base, scale, Some(3), w);
                // A second press moment, mid-scene: the game's fade length is
                // `min(length − clock, 45)` from the PRESS [M 0x1dac78:48-51], so a later press
                // freezes the anim at a later pose — the hand-back depends on when the player
                // skipped.
                let late = (e6_tick as usize / 2).max(4);
                let (hb_late, k_late, fl_late) = drive(&lib, placements, pi, h, base, scale, Some(late), w);
                eprintln!(
                    "{name} plc#{pi} {h:#09x}: play {} skip@{k_clock}+{fade_len} {} late@{k_late}+{fl_late} {} e6@{e6_tick} (s{e6_stream}) prev_anim {prev_kind}",
                    fmt(hb_play), fmt(hb_skip), fmt(hb_late)
                );

                let freeze_clock = k_clock + fade_len as f32;
                if !prev_kind {
                    // 07000014's vent-drop 0x06000833 is NOT asserted: its prev stream is
                    // Wait/Marker only [D the pack's stream dump], so the strict [M] rule is the
                    // player-matrix fallback — but the port carries the level-14-vent-drop
                    // capture arm there ([S] unverified, its own test pins it), which reads the
                    // scripted Bond body's live root and freezes with the kind-2 gate on a skip.
                    // An [S] item never passes/fails (the task rules); printed for the report.
                    if h == 0x0600_0833 {
                        eprintln!("{name} plc#{pi} {h:#09x}: [S] vent-drop arm — play {} skip@{k_clock}+{fade_len} {} late@{k_late}+{fl_late} {} (not asserted)", fmt(hb_play), fmt(hb_skip), fmt(hb_late));
                        continue;
                    }
                    // No anim prev stream: either a kind-1 entity (no freeze gate [M 0x1d9a08:91-139]
                    // — the spline keys through the catch-up) or the game's player-matrix fallback
                    // (disabled Bond does not move). Either way the snapshot equals the play path's.
                    for (tag, hb) in [("early", hb_skip), ("late", hb_late)] {
                        assert!(
                            close(hb, hb_play, 0.05),
                            "{name} {h:#09x}: {tag} skip hand-back {} != play {} — a non-anim prev stream must snapshot the same pose (no freeze gate on kind 1 [M 0x1d9a08:91-139], the player fallback is static)",
                            fmt(hb), fmt(hb_play)
                        );
                    }
                    continue;
                }
                let Some((ascript, skin, start_frame, start_clock)) = anim_prev_stream(nis, e6_stream) else {
                    // An AnimEnd killed the prev stream before the fire — the kind byte is 0 at
                    // event time [M 0x1d9860:107]: the fallback rule, same as the non-anim rows.
                    for (tag, hb) in [("early", hb_skip), ("late", hb_late)] {
                        assert!(
                            close(hb, hb_play, 0.05),
                            "{name} {h:#09x}: {tag} skip hand-back {} != play {} — the prev stream's anim was killed before the fire (kind byte 0 [M 0x1d9860:107]), the snapshot must equal the play path's",
                            fmt(hb), fmt(hb_play)
                        );
                    }
                    continue;
                };
                let alen = w.anim_length(ascript).unwrap_or(1).max(1) as f32;
                // The [M] freeze rule for one skip result: an event 6 at or before the freeze
                // snapshots the play pose (the anim was live at the fire); an event 6 after the
                // freeze snapshots the anim object's FROZEN pose — its clamped local frame at the
                // freeze clock [M 0x1d9a08 case 2 + 0x1da998:84-98] — and, unless the anim had
                // already clamped to its last frame by then, that is NOT the anim's end pose.
                let check = |tag: &str, hb: Option<[f32; 3]>, k: f32, fl: u8| {
                    let fc = k + fl as f32;
                    if e6_tick <= fc {
                        assert!(
                            close(hb, hb_play, 0.05),
                            "{name} {h:#09x}: {tag} skip hand-back {} != play {} — event 6 fires at {e6_tick} ≤ freeze {fc}, the anim was still live",
                            fmt(hb), fmt(hb_play)
                        );
                    } else {
                        let f = (fc - start_clock + start_frame as f32).clamp(1.0, alen);
                        let want = w.anim_obj_matrix(skin, ascript, f).map(|m| [m[3][0], m[3][1], m[3][2]]);
                        assert!(
                            close(hb, want, 0.05),
                            "{name} {h:#09x}: {tag} skip hand-back {} != the frozen pose at frame {f} {} (anim {ascript:#09x} len {alen}, start {start_frame}@{start_clock})",
                            fmt(hb), fmt(want)
                        );
                        if f < alen {
                            let end = w.anim_obj_matrix(skin, ascript, alen).map(|m| [m[3][0], m[3][1], m[3][2]]);
                            assert!(
                                !close(hb, end, 0.05),
                                "{name} {h:#09x}: {tag} skip hand-back {} == the anim's END pose {} — the freeze should have caught the anim mid-run (frame {f} < {alen})",
                                fmt(hb), fmt(end)
                            );
                        }
                    }
                };
                check("early", hb_skip, k_clock, fade_len);
                check("late", hb_late, k_late, fl_late);
            }
        }
    }
}

/// The [L] anchor: the port's play hand-back for the 07000002 intro scene is the game's measured
/// (-27.504, 1.131, 32.348) (fresh-boot PCSX2).
#[test]
fn play_handback_07000002_matches_the_live_pin() {
    let Some(path) = iso_path() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap();
    let lib = script_lib(&iso);
    let (entries, bin) = disc::files_bin_entries(&iso).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000002.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let mi = level.main_map().unwrap();
    let placements = &level.maps[mi].placements;
    let mut anim = AnimWorld::default();
    let mut hit = false;
    for (pi, p) in placements.iter().enumerate() {
        if p.flags & 0xff != 0xd9 { continue; }
        let g = |k: u32| p.params.iter().find(|x| x.0 == k).map_or(0, |x| x.1);
        for h in [g(0), g(1)] {
            if h == 0 { continue; }
            let Some(nis) = lib.get(&h) else { continue };
            let fires = event6_fires(nis);
            if fires.is_empty() { continue; }
            let (e6_stream, _) = *fires.last().unwrap();
            if nis.streams.get(e6_stream.wrapping_sub(1))
                .map(|st| st.decode().iter().any(|c| matches!(c, Cmd::Anim { .. }))).unwrap_or(false)
            {
                if anim.scripts.is_empty() { anim = AnimWorld::for_level(&iso, &level, &fe.name, &data); }
            }
            let base = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
            let scale = if g(10) == 0 { 1.0 } else { g(10) as f32 * 0.01 };
            let (hb, _, _) = drive(&lib, placements, pi, h, base, scale, None, &anim);
            eprintln!("07000002 plc#{pi} {h:#09x}: play {}", fmt(hb));
            if let Some(x) = hb {
                if (x[0] - -27.504).abs() < 0.05 && (x[1] - 1.131).abs() < 0.05 && (x[2] - 32.348).abs() < 0.05 {
                    hit = true;
                }
            }
        }
    }
    assert!(hit, "no 07000002 event-6 scene hands Bond back at the [L] pin (-27.504, 1.131, 32.348)");
}
