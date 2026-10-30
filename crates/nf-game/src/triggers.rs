//! Switch channels and trigger objects (docs/systems/triggers.md): placements 0xEA TouchOnce, 0xEB Touch, 0xEC
//! MultiplexIn (AND), 0xED MultiplexSIn (ordered AND), 0xEE MultiplexOut, 0xEF MultiplexOrIn (OR), 0xDC Trigger (type
//! = p0). Volumes are the trigger model's collision triangles at the placement transform; contacts are capsules
//! (Bond: radius 0.55 from feet + 0.55 to feet + 1.86; drones: *suggested* radius 0.4 from the feet to the head).

use crate::world::V3;
use nf_formats::level::Level;
use nf_formats::placement::{transform, view_rot_trans_matrix};

pub mod kind {
    pub const TOUCH_ONCE: u8 = 1;
    pub const TOUCH: u8 = 2;
    pub const AND: u8 = 3;
    pub const SEQ_AND: u8 = 4;
    pub const OUT: u8 = 5;
    pub const LOAD_LEVEL: u8 = 10;
    pub const OR: u8 = 0xc;
    /// The movie-player trigger (`Trigger_MoviePlayer` 0x1b2a08 writes action 0xd into slot 0).
    pub const MOVIE: u8 = 0xd;
}

/// `GetLevelToLoadFromMovieCode` (0x1d3560), disasm-checked against the decomp: the movie-code →
/// next-level table the load pipeline's phase 2 consults (`ResetMap_Load` 0x1d6360:173). The 0x72xxxx
/// codes are the story cut-scene movies that play BETWEEN levels (a level's script event 0x12 requests
/// them: 0700004a's pack script 0x060006d3 stream 9 fires `0x12 [0x0720004a]` — its own archive holds
/// no 0xE8 and no other chain); the 0x73xxxx codes are the DRIVING.ELF interludes, and the table sends
/// every one of them to the front-end map 0x07000048. Any other code (including the 0x71xxxx briefing
/// wrappers and plain 0x0700xxxx levels) falls through to `GameState+8 & 0xff00ffff` — the pending
/// load with its stream byte (bits 16-23) masked off [M 0x1d3560:97-98]; the caller passes it.
pub fn movie_code_next_level(code: u32, fallback: u32) -> u32 {
    match code {
        // The story cut-scenes [M 0x1d3560:52-96 — every arm of the binary search tree].
        0x0720_0001 => 0x0710_0002,
        0x0720_0002 => 0x0710_0003,
        0x0720_0003 => 0x0710_0004,
        0x0720_0005 => 0x0710_0006,
        0x0720_0006 => 0x0710_0007,
        0x0720_0007 => 0x0710_0008,
        0x0720_0009 => 0x0710_000a,
        0x0720_000a => 0x0710_000b,
        0x0720_000c => 0x0710_000d,
        0x0720_000e => 0x0710_000f,
        0x0720_0011 => 0x0710_0012,
        0x0720_0012 => 0x0710_004a,
        0x0720_0014 => 0x0710_0015,
        0x0720_0015 => 0x0710_0016,
        0x0720_004a => 0x0710_0013,
        // The driving interludes → the front end [M 0x1d3560:24-50].
        0x0730_0004 | 0x0730_0008 | 0x0730_000b | 0x0730_000d | 0x0730_000f | 0x0730_0010
        | 0x0730_0013 | 0x0730_0017 | 0x0730_001b => 0x0700_0048,
        _ => fallback,
    }
}

/// The level-exit request's resolve: the port's stand-in for `ResetMap_LevelToLoad` (0x1d3800) +
/// `ResetMap_Load`'s phase-0/phase-2 cycle compressed to the archive the port finally loads [M
/// 0x1d6360:29-36, 173-182]. A plain 0x0700xxxx code loads as-is. A 0x0720xxxx cut-scene code runs
/// phase 2 (`GetLevelToLoadFromMovieCode` → a 0x0710xxxx briefing code) and then the second cycle's
/// phase-2 fallback masks it to the playable level (0x1d3560:97-98: `GameState+8 & 0xff00ffff`).
/// A 0x0730xxxx driving code resolves straight to its front-end landing (the table has every driving
/// code → 0x07000048; the port cannot run DRIVING.ELF, so the interlude is skipped — the game's own
/// X-skip shortens it to nothing observable too [M docs/systems/movies.md §Skipping]). A 0x0710xxxx
/// briefing wrapper masks directly.
pub fn resolve_exit_code(code: u32) -> u32 {
    let masked = code & 0xff00_ffff;
    match (code >> 20) & 0xf {
        1 => masked, // 0x0710xxxx briefing wrapper → its level
        2 => movie_code_next_level(code, masked) & 0xff00_ffff, // 0x0720xxxx cut-scene movie
        3 => movie_code_next_level(code, masked),               // 0x0730xxxx driving interlude
        _ => code,   // a plain 0x0700xxxx hashcode loads as-is
    }
}

/// `ResetMap_LevelToLoad__FUiScSc` 0x1d3800's wrap switch (no_intro == 0) [M 0x1d3800:27-101;
/// jump table 0x305030, 74 slots for 0x07000001-0x0700004a, 24 `lui 0x710`/`ori` pairs]: every
/// story code loads through its 0x0710000N briefing code — identity on the low byte for
/// 0x07000001..0x07000016, plus the two special cases 0x0700001b → 0x07100018 (the Equinox
/// briefing is movie 18; 0x0710001b is not on the disc [D /MOVIES/30_FPS]) and 0x0700004a →
/// 0x0710004a. Every other code — the wrappers 0x07000041/43/46/49, the front end 0x07000048,
/// the 0x0720/0x0730 movie codes — has no case and loads unwrapped [M]. The trigger exits pass
/// no_intro = 0 [M 0x1b2318:111]; the only no_intro = 1 callers are P_ENDMISSION's restart rows
/// [M 0x216618:16,20]. The loader's phase 2 then resolves the wrapped code back
/// (`GetLevelToLoadFromMovieCode` 0x1d3560 → `resolve_exit_code`); a code whose briefing is not
/// in `UsingMovie` 0x107eb8 never starts a movie, so state 6 pops on its first tick [M
/// 0x1c98c8 case 6] and the straight load stands.
pub fn wrap_intro(code: u32) -> u32 {
    match code {
        0x0700_0001..=0x0700_0016 => 0x0710_0000 | (code & 0xff),
        0x0700_001b => 0x0710_0018,
        0x0700_004a => 0x0710_004a,
        _ => code,
    }
}

#[derive(Clone, Debug)]
    /// unported-misc-5 — `Discrete_Delete` 0x1c64d8 (`obj->pE0 = 0`) IS reached — through two
    /// object-kind handler tables (word refs at 0x169a74 and 0x216798 [D ELF scan], no direct
    /// `jal`), the game's per-kind delete slot clearing the discrete pointer. The port has no
    /// per-object discrete pointer to clear: triggers live in the owned `triggers` Vec (removal
    /// subsumes the store), so the function reduces to this model note.
pub struct Trigger {
    /// Position in the main map's placement list (objects update in reverse placement order).
    pub placement: usize,
    /// The trigger's discrete type, ext+0x38 [M `Trigger_Type` 0x1b2fa8: `return *(u16*)(obj->pE0 + 0x38)`;
    /// `Trigger_Create` 0x1b1720:70 stores placement param +0x2c there]. The port's `kind` IS that field
    /// (`kind::LOAD_LEVEL` == 10 — `Trigger_LoadLevelCreate` 0x1b2950:10 forces the param to 10 before
    /// `Trigger_Create`). `Check_Target`'s 0x1d arm reads it via `Trigger_Type` [M 0x1a7810:193].
    pub kind: u8,
    pub out: u8,
    /// Contact mask (8 player, 0x10 drone, 0x18 either with the filter list).
    pub mask: u32,
    /// Touch: filter list; muxes: input channels; Out: target channels.
    pub list: Vec<u32>,
    /// The lock channel, ext+0x24 for non-LOAD_LEVEL families [M `Trigger_Locked` 0x1b2fc8:24-29:
    /// locked = channel non-zero AND `switch_channels[ch]` non-zero].
    pub lock: u8,
    pub hide: bool,
    /// `obj+0xF0 & 0x10` — the glist flag `Trigger_Update` requires before it deletes a finished TouchOnce
    /// (0x1b1d78-0x1b1d90) or a Bond-moment AND (out 0x65-0x6E, 0x1b1da0). `Trigger_Create` sets `obj+0xF0` from
    /// the object's glist [M 0x1b1720]; the port does not read glist+0x44 yet — live data: every TouchOnce on
    /// 07000005 carries the bit [L, trigger-port-check #6] — so the flag is modelled as set.
    ///
    /// The same word's low bits are the EFFECT TYPE: `Trigger_Update` 0x1b1a78:45 calls
    /// `Control_SetEffectType` 0x134940 every tick — `*(obj+0xF0) = effect | *(obj+0xF0) & 0x30008000`
    /// [M 0x134940:6-7], the effect taken from the trigger's glist+0x44 (or the obj's own word when
    /// the glist is 0) [M 0x1b1a78:31-44]. The effect-type readers are render-side (the spawned
    /// effect class); the port carries no per-trigger effect word — recorded here at the word this
    /// field models, not silently dropped.
    pub glist_0x10: bool,
    /// `ext+0x2E` (`mask & 1`'s base channel, `Trigger_Hit` 0x1b1fc8:63-68): the filter list's 5th entry — for the
    /// Touch families that is param 6 (the same param the lock reads; both live at ext+0x24/+0x2E from slot p6).
    pub filter_gate: u8,
    pub tris: Vec<[V3; 3]>,
    pub lo: V3,
    pub hi: V3,
    /// TouchOnce removed itself (its channel was on).
    pub gone: bool,
    /// Made by a script (`Script_CreateEntity` 0xEA / 0xEB): the script object's index; its volume follows the object.
    pub script: Option<usize>,
    /// `LOAD_LEVEL` (0xE8): the next level's hashcode (placement param 0). `Trigger_Activate` (0x1b2318:111)
    /// calls `ResetMap_LevelToLoad(next_level, 1, 0)` [M].
    pub next_level: u32,
    /// `LOAD_LEVEL` gate channel: `Trigger_Update` case 10 (0x1b1a78:163) fires `Trigger_Activate` only when
    /// `switch_channels[gate]` is non-zero. The gate is `DOORINFO+0x28` = original `param[2]` of the placement
    /// (verified against MIPS disassembly: `lhu v1,56(s1); sh v1,40(s0)`) [M].
    pub gate: u8,
    /// `LOAD_LEVEL` key gate (`Trigger_Activate` 0x1b2318:76-90): param 1 — if non-zero and the channel is off,
    /// the trigger is locked (the fallback channel is set, no level load). [M] The channel lives at ext+0x26 —
    /// the same word `Trigger_Locked` 0x1b2fc8:14-22 reads for type-10 triggers (`Trigger_LoadLevelCreate`
    /// 0x1b2950:13 shifts the placement's slot 10 into the +0x26 slot before `Trigger_Create`).
    pub key_gate: u8,
    /// `LOAD_LEVEL` success channel (`Trigger_Activate` 0x1b2318:100-105): param 4 — set on a successful load. [M]
    pub success: u8,
    /// `LOAD_LEVEL` fallback channel (`Trigger_Activate` 0x1b2318:92-98): param 3 — set when the key gate blocks. [M]
    pub fallback: u8,
    /// Movie trigger (0xF4): the OUT slot's FULL 32-bit value — `Trigger_MoviePlayer` 0x1b2a08 copies
    /// old param 0 into slot 1 (`sw v0,48(s0)` @0x1b2a6c) and `Trigger_Create` 0x1b1720:68 takes it as a
    /// u32 into ext+0x20 (`__s[8] = *(undefined4 *)(param_3 + 0x30)`), the word both case-0xd paths
    /// latch into `GameState+0x10` (0x1b1f68 / 0x1b2318:58). The port's `out: u8` would truncate it.
    pub movie_code: u32,
    /// Movie trigger: case 0xd fired this tick (its input channel was armed and on — the channel clear
    /// already applied). The control-list walk drains it into `Drones::movie_request` [M 0x1b1f24-0x1b1f78].
    pub movie_fired: bool,
}

/// A body that can touch triggers: capsule axis from `a` to `b`, radius, drone (true) or Bond, drone skin hash.
#[derive(Clone, Copy, Debug)]
/// `contain`: whether the volume-containment contact (see `Trigger::touched_by`) applies to this toucher.
pub struct Toucher { pub a: V3, pub b: V3, pub r: f32, pub drone: bool, pub skin: u32, pub contain: bool }

impl Toucher {
    /// Bond in the normal stance (`Player_Collision` 0x194DF0, hip height 1.035 read live).
    /// level-0c-display-hatch-2 measured two tighter models against the full battery and
    /// REVERTED both: the walk-capsule SPINE with r 0 (A = pos+0.275, B = pos−0.485 [M
    /// 0x194df0's A/B arms]) reproduces the 0c [L] rest state (ch 27 = 0 at the bay-floor rest)
    /// but breaks the master-green chain/gatehouse touches (the truck plate 27.036..27.071 vs
    /// the spine top 26.93 never fires); the spine with r 0.55 still lights the 0c rest state
    /// (the top reach 13.70 vs the parsed volume floor 13.63) AND broke
    /// interactables_07000011_live (the disc-gate sound triggers) + the 07000006 party-touch
    /// chain in the final-tree suite run. The box's extra top reach (pos+1.26 vs the capsule's
    /// pos+0.275) is what those contacts need; the 0c [L] rest divergence (the game's ch 27 = 0
    /// at the bay-floor rest, the box's reach 14.14 over the parsed volume floor 13.63) stays a
    /// documented live check — the game's true contact is `Intersect_CylGeom` vs the trigger's
    /// glist [M Trigger_Create 0x1b1720:92 stores the glist at obj+0xD8], whose ref-gate/extent
    /// the port has not modelled.
    pub fn player(feet: V3) -> Toucher { Toucher { a: [feet[0], feet[1] + 0.55, feet[2]], b: [feet[0], feet[1] + 1.86, feet[2]], r: 0.55, drone: false, skin: 0, contain: true } }
}

