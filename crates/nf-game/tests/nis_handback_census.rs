//! Census [D]: every SP script's event-6 hand-back position the port produces, checked against the
//! level's own floor geometry.
//!
//! The game's hand-back: `Script_EventHandler` (0x1d94c0) case 6 snapshots the PREVIOUS stream's
//! bound object's `+0x90` matrix into `SCRIPTINFO+0x40` (prev stream = the record at `si+0xab3`,
//! gated on its kind byte ∈ {1, 2}; with no object, the fallback copies `glb_players+0x90` — the
//! player's live matrix at EVENT time) [M 0x1d94c0:73-84, 125-134], and `Script_Free`
//! (0x1da880:44-48) re-enables the player at that stored matrix [M 0x1da880:46-49]. The port runs
//! the same shape (`Script::event` case 6 → `player_at`, `Script::free` →
//! `Request::PlayerEnable { at }`), with the entity matrix produced by the port's interp — which
//! carries cutscene-kind31-frame-2's base rule (`keys_alone = camera_started && parent < 0`
//! [M 0x1d9c0c flag = (parent == -1), 0x1da838-0x1da844 tail; si+0x00 = the SP placement base from
//! `Script_Load` 0x1d8248:148 until `Script_CameraStart` overwrites it with the identity,
//! permanently, 0x1d916c / 0x1db348]).
//!
//! This census drives every SP placement's script (kind-0xd9 placements of every on-foot SP level
//! 0700*.bin [D], the same set `Scripts::new` mounts) through the REAL port runtime —
//! `Script::load` at the placement's `View_RotTransMatrix` base [M 0x1db9a8 → 0x1344e0], one
//! `Script::update` per 60 Hz tick until the script ends, then `Script::free` — and reads the
//! `Request::PlayerEnable` position. Each snapshot is checked against the level's static collision
//! (the main map's triangles via `CollisionWorld::build`): a hand-back must land ON A FLOOR
//! (a surface within 2.5 units straight down, the game's own settle distance for a re-enabled
//! player) or be the game's own fallback (no snapshot → the player stays where he is). A snapshot
//! over void is a hand-back into the abyss and fails the census.
//!
//! The kind-2 (anim) prev streams: the game's gate admits them and snapshots the ANIM object's
//! +0x90 — its live world matrix [M 0x1d94c0:125-134; built by `AnimObjectUpdate` 0x1befd8:290 →
//! `Control_GetObjMatrix` 0x134210 = `View_RotTransMatrix(obj+0x50, obj+0x30)`, obj+0x30 carrying
//! the animation's root track 0x1c35f8:~428-432]. The port carries that matrix on `Obj::Anim`
//! (`World::anim_obj_matrix`, the posed root) and its event-6 arm snapshots `Obj::Anim` too; the
//! four scenes that used to fall back — 07000002's 0x0600007f and 0x06000084, 07000003's
//! 0x06000080, 0700004a's 0x060006d3 [D decode, each prev stream an Anim stream] — are HARD
//! assertions: each must produce a snapshot, and the shared geometry verdict must give it a floor.
//!
//! Run: `NF_ISO=… cargo test -p nf-game --test nis_handback_census -- --nocapture`
//! (the full table prints; the hard assertions are the four kind-2 rows + the geometry verdicts).

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::{disc, nis::Cmd};
use nf_game::nis::{Objects, Request, Script, World};
use nf_game::triggers::Channels;
use nf_game::world::CollisionWorld;
use std::collections::HashMap;
use std::sync::Arc;

type Mat4 = nf_formats::anim::Mat4;

/// The anim data the census resolves kind-2 streams through: the level archive's own skins
/// (kind-3), skeletons (kind-6) and anim scripts (kind-5) plus its `07F…` sequence pack (kind-4)
/// [D the three packs the four kind-2 rows need: 07000002.bin has skins 05000009/34/4f and anim
/// scripts 06000501/02/03/04, 07000003.bin has 05000009 and 06000552, 0700004a.bin has
/// 05000004/06/07 and 060006d4/d5/d6; sequences in 07F00002/07F00003/07F0004a].
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
    /// The level's own anim set: skins (the parsed level's kind-3), skeletons (kind-6) and anim
    /// scripts (kind-5) from the level archive, sequences (kind-4) from its `07F…` pack
    /// (the same per-level set the runtime loads [M nf-view's loader; disc::level_sequences]).
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
        // The level's cut-scene packs (078/079/07A-000NN — the loadable archives its triggers
        // load [M LoadableLoad 0x1d69e8]): their kind-4 sequences carry the cut-scene anims' root
        // tracks [D 06000501's windows 0x04000584/0x04000474 live in 07900002.bin, 06000504's
        // 0x04000477/0x04000551 in 07800002.bin, 060006d5's 0x040006f0/0x04000601 in 0790004a.bin
        // — none of them in the level's 07F pack]. The level's own set stays first-wins.
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
    /// runtime's `Data` implements (drone.rs): `script_pose_together`'s bone 0 through
    /// `world_pose` [M 0x1befd8:290 → 0x134210 View_RotTransMatrix(obj+0x50, obj+0x30); the root
    /// keeps its translation]. `None` when the skin/script/sequence is not loaded — the census row
    /// then falls back exactly as before this change.
    fn anim_obj_matrix(&self, skin: u32, script: u32, frame: f32) -> Option<Mat4> {
        let sk = self.skins.get(&skin)?;
        let skel = self.skeletons.get(&sk.skeleton)?;
        let pose = nf_game::anim::script_pose_together(&self.scripts, &self.sequences, skel, sk, script, frame)?;
        nf_formats::anim::world_pose(&pose, sk).first().copied()
    }
}