/// One class-5 (BULLET) contact for this tick: a projectile's per-tick flight segment and its
/// weapon-definition id — the two facts `Trigger_Hit` 0x1b1fc8:124-159 reads off a class-5 node in
/// the trigger's `obj+0xD0` list. The class byte itself is fixed (a bullet object's `obj+0xff` = 5,
/// `Bullet_init` 0x12c5a0:87), so the port folds it into the type.
/// - `a` → `b`: this tick's swept path. The game's picker is the BULLET (`Bullet_update` 0x12e7b0
///   arms its embedded HITTEST `obj+0xD4` = BU+0x50 every flying tick — `Bullet_init` 0x12c5a0:102;
///   mode ht+0x94 = 0x201 swept-segment or 0x101 point-sphere written at 0x12f134/0x12f154, closed
///   to 0 at 0x12e7f4/0x12e818 and whenever state ≥ 2, 0x12f030-0x12f03c) and `Collide_PickObj`
///   0x1eae80:93-107 tests the closest point on that segment against the TARGET object's world
///   sphere (centre obj+0x80, radius obj+0x8C). `Collide_Intersect` 0x1ebaa8 then tests the real
///   geometry — the trigger's glist collision triangles (`Trigger_Create` 0x1b1720:92 stores the
///   glist at obj+0xD8) via `Intersect_CylGeom` 0x109da0, which writes the HITDATA distance as 1e8
///   on a miss (:587), and the distance sort drops every 1e8 node (`Collide_Update` 0x1ea398:118-158).
///   So the surviving contact = the segment actually crossing the trigger's collision triangles —
///   the port's `touched_by` mesh test with a zero-radius capsule.
/// - `id`: `*(s16*)(weapon_definition + 2)` — `Trigger_Hit` 0x1b1fc8:146-147 dereferences the
///   bullet's `*(*(obj+0xE0)+0x44)+2`; BU+0x44 is the `weapon_definition_tag*` the round was born
///   with (`Bullet_init` 0x12c5a0:109 `pfVar3[0x11] = param_4`), and +2 is the record's weapon id
///   (the grenade doc's `weapon_type`, e.g. 0x3b satchel [L weapon_data]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BulletContact { pub a: V3, pub b: V3, pub id: u16 }

/// The channel table (`switch_channels`) with the frame each channel was last set (`switch_channels_time`).
#[derive(Clone, Debug)]
pub struct Channels { pub v: Vec<u8>, pub time: Vec<u32>,
    /// channel-persistence: when `Some`, every `set` (and the direct `v[..]` writers) records
    /// `(channel, value, frame, source)` — the per-tick writer census's probe. Source 0 = an object
    /// writer (the trigger/door/sensor/flicker/emitter families), 1 = the script path (the script
    /// event 0xD / 0x11 writes and the SP out — data-driven from the disc's own scripts [M
    /// 0x1d94c0]). `None` otherwise (no cost).
    pub writes: Option<Vec<(u8, u8, u32, u8)>>,
    /// channel-persistence: every channel a script op has ever written in this level — the script
    /// event 0xD / 0x11 writes (`Script_EventHandler` 0x1d94c0) and the SP out writes — accumulated
    /// so the census can tell script-owned channels (data-driven from the disc's own scripts) from
    /// a port-only writer. The [S] stand-ins deliberately do NOT land here.
    pub script_channels: std::collections::BTreeSet<u8> }

impl Default for Channels { fn default() -> Channels { Channels { v: vec![0; 256], time: vec![0; 256], writes: None, script_channels: std::collections::BTreeSet::new() } } }

impl Channels {
    /// channel-persistence: the script-path write — the game's `Script_EventHandler` 0x1d94c0 case
    /// 0xD / 0x11 and the SP out write (`SP_Update`'s tail). Same array write + nonzero stamp as
    /// `set` (the script path stamps [M 0x1d94c0:112]), recorded with source 1.
    pub fn set_script(&mut self, c: u32, value: u8, frame: u32) {
        self.set(c, value, frame);
        if let Some(w) = &mut self.writes { if let Some(last) = w.last_mut() { last.3 = 1; } }
    }

    /// Channel 0 is a real channel: the game's readers index `switch_channels[ch]` directly —
    /// `SS_Update` 0x1a9948:20-23 reads AND writes `switch_channels[ss.ch]` for any channel word
    /// including 0 (the 0c pulley winches #351/#364/#377 listen on ch 0 [D]), and the trigger/AND
    /// writers copy Out into the array unconditionally [M 0x1b1a78]. The old port skipped ch 0 in
    /// both `set` and `on`, so a ch-0 SS could never run.
    pub fn on(&self, c: u32) -> bool { self.v.get(c as usize).is_some_and(|&x| x != 0) }
    pub fn set(&mut self, c: u32, value: u8, frame: u32) {
        if c > 255 { return; }
        // channel-persistence: the NF_CHWATCH write watch — env holds the channel number; every
        // matching write prints the value and a backtrace (the writer hunt's probe; additive).
        {
            static WATCH: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
            let w = *WATCH.get_or_init(|| std::env::var("NF_CHWATCH").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()));
            if w == Some(c) {
                eprintln!("[chwatch] frame {frame} set ch 0x{c:02x} = {value} from\n{}", std::backtrace::Backtrace::force_capture());
            }
        }
        // The game stamps `switch_channels_time` on EVERY nonzero write in its trigger and script paths
        // [M 0x1b1a78:87,107,188-189 — the type-2 OR, the AND write, the Out copy; Script_EventHandler 0xD],
        // not only on the rising edge (trigger-port-check #4): a re-set channel refreshes its SEQ_AND window.
        if value != 0 { self.time[c as usize] = frame; }
        if let Some(w) = &mut self.writes { w.push((c as u8, value, frame, 0)); }
        // NF_LEVELLOG=1: every switch-channel write that turns a channel on (rising edge), with the frame
        // and the new value. Traces e.g. ch 0x1D being set by the EndLevel script's event 0x11.
        if std::env::var("NF_LEVELLOG").is_ok_and(|v| v == "1") && value != 0 && self.v[c as usize] == 0 {
            eprintln!("[levellog] frame {} channel 0x{:02x} = {}", frame, c, value);
        }
        self.v[c as usize] = value;
    }
}

fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn lerp(a: V3, b: V3, t: f32) -> V3 { [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t] }

/// Closest distance between point p and triangle t.
fn point_triangle(p: V3, t: &[V3; 3]) -> f32 {
    let (e0, e1) = (sub(t[1], t[0]), sub(t[2], t[0]));
    let n = cross(e0, e1);
    let nn = dot(n, n);
    if nn > 1e-12 {
        let d = dot(sub(p, t[0]), n) / nn;
        let q = sub(p, [n[0] * d, n[1] * d, n[2] * d]);
        let inside = (0..3).all(|k| { let (u, v) = (t[k], t[(k + 1) % 3]); dot(cross(sub(v, u), sub(q, u)), n) >= 0.0 });
        if inside { return (d * d * nn).sqrt(); }
    }
    (0..3).map(|k| segment_segment(p, p, t[k], t[(k + 1) % 3])).fold(f32::INFINITY, f32::min)
}

/// Closest distance between segments p1–q1 and p2–q2.
fn segment_segment(p1: V3, q1: V3, p2: V3, q2: V3) -> f32 {
    let (d1, d2, r) = (sub(q1, p1), sub(q2, p2), sub(p1, p2));
    let (a, e, f) = (dot(d1, d1), dot(d2, d2), dot(d2, r));
    let (s, t) = if a <= 1e-12 && e <= 1e-12 { (0.0, 0.0) } else if a <= 1e-12 { (0.0, (f / e).clamp(0.0, 1.0)) } else {
        let c = dot(d1, r);
        if e <= 1e-12 { ((-c / a).clamp(0.0, 1.0), 0.0) } else {
            let b = dot(d1, d2);
            let den = a * e - b * b;
            let mut s = if den > 1e-12 { ((b * f - c * e) / den).clamp(0.0, 1.0) } else { 0.0 };
            let mut t = (b * s + f) / e;
            if t < 0.0 { t = 0.0; s = (-c / a).clamp(0.0, 1.0); } else if t > 1.0 { t = 1.0; s = ((b - c) / a).clamp(0.0, 1.0); }
            (s, t)
        }
    };
    let (c1, c2) = (lerp(p1, q1, s), lerp(p2, q2, t));
    dot(sub(c1, c2), sub(c1, c2)).sqrt()
}

/// The capsule-vs-mesh contact the trigger `contains` uses, shared with the hurt volumes
/// (the game's object collision picks a class-0x27 hurt volume's glist cels with the player's
/// capsule [M Player_DealWithObjHit 0x194458:95-103; the exact BL pick rule is the port's
/// trigger-contact approximation]).
pub fn capsule_hits_tris(a: V3, b: V3, r: f32, tris: &[[V3; 3]]) -> bool {
    tris.iter().any(|t| {
        segment_hits_triangle(a, b, t) || point_triangle(a, t) <= r || point_triangle(b, t) <= r
            || (0..3).any(|k| segment_segment(a, b, t[k], t[(k + 1) % 3]) <= r)
    })
}

/// The object-collision REFINE boundary: strict `dist < radius` [M Intersect_CylGeom 0x109da0:447
/// pushes only while `fVar30 < *(hittest+0x8C)` and :524 tracks the minimum only while
/// `fVar30 < fVar19` with the minimum seeded AT the radius — a triangle at exactly the radius is
/// never tracked, the candidate distance stays 1e8 and `Collide_Intersect` 0x1ebaa8:320 returns no
/// hit]. The ladder pick's stage 2 rides this (`Ladder`'s refine); the inclusive
/// `capsule_hits_tris` stays for the hurt volumes and the world objects, whose [M] citations
/// measure an `<=` contact.
pub fn capsule_hits_tris_strict(a: V3, b: V3, r: f32, tris: &[[V3; 3]]) -> bool {
    tris.iter().any(|t| {
        segment_hits_triangle(a, b, t) || point_triangle(a, t) < r || point_triangle(b, t) < r
            || (0..3).any(|k| segment_segment(a, b, t[k], t[(k + 1) % 3]) < r)
    })
}

/// `Intersect_PointGeom` 0x1e92d8 for the use pick (mode 0x101): the point reaches a triangle when
/// it sits within the triangle's plane window AND the projection of the point onto the tri's plane
/// lands ON the triangle. The plane window is two-sided [−radius, +radius] (see the comment at the
/// gate — the exit planes pin the behind-side admission); the projection-inside requirement is
/// pinned by 07000003's locked-door audit (see below). Per triangle the game computes the signed
/// plane distance X of the (object-space) point and skips when `X > radius` (0x1e92d8:200) or when
/// `X < fVar25` (0x1e92d8:227 — fVar25 = 0 for a flags-clear pick, −radius with flags bit 8,
/// 0x1e92d8:116-121); the surviving triangle's distance comes from
/// `vecutil_Dist2Tri_PointGeom` 0x1dfac8. The port registers only the routine's plane-projection
/// arm (0x1dfac8:77-85, the `default` case — the point's footprint projection inside the
/// triangle); the edge/vertex clamp arms (0x1dfac8:86-106) do NOT register a use-pick hit —
/// pinned by 07000003's locked-door audit [D: from the doorway stance the use point sits 0.07/0.16
/// outside plate #504's top sheet's east/south edges and 0.74 above it — the clamped surface
/// distance 0.76 is inside the 1.0 reach, yet the game refuses the plate and plays the locked-door
/// hit sound]. `// [S] unverified`: which of the two readings the game's registers enforce is not
/// decidable from the decompile alone (the registration gate rides the caller's fVar24 compare);
/// the live check in docs/reports/level-03-locked-door-use-volume.md asks the main agent's
/// gdbwatch to settle it.
///
/// The pick radius is the hittest's +0x8C = 1.0 for `Player_Activate`'s use sphere [M 0x1a5170:106-108].
/// The port's triangles carry the game's stored plane direction in their winding cross
/// (`nf_formats::collision::triangles_with_material` swaps the verts where the cross opposes the
/// stored s16·2⁻¹⁴ normal), so the winding normal here is the game's plane normal.
pub fn point_reaches_tris(p: V3, tris: &[[V3; 3]], r: f32) -> bool {
    tris.iter().any(|t| {
        let e1 = sub(t[1], t[0]);
        let e2 = sub(t[2], t[0]);
        let n = cross(e1, e2);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len < 1e-9 { return false; }
        let n = [n[0] / len, n[1] / len, n[2] / len];
        let x = n[0] * (p[0] - t[0][0]) + n[1] * (p[1] - t[0][1]) + n[2] * (p[2] - t[0][2]);
        // The signed plane distance window. `Intersect_PointGeom` skips a triangle when
        // `X > radius` [M 0x1e92d8:200] and when `X < fVar25` [M 0x1e92d8:227], where fVar25 = 0
        // for a flags-clear pick and −radius when the hittest flags carry bit 8
        // [M 0x1e92d8:116-121]. The use pick's call site drops the fourth constant
        // (`0,0,0` for the three trailing args the decompile shows); the game's behaviour pins
        // the two-sided window: 07000015's exit planes all face +z and the walking player's use
        // point sits up to 0.45 BEHIND them when the E loads the level [D walk_07000015_live's
        // gate planes at z −24.60..−24.96, normals +z, the stop ~0.55 short], which only
        // fVar25 = −radius admits. [S] unverified: the flags value itself is not readable from
        // the decompile (the dropped argument) — the live check in the report asks gdbwatch to
        // confirm; no test asserts the flags directly.
        if x < -r || x > r { return false; }
        // The projection of the point onto the tri's plane must land inside the triangle
        // (`vecutil_Dist2Tri_PointGeom`'s plane-projection arm [M 0x1dfac8:77-85]; the edge-clamp
        // arms' refusal pinned by the 07000003 audit — see the header).
        let q = [p[0] - n[0] * x, p[1] - n[1] * x, p[2] - n[2] * x];
        point_in_tri(q, t)
    })
}

/// Whether `q` lies inside triangle `t` (the barycentric sign test, CW/CCW agnostic — the winding
/// cross is the plane normal, both orientations occur in the disc's data pre-alignment).
fn point_in_tri(q: V3, t: &[V3; 3]) -> bool {
    let sign = |a: [f32; 2], b: [f32; 2], c: [f32; 2]| (a[0] - c[0]) * (b[1] - c[1]) - (b[0] - c[0]) * (a[1] - c[1]);
    // Work in the plane's dominant 2D projection to avoid the degenerate axis.
    let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
    let (i0, i1) = if n[0].abs() >= n[1].abs() && n[0].abs() >= n[2].abs() {
        (1, 2)
    } else if n[1].abs() >= n[2].abs() {
        (0, 2)
    } else {
        (0, 1)
    };
    let p2 = |v: &V3| [v[i0], v[i1]];
    let (a, b, c, q2) = (p2(&t[0]), p2(&t[1]), p2(&t[2]), p2(&q));
    let d1 = sign(q2, a, b);
    let d2 = sign(q2, b, c);
    let d3 = sign(q2, c, a);
    let neg = d1 < -1e-9 || d2 < -1e-9 || d3 < -1e-9;
    let pos = d1 > 1e-9 || d2 > 1e-9 || d3 > 1e-9;
    !(neg && pos)
}

/// Whether the segment a–b crosses triangle t.
fn segment_hits_triangle(a: V3, b: V3, t: &[V3; 3]) -> bool {
    let d = sub(b, a);
    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-12 { return false; }
    let s = sub(a, t[0]);
    let u = dot(s, p) / det;
    if !(0.0..=1.0).contains(&u) { return false; }
    let q = cross(s, e1);
    let v = dot(d, q) / det;
    if v < 0.0 || u + v > 1.0 { return false; }
    (0.0..=1.0).contains(&(dot(e2, q) / det))
}

impl Trigger {
    /// A trigger of `kind` from its parameters (placement or script entity: parameter k at `level_tag+0x2C+4k`) and its
    /// world triangles; `None` for a Touch on channel 0x7D (only live on 0x07001500). The wrapper families shift the
    /// slots differently [M 0x1d04f0 dispatch; core-loop port map §3.1]: the Touch wrappers keep slots 2-7 in place
    /// (0x1b2aa8/0x1b2b88), the mux wrappers shift every param down by one (out p0 → slot 1, inputs = raw p1..p7,
    /// mask = raw p8, lock = raw p9, hide = raw p10 — 0x1b2c68/0x1b2e08/0x1b2d38 loops), and `Trigger_LoadLevelCreate`
    /// (0x1b2950, disasm) forces mask = 0xE and maps key/gate/fallback/success into the input slots.
    pub fn from_params(placement: usize, kind: u8, q: impl Fn(u32) -> u32, tris: Vec<[V3; 3]>) -> Option<Trigger> {
        let (out_ch, mask, list, lock, hide, next_level, gate, key_gate, success, fallback): (u32, u32, Vec<u32>, u32, bool, u32, u8, u8, u8, u8) = match kind {
            kind::TOUCH_ONCE => (q(0), q(1), (2..=8).map(&q).collect(), q(7), q(8) != 0, 0, 0, 0, 0, 0),
            kind::TOUCH => (q(0), q(1), (2..=8).map(&q).collect(), q(6), q(7) != 0, 0, 0, 0, 0, 0),
            // `LOAD_LEVEL` (0xE8): `Trigger_Update` case 10 fires when `switch_channels[gate]` is non-zero —
            // NO touch check. `next_level` = param 0. `gate` = param 2 (the gate channel; set by the EndLevel
            // script's event 0x11 on 07000005). `key_gate` = param 1, `success` = param 4, `fallback` = param 3.
            // The wrapper hard-sets mask 0xE (`addiu t0,zero,14` @0x1b299c → tag+0x50) [M]. Only the input slots
            // p1 (key) and p2 (gate) are read back by `Trigger_Activate` (+0x26/+0x28) — the list keeps p1..p4.
            // [M Trigger_Update 0x1b1a78:162-166, Trigger_Activate 0x1b2318:64-111, 0x1b2950].
            kind::LOAD_LEVEL => (0, 0xE, (1..=4).map(&q).collect(), q(10), q(11) != 0, q(0), q(2) as u8, q(1) as u8, q(4) as u8, q(3) as u8),
            // `Trigger_MoviePlayer` 0x1b2a08 (disasm): backs the 0x84-byte param block up to TmpParam
            // (0x319b88), then writes slot 0 = 0xd (@0x1b2a58), slot 1 = TmpParam[0] = old p0 (@0x1b2a6c),
            // slot 2 = TmpParam[2] = old p2 (@0x1b2a74) and slot 9 = TmpParam[1] = old p1 (@0x1b2a7c) —
            // NOT the mux shift (slot 2 keeps old p2, and slot 9 is REWRITTEN with old p1). Slots 3-8 and
            // 10-11 keep the placement's raw p3..p8/p10/p11. `Trigger_Create` reads out = slot 1 (u32),
            // inputs = slots 2..8, mask = slot 9, lock = slot 10, hide = slot 11 [M 0x1b1720:67-88].
            kind::MOVIE => (q(0), q(1), (2..=8).map(&q).collect(), q(9), q(10) != 0, 0, 0, 0, 0, 0),
            // The mux family (0xEC AND / 0xED SEQ_AND / 0xEE Out / 0xEF OR): the wrapper backs the params up and
            // writes slot[k] = p(k-1) for k = 2..7 ONLY — the loop bound is `while (uVar3 < 8)` [M 0x1b2c68], so
            // slot 8 keeps raw p8 and slots 9-11 are never touched. `Trigger_Create` copies 7 input ushorts from
            // slots 2..8 into ext+0x26..0x32 (ext+0x34 stays 0) and reads mask/lock/hide from slots 9/10/11
            // [M 0x1b1720:80-91 — the audit's correction of the port-map's "p1..p7 / p8 / p9 / p10" table].
            // `Trigger_MultiplexOut` 0x1b2ed8 — the 0xEE placement wrapper [M 0x1b2ed8:11-28].
            _ => (q(0), q(9), (1..=6).map(&q).chain(std::iter::once(q(8))).collect(), q(10), q(11) != 0, 0, 0, 0, 0, 0),
        };
        // `Trigger_Create` defaults a zero mask to 2 (class-5 contacts) [M 0x1b1720:80-84].
        let mask = if mask == 0 { 2 } else { mask };
        if matches!(kind, kind::TOUCH | kind::TOUCH_ONCE) && out_ch == 0x7d { return None; }
        let filter_gate = if matches!(kind, kind::TOUCH | kind::TOUCH_ONCE) { list.get(4).copied().unwrap_or(0) as u8 } else { 0 };
        let mut t = Trigger { placement, kind, out: out_ch as u8, mask, list, lock: lock as u8, hide, glist_0x10: true, filter_gate, tris: Vec::new(), lo: [0.0; 3], hi: [0.0; 3], gone: false, script: None, next_level, gate, key_gate, success, fallback, movie_code: 0, movie_fired: false };
        if kind == kind::MOVIE { t.movie_code = out_ch; }
        t.set_tris(tris);
        Some(t)
    }