fn iso_path() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Every archive's kind-7 NIS scripts (the 079*/078*/07A* loadable packs — SP scripts live there,
/// not in the level bins) [M level.rs's own nis-map build; the frame-2 census idiom].
fn script_lib(iso: &IsoImage) -> HashMap<u32, Arc<nf_formats::nis::Nis>> {
    let (entries, bin) = disc::files_bin_entries(iso).expect("FILES.BIN");
    let mut lib = HashMap::new();
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = iso
            .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
            .unwrap_or_default();
        let Ok(lb) = LevelBin::parse(&data) else { continue };
        for e in lb.entries.iter().filter(|e| e.kind == 7) {
            let Ok(h) = u32::from_str_radix(&e.name, 16) else { continue };
            // Retry every copy: a failed parse in one pack must not shadow a good copy in another.
            if lib.contains_key(&h) { continue; }
            if let Ok(n) = nf_formats::nis::Nis::parse(h, lb.data(&data, e)) {
                lib.insert(h, Arc::new(n));
            }
        }
    }
    lib
}

/// Does any stream raise the event-6 hand-back?
fn has_event6(nis: &nf_formats::nis::Nis) -> bool {
    nis.streams.iter().any(|st| st.decode().iter().any(|c| matches!(c, Cmd::Event { kind: 6, .. })))
}

/// Is the stream before `e6_stream` an ANIM stream (the game's kind-2 prev gate [M 0x1d94c0:73-84])?
/// Only these rows need the anim world; the sweep builds it lazily per level.
fn prev_stream_is_anim(nis: &nf_formats::nis::Nis, e6_stream: usize) -> bool {
    let Some(s) = nis.streams.get(e6_stream.wrapping_sub(1)) else { return false };
    s.decode().iter().any(|c| matches!(c, Cmd::Anim { .. }))
}

/// The script's first camera-op tick under the runner's timing (ops between Wait/End holds fire at
/// the held clock) [M Script_Run 0x1d8548] — informational for the table.
fn cam_tick(nis: &nf_formats::nis::Nis) -> f32 {
    let mut cam = f32::INFINITY;
    for st in &nis.streams {
        let mut clock = 0.0f32;
        for cmd in st.decode() {
            match &cmd {
                Cmd::Wait(t) | Cmd::End(t) => clock = clock.max(*t as f32),
                Cmd::Camera { .. } => cam = cam.min(clock),
                _ => {}
            }
        }
    }
    cam
}