    /// New world triangles (a moving trigger).
    pub fn set_tris(&mut self, tris: Vec<[V3; 3]>) {
        let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for t in &tris { for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
        (self.tris, self.lo, self.hi) = (tris, lo, hi);
    }

    /// The trigger's bounding-box centre (for scenario placement — put Bond at a TouchOnce trigger's centre to
    /// fire it). B-11: pub so nf-view's cutscene-ai-pause scenario + its regression test can find the gatehouse.
    pub fn centre(&self) -> V3 {
        if self.tris.is_empty() { return [0.0; 3]; }
        [(self.lo[0] + self.hi[0]) * 0.5, (self.lo[1] + self.hi[1]) * 0.5, (self.lo[2] + self.hi[2]) * 0.5]
    }

    /// Whether a capsule touches the trigger's triangles.
    /// `Trigger_Hit` 0x1b1fc8 itself does NO geometry: it classifies the object's CONTACT LIST
    /// (obj+0xD0, built by the collision pass — a toucher whose sphere overlaps the trigger's glist
    /// bounds counts, including one standing INSIDE a room-shaped volume where no triangle is
    /// within reach). The port's triangle tests cover the plate/quad cases; the containment branch
    /// here covers the volume case (a toucher capsule fully inside the trigger's bounds — e.g.
    /// 07000006's LoadPoint #746, an open room-shaped glist [D measured tris]).
    pub fn touched_by(&self, c: &Toucher) -> bool {
        let grow = c.r + 0.1;
        let lo = [c.a[0].min(c.b[0]) - grow, c.a[1].min(c.b[1]) - grow, c.a[2].min(c.b[2]) - grow];
        let hi = [c.a[0].max(c.b[0]) + grow, c.a[1].max(c.b[1]) + grow, c.a[2].max(c.b[2]) + grow];
        if (0..3).any(|k| hi[k] < self.lo[k] || lo[k] > self.hi[k]) { return false; }
        // Containment: both capsule ends inside the trigger's own bounds (the bounding-sphere
        // contact the game's collision pass registers).
        // Not for Bond hanging on a wire / zipline: there the bounds of the chasm's plunge volume enclose him, and
        // the containment contact fired the falling-to-death cut-scene the moment he grabbed the 07000005 cable
        // (ch 0xBE, merge of map-mayhew over fix-wire). In the game the crossing has no fall or death path at all
        // [M Player_Wire 0x197658]. [S]: the game's exact contact rule for a hanging Bond isn't read yet.
        // fix-objects2: containment is a PLATE/ROOM contact, not a valley-shell one. The game's contact is the
        // capsule against the volume's own mesh (the shells fire on crossing; standing strictly inside a hollow
        // volume touches nothing — 07000008's boarding platform sits INSIDE plunge #54's AABB and the game's player
        // does not die there [D #54 tris: a floor quad at y 17.5 + a wall at x 136.6 over the valley; the platform
        // at y ≈ 24–27 touches neither]). The containment approximation stays only for volumes whose triangles are
        // small (the room-exit shells and walk-over plates — 07000006's LoadPoint #746, edges ≤ 10.4); the plunge
        // family's valley-sized quads (edges 25.8–46.5 [D]) fall back to the mesh test alone.
        let contain_ok = self.tris.iter().all(|t| (0..3).all(|k| {
            let a = t[k];
            let b = t[(k + 1) % 3];
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)) < 24.0 * 24.0
        }));
        if c.contain && contain_ok
            && (0..3).all(|k| c.a[k] >= self.lo[k] && c.a[k] <= self.hi[k])
            && (0..3).all(|k| c.b[k] >= self.lo[k] && c.b[k] <= self.hi[k]) {
            return true;
        }
        capsule_hits_tris(c.a, c.b, c.r, &self.tris)
    }

    /// `Trigger_Hit` (0x1B1FC8) for a Touch / TouchOnce trigger. The 0x18 branch is an AND-chain [M 0x1b1fc8:56-123]:
    /// the base is 1, or `ch[ext+0x2E]` when `mask & 1`; every nonzero filter entry must then pass (0xB needs a drone
    /// contact, 0xE the player, anything else a drone whose skin hash matches the entry's low 16 bits). An empty
    /// filter list returns the base alone.
    /// `bullets` are this tick's class-5 (BULLET) contacts — `Trigger_Hit` reads them off the trigger's
    /// `obj+0xD0` HITDATA list, which `Collide_Update` 0x1ea398 rebuilds EVERY tick (cleared :25-53, refilled
    /// by the bullet picker + the :160-233 reflection into the trigger's list) — with the mask-2 arm at
    /// :124-163: any class-5 contact lights bit 1 of the collected class word; types 1/2 then require EVERY
    /// nonzero filter entry (+0x26.., up to 8) to equal some touching bullet's weapon-definition id
    /// (`*(*(obj+0xE0)+0x44)+2`, :146-147), an EMPTY filter list returning 1 (:130-135); any other type falls
    /// through to the fallback `(mask & bits) != 0` (:161-163) — any class-5 contact fires. The arm is only
    /// reached when the 0x18 filter branch did not already return (no player/drone contact, or mask & 0x18 == 0).
    fn hit(&self, ch: &Channels, touchers: &[Toucher], bullets: &[BulletContact]) -> bool {
        let player = touchers.iter().any(|c| !c.drone && self.touched_by(c));
        let drone_touch = touchers.iter().any(|c| c.drone && self.touched_by(c));
        // The class-5 contacts [M 0x1b1fc8:38-39 + the collide walk — see BulletContact]: the bullet's
        // per-tick segment vs this trigger's collision triangles (zero-radius capsule; the game's exact
        // pass `Intersect_CylGeom` 0x109da0:587 vetoes a sphere-picked node whose segment misses the glist).
        let bullets5: Vec<&BulletContact> = bullets.iter()
            .filter(|b| self.touched_by(&Toucher { a: b.a, b: b.b, r: 0.0, drone: false, skin: 0, contain: false }))
            .collect();
        let m = self.mask;
        // The filter branch runs only for types 1/2 with mask & 0x18 and a player/drone contact [M 0x1b1fc8:56].
        if m & 0x18 != 0 && (player || drone_touch) {
            if m == 8 { return player; }
            if m == 0x10 && !drone_touch { return false; }
            // Base: 1, or the ext+0x2E channel when mask bit 0 is set [M 0x1b1fc8:63-68].
            let mut acc = if m & 1 != 0 { ch.on(self.filter_gate as u32) } else { true };
            for &f in self.list.iter().take_while(|&&f| f != 0) {
                let ok = match f & 0xffff {
                    0xb => drone_touch,
                    0xe => player,
                    // (the drone side of the hash test is `Drone_GetSkinHash` 0x13a578 [M
                    // 0013a578 decompile — `AI+0xE0`, 0 when the AI block is absent], read by
                    // `Trigger_Hit` 0x1b1fc8 [M caller grep]; the port's `Toucher.skin` is that hash.
                    // NOT YET FED: the port's net-body touchers still pass `skin: 0` — the SP drone
                    // toucher build is remaining work.)
                    h => touchers.iter().any(|c| c.drone && self.touched_by(c) && c.skin & 0xffff == h),
                };
                acc = acc && ok;
            }
            return acc;
        }
        // mask & 2 (class-5 contacts, 0x1b1fc8:124-163): the arm reads the trigger's obj+0xD0 list, which the
        // collide pass fills with the class-5 objects whose per-tick flight segment crossed the trigger's
        // collision triangles (BulletContact — see the struct note). With a contact and types 1/2, every
        // nonzero filter entry must match some touching bullet's weapon-definition id (`*(*(obj+0xE0)+0x44)+2`,
        // :146-147; the port's `BulletContact.id`); an EMPTY filter list returns 1 [:130-135]. A class-5
        // contact with a non-1/2 type falls through to the fallback [:161-163] — any contact fires. The port
        // used to model no class-5 contacts at all ("contacts bit 1 never sets", trigger-port-check #11 —
        // the stand-ins-gadget-937 stopgap fed the arm the explosion list instead; the disc arm has no
        // explosion input: the reader is the bullet object's own weapon def, :147).
        if m & 2 != 0 && !bullets5.is_empty() {
            if matches!(self.kind, kind::TOUCH_ONCE | kind::TOUCH) {
                let mut acc = true;
                let mut any = false;
                for &f in self.list.iter().take_while(|&&f| f != 0) {
                    any = true;
                    acc = acc && bullets5.iter().any(|b| b.id == (f & 0xffff) as u16);
                }
                if !any { return true; }
                return acc;
            }
            return true;
        }
        // The fallback [M :160-163 `return (uVar1 & uVar8) != 0`]: the full mask vs the collected class
        // word (8 player, 0x10 drone, 2 class-5).
        (m & 8 != 0 && player) || (m & 0x10 != 0 && drone_touch) || (m & 2 != 0 && !bullets5.is_empty())
    }

    /// `Trigger_Locked` (0x1b2fc8) — the aim-hint/exit lock test over the port's fields. Type 10
    /// (`LOAD_LEVEL`): the key gate at ext+0x26 — locked when non-zero and its `switch_channels`
    /// entry is still OFF [M 0x1b2fc8:14-22, the channel = the LoadLevelCreate-shifted key gate].
    /// Every other type: the lock channel at ext+0x24 — locked when non-zero and its channel is ON
    /// [M 0x1b2fc8:24-29]. `Check_Target`'s 0x1d arm calls this before showing the exit hint
    /// [M 0x1a7810:197-208]; `Trigger_Activate`'s key-gate block (0x1b2318:76-90) is the same test
    /// inlined on the load path.
    pub fn locked(&self, ch: &Channels) -> bool {
        if self.kind == kind::LOAD_LEVEL {
            self.key_gate != 0 && !ch.on(self.key_gate as u32)
        } else {
            self.lock != 0 && ch.on(self.lock as u32)
        }
    }

    /// `Trigger_Activate` (0x1b2318): fires the trigger immediately from `Player_Activate`'s E-key path (routes
    /// b and c on 07000005 — the on-foot gate door and the zip-wire exit). Returns the channel writes to apply
    /// and `Some(next_level)` if a level load is requested. Does NOT mutate — the caller applies the writes
    /// (avoids the `&mut self.triggers[i]` + `&mut self.channels` aliasing). [M 0x1b2318:34-111].
    pub fn activate_request(&self, ch: &Channels) -> (Vec<(u8, u8)>, Option<u32>) {
        match self.kind {
            kind::LOAD_LEVEL => {
                // The game latches `ext+0x38 = 99` (type 99 = dead) after the load request [M
                // 0x1b2318:113], so the E-path can never re-fire a spent exit either (`gone`).
                if self.gone { return (Vec::new(), None); }
                // Cancel channels 0x60–0x63 (99, 0x62, 0x61, 0x60) [M 0x1b2318:64-75]: if any is on, abort (an
                // alarm blocks the exit).
                if (0x60..=0x63).any(|c| ch.on(c)) { return (Vec::new(), None); }
                // Key gate [M 0x1b2318:76-90 — `Trigger_Locked` 0x1b2fc8's type-10 arm]: if non-zero and off,
                // the trigger is locked — locked message (label
                // 0x2000021) + ext+0x3C cooldown (no text system in the port), NO channel write, NO load.
                if self.locked(ch) { return (Vec::new(), None); }
                // Auto channel [M 0x1b2318:96-108]: gate (+0x28) on → the success write (+0x2C); gate off →
                // the fallback write (+0x2A) — and BOTH branches fall through to the level load @0x1b24b8.
                if self.gate != 0 && ch.on(self.gate as u32) {
                    let sets = if self.success != 0 { vec![(self.success, 1)] } else { Vec::new() };
                    (sets, Some(self.next_level))
                } else {
                    let sets = if self.fallback != 0 { vec![(self.fallback, 1)] } else { Vec::new() };
                    (sets, Some(self.next_level))
                }
            }
            kind::TOUCH_ONCE => {
                // `Trigger_Activate` (0x1b2318:38-49) type-1 arm: begins with the `mask & 4` gate — a
                // trigger whose mask lacks bit 4 is NOT player-activatable (07000005's plunge/death
                // triggers are mask 8: pressing E near one must never fire it). The port
                // used to fire them from the use-key path. Then the out write (skipped when out = 0,
                // :41-44) — the glist-0x10 self-delete latch is the caller's (`gone`, set by
                // player_use_triggers like the game's `obj+0xFE |= 1`).
                if self.mask & 4 == 0 { return (Vec::new(), None); }
                if !self.gone && self.out != 0 { (vec![(self.out, 1)], None) } else { (Vec::new(), None) }
            }
            kind::TOUCH => {
                // `Trigger_Activate` (0x1b2318:23-33) type-2 arm: the same `mask & 4` gate, then the out
                // write with out != 0 required.
                if self.mask & 4 == 0 || self.out == 0 { return (Vec::new(), None); }
                (vec![(self.out, 1)], None)
            }
            _ => (Vec::new(), None),
        }
    }

    /// `Trigger_Update` (0x1B1A78) for every type except Touch (run in `calc`). The mux input scan walks the slots
    /// as a PREFIX — a zero slot ends the list (a mid-list 0 hides everything after it) [M 0x1b1a78 case 3/4/0xC
    /// loops: `while input[k] != 0`].
    pub fn update(&mut self, ch: &mut Channels, touchers: &[Toucher], bullets: &[BulletContact], frame: u32) {
        if self.gone || (self.lock != 0 && ch.on(self.lock as u32)) { return; }
        match self.kind {
            kind::TOUCH_ONCE => {
                // Case 1: channel already on → delete only when the glist carries 0x10 [M 0x1b1d78-0x1b1d90];
                // otherwise it FALLS THROUGH into case 2 every frame while the channel is off.
                if ch.on(self.out as u32) {
                    if self.glist_0x10 { self.gone = true; }
                    return;
                }
                if self.hit(ch, touchers, bullets) { ch.set(self.out as u32, 1, frame); }
            }
            kind::AND | kind::OR => {
                // Case 3 / 0xC: the accumulator starts at 1 for the AND (an empty/zero prefix writes 1 —
                // 0x1b1cbc `addiu a3,zero,1`) and at 0 for the OR; a zero input slot ends the scan.
                let mut acc = self.kind == kind::AND;
                for &c in self.list.iter().take_while(|&&c| c != 0) {
                    let on = ch.on(c);
                    acc = if self.kind == kind::AND { acc && on } else { acc || on };
                }
                ch.set(self.out as u32, acc as u8, frame);
                // The Bond-moment self-delete: out 0x65-0x6E, result on, glist 0x10 [M 0x1b1da0-0x1b1da8].
                if acc && (0x65..=0x6e).contains(&self.out) && self.glist_0x10 { self.gone = true; }
            }
            kind::SEQ_AND => {
                // Case 4: first input read directly (an empty list reads ch[0] = off → writes 0); every later
                // input must be on AND satisfy `time[prev] − 4 <= time[cur]` — the current stamp may be up to 4
                // frames OLDER than the previous one's, with no upper bound [M 0x1b1e04-0x1b1e1c].
                let mut it = self.list.iter().take_while(|&&c| c != 0);
                let mut acc = it.next().is_some_and(|&c| ch.on(c));
                let mut prev = self.list.first().copied().unwrap_or(0);
                for &c in it {
                    if !ch.on(c) { acc = false; break; }
                    let (a, b) = (ch.time[prev as usize] as i64, ch.time[c as usize] as i64);
                    if a - 4 > b { acc = false; break; }
                    prev = c;
                }
                ch.set(self.out as u32, acc as u8, frame);
                if acc && (0x65..=0x6e).contains(&self.out) && self.glist_0x10 { self.gone = true; }
            }
            kind::OUT => {
                let v = ch.v.get(self.out as usize).copied().unwrap_or(0);
                for &t in &self.list { if t != 0 { ch.set(t, v, frame); } }
            }
            // `LOAD_LEVEL` (0xE8): `Trigger_Update` case 10 fires when `switch_channels[gate]` is non-zero —
            // NO touch check [M 0x1b1a78:162-166] — and then runs the FULL `Trigger_Activate` [M 0x1b2318]:
            // the cancel channels, the key gate (locked → no write, no load), the success write, and the
            // type←99 latch (`gone`). The level-load side effects (Music_Event(3,1), `GS_PauseGame`,
            // `ResetMap_LevelToLoad`) are carried by `gone` — `drone.rs`'s LOAD_LEVEL arm posts them.
            kind::LOAD_LEVEL => {
                if self.gone || self.gate == 0 { return; }
                if !ch.on(self.gate as u32) { return; }
                if (0x60..=0x63).any(|c| ch.on(c)) { return; }
                if self.locked(ch) { return; } // the key gate [M 0x1b2318:76-90, `Trigger_Locked` 0x1b2fc8 type-10 arm]
                if self.success != 0 { ch.set(self.success as u32, 1, frame); }
                self.gone = true;
            }
            // `Trigger_Update` case 0xd [M 0x1b1a78 case 0xd, disasm 0x1b1f24-0x1b1f78]: fires only when
            // the FIRST input slot (ext+0x26) is non-zero AND that channel is on; then it CLEARS the
            // channel (`sb zero,0(a0)` @0x1b1f48), latches ext+0x20 (the out/movie code) into
            // `GameState+0x10` and 0 into `GameState+0x14` (0x1b1f60/0x1b1f68 — GameState = 0x2a3768
            // [L symtab]), and pushes GameFlow states 6 (movie; f12 = 0.0) and 7 (→ `Camera_SetFade(6,
            // 30.0, 1)` + state 8, `GameFlow_PushState` 0x1c9f38:16-22) — 0x41f00000 = 30.0f @0x1b1f6c.
            // NOTHING on this path starts a movie or loads a level: state 6's body pops immediately
            // when `BGFMVPlaying == 0` (`psiMovieFinished` 0x1082e8) and only `ResetMap_Load` phase 2
            // (0x1d6360:173) ever reads `GameState+0x10` — with phase 0 overwriting it first
            // (0x1d6360:103). The two state pushes are the view's business; the port carries the
            // code in `Drones::movie_request` via `movie_fired`. The game has NO spent latch here —
            // a re-armed input fires again (`gone` stays false). The disc's single record (07000041
            // #81: params (0, 4, 0) [D]) has input slot 0 → the FIRST guard fails every frame —
            // inert from this path by design.
            kind::MOVIE => {
                let input = self.list.first().copied().unwrap_or(0);
                if input != 0 && ch.on(input) {
                    ch.set(input, 0, frame);
                    self.movie_fired = true;
                }
            }
            _ => {}
        }
    }
}

/// Every trigger of the level's main map.
pub fn build(level: &Level) -> Vec<Trigger> {
    let Some(m) = level.main_map() else { return Vec::new() };
    let map = &level.maps[m];
    let mut out = Vec::new();
    for (i, p) in map.placements.iter().enumerate() {
        let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
        // Kind 0xF9 is NOT a trigger: `parsemap_create_dynamic_objects` case 0xf9 calls
        // `SoundTrigger_Create` 0x1b2628 (class 0x40) [M 0x1d04f0:504-510] — p0 is the ON-channel
        // gate, p1 the one-shot flag, p2 the sound id [M 0x1b2628:41-48; 0x1b2790]. The port's
        // sound triggers (crate::sound) own them. The game's InverterTrigger (0x1b3040, class
        // 0x53) comes from case 0x41 [M 0x1d04f0:265-268] and worldobj.rs owns it.
        let raw_dc = p.flags == 0xdc;
        let kind = match p.flags { 0xea => kind::TOUCH_ONCE, 0xeb => kind::TOUCH, 0xec => kind::AND, 0xed => kind::SEQ_AND, 0xee => kind::OUT,
            0xef => kind::OR, 0xe8 => kind::LOAD_LEVEL, 0xf4 => kind::MOVIE, 0xdc => param(0) as u8, _ => continue };
        // The typed 0xEA-0xEF wrappers rewrite the param block before `Trigger_Create` (slot 0 = the
        // type, slot 1 = the out, the mask/lock/hide re-homed per wrapper) — `from_params`' arms read
        // the wrapper-shifted slots. The RAW 0xDC rows run the direct `Trigger_Create` map instead:
        // out = slot 1 (p1), inputs = slots 2..8 (p2..p8), mask = slot 9 (p9, 0 -> the create's
        // default 2), lock = slot 10 (p10), hide = slot 11 (p11), the type byte = p0 [M 0x1b1720:61-91
        // — rec 0x20 = slot 1, ext+0x26..0x32 = slots 2..8, the mask/lock/hide from slots 9/10/11;
        // re-derived against 0x1b2b88/0x1b2c68's wrapper stores this task]. 0700000c's skylight plates
        // #275-277 are raw rows: p0=2 (Touch), p1=0x1b (the OUT — the objective channel, NOT ch 2),
        // p9=0 -> the mask defaults to 2 (class-5 bullets).
        let q = |k: u32| param(k);
        let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < map.objects.len() { Some((m, p.index as usize)) } else { None };
        // Trigger_Create builds obj+0x90 with control_create_object's quaternion-0 arm →
        // View_RotTransMatrix [M 0x1b1720:52, 0x1344e0:26-29].
        let mx = view_rot_trans_matrix(p.rotation, p.translation);
        let tris: Vec<[V3; 3]> = target.and_then(|(om, oi)| level.maps[om].objects[oi].collision.as_ref())
            .map(|c| c.triangles().into_iter().map(|t| t.map(|v| transform(&mx, v))).collect()).unwrap_or_default();
        // Touch triggers on channel 0x7D only work on level 0x07001500.
        match Trigger::from_params(i, kind, q, tris) {
            Some(mut t) if raw_dc => {
                apply_raw_dc_map(&mut t, &param);
                if !matches!(t.kind, kind::TOUCH | kind::TOUCH_ONCE) || t.out != 0x7d { out.push(t); }
            }
            Some(t) => out.push(t),
            None => {}
        }
    }
    out
}

/// The RAW 0xDC record map (no wrapper): `Trigger_Create`'s direct stores — out = slot 1, the input
/// list = slots 2..8, mask = slot 9 (0 -> the create's default 2 [M 0x1b1720:80-84]), lock = slot 10,
/// hide = slot 11 [M 0x1b1720:61-91]. Applied after `from_params` built the wrapper-shaped record.
fn apply_raw_dc_map(t: &mut Trigger, q: &impl Fn(u32) -> u32) {
    t.out = q(1) as u8;
    let m = q(9);
    t.mask = if m == 0 { 2 } else { m };
    t.list = (2..=8).map(q).collect();
    t.lock = q(10) as u8;
    t.hide = q(11) != 0;
    t.filter_gate = t.list.get(4).copied().unwrap_or(0) as u8;
}

/// The level exit's real use target, resolved the way `Player_Activate` 0x1a5170's pick and the
/// level's own wiring see it, for the campaign/flow legs and any caller that wants to stand where
/// the player stands at an exit:
///
/// 1. **the LOAD_LEVEL trigger itself** when it owns collision — the game's E-use pick is a
///    radius-1.0 sphere at head+look vs COLLISION geometry (`Collide_SphereIntersect` [M
///    0x1a5170:100-108]), hits dispatch on the object's class byte — `Trigger_Create` stamps 0x1d
///    on every trigger [M 0x1b1720:93] — and the class-0x1d arm runs `Trigger_Activate` [M
///    0x1a5170:134-137]. With a mesh, E at the 0xE8 fires it (the gate-off fallback arm still
///    loads [M 0x1b2318:96-111]).
/// 2. **else the Touch plate that feeds its gate channel** (`out == ` the 0xE8's `gate`) whose
///    bounds are real: a disc 0xE8 record can carry NO collision at all — 07000001 #18 "Load
///    Point" resolves to an object with a gfx box but `collision None` [D placements dump], so
///    the port's tris are empty and the game's E-sphere can never reach the 0xE8 either. The
///    level's exit then runs by PRESENCE: the plate's `Trigger_Hit` raises the gate channel [M
///    0x1b1a78:162-166 case 10; 07000001 #180 "LoadPointActivator" out 0x10 == the exit's gate
///    [D]], and the player's part is standing on the plate. E on that plate is REFUSED by the
///    game's `mask & 4` gate (0x18 & 4 = 0 [D #180 mask; M 0x1b2318:23-25]).
///
/// Returns `(trigger, e_can_fire)`. The fallback choice itself is the port's survey mechanic
/// (`// [S] unverified` as a resolution rule); every fact it rests on is tagged above. `None`
/// when the level has no LOAD_LEVEL exit, or a mesh-less one with no reachable gate plate.
pub fn exit_use_target(triggers: &[Trigger]) -> Option<(&Trigger, bool)> {
    let exit = triggers.iter().find(|t| t.kind == kind::LOAD_LEVEL && t.next_level != 0)?;
    if !exit.tris.is_empty() {
        return Some((exit, true));
    }
    let plate = triggers.iter()
        .find(|t| t.kind == kind::TOUCH && !t.tris.is_empty() && t.out as u32 == exit.gate as u32)?;
    Some((plate, false))
}