/// The event-6 ticks per stream, for the table.
fn event6_ticks(nis: &nf_formats::nis::Nis) -> Vec<(usize, f32)> {
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

/// Run one script instance the way the port's runtime does and return the hand-back:
/// `(snapshot, mode, ticks, prev_stream_kind)`. `None` snapshot means no snapshot was taken (the
/// game's fallback — the player stays where he is); `prev_stream_kind` is the port's kind byte of
/// the event-6's previous stream after the run — the game's snapshot gate reads the same record's
/// kind byte ∈ {1, 2} [M 0x1d94c0:73-84], so a fallback with prev kind 1/2 is a divergence row.
fn run_handback(
    nis: Arc<nf_formats::nis::Nis>, base: Mat4, scale: f32, lib: &HashMap<u32, Arc<nf_formats::nis::Nis>>,
    e6_stream: usize, w: &AnimWorld,
) -> (Option<Mat4>, u8, usize, Option<u8>) {
    let mut subs: Vec<Option<Script>> = Vec::new();
    let mut objs = Objects::default();
    let mut ch = Channels::default();
    let mut out: Vec<Request> = Vec::new();
    let mut s = Script::load(nis, base, scale);
    s.start(false);
    // The script's own length in 60 Hz ticks (ffwd's `nis.length − clock` in frames) plus slack;
    // the cap only guards a pathological non-terminating stream.
    let cap = 5400usize;
    let mut ticks = 0usize;
    while s.playing() && ticks < cap {
        s.update(&mut subs, lib, &mut objs, &mut ch, w, ticks as u32, &mut out);
        ticks += 1;
    }
    // BEFORE free: `kill` clears every stream's kind byte at free time (s.kind = 0), so the
    // event-time kind must be read while the script is still intact.
    let prev_kind = s.stream_kind(e6_stream.wrapping_sub(1));
    s.free(&mut subs, &mut objs, &mut out);
    let hb = out.iter().find_map(|r| match r {
        Request::PlayerEnable { at, mode } => Some((*at, *mode)),
        _ => None,
    });
    match hb {
        Some((at, mode)) => (at, mode, ticks, prev_kind),
        None => (None, 0, ticks, prev_kind),
    }
}

#[test]
fn census_sp_handbacks_land_on_floors() {
    let Some(path) = iso_path() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap();
    let lib = script_lib(&iso);
    eprintln!("script lib: {} scripts", lib.len());

    let (entries, bin) = disc::files_bin_entries(&iso).expect("FILES.BIN");
    let mut rows = 0usize;
    let mut fallbacks = 0usize;
    let mut misses = 0usize;
    let mut failures: Vec<String> = Vec::new();

    // The four kind-2 prev-stream hand-backs the game snapshots through the anim object's +0x90
    // [M 0x1d94c0:73-84, 125-134]: the level bin and script hash of each [D nf-extract decode —
    // 0600007f s3→s2 (Anim 0x06000501, skin 0x05000009), 06000084 s3→s2 (Anim 0x06000504, skin
    // 0x05000009), 06000080 s2→s1 (Anim 0x06000552, skin 0x05000009), 060006d3 s4→s3 (Anim
    // 0x060006d5, skin 0x0500004)]. Before the anim-matrix carry the port fell back to where the
    // player was disabled on exactly these four scenes; now each row must snapshot.
    const KIND2_ROWS: [(&str, u32); 4] = [
        ("07000002.bin", 0x0600_007f),
        ("07000002.bin", 0x0600_0084),
        ("07000003.bin", 0x0600_0080),
        ("0700004a.bin", 0x0600_06d3),
    ];

    for fe in entries.iter().filter(|e| {
        let n = e.name.to_ascii_lowercase();
        n.starts_with("0700") && n.ends_with(".bin")
    }) {
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64);
        let Ok(data) = data else { continue };
        let Ok(level) = Level::parse(&data) else { continue };
        let Some(m) = level.main_map() else { continue };
        let ps = &level.maps[m].placements;
        // The level's static collision: the same build every live test floors against.
        let world = CollisionWorld::build(&level, &level.instances(true).0);
        let mut level_rows = 0usize;
        // The anim world, built lazily on the level's first kind-2 prev row (the parse of the
        // level's 07F sequence pack is the cost; only the levels with kind-2 rows pay it).
        let mut anim: Option<AnimWorld> = None;

        for (pi, p) in ps.iter().enumerate() {
            if p.flags & 0xff != 0xd9 { continue; }
            let g = |k: u32| p.params.iter().find(|x| x.0 == k).map_or(0, |x| x.1);
            let base = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
            let scale = if g(10) == 0 { 1.0 } else { g(10) as f32 * 0.01 };
            for h in [g(0), g(1)] {
                if h == 0 { continue; }
                let Some(nis) = lib.get(&h) else {
                    // The game's own loader streams this hash from the packs [M 0x1dbc08 /
                    // 0x1d69a0]; a miss means the census's lib is incomplete. The port's Player::load
                    // has a missing-script arm (state 5, deleted), so a non-NIS hash on an SP is
                    // survivable in-game — printed, not asserted.
                    misses += 1;
                    eprintln!("{} plc#{pi} script {h:#09x}: NOT in any pack (not censusable)", fe.name);
                    continue;
                };
                if !has_event6(nis) { continue; }
                let e6 = event6_ticks(nis);
                let e6_stream = e6.first().map(|(si, _)| *si).unwrap_or(0);
                let need_anim = prev_stream_is_anim(nis, e6_stream);
                if need_anim && anim.is_none() {
                    anim = Some(AnimWorld::for_level(&iso, &level, &fe.name, &data));
                }
                let empty = AnimWorld::default();
                let aw = anim.as_ref().unwrap_or(&empty);
                let (at, mode, ticks, prev_kind) = run_handback(Arc::clone(nis), base, scale, &lib, e6_stream, aw);
                rows += 1;
                level_rows += 1;
                // The kind-2 hard rows: the game snapshots the anim object's +0x90 here
                // [M 0x1d94c0:73-84, 125-134], so the port must produce a snapshot (the geometry
                // verdict runs through the shared path below).
                if KIND2_ROWS.iter().any(|(l, hh)| *l == fe.name && *hh == h) {
                    assert!(
                        at.is_some(),
                        "{} script {h:#09x}: no hand-back snapshot from the kind-2 prev stream — the port fell back where the game snapshots the anim object's +0x90 [M 0x1d94c0:125-134]",
                        fe.name
                    );
                }
                let ct = cam_tick(nis);
                match at {
                    None => {
                        fallbacks += 1;
                        eprintln!(
                            "{} plc#{pi} {h:#09x} mode {mode}: FALLBACK (no snapshot; prev kind {prev_kind:?}) cam@{ct} e6@{e6:?} {ticks}t",
                            fe.name
                        );
                    }
                    Some(mat) => {
                        let pos = [mat[3][0], mat[3][1], mat[3][2]];
                        let floor = world.floor_below(pos, 2.5);
                        // The [L]-pinned exceptions: hand-backs whose spot carries no STATIC floor
                        // because the floor is a dynamic object, where the game's own position is
                        // live-measured. 0x060006cf (the 4a landing pad): the game re-enables Bond
                        // at the hash-0 kind-0x19 marker key's own coordinates [L fresh boot,
                        // level-4a-lift-4; D walkreal_probe_l3 key t575]; the "floor" is the lift
                        // platform — SP #289's own placed object, dynamic [D placement base
                        // (0,−53.534,−3.377)] — invisible to the static collision build.
                        let pinned = [
                            ([1.79945f32, -54.12811, -3.45340], "4a landing pad [L level-4a-lift-4]"),
                        ];
                        let on_floor = floor.is_some()
                            || pinned.iter().any(|(p, _)|
                                (p[0] - pos[0]).abs() < 0.05 && (p[1] - pos[1]).abs() < 0.05 && (p[2] - pos[2]).abs() < 0.05);
                        let d_base = ((pos[0] - base[3][0]).powi(2)
                            + (pos[1] - base[3][1]).powi(2)
                            + (pos[2] - base[3][2]).powi(2))
                        .sqrt();
                        let verdict = if on_floor { "ok" } else { "VOID" };
                        eprintln!(
                            "{} plc#{pi} {h:#09x} mode {mode}: ({:.2},{:.2},{:.2}) floor {:?} base-d {d_base:.1} cam@{ct} e6@{e6:?} {ticks}t -> {verdict}",
                            fe.name, pos[0], pos[1], pos[2], floor
                        );
                        if !on_floor {
                            failures.push(format!(
                                "{} plc#{pi} script {h:#09x}: hand-back ({:.2},{:.2},{:.2}) has no floor within 2.5 below (mode {mode}, cam@{ct}, e6@{e6:?})",
                                fe.name, pos[0], pos[1], pos[2]
                            ));
                        }
                    }
                }
            }
        }
        if level_rows > 0 {
            eprintln!("-- {}: {level_rows} hand-back rows", fe.name);
        }
    }

    eprintln!("CENSUS: {rows} hand-back rows, {fallbacks} fallback, {misses} not-in-pack, {} VOID", failures.len());
    for f in &failures { eprintln!("FAIL: {f}"); }
    assert!(failures.is_empty(), "{} hand-back(s) landed over void:\n{}", failures.len(), failures.join("\n"));
}