/// `Trigger_Calc` (0x1B1900): level latches, then the Touch triggers (their channels cleared, then OR-ed).
pub fn calc(level: u32, triggers: &mut [Trigger], ch: &mut Channels, touchers: &[Toucher], bullets: &[BulletContact], frame: u32) {
    if level == 0x0700_0005 && ch.on(0xa4) { ch.set(0x6f, 1, frame); }
    if level == 0x0700_000b && ch.on(0x34) { ch.set(0x2d, 1, frame); }
    for t in triggers.iter().filter(|t| t.kind == kind::TOUCH) { ch.set(t.out as u32, 0, frame); }
    let hits: Vec<u8> = triggers.iter().filter(|t| t.kind == kind::TOUCH && !(t.lock != 0 && ch.on(t.lock as u32)) && t.hit(ch, touchers, bullets)).map(|t| t.out).collect();
    for out in hits { ch.set(out as u32, 1, frame); }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plate(kind: u8, out: u8, mask: u32, list: Vec<u32>) -> Trigger {
        // A 2 × 2 horizontal quad at y = 0 around the origin.
        let (a, b, c, d) = ([-1.0, 0.0, -1.0], [1.0, 0.0, -1.0], [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0]);
        Trigger { placement: 0, kind, out, mask, list, lock: 0, hide: false, glist_0x10: true, filter_gate: 0, tris: vec![[a, b, c], [a, c, d]], lo: [-1.0, 0.0, -1.0], hi: [1.0, 0.0, 1.0], gone: false, script: None, next_level: 0, gate: 0, key_gate: 0, success: 0, fallback: 0, movie_code: 0, movie_fired: false }
    }

    #[test]
    fn touch_once_latches_and_muxes_follow_inputs() {
        let mut ch = Channels::default();
        let mut once = plate(kind::TOUCH_ONCE, 0x23, 8, vec![]);
        let mut or = Trigger { kind: kind::OR, out: 0x22, list: vec![0x23, 0x24], ..plate(kind::OR, 0, 0, vec![]) };
        let away = [Toucher::player([5.0, 0.0, 5.0])];
        let on = [Toucher::player([0.5, -0.3, 0.0])];
        once.update(&mut ch, &away, &[], 1);
        or.update(&mut ch, &away, &[], 1);
        assert!(!ch.on(0x22));
        once.update(&mut ch, &on, &[], 2);
        or.update(&mut ch, &on, &[], 2);
        assert!(ch.on(0x23) && ch.on(0x22));
        // Latched: stays on after Bond leaves; the TouchOnce removes itself.
        once.update(&mut ch, &away, &[], 3);
        or.update(&mut ch, &away, &[], 3);
        assert!(ch.on(0x22) && once.gone);
        // A drone-only plate ignores Bond.
        let drone_plate = plate(kind::TOUCH_ONCE, 0x30, 0x10, vec![]);
        assert!(!drone_plate.hit(&ch, &on, &[]));
        assert!(drone_plate.hit(&ch, &[Toucher { drone: true, ..on[0] }], &[]));
    }

    /// `Trigger_Update` case 5 — the OUT mux (`Trigger_MultiplexOut` 0x1b2ed8 forces subtype 5
    /// [M 0x1b2ed8:14-22]): every tick it copies the OUT channel's state into each target
    /// [M 0x1b1a78:147-158, `switch_channels[list[i]] := switch_channels[out]`], a 0 state
    /// included — the mirror, not a latch. 0700000b's #304 is the disc shape: out 0x41 →
    /// targets [0x5A, 0x64(100)] [D placements-all #304] — the parachute-jump chain's last hop.
    #[test]
    fn out_mux_mirrors_the_out_channel_both_ways_each_tick() {
        let mut ch = Channels::default();
        let mut out = Trigger { kind: kind::OUT, out: 0x41, list: vec![0x5a, 0x64], ..plate(kind::OUT, 0, 0, vec![]) };
        // Source off → the targets are driven to 0 (a stale 1 would be cleared — the mirror law).
        ch.set(0x5a, 1, 1);
        out.update(&mut ch, &[], &[], 2);
        assert!(!ch.on(0x5a) && !ch.on(0x64));
        // Source on → both targets rise.
        ch.set(0x41, 1, 3);
        out.update(&mut ch, &[], &[], 4);
        assert!(ch.on(0x5a) && ch.on(0x64));
        // Source falls again → the targets follow (not latched).
        ch.set(0x41, 0, 5);
        out.update(&mut ch, &[], &[], 6);
        assert!(!ch.on(0x5a) && !ch.on(0x64));
    }

    /// `Trigger_Calc`'s level latches [M 0x1b1900 — the hard-coded per-level copies]: on
    /// 0700000b ch 0x34 on → ch 0x2d rises every calc (the roof cut-scene SP #177's arming law;
    /// the disc carries no placement writer for 0x2d [D trig dump — 60 records, no 0x2d out]),
    /// while 0x34 low leaves 0x2d alone.
    #[test]
    fn the_0b_calc_latch_copies_0x34_into_0x2d() {
        let mut ch = Channels::default();
        let mut trigs: Vec<Trigger> = vec![];
        ch.set(0x34, 1, 1);
        calc(0x0700_000b, &mut trigs, &mut ch, &[], &[], 2);
        assert!(ch.on(0x2d));
        // A different level does not latch.
        let mut ch2 = Channels::default();
        ch2.set(0x34, 1, 1);
        calc(0x0700_000a, &mut trigs, &mut ch2, &[], &[], 2);
        assert!(!ch2.on(0x2d));
        // 0x34 off → no rise on 0b either.
        let mut ch3 = Channels::default();
        calc(0x0700_000b, &mut trigs, &mut ch3, &[], &[], 2);
        assert!(!ch3.on(0x2d));
    }

    /// The movie trigger (0xF4 → action 0xd), `Trigger_Update` case 0xd [M 0x1b1a78:190-201, disasm
    /// 0x1b1f24-0x1b1f78]: fires only when the FIRST input slot is non-zero AND its channel is on;
    /// the fire CLEARS that channel (so it cannot re-fire until something re-arms it) and raises
    /// `movie_fired` for the walk to carry into `Drones::movie_request` (the game latches ext+0x20
    /// into GameState+0x10 and pushes states 6/8 — no level load, no spent latch).
    /// Sister 1: the input slot zero → the first guard (`lhu v0,38(s0); beq v0,zero` @0x1b1f24-28)
    /// fails even with the channel itself armed — the disc's only 0xF4 record (07000041 #81,
    /// params (0, 4, 0) [D]) is in this shape, so its update path is inert by design.
    /// Sister 2: input slot armed but the channel off → the second guard
    /// (`lbu v1,0(a0); beq v1,zero` @0x1b1f3c-40) fails; the channel is NOT cleared.
    #[test]
    fn movie_trigger_fires_on_armed_input_and_the_discs_record_cannot() {
        let mut ch = Channels::default();
        // A movie trigger whose input slot is armed (the game's ext+0x26 = the placement's p2).
        let mut movie = plate(kind::MOVIE, 0, 4, vec![0x2a]);
        movie.movie_code = 0x0720_004a;
        let touchers: [Toucher; 0] = [];
        // Sister 2 first: the channel off → nothing (and the channel is not cleared).
        movie.update(&mut ch, &touchers, &[], 2);
        assert!(!movie.movie_fired, "channel off → no fire");
        // Fire: the channel on → cleared + the movie code carried.
        ch.set(0x2a, 1, 1);
        movie.update(&mut ch, &touchers, &[], 3);
        assert!(movie.movie_fired, "input slot non-zero + channel on → case 0xd fires [M 0x1b1f24-0x1b1f48]");
        assert!(!ch.on(0x2a), "the fire clears the input channel [M 0x1b1f48 `sb zero,0(a0)`]");
        assert_eq!(movie.movie_code, 0x0720_004a, "the out slot's full u32 rides (ext+0x20 → GameState+0x10 [M 0x1b1f68])");
        assert!(!movie.gone, "case 0xd has no spent latch — the trigger stays [M: no ext+0x38=99 write in the arm]");
        // The port's re-arm rule: the walk drains `movie_fired`; a re-armed channel fires again.
        movie.movie_fired = false;
        ch.set(0x2a, 1, 4);
        movie.update(&mut ch, &touchers, &[], 5);
        assert!(movie.movie_fired, "re-armed → fires again (no latch in the game's arm)");
        // Sister 1: the disc's record shape — input slot 0 (p2 = 0 [D]) — never fires, even with
        // channel 0 armed (channel 0 is a real channel, but the SLOT guard fails first).
        let mut discs = plate(kind::MOVIE, 0, 4, vec![0]);
        discs.movie_code = 0;
        ch.set(0, 1, 6);
        discs.update(&mut ch, &touchers, &[], 7);
        assert!(!discs.movie_fired, "input slot 0 → the first guard fails [M 0x1b1f24-0x1b1f28] — the 07000041 record's update path is inert");
        assert!(ch.on(0), "channel 0 untouched (the clear only runs on a fire)");
    }

    /// The E-key path: `Trigger_Activate` 0x1b2318's type-0xd arm [M :53-61] — the `mask & 4` gate
    /// (the port models as: activate_request's `_` arm writes nothing, and the walk checks the kind
    /// + mask itself, 0x1b2318 has NO cancel-channel, key-gate or channel-write arm for 0xd and NO
    /// spent latch). Sister: a mask without bit 4 → nothing.
    #[test]
    fn movie_trigger_activate_gates_on_mask_bit4() {
        let ch = Channels::default();
        let mut movie = plate(kind::MOVIE, 0, 4, vec![0]);
        movie.movie_code = 0;
        // The port's contract: the `_` arm of activate_request returns no writes and no level for
        // action 0xd — the movie request is raised by the walk's kind check (mask & 4 [M 0x1b2318:55]).
        let (sets, next) = movie.activate_request(&ch);
        assert!(sets.is_empty() && next.is_none(), "action 0xd writes no channel and loads no level [M 0x1b2318:53-61]");
        assert_eq!(movie.movie_code, 0, "the disc's record carries movie code 0 [D placement params]");
        // The walk's gate shape, pinned directly: mask & 4 decides.
        assert!(movie.mask & 4 != 0, "the disc's mask (p1 = 4 [D]) passes the gate");
        let blocked = Trigger { mask: 8, ..movie.clone() };
        assert!(blocked.mask & 4 == 0, "sister: mask 8 fails the `mask & 4` gate [M 0x1b2318:55-57]");
    }

    /// `LOAD_LEVEL` (0xE8): `Trigger_Update` case 10 fires when `switch_channels[gate]` is non-zero — no touch check.
    /// The gate is set by a script event. When the gate is on, the trigger latches `gone` and the caller reads
    /// `next_level`. Sister: gate off → never fires.
    #[test]
    fn load_level_trigger_fires_on_gate_channel() {
        let mut ch = Channels::default();
        let mut exit = Trigger { kind: kind::LOAD_LEVEL, gate: 0x1D, next_level: 0x0700_0006, ..plate(kind::LOAD_LEVEL, 0, 8, vec![]) };
        let touchers: [Toucher; 0] = [];
        // Sister: gate off → nothing.
        exit.update(&mut ch, &touchers, &[], 1);
        assert!(!exit.gone);
        // Gate on → fires, latches.
        ch.set(0x1D, 1, 1);
        exit.update(&mut ch, &touchers, &[], 2);
        assert!(exit.gone);
        assert_eq!(exit.next_level, 0x0700_0006, "carries the next level hashcode");
        // Stays gone.
        exit.update(&mut ch, &touchers, &[], 3);
        assert!(exit.gone);
    }

    /// The exit's use-target resolution (`exit_use_target`), pinned on the 07000001 shape:
    /// the disc's #18 "Load Point" has NO collision (its object record carries a gfx box but
    /// `collision None` [D placements dump]) — the game's E-sphere hits collision only [M
    /// 0x1a5170:106-108], so the mesh-less 0xE8 can never be an E target, and the resolution
    /// falls to the gate plate (#180 "LoadPointActivator", Touch out 0x10 == the 0xE8's gate,
    /// 44 collision tris [D]). Sister: a plate wired to another channel is not the exit's target.
    #[test]
    fn meshless_exit_resolves_to_its_gate_plate() {
        // The 07000001 shapes: a mesh-less LOAD_LEVEL (gate 0x10, → 0x07000002) and the Touch plate.
        let exit = Trigger { kind: kind::LOAD_LEVEL, gate: 0x10, next_level: 0x0700_0002, tris: Vec::new(), lo: [f32::INFINITY; 3], hi: [f32::NEG_INFINITY; 3], ..plate(kind::LOAD_LEVEL, 0, 8, vec![]) };
        let mut gate_plate = plate(kind::TOUCH, 0x10, 0x18, vec![]);
        gate_plate.tris = vec![[[-33.8, 0.64, 12.17], [-32.41, 0.64, 12.17], [-32.41, 0.64, 18.68]]];
        gate_plate.lo = [-33.8, 0.64, 12.17];
        gate_plate.hi = [-32.41, 0.64, 18.68];
        let triggers = vec![exit, gate_plate];
        let (target, e_can_fire) = exit_use_target(&triggers).expect("the gate plate resolves");
        assert_eq!(target.kind, kind::TOUCH, "the resolved target is the Touch plate, not the mesh-less 0xE8 [D collision None]");
        assert_eq!(target.out, 0x10, "the plate feeds the 0xE8's gate ch 0x10 [D #180 out]");
        assert!((target.lo[0] - -33.8).abs() < 1e-4, "the resolved bounds are the plate's real ones [D #180 box]");
        assert!(!e_can_fire, "E on the plate is refused by the game's mask & 4 gate [M 0x1b2318:23-25; #180 mask 0x18 [D]]");
        // Sister: a Touch plate wired to another channel does not resolve — the exit has no real
        // use target (the caller keeps its honest skip).
        let wrong_plate = Trigger { out: 0x11, ..triggers[1].clone() };
        assert!(exit_use_target(&[triggers[0].clone(), wrong_plate]).is_none(),
            "a plate that does not feed the gate is not the exit's use target");
    }

    /// A LOAD_LEVEL exit that OWNS collision is its own E target: the game's E-sphere hits its
    /// mesh and `Trigger_Activate`'s type-10 arm loads the level (07000005's zip-wire exit shape).
    /// Sister: the same trigger with the tris stripped resolves to no plate → None.
    #[test]
    fn exit_with_mesh_is_its_own_e_target() {
        // plate() builds its own quad — strip it for the mesh-less sister, then restore.
        let mut exit = Trigger { kind: kind::LOAD_LEVEL, gate: 0, next_level: 0x0700_0066, tris: Vec::new(), lo: [f32::INFINITY; 3], hi: [f32::NEG_INFINITY; 3], ..plate(kind::LOAD_LEVEL, 0, 8, vec![]) };
        assert!(exit_use_target(&[exit.clone()]).is_none(), "no tris and no gate plate → None");
        exit.tris = vec![[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0]]];
        exit.lo = [0.0, 0.0, 0.0];
        exit.hi = [1.0, 0.0, 1.0];
        let exits = vec![exit];
        let (target, e_can_fire) = exit_use_target(&exits).expect("the exit itself resolves");
        assert_eq!(target.kind, kind::LOAD_LEVEL, "the mesh-less fallback is not consulted for a meshed exit");
        assert!(e_can_fire, "E fires it: the class-0x1d hit runs Trigger_Activate's load arm [M 0x1a5170:134-137]");
    }

    /// The E-refusal on the LoadPointActivator shape, pinned at the request surface the port's
    /// E-path calls: mask 0x18 (Bond AND the contact, [D #180 p1]) lacks bit 4, so `Trigger_Activate`
    /// returns before any write [M 0x1b2318:23-25] — holding E on the plate can never raise the
    /// exit's gate channel. Sister: the same plate with bit 4 set fires the out write (the gate is
    /// the deciding bit, not the plate's presence).
    #[test]
    fn e_on_the_loadpointactivator_shape_is_refused() {
        let ch = Channels::default();
        let plate0x18 = plate(kind::TOUCH, 0x10, 0x18, vec![]);
        let (sets, next) = plate0x18.activate_request(&ch);
        assert!(sets.is_empty() && next.is_none(), "mask 0x18 & 4 == 0 → Trigger_Activate refuses [M 0x1b2318:23-25]");
        let plate0x1c = plate(kind::TOUCH, 0x10, 0x1c, vec![]);
        let (sets, next) = plate0x1c.activate_request(&ch);
        assert_eq!(sets, vec![(0x10, 1)], "sister: with bit 4 the same plate writes its out channel");
        assert!(next.is_none(), "a Touch never loads a level");
    }

    /// `Trigger_Activate` (the E-key path, routes b/c on 07000005): pressing E on a LOAD_LEVEL trigger fires it
    /// immediately — no gate channel needed. Gate on → the success channel; gate OFF → the fallback channel AND
    /// the level still loads (both branches fall through to the load @0x1b24b8) [M 0x1b2318:96-111]. Sister: a
    /// key-gated trigger with the key off → NO write and NO load (locked message + cooldown, 0x1b2318:76-90).
    #[test]
    fn trigger_activate_fires_on_e_key_path() {
        let ch = Channels::default();
        let exit = Trigger { kind: kind::LOAD_LEVEL, next_level: 0x0700_0066, success: 0x1F, fallback: 0x1E, key_gate: 0, gate: 0, ..plate(kind::LOAD_LEVEL, 0, 8, vec![]) };
        // Gate 0 = the auto channel is off → the fallback write AND the load [M 0x1b2318:96-108].
        let (sets, next) = exit.activate_request(&ch);
        assert_eq!(next, Some(0x0700_0066), "requests the next level");
        assert_eq!(sets, vec![(0x1E, 1)], "the gate-off fallback write");
        // Gate off (and key gate clear) → the fallback write AND the load still happen [M 0x1b2318:105-108].
        let gated = Trigger { gate: 0x10, ..exit.clone() };
        let (sets, next) = gated.activate_request(&ch);
        assert_eq!(next, Some(0x0700_0066), "the gate-off fallback branch STILL loads");
        assert!(sets.contains(&(0x1E, 1)), "sets the fallback channel");
        // Gate on → the success channel.
        let mut ch_on = Channels::default();
        ch_on.set(0x10, 1, 1);
        let (sets, next) = gated.activate_request(&ch_on);
        assert_eq!(next, Some(0x0700_0066));
        assert!(sets.contains(&(0x1F, 1)), "sets the success channel");
        // Sister: key gate on and off → no write, no load (the game shows the locked message instead).
        let ch2 = Channels::default();
        let locked = Trigger { kind: kind::LOAD_LEVEL, next_level: 0x0700_0066, success: 0x1F, fallback: 0x1E, key_gate: 0x10, gate: 0, ..plate(kind::LOAD_LEVEL, 0, 8, vec![]) };
        let (sets2, next2) = locked.activate_request(&ch2);
        assert_eq!(next2, None, "locked trigger does not load");
        assert!(sets2.is_empty(), "the locked path writes NO channel (0x1b2318 returns at :90)");
        // Sister: an alarm channel (0x60-0x63) blocks the exit entirely.
        let mut ch3 = Channels::default();
        ch3.set(0x61, 1, 1);
        let (sets3, next3) = locked.activate_request(&ch3);
        assert_eq!(next3, None, "an alarm blocks the level load");
        assert!(sets3.is_empty());
    }

    /// The mux wrappers write slot[k] = p(k-1) for k = 2..7 only [M 0x1b2c68 loop bound `while (uVar3 < 8)`],
    /// and `Trigger_Create` copies 7 inputs from slots 2..8 [M 0x1b1720]: mux inputs = raw p1..p6 + raw p8 (p7 is
    /// overwritten and lost), mask = p9 with the default-2 fallback [M 0x1b1720:80-84], lock = p10, hide = p11.
    #[test]
    fn mux_params_read_the_shifted_slots() {
        let q = |k: u32| 0x10 + k; // param k reads as 0x10+k so each slot is distinguishable
        let t = Trigger::from_params(0, kind::AND, q, Vec::new()).unwrap();
        assert_eq!(t.out, 0x10, "out = p0");
        assert_eq!(t.list, vec![0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x18], "inputs = p1..p6, p8 (p7 lost to the wrapper)");
        assert_eq!(t.mask, 0x19, "mask = p9");
        assert_eq!(t.lock, 0x1a, "lock = p10");
        assert!(t.hide, "hide = p11 (0x1b != 0)");
        // Sister: a zero mask reads as the game's default 2 [M 0x1b1720:80-84].
        let t = Trigger::from_params(0, kind::AND, |_| 0, Vec::new()).unwrap();
        assert_eq!(t.mask, 2, "Trigger_Create defaults a zero mask to 2");
        // LOAD_LEVEL keeps its own mapping and the hard 0xE mask [M 0x1b2950: addiu t0,zero,14 → tag+0x50].
        let t = Trigger::from_params(0, kind::LOAD_LEVEL, q, Vec::new()).unwrap();
        assert_eq!(t.mask, 0xE, "Trigger_LoadLevelCreate forces mask 0xE");
        assert_eq!(t.next_level, 0x10, "next level = p0");
    }

    /// Case 3 [M 0x1b1a78:91-103]: the AND accumulator starts at 1 — an empty input list writes 1 — and the scan
    /// is a prefix: a zero input slot ends it (a deliberate 0 mid-list hides the inputs after it). Sister: the OR
    /// starts at 0.
    #[test]
    fn and_accumulator_starts_1_and_scans_the_prefix() {
        let mut ch = Channels::default();
        ch.set(0x20, 1, 1);
        // Empty list → AND writes 1 (the port used to write 0).
        let mut empty = plate(kind::AND, 0x21, 0, vec![]);
        empty.update(&mut ch, &[], &[], 2);
        assert!(ch.on(0x21), "an empty MuxIn latches its output ON (0x1b1cbc)");
        // Mid-list zero: inputs after it are invisible to the scan.
        ch.set(0x22, 1, 3);
        let mut mid = Trigger { kind: kind::AND, out: 0x23, list: vec![0x20, 0, 0x22], ..plate(kind::AND, 0, 0, vec![]) };
        mid.update(&mut ch, &[], &[], 4);
        assert!(ch.on(0x23), "AND stops at the zero slot: only the prefix matters");
        // The same list on an OR: the zero slot also ends the scan (0x22 ignored).
        let mut orm = Trigger { kind: kind::OR, out: 0x24, list: vec![0, 0x22], ..plate(kind::OR, 0, 0, vec![]) };
        orm.update(&mut ch, &[], &[], 5);
        assert!(!ch.on(0x24), "OR over an all-zero prefix writes 0");
    }

    /// Case 4's window predicate is `time[prev] − 4 <= time[cur]` [M 0x1b1e04-0x1b1e1c] — the current input may be
    /// up to 4 frames OLDER than the previous one and there is NO upper bound. The port's old test
    /// (`b >= a && b - a <= 4`) rejected the >4-frame gaps the game accepts.
    #[test]
    fn seq_and_window_is_lower_bound_only() {
        let mut ch = Channels::default();
        // A 100-frame gap between input 1 and input 2: the game fires, the old port never did.
        ch.set(0x30, 1, 10);
        ch.set(0x31, 1, 110);
        let mut seq = Trigger { kind: kind::SEQ_AND, out: 0x32, list: vec![0x30, 0x31], ..plate(kind::SEQ_AND, 0, 0, vec![]) };
        seq.update(&mut ch, &[], &[], 111);
        assert!(ch.on(0x32), "no upper bound on the SEQ_AND window");
        // Up to 4 frames of skew the other way is also accepted (cur older than prev).
        let mut ch = Channels::default();
        ch.set(0x30, 1, 50);
        ch.set(0x31, 1, 47); // 4 frames older: 50-4 <= 47
        let mut seq = Trigger { kind: kind::SEQ_AND, out: 0x32, list: vec![0x30, 0x31], ..plate(kind::SEQ_AND, 0, 0, vec![]) };
        seq.update(&mut ch, &[], &[], 51);
        assert!(ch.on(0x32), "the current input may be up to 4 frames older");
        // Sister: more than 4 frames of skew the other way fails.
        let mut ch = Channels::default();
        ch.set(0x30, 1, 50);
        ch.set(0x31, 1, 45); // 5 frames older: 50-4 > 45
        let mut seq = Trigger { kind: kind::SEQ_AND, out: 0x32, list: vec![0x30, 0x31], ..plate(kind::SEQ_AND, 0, 0, vec![]) };
        seq.update(&mut ch, &[], &[], 51);
        assert!(!ch.on(0x32));
    }

    /// `Channels::set` stamps `time` on EVERY nonzero write (the game's trigger and script paths re-stamp;
    /// 0x1b1a78:87,107, trigger-port-check #4) — a re-set channel refreshes its SEQ_AND window.
    #[test]
    fn channels_restamp_on_every_nonzero_write() {
        let mut ch = Channels::default();
        ch.set(0x40, 1, 10);
        assert_eq!(ch.time[0x40], 10);
        ch.set(0x40, 1, 77); // same value, still nonzero: the stamp refreshes
        assert_eq!(ch.time[0x40], 77, "every nonzero write re-stamps");
        ch.set(0x40, 0, 90);
        assert_eq!(ch.time[0x40], 77, "a zero write does not stamp");
    }

    /// Case 1 deletes the TouchOnce only when the glist flags carry 0x10 [M 0x1b1d78-0x1b1d90]; without the bit
    /// the trigger stays live (and case 1 keeps falling through into case 2 while the channel is off).
    #[test]
    fn touch_once_delete_needs_the_glist_flag() {
        let mut ch = Channels::default();
        let on = [Toucher::player([0.5, -0.3, 0.0])];
        let away = [Toucher::player([5.0, 0.0, 5.0])];
        // Latch via touch, leave, then re-arm the channel externally: with the flag the plate deletes itself.
        let mut t = plate(kind::TOUCH_ONCE, 0x50, 8, vec![]);
        t.update(&mut ch, &on, &[], 1);
        assert!(ch.on(0x50));
        t.update(&mut ch, &away, &[], 2);
        assert!(t.gone, "glist 0x10 set: the TouchOnce deletes itself");
        // Without the glist bit the delete is refused (the port models every TouchOnce with the bit — live data).
        let mut ch2 = Channels::default();
        ch2.set(0x50, 1, 1);
        let mut keep = Trigger { glist_0x10: false, ..plate(kind::TOUCH_ONCE, 0x50, 8, vec![]) };
        keep.update(&mut ch2, &away, &[], 2);
        assert!(!keep.gone, "no glist 0x10: the trigger survives");
    }

    /// `GetLevelToLoadFromMovieCode` (0x1d3560): the story cut-scene codes map to the NEXT level's
    /// briefing code, the driving interludes map to the front end, and everything else falls back to
    /// `GameState+8 & 0xff00ffff` [M the decomp's every arm]. The numbers are the decomp's chain table:
    /// 06→07, 09→0a, 0a→0b, 11→12, 12→4a, 4a→13 ride exactly these pairs.
    #[test]
    fn movie_code_table_matches_the_decomp() {
        // The story cut-scene pairs (the movie AFTER level N loads level N+1) [M 0x1d3560:52-96].
        for (code, want) in [
            (0x0720_0001u32, 0x0710_0002), (0x0720_0002, 0x0710_0003), (0x0720_0003, 0x0710_0004),
            (0x0720_0005, 0x0710_0006), (0x0720_0006, 0x0710_0007), (0x0720_0007, 0x0710_0008),
            (0x0720_0009, 0x0710_000a), (0x0720_000a, 0x0710_000b), (0x0720_000c, 0x0710_000d),
            (0x0720_000e, 0x0710_000f), (0x0720_0011, 0x0710_0012), (0x0720_0012, 0x0710_004a),
            (0x0720_0014, 0x0710_0015), (0x0720_0015, 0x0710_0016), (0x0720_004a, 0x0710_0013),
        ] {
            assert_eq!(movie_code_next_level(code, 0x0bad_f00d), want, "movie {code:08x}");
        }
        // The driving interludes all land on the front end [M 0x1d3560:24-50].
        for code in [0x0730_0004u32, 0x0730_0008, 0x0730_000b, 0x0730_000d, 0x0730_000f, 0x0730_0010, 0x0730_0013, 0x0730_0017, 0x0730_001b] {
            assert_eq!(movie_code_next_level(code, 0x0bad_f00d), 0x0700_0048, "driving {code:08x}");
        }
        // Sisters: an unknown code and a plain level code fall back to the masked pending load.
        assert_eq!(movie_code_next_level(0x0720_0099, 0x0700_0004), 0x0700_0004, "unknown 0x72 → the masked fallback");
        assert_eq!(movie_code_next_level(0x0710_0006, 0), 0, "0x71 codes have no arm — the caller's fallback");
    }

    /// `resolve_exit_code`: the port's compression of `ResetMap_LevelToLoad` + `ResetMap_Load`'s
    /// phase-0/phase-2 cycle [M 0x1d3800/0x1d6360/0x1d3560]. The numbers are the decomp's chains:
    /// 4a's cut-scene request lands on 13, a driving exit lands on the front end, and a plain
    /// 0xE8 hashcode passes through untouched.
    #[test]
    fn resolve_exit_code_walks_the_game_pipeline() {
        assert_eq!(resolve_exit_code(0x0720_004a), 0x0700_0013, "4a's cut-scene movie → 13 (0x720004a → 0x7100013 → masked)");
        assert_eq!(resolve_exit_code(0x0720_0006), 0x0700_0007, "06's cut-scene movie would land on 07");
        assert_eq!(resolve_exit_code(0x0730_0013), 0x0700_0048, "13's driving interlude → the front end");
        assert_eq!(resolve_exit_code(0x0710_0006), 0x0700_0006, "a briefing wrapper masks to its level");
        assert_eq!(resolve_exit_code(0x0700_0007), 0x0700_0007, "a plain exit hashcode passes through");
        // Sister: the 0x720004a request the 4a SP actually fires is resolvable end to end.
        assert_eq!(resolve_exit_code(0x0720_004a), 0x0700_0013);
    }

    /// `ResetMap_LevelToLoad`'s wrap switch (no_intro == 0) [M 0x1d3800:27-101]: the Double Cross
    /// parts wrap to their 0x0710000N briefing codes (identity on the low byte), the Equinox
    /// special case and 4a wrap, and everything outside the jump table's 74 slots (0x07000001-4a)
    /// — the wrappers, the front end, the 0x0720/0x0730 movie codes — loads unwrapped.
    #[test]
    fn wrap_intro_follows_the_level_to_load_switch() {
        // The Double Cross chain [D: the parts' 0xE8 param-0s are exactly these codes].
        assert_eq!(wrap_intro(0x0700_0001), 0x0710_0001, "[M 0x1d3800:29-31]");
        assert_eq!(wrap_intro(0x0700_0002), 0x0710_0002, "[M 0x1d3800:33-36]");
        assert_eq!(wrap_intro(0x0700_0003), 0x0710_0003, "[M 0x1d3800:37-40]");
        assert_eq!(wrap_intro(0x0700_0004), 0x0710_0004, "[M 0x1d3800:41-44]");
        // The two special cases [M 0x1d3800:95-99].
        assert_eq!(wrap_intro(0x0700_001b), 0x0710_0018, "the Equinox briefing is movie 18");
        assert_eq!(wrap_intro(0x0700_004a), 0x0710_004a);
        // Outside the table: the wrappers, the front end, the movie codes load unwrapped [M].
        assert_eq!(wrap_intro(0x0700_0048), 0x0700_0048, "the front end has no wrap case");
        assert_eq!(wrap_intro(0x0700_0043), 0x0700_0043, "the wrappers load unwrapped");
        assert_eq!(wrap_intro(0x0720_004a), 0x0720_004a, "the 0x0720 cut-scene codes skip the wrap");
        assert_eq!(wrap_intro(0x0730_0013), 0x0730_0013, "the 0x0730 interlude codes skip the wrap");
        // Sister: a code inside the table's range but past the last case (0x0700001c-0x0700001a
        // gap — 0x1c..0x49 have no case [M 0x1d3800's table]) stays unwrapped.
        assert_eq!(wrap_intro(0x0700_001c), 0x0700_001c);
    }

    /// The Double Cross hops wrap and resolve straight back: `wrap_intro` (0x1d3800's switch)
    /// maps each part to its briefing code and phase 2 (`GetLevelToLoadFromMovieCode`
    /// 0x1d3560:97-98) strips the middle byte back off — the net code the loader finally loads is
    /// the part itself. The briefing codes 0x07100002/03/04 are NOT in `UsingMovie` 0x107eb8 [M
    /// :8-85 — only 0x07100001/05/09/0c/11/14/18], so no movie starts and state 6 pops on its
    /// first tick [M 0x1c98c8 case 6]; the whitelist intersection itself is pinned against the
    /// port's `nf_view::movie::USING_MOVIE` in the nf-view tests (the constant lives there).
    /// Sister: the mission-start code 0x07000001 wraps to 0x07100001, the one whitelisted Double
    /// Cross briefing.
    #[test]
    fn a_double_cross_hop_wraps_and_resolves_straight_back() {
        for (part, wrapped) in [(0x0700_0002, 0x0710_0002), (0x0700_0003, 0x0710_0003), (0x0700_0004, 0x0710_0004)] {
            assert_eq!(wrap_intro(part), wrapped, "[M 0x1d3800]");
            assert_eq!(resolve_exit_code(wrapped), part, "phase 2 strips the wrap back to the part [M 0x1d3560:97-98]");
        }
        // Sister: the mission-start briefing wrap is a different code (and the only Double Cross
        // one in UsingMovie — pinned against the port's whitelist in nf-view).
        assert_eq!(wrap_intro(0x0700_0001), 0x0710_0001);
    }

    // ===== mask & 2 — the class-5 (BULLET) contact arm, Trigger_Hit 0x1b1fc8:124-163 =====
    //
    // The contacts are the port's LIVE bullet objects: a grenade ticked through its real
    // `Grenade::update` (gravity, the real step integration) contributes its actual per-tick
    // swept segment, exactly what `Drones::tick` pushes into `bullet_contacts_now` and what the
    // game's bullet picker (`Bullet_update` arms its HITTEST 0x201/0x101, `Collide_PickObj`
    // 0x1eae80:93-107 + `Collide_Intersect`/`Intersect_CylGeom` vs the trigger's glist tris)
    // turns into the trigger's obj+0xD0 entry.

    /// The trigger-plate quad as a `BulletContact` source: a real satchel (0x3b) dropped through
    /// it. `satchel_seg` ticks the grenade once and returns the swept segment.
    fn tick_grenade_seg(g: &mut crate::grenade::Grenade, rng: &mut crate::drone::Rng) -> BulletContact {
        let prev = g.pos;
        let alive = g.state < 2;
        let _ = g.update(&crate::world::CollisionWorld::empty(), rng);
        assert!(alive, "grenade alive through the tick");
        BulletContact { a: prev, b: g.pos, id: g.weapon_type }
    }

    fn falling_grenade(id: u16) -> crate::grenade::Grenade {
        crate::grenade::Grenade {
            pos: [0.0, 1.5, 0.0], velocity: [0.0, -1.0, 0.0], speed: 1.0, travelled: 0.0, range: 300.0,
            damage: 0.0, radius: 0.0, mesh: 0, owner: usize::MAX, state: 0, bounces: 0,
            smoke_popped: false, resting: false, stuck: false, seek_target: None, phase: None,
            weapon_type: id,
        }
    }

    /// A satchel's flight segment crossing the plate fires a Touch plate whose filter wants the
    /// satchel's weapon-def id 0x3b [M 0x1b1fc8:136-158 — `*(short *)(filter) ==
    /// *(short *)(*(*(obj+0xE0)+0x44)+2)`; the satchel's `Grenade.weapon_type` IS that id, the
    /// throw site's `[wd[2], wd[3]]`]. The filter list here is the 937-census shape: one entry.
    #[test]
    fn a_satchel_flight_segment_fires_the_satchel_filtered_plate() {
        let mut ch = Channels::default();
        let mut plate = plate(kind::TOUCH, 0x40, 2, vec![0x3b]);
        let mut g = falling_grenade(0x3b);
        let mut rng = crate::drone::Rng::new(0x5eed);
        let mut fired_at = None;
        for t in 1..200u32 {
            let seg = tick_grenade_seg(&mut g, &mut rng);
            crate::triggers::calc(0, std::slice::from_mut(&mut plate), &mut ch, &[], &[seg], t);
            if ch.on(0x40) { fired_at = Some(t); break; }
        }
        assert!(fired_at.is_some(), "the satchel's flight through the plate fired it");
    }

    /// Sister [M 0x1b1fc8:144-148]: a different weapon's round does not satisfy the filter — the
    /// frag's id 0x34 walks the class-5 nodes, misses the 0x3b compare on every one, and the
    /// entry's carry (`cVar10 = cVar9`) returns 0.
    #[test]
    fn a_frags_flight_segment_does_not_fire_the_satchel_filtered_plate() {
        let mut ch = Channels::default();
        let mut plate = plate(kind::TOUCH, 0x40, 2, vec![0x3b]);
        let mut g = falling_grenade(0x34);
        let mut rng = crate::drone::Rng::new(0x5eed);
        for t in 1..200u32 {
            let seg = tick_grenade_seg(&mut g, &mut rng);
            crate::triggers::calc(0, std::slice::from_mut(&mut plate), &mut ch, &[], &[seg], t);
            assert!(!ch.on(0x40), "tick {t}: the frag's id 0x34 must not satisfy the 0x3b filter");
        }
    }

    /// Every nonzero filter entry must be satisfied [M 0x1b1fc8:136-158 — the AND carry across the
    /// ≤8 entries], each by SOME touching bullet. Two entries (0x3b + 0x34): both rounds through
    /// the plate in one tick fire it; either one alone does not.
    #[test]
    fn every_filter_entry_needs_a_matching_bullet() {
        let mut ch = Channels::default();
        let mut both = plate(kind::TOUCH, 0x40, 2, vec![0x3b, 0x34]);
        let mut satchel = falling_grenade(0x3b);
        let mut frag = falling_grenade(0x34);
        let mut rng = crate::drone::Rng::new(0x5eed);
        let mut satchel_crossed = false;
        for t in 1..200u32 {
            let seg_a = tick_grenade_seg(&mut satchel, &mut rng);
            let seg_b = tick_grenade_seg(&mut frag, &mut rng);
            crate::triggers::calc(0, std::slice::from_mut(&mut both), &mut ch, &[], &[seg_a], t);
            if ch.on(0x40) { panic!("tick {t}: the satchel alone must not satisfy both entries"); }
            crate::triggers::calc(0, std::slice::from_mut(&mut both), &mut ch, &[], &[seg_a, seg_b], t);
            if ch.on(0x40) { satchel_crossed = true; break; }
        }
        assert!(satchel_crossed, "both rounds through the plate satisfy both entries");
    }

    /// An EMPTY filter list returns 1 [M 0x1b1fc8:130-135 — `*(short *)(param_2 + 0x26) == 0`
    /// returns `cVar9 = 1`], and the unfiltered default-mask-2 plate (the `Trigger_Create`
    /// mask default [M 0x1b1720:80-84]) fires on ANY class-5 contact.
    #[test]
    fn an_unfiltered_mask2_plate_fires_on_any_round_through_it() {
        let mut ch = Channels::default();
        let mut any = plate(kind::TOUCH, 0x40, 2, vec![]);
        let mut g = falling_grenade(0x34);
        let mut rng = crate::drone::Rng::new(0x5eed);
        let mut fired = false;
        for t in 1..200u32 {
            let seg = tick_grenade_seg(&mut g, &mut rng);
            crate::triggers::calc(0, std::slice::from_mut(&mut any), &mut ch, &[], &[seg], t);
            if ch.on(0x40) { fired = true; break; }
        }
        assert!(fired, "the frag's flight through the unfiltered plate fired it");
    }

    /// A TouchOnce plate fires the same way through `Trigger_Update`'s case-1 path (the port's
    /// `update`) [M 0x1b1a78's case 1 → Trigger_Hit], and latches.
    #[test]
    fn a_touch_once_mask2_plate_fires_and_latches() {
        let mut ch = Channels::default();
        let mut once = plate(kind::TOUCH_ONCE, 0x41, 2, vec![0x3b]);
        let mut g = falling_grenade(0x3b);
        let mut rng = crate::drone::Rng::new(0x5eed);
        let mut fired = false;
        for t in 1..200u32 {
            let seg = tick_grenade_seg(&mut g, &mut rng);
            once.update(&mut ch, &[], &[seg], t);
            if ch.on(0x41) { fired = true; break; }
        }
        assert!(fired, "the satchel's flight fired the TouchOnce plate");
        // The case-1 delete runs on a LATER update while the channel is on [M 0x1b1d78-0x1b1d90].
        once.update(&mut ch, &[], &[], 999);
        assert!(once.gone, "a spent TouchOnce with glist 0x10 removes itself");
    }

    /// A STUCK gadget contributes nothing: the game's bullet picker gate (ht+0x94) is written only
    /// while the bullet's state < 2 [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f03c — `slti
    /// v0, state, 2` closes it otherwise; `Bullet_init` 0x12c668 zeroes it at spawn], so a satchel
    /// resting on the plate pushes no contact. The port's model of the same fact here: a
    /// zero-length segment (a == b — what a resting gadget would sweep) touches no triangle. A
    /// TOUCH plate's channel is the per-tick clear + OR [M Trigger_Calc 0x1b1900], so under
    /// rest-only contacts it reads 0 every tick — a resting gadget neither fires a fresh plate nor
    /// holds a crossed one on.
    #[test]
    fn a_stuck_gadget_resting_on_the_plate_neither_fires_nor_holds_it() {
        let mut ch = Channels::default();
        // The resting gadget: zero-length segment inside the volume, every tick.
        let rest = BulletContact { a: [0.0, 0.02, 0.0], b: [0.0, 0.02, 0.0], id: 0x3b };
        let mut fresh = plate(kind::TOUCH, 0x42, 2, vec![0x3b]);
        for t in 1..100u32 {
            crate::triggers::calc(0, std::slice::from_mut(&mut fresh), &mut ch, &[], &[rest], t);
            assert!(!ch.on(0x42), "tick {t}: the resting gadget's zero-length segment never fires the plate");
        }
        // The same rest-only contacts cannot hold a plate whose flight already crossed (the
        // per-tick clear + OR re-evaluates from THIS tick's contacts alone).
        let mut crossed = plate(kind::TOUCH, 0x40, 2, vec![0x3b]);
        let seg = BulletContact { a: [0.0, 0.5, 0.0], b: [0.0, -0.5, 0.0], id: 0x3b };
        crate::triggers::calc(0, std::slice::from_mut(&mut crossed), &mut ch, &[], &[seg], 1);
        assert!(ch.on(0x40), "the crossing segment fired the plate");
        crate::triggers::calc(0, std::slice::from_mut(&mut crossed), &mut ch, &[], &[rest], 2);
        assert!(!ch.on(0x40), "rest-only contacts: the Touch channel re-evaluates off");
        // The latch semantics live on the TouchOnce arm (pinned in the test above).
    }

    /// The 0x18 filter branch still returns first [M 0x1b1fc8:56-123]: with a player/drone contact
    /// on a type-1/2 trigger whose mask carries 0x18 bits, the filter chain consumes the call — its
    /// list entries are drone skin hashes there, NOT weapon ids — and the mask-2 arm never runs.
    /// Without a player/drone contact the same plate reads the mask-2 arm, where the list entries
    /// ARE weapon ids.
    #[test]
    fn the_player_arm_returns_before_the_mask2_filter() {
        let mut ch = Channels::default();
        let dual = plate(kind::TOUCH, 0x43, 0x1a, vec![0x3b]);
        let on = [Toucher::player([0.5, -0.3, 0.0])];
        let satchel = BulletContact { a: [0.0, 0.5, 0.0], b: [0.0, -0.5, 0.0], id: 0x3b };
        let frag = BulletContact { a: [0.0, 0.5, 0.0], b: [0.0, -0.5, 0.0], id: 0x34 };
        // No player: the mask-2 arm runs, the list entries are weapon ids.
        assert!(dual.hit(&ch, &[], &[satchel]), "the mask-2 filter matches the satchel's id [M :146-147]");
        assert!(!dual.hit(&ch, &[], &[frag]), "the mask-2 filter refuses the frag [M :144-148]");
        // A player contact: the 0x18 branch consumes the call — 0x3b is a drone-hash entry there
        // (no drone toucher → the entry fails) and the mask-2 arm is never reached.
        assert!(!dual.hit(&ch, &on, &[satchel]), "the 0x18 branch returned before the mask-2 arm [M :56-123]");
        assert!(!dual.hit(&ch, &on, &[]), "the empty-bullet case reads the same 0x18 chain");
        // The empty-list 0x1a plate: the 0x18 chain's base alone [M :70-72] — any player contact
        // fires with or without class-5 traffic.
        let mixed = plate(kind::TOUCH, 0x44, 0x1a, vec![]);
        assert!(mixed.hit(&ch, &on, &[]));
        assert!(mixed.hit(&ch, &on, &[frag]));
        assert!(mixed.hit(&ch, &[], &[satchel]), "no player: the mask-2 arm — an empty filter fires on any contact [M :130-135]");
    }
    /// The RAW 0xDC map (no wrapper): out = p1, the inputs = p2..p8, mask = p9 with the create's
    /// 0 -> 2 default, lock = p10, hide = p11 [M 0x1b1720:61-91]. 0700000c's skylight plates
    /// #275-277 are the shipped rows: p0=2 (Touch), p1=0x1b (the OUT — the objective channel),
    /// p9=0 -> mask 2 (class-5 bullets) [D census].
    #[test]
    fn the_raw_dc_rows_read_the_direct_create_map() {
        let params = [2u32, 0x1b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let q = move |k: u32| params.get(k as usize).copied().unwrap_or(0);
        let mut t = Trigger::from_params(0, q(0) as u8, q, Vec::new()).unwrap();
        apply_raw_dc_map(&mut t, &q);
        assert_eq!(t.out, 0x1b, "the raw out = slot 1 [M 0x1b1720:67-69]");
        assert_eq!(t.mask, 2, "mask slot 9 = 0 -> the create's default 2 [M 0x1b1720:80-84]");
        assert_eq!(t.lock, 0);
        assert!(!t.hide);
        assert_eq!(t.list, vec![0, 0, 0, 0, 0, 0, 0], "the inputs = slots 2..8");
        // A sister with the mask slot populated keeps the placed value.
        let params2 = [2u32, 0x1b, 0, 0, 0, 0, 0, 0, 0, 5, 7, 1];
        let q2 = move |k: u32| params2.get(k as usize).copied().unwrap_or(0);
        let mut t2 = Trigger::from_params(0, q2(0) as u8, q2, Vec::new()).unwrap();
        apply_raw_dc_map(&mut t2, &q2);
        assert_eq!(t2.mask, 5);
        assert_eq!(t2.lock, 7);
        assert!(t2.hide);
    }
}