/// Probe: where does a missing script hash live on the disc? Run with `--ignored -- --nocapture`.
#[test]
#[ignore = "assertion-free probe"]
fn probe_missing_hash() {
    let Some(path) = iso_path() else { eprintln!("skip: NF_ISO not set"); return };
    let target = std::env::var("NF_PROBE_HASH").unwrap_or_else(|_| "06000074".into());
    let want = u32::from_str_radix(&target, 16).unwrap();
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap_or_default();
        let Ok(lb) = LevelBin::parse(&data) else { continue };
        for e in lb.entries.iter() {
            let name_hit = u32::from_str_radix(&e.name, 16).ok() == Some(want);
            if e.kind == 0xC {
                let r = lb.data(&data, e);
                for w in r.chunks(5) {
                    if w.len() == 5 && u32::from_le_bytes([w[0], w[1], w[2], w[3]]) == want {
                        eprintln!("{}: kind-0xC record hash {target} flag {} (loadable pack nibble {})", fe.name, w[4], w[4] as u32 + 7);
                    }
                }
            }
            if name_hit {
                eprintln!("{}: kind-{} entry named {target} (len {})", fe.name, e.kind, e.length);
                if e.kind == 7 {
                    match nf_formats::nis::Nis::parse(want, lb.data(&data, e)) {
                        Ok(n) => eprintln!("    PARSES: {} streams, {} keys, len {}", n.streams.len(), n.keys.len(), n.length),
                        Err(err) => eprintln!("    parse FAILED: {err}"),
                    }
                }
            }
        }
    }
    eprintln!("probe {target} done");
}
