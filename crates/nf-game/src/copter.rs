//! The helicopter (`Copter_Create` 0x17d340, `Copter_Update` 0x17dbb8, `Copter_Move` 0x17d628;
//! placement type 0x2F — fix-objects2; the route — copter-path-flight).
//!
//! The game's model [M] (all addresses in `work/decomp/ACTION.ELF/`):
//! - `Copter_Create` (0x17d340): EVERY 0x2F placement first builds a 0x1c path node into
//!   `CopterPaths` (`LList_Add(0x316fe0, node)`, :42); only a placement whose parameter 0
//!   (level_tag+0x2C, the record's `+0x18` kind) is 0 then goes on to create the copter object
//!   (`if (*(int *)(param_3 + 0x2c) != 0) return 0;`, :43-45) — so kind-2 placements are
//!   route-only records. The node: `+0x8` the framelist (level_tag+0x8 = `m_paths[record+0x44]`,
//!   built by `parsemap_block_map_data_dynamic` 0x1d1b68 — the same resolution the doors use),
//!   `+0xc` the per-tick t step `count · REC_FRAME_RATE · (FRAME_RATE / scale)` with
//!   `REC_FRAME_RATE` = 1/60 [M 0x1bba38:26-27] and `FRAME_RATE` = 30 at load [M 0x30d0d0 — the
//!   doors' CREATE_FRAME_RATE; `Copter_Create` also runs at load], `+0x10` count =
//!   `frames - 1` (0 when frames is 0), `+0x12` group = param 1, `+0x14` reset-t =
//!   `min(count, param 3)`, `+0x16` pick index = param 2, `+0x18` kind = param 0. The record:
//!   `+0x54` = its own path node, `+0x5c` = 0 (the spline t), `+0x70..+0x76` = params 6..9 (the
//!   gate, the arrival channel, the countdown channel, the countdown value), `+0x78` = 0, the 9
//!   part slots `+0x30..+0x54` zeroed (the SP callback `Copter_Register` 0x17e530 fills slot i
//!   with script entity i and sets its +0xF0 bit 0x20 — slot 2 = the body, `Copter_GetBody`
//!   0x17e4b8 reads `*(rec+0x38)`), hp `+0x68` = `DiffMod · 299` [M :76-94].
//! - `Copter_Update` (0x17dbb8) head: a set gate whose channel is off returns; the first ON tick
//!   unhides the SP (`SP_HideObj(*(rec+0x2c), 0)`) and zeroes +0x70 [M :91-97]; then the +0x76
//!   countdown writes the +0x74 channel when it reaches 0 [M :98-103]; then the parts[6] recharge
//!   (gun B's +0xF4 counter → its +0xF0 bit 0x10, [M :104-113] — the port keeps no part objects;
//!   not modelled). The state machine at obj+0xF4:
//!   - case 0 (the first post-gate tick): walk `CopterPaths` for the record's own group
//!     (`+0x12 == my path's +0x12`): kind 2 → `rec+0x58` (the arrival path), every other match
//!     counts into `rec+0x78`; then state 1 [M :115-137].
//!   - case 1 (fly): `Copter_Move`, then the spot check behind `rec+0x74 != 100` [M :140-187]:
//!     the +0xF6 timer counts every case-1 tick; once it passes `FRAME_RATE_INT·4 / DiffMod`
//!     (240/DiffMod — `FRAME_RATE_INT` = 60 in play, the port's fixed rate) the spot test runs
//!     every tick until it passes: the player inside 1000 units (`Vec_SqDist3D(player+0x30,
//!     obj+0xC0) ≤ 1e6` [M :149-151]), inside the view cone (`Intersect_ConeSphere(COPT_COS,
//!     COPT_SIN, obj+0xC0, Mat_GetDir(obj+0x90), player+0x80)` [M :152-155 — COPT_COS =
//!     `PS2Sinf(3π/4)`, COPT_SIN = `PS2Sinf(π/4)` ≈ 0.7071, a 45° half-angle cone, `___static_
//!     initialization_and_destruction_0` 0x17e468:7-8]) and on line of sight
//!     (`Collide_LineOfSight(obj+0xC0 + dir·4, player+0x80, …, 0x2e)` [M :156-172]). A pass
//!     zeroes the timer, goes to state 2, zeroes rec+0x6c and draws `attackmode_159 =
//!     Rand_Rand(10)` [M :179-184].
//!   - case 2 (attack): `Music_Event(2,1)` every tick [M :189], `Copter_Move`, the +0xF6
//!     increment, then the shot: `attackmode_159 & 1` picks `Copter_FireGun` (odd) or
//!     `Copter_FireMissile` (even) [M :191-198]. `Copter_FireMissile` 0x17d9f0 draws
//!     `Rand_Rand(0x86e6)` EVERY call [M :47] — the low bit picks pod slot 8 (rec+0x50, even)
//!     or slot 7 (rec+0x4c, odd) [M :49-54] — then, on a tick where `GameState+0x34 %
//!     (FRAME_RATE_INT·DiffMod) == 0` [M :58-64] and the pod is registered, on 0x07000008 a
//!     further `Rand_Rand(8/DiffMod) == 0` aims the missile straight at the player (else the
//!     pod's `Mat_GetDir` facing) and `Bullet_init`s weapon 0x62 from `pod+0x80 + dir·(glist
//!     radius·1.5)` [M :65-97]. `Copter_FireGun` 0x17d828 keeps its 6-tick cadence (the port's
//!     `fire_gun_tick`). The exits [M :199-242]: no player or the timer past
//!     `FRAME_RATE_INT·5·DiffMod` (300·DiffMod) resets the timer; else with rec+0x6c ≤ 100 the
//!     spot test re-runs — a pass STAYS in state 2 (the damage tail runs below), a fail resets
//!     the timer; rec+0x6c > 100 resets it too. Every non-staying exit lands in state 1 with
//!     rec+0x6c zeroed [M :241-242] — the fly/attack pass cycle.
//!   - case 4 (crash/arrive): `Copter_Move`; while it returns 0 the state stays; when it returns
//!     1 the record adopts `rec+0x58` with t = 0, writes the arrival channel `+0x72` and goes to
//!     state 5 [M :244-254].
//!   - case 5 (arrived): `Copter_Move`; 0 → plain return (no damage tail); 1 → re-write `+0x72`
//!     and set obj+0xFE bit 1, every tick [M :255-275].
//!   - case 6 (explode): write `+0x72`, `Explode_Create` at obj+0x30, obj+0xFE bit 1 [M
//!     :265-275] — the explosion itself is not ported (no Explode objects in the sim).
//!   - the damage tail (reached by cases 0-4): the 9 part slots' hit chains accumulate
//!     `rec+0x64` (damage) and `rec+0x6c` [M :277-320] — the port keeps no part hit chains, so
//!     `damage` only moves if a caller pokes it (documented gap; live checks requested). The
//!     death: `sltiu(state-4, 2); bnez → skip` [M disasm 0x17e390-0x17e39c] — states 4 and 5 are
//!     exempt, every other state with `hp < damage` goes to state 4 — or straight to state 6 on
//!     0x0700000b/0x07000012 — and writes the arrival channel [M disasm 0x17e3a4-0x17e3c8].
//! - `Copter_Move` (0x17d628): the record's current path `+0x54` null → return 1; its count
//!   `+0x10` == 0 → return 0. Otherwise `Spline_Interp(t, framelist, 0, count, out, quat, 2)`
//!   (the door module's `interp`, the same port), the matrix lands in obj+0x90 (the C call
//!   passes param_3 = 0 → `Mat_Copy(out, obj+0x90)`), `t += path->step` [M :45-53]; when t
//!   overflows the count [M :63-114]: the +0x74 == 100 latch sets obj+0xFE bit 1, the next path
//!   is picked as `ratio = damage / (hp - hp·0.1)`; `ratio >= 0.5` → `Rand_Rand(rec+0x78)` [M
//!   disasm 0x17d718/0x17d748 — the decompile dropped the argument], else the deterministic
//!   `(uint)(rec+0x78 · ratio + 0.5)`, clamped to `rec+0x78`; the walk takes the FIRST CopterPaths
//!   entry whose group matches the current path's, whose pick index `+0x16` equals the pick and
//!   whose kind `+0x18` is not 2 (arrival paths are never adopted here); none → keep the current
//!   path; `t = new path's reset-t`; return 1 (transitioned). While flying it returns 0. Every
//!   flown tick ends with `SP_SetPosRot(*(rec+0x2c), obj+0x90)` — the SP's entities ride the
//!   copter matrix [M :115].

use crate::door::{self, Key};
use crate::triggers::Channels;
use crate::world::V3;
use nf_formats::placement::{object_matrix, Placement};
use nf_formats::anim::Mat4;

/// The load-time `FRAME_RATE` = 30 [M 0x30d0d0 — the doors' CREATE_FRAME_RATE; `Copter_Create`
/// also runs at load]. The step divides by `REC_FRAME_RATE` = 1/60 separately [M 0x1bba38:26-27].
const CREATE_FRAME_RATE: f32 = 30.0;

/// One `CopterPaths` node — the 0x1c record `Copter_Create` mallocs per 0x2F placement
/// (`Mem_Malloc(0x1c, 0x3304, 0)`, memset 0 [M 0x17d340:18-20]).
#[derive(Clone, Debug, Default)]
pub struct CopterPath {
    /// The node's placement (the port's list index source).
    pub placement: usize,
    /// `+0x8`: the framelist — `m_paths[record+0x44]` [M 0x1d1b68; the doors' resolution].
    pub keys: Option<Vec<Key>>,
    /// `+0xc`: the per-tick t step `count · REC_FRAME_RATE · (FRAME_RATE / scale)` [M 0x17d340:33-41].
    pub step: f32,
    /// `+0x10`: `frames - 1`, 0 when frames is 0 [M 0x17d340:28-32].
    pub count: u16,
    /// `+0x12`: the route group (param 1) [M 0x17d340:26].
    pub group: u16,
    /// `+0x14`: `min(count, param 3)` — the t a transition resets to [M 0x17d340:34-38].
    pub reset_t: u16,
    /// `+0x16`: the pick index (param 2) [M 0x17d340:39].
    pub pick: u16,
    /// `+0x18`: the kind (param 0): 0 = the copter's own path, 2 = the arrival path [M 0x17d340:25, 43-45].
    pub kind: u16,
}

impl CopterPath {
    /// `Copter_Create`'s node build over the main map's 0x2F placements [M 0x17d340:18-45].
    /// `path_blocks` is the level's 0x19 payloads (`level.path_blocks()` — the game's `m_paths`).
    pub fn create(placements: &[Placement], path_blocks: &[&[u8]]) -> Vec<CopterPath> {
        let mut out = Vec::new();
        for (i, p) in placements.iter().enumerate() {
            if p.flags & 0xff != 0x2f { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0u32, |&(_, v)| v);
            // The runtime record `parsemap_block_map_data_dynamic` (0x1d1b68) builds: record +0x40
            // (count) → level_tag+0x2, +0x42 (rate) → +0x4, `m_paths[record+0x44]` → +0x8 —
            // extra[7]'s halves and extra[8]'s low half (the doors' mapping, door.rs).
            let frames = (p.extra[7] & 0xffff) as u16;
            let count = frames.wrapping_sub(1); // [M 0x17d340:28-32 — 0 when frames is 0]
            let scale = {
                let over = param(10); // level_tag+0x54 [M 0x17d340:21-24]
                if over != 0 { over } else { (p.extra[7] >> 16) as u32 }
            };
            let step = count as f32 * (1.0 / 60.0) * (CREATE_FRAME_RATE / scale as f32);
            let keys = if frames != 0 {
                path_blocks.get((p.extra[8] & 0xffff) as usize).map(|b| door::parse_keys(b))
            } else {
                None
            };
            out.push(CopterPath {
                placement: i,
                keys,
                step,
                count,
                group: param(1) as u16,
                reset_t: (count as u32).min(param(3)) as u16,
                pick: param(2) as u16,
                kind: param(0) as u16,
            });
        }
        out
    }
}

#[derive(Clone, Debug)]
pub struct Copter {
    pub placement: usize,
    /// The placement's world matrix — obj+0x90, overwritten by `Copter_Move`'s spline pose.
    pub matrix: Mat4,
    /// record+0x70 = param 6: the gate channel [M 0x17d340:55, 0x17dbb8 head].
    pub gate: u16,
    /// record+0x72 = param 7: the arrival channel, written by cases 4/5/6 and the death [M].
    pub out: u16,
    /// record+0x74/+0x76 = params 8/9: the countdown pulse written when the countdown expires
    /// [M 0x17dbb8:98-103]. Both 0 on 07000008 [L the live read] — inert there, implemented anyway.
    pub arrival: u16,
    pub countdown: u16,
    /// The SP scripts (params 4/5): [assembly, flight] [D #45 = 0x06000051 / 0x06000050].
    pub scripts: [u32; 2],
    /// `rec+0x54`: the current path (its own node from create) [M 0x17d340:52].
    pub path: Option<usize>,
    /// `rec+0x58`: the group's arrival path (case 0's scan) [M 0x17dbb8:115-135].
    pub arrival_path: Option<usize>,
    /// `rec+0x5c`: the spline t [M 0x17d340:54; Copter_Move :53].
    pub t: f32,
    /// `rec+0x78`: the group's non-arrival path count (case 0's scan; the pick's range) [M].
    pub group_paths: u16,
    /// `obj+0xF4`: the state (0 scan, 1 fly, 4 crash/arrive, 5 arrived, 6 explode) [M 0x17dbb8].
    pub state: u16,
    /// `obj+0xF6`: the spot/attack timer [M 0x17dbb8:141]. Never moves while the spot check is
    /// not ported.
    pub timer: i16,
    /// `rec+0x64`: the accumulated damage [M 0x17dbb8:306 — the part walk sums every counted
    /// hit × the slot's Bits scale into it].
    pub damage: f32,
    /// `obj+0x68` = 299.0 x the difficulty multiplier [M `Copter_Create` 0x17d340:76-94]:
    /// d1 x1.0 (the decomp's fall-through default), d2 x1.5, d3 x2.0, d4 x4.0.
    pub hp: f32,
    /// obj+0xFE bit 1 — set by the +0x74 == 100 latch (Copter_Move) and cases 5/6 [M].
    pub landed: bool,
    /// The walk's per-tick `PlrStat_LogShotHitEnemy(0)` roll [M 0x17dbb8:314-316]: true when a
    /// counted hit's bullet traces back to the player (every port player bullet does). The
    /// caller folds it into the stats outbox; overwritten each `update`.
    pub shot_hit_enemy: bool,
    /// `*(rec+0x38)` — part slot 2, the body entity id (`Copter_Register` 0x17e530:7-8 fills the
    /// slots from the SP script entities; the port captures the body when the assembly script
    /// spawns it). None until captured.
    pub body_obj: Option<u32>,
    /// The gun parts' fire counters — the game keeps `*(short*)(gun+0xf4)` per gun obj
    /// (slots 5/6) [M `Copter_FireGun` 0x17d828:63-65, 93-96]. [0] = gun A (rec+0x44),
    /// [1] = gun B (rec+0x48).
    pub gun_counters: [u16; 2],
    /// `rec+0x6c`: the attack accumulator — the same walk feeds it beside rec+0x64 [M
    /// 0x17dbb8:307]; case 2's bail gate reads it against 100.0 [M :205]. Unported state 2
    /// reads nothing yet; the field exists because the game writes it on every counted hit.
    pub attack_acc: f32,
    /// This tick's bullet contacts on the part objects (`update_player_bullets` feeds them;
    /// the game's collide pass fills each part's obj+0xD0 chain and `Collide_Update` clears
    /// the chains at the head of the next frame [M 0x1ea398 head]). Consumed by `update`'s
    /// damage walk, then cleared.
    pub part_hits: Vec<PartHit>,
    /// Slot 3's object bounds centre (obj+0x80) — the gas emitter [M 0x17dbb8:331-348 reads
    /// `*(int*)(rec+0x3c)` and its +0x80]. None = the slot unregistered (`rec+0x3c == 0`,
    /// the branch's own gate). The caller refreshes it from the SP part targets each tick.
    pub slot3_pos: Option<V3>,
    /// The game's `attackmode_159` static — the state-2 entry draw `Rand_Rand(10)` [M 0x17dbb8
    /// :184] and its `& 1` gun/missile pick [M :191]. The game keeps ONE static shared by every
    /// copter; each level's attacking set is a single copter (05's patrols never leave state 1 —
    /// +0x74 = 100 keeps the spot check off, [D the placement params] — and 08/0b/12 carry one
    /// record each, [D the route dumps]), so the per-record field is observationally the same.
    pub attackmode: u16,
}

/// One bullet contact on a part object, the port's HITDATA entry (`Coll_AddHitToList` node: +4
/// next, +8 the damage float the caller stores [M Explode_Propagate 0x17c518:215-217 stores the
/// blast amount to +8; the bullet collide pass stores the weapon's contact damage], +0x58 the
/// class-5 bullet object). The port's player bullets are always Bond's class-3 shots, so the
/// walk's owner roll (PlrStat) folds into the stats outbox, not the entry.
#[derive(Clone, Copy, Debug)]
pub struct PartHit {
    /// The part slot the contact landed on (the game's `Coll_AddHitToList(part+0xD0)` — the
    /// port's slot = the SP stream index, `Copter_Register` 0x17e530:6).
    pub slot: u8,
    /// `HITDATA+8`: the hit's damage [M 0x17dbb8:291 reads it].
    pub damage: f32,
    /// The bullet's weapon class word (`weapon_data+0x10`) — what `Collide_FilterBullets`
    /// masks the slot's filter against [M 0x1ece30:26].
    pub class: u16,
}

/// The 9-entry `Bits` table the damage walk scales and filters with [D ACTION.ELF .data 0x2a37e8,
/// 72 bytes: scales 2.0, 3.0, 1.0, 2.0, 100.0 (filter 0x10), 1.0, then three zero entries; L the same 72 bytes,
/// identical in work/ee_snapshot.bin and work/ee_start_05.bin — `{u16 filter, f32 scale}` per
/// part]. Filters are 0 everywhere but slot 4 (0x10): a 0 filter keeps every hit
/// [M Collide_FilterBullets 0x1ece30:26 — `mask & 0 == 0` never drops]; slot 4 keeps only
/// weapon classes outside bit 0x10 (records 51/96/97/103/106-113 [L the same snapshot's
/// weapon_data 0x2bf150 census]) and scales the rest x100.
pub const PART_BITS: [(u16, f32); 9] = [
    (0x0000, 2.0),
    (0x0000, 3.0),
    (0x0000, 1.0),
    (0x0000, 2.0),
    (0x0010, 100.0),
    (0x0000, 1.0),
    (0x0000, 0.0),
    (0x0000, 0.0),
    (0x0000, 0.0),
];

/// `Mat_GetDir` 0x1e3398: the direction is the matrix's row 2 (bytes 0x20..0x2c) [M].
fn dir_of(m: &Mat4) -> V3 { [m[2][0], m[2][1], m[2][2]] }

/// `Intersect_ConeSphere` 0x1ee100 verbatim: 1 = the sphere (centre, radius) meets the
/// half-angle cone (cos, sin) from `apex` along `dir`. The apex inside the sphere returns 1
/// without the cone test [M :36-37, :54]; the centre inside the axial wedge returns 1 [M :47's
/// else]; otherwise the surface test `t = cos·proj + sin·√(|d² − proj²|)`, hit when
/// `t ≥ 0 && t² − d² + r² ≥ 0` [M :49-52].
fn cone_sphere(cos: f32, sin: f32, apex: V3, dir: V3, center: V3, radius: f32) -> bool {
    let d = [center[0] - apex[0], center[1] - apex[1], center[2] - apex[2]];
    let d2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    if radius * radius >= d2 {
        return true; // [M :37 — the outer `if (r² < d²)` never runs]
    }
    let proj = d[0] * dir[0] + d[1] * dir[1] + d[2] * dir[2];
    if proj * proj < d2 * cos * cos || proj <= 0.0 {
        let t = cos * proj + sin * (d2 - proj * proj).abs().sqrt();
        t >= 0.0 && t * t - d2 + radius * radius >= 0.0 // [M :49-52]
    } else {
        true // [M :47 — the centre inside the wedge returns 1]
    }
}

/// The per-difficulty health multiplier (`Copter_Create` 0x17d340:76-93): 2 -> 1.5, 3 -> 2.0,
/// 4 -> 4.0, everything else (d1) -> 1.0 [M].
pub fn copter_hp_mult(difficulty: u32) -> f32 {
    match difficulty {
        2 => 1.5,
        3 => 2.0,
        4 => 4.0,
        _ => 1.0,
    }
}

/// `DiffMod` — the same table `Copter_Create` 0x17d340:76-90 loads beside the hp (the hp IS
/// `DiffMod · 299`, [M :94]): d2 1.5, d3 2.0, d4 4.0, d1 (else) 1.0. The case machine reads it
/// for the spot-check period (240/DiffMod), the attack bail (300·DiffMod) and the missile fire
/// gate (60·DiffMod) [M 0x17dbb8:142, 0x17dbb8:200, 0x17d9f0:58-64].
pub fn diff_mod(difficulty: u32) -> f32 { copter_hp_mult(difficulty) }

/// The spot cone's `COPT_COS`/`COPT_SIN` [M 0x17e468:7-8 — `PS2Sinf(0x4016cbe4)`,
/// `PS2Sinf(0x3f490fdb)`]: 3π/4 and π/4, both sin ≈ 0.7071 — a 45° half-angle view cone.
pub const COPT_COS: f32 = 0.707_106_77;
pub const COPT_SIN: f32 = 0.707_106_77;

/// One registered part slot's pose (`Copter_Register` 0x17e530 fills `rec+0x30 + 4i` with SP
/// stream entity i): what `Copter_FireGun`/`Copter_FireMissile` read off the part objects.
#[derive(Clone, Copy, Debug, Default)]
pub struct PartPose {
    /// The part obj's position — obj+0x30, with obj+0xC0 in step (the sensor module's own
    /// equivalence, [M 0x1af010; the port keeps one translation per stream pose]).
    pub pos: V3,
    /// The part's world sphere centre (obj+0x80, `Control_BuildWorldSph`) — the missile's muzzle
    /// base [M 0x17d9f0:83].
    pub center: V3,
    /// The cel-glist sphere radius (`*(obj+0xD8)+0x38`) — the gun's muzzle offset, the missile's
    /// ×1.5 [M 0x17d828:76, 0x17d9f0:82].
    pub radius: f32,
    /// The part's world matrix — `Mat_GetDir` reads its row 2 (the pod's facing) [M 0x17d9f0:78].
    pub matrix: Mat4,
}

/// The spot test's inputs, resolved by the caller once per tick.
pub struct SpotCtx<'a> {
    /// The shared collision world — the spot check's `Collide_LineOfSight` [M 0x17dbb8:170-172].
    pub world: &'a crate::world::CollisionWorld,
    /// `glb_players+0x30` — the spot test's distance point [M :149].
    pub player_pos: V3,
    /// The player's world-sphere centre (obj+0x80) — the cone's target and the LOS endpoint
    /// [M :154-155, :172; L obj-tag-player.md].
    pub player_sph: V3,
    /// The world-sphere radius (obj+0x8C) [L obj-tag-player.md — 1.0576, the sensor's radius].
    pub player_radius: f32,
    /// `glb_players != 0` [M :144, :199].
    pub player_present: bool,
    /// `DiffMod` [M 0x17d340:76-90].
    pub diff_mod: f32,
    /// `GameState+0x34` — the tick counter the missile fire gate mods [M 0x17d9f0:58-64].
    pub frame: u32,
    /// `GameState+0x12` — the level id (0x07000008's missile arm [M 0x17d9f0:65]).
    pub level: u32,
}

/// One `Bullet_init` the case machine raised this tick — the caller spawns the shared `Bullet`.
#[derive(Clone, Copy, Debug)]
pub struct CopterShot {
    /// The muzzle [M 0x17d828:75-84 (gun) / 0x17d9f0:87-94 (missile)].
    pub muzzle: V3,
    pub dir: V3,
    /// The weapon record: the gun 0x61 / the missile 0x62 on 0x07000008, else 0x71 / 0x6d
    /// [M 0x17d828:86-89, 0x17d9f0:81-86].
    pub weapon: u16,
    /// The gun's `Sound_Play3D(100, −1, −1, 0x5df, muzzle, 0, 0, 0)` [M 0x17d828:85]; the
    /// missile's spawn is silent.
    pub sound: Option<u32>,
}

impl Copter {
    /// `Copter_Create` over the main map's 0x2F placements: the kind-0 ones get the copter
    /// object, every 0x2F one already made a `CopterPath` [M 0x17d340:18-45]. `paths` is
    /// `CopterPath::create`'s output over the same placements (the copter's own node is the last
    /// path built from its placement). `difficulty` is the selected difficulty (`GameState+0x28`,
    /// 0x2a3790) — the HP scales with it [M 0x17d340:76-94].
    pub fn create(placements: &[Placement], paths: &[CopterPath], difficulty: u32) -> Vec<Copter> {
        let mut out = Vec::new();
        for (i, p) in placements.iter().enumerate() {
            if p.flags & 0xff != 0x2f { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            if param(0) != 0 {
                continue; // [M 0x17d340:43-45 — a non-zero kind builds no copter]
            }
            let own = paths.iter().rposition(|c| c.placement == i);
            out.push(Copter {
                placement: i,
                matrix: object_matrix(p.rotation, p.translation),
                gate: param(6) as u16,
                out: param(7) as u16,
                arrival: param(8) as u16,
                countdown: param(9) as u16,
                scripts: [param(4), param(5)],
                path: own,
                arrival_path: None,
                t: 0.0, // [M 0x17d340:54]
                group_paths: 0, // [M 0x17d340:59]
                state: 0, // [M 0x17d340:63 — obj+0xF4 = 0]
                timer: 0,
                damage: 0.0, // [M 0x17d340:93]
                hp: copter_hp_mult(difficulty) * 299.0, // [M 0x17d340:94]
                landed: false,
                shot_hit_enemy: false,
                body_obj: None,
                gun_counters: [0; 2],
                attack_acc: 0.0, // [M 0x17d340:93 — +0x6c zeroed at create (memset + the +0x64 store)
                part_hits: Vec::new(),
                slot3_pos: None,
                attackmode: 0, // [M 0x17d340 — the static starts 0 (the C initializer)]
            });
        }
        out
    }

    /// `Copter_Move` 0x17d628 — one flight tick. Advances the spline t along the current path,
    /// poses obj+0x90 (`self.matrix`) from the interpolation, and on a path's end picks the next
    /// one (the damage-weighted rule). Returns `(game result, posed)`: the game's return — true =
    /// the path pointer is null or the path just transitioned, false = still flying (or an empty
    /// path) — and whether `SP_SetPosRot(*(rec+0x2c), obj+0x90)` ran (the valid-path branch,
    /// [M :49-115]).
    pub fn move_tick(&mut self, paths: &[CopterPath], rng: &mut crate::drone::Rng) -> (bool, bool) {
        let Some(pi) = self.path else { return (true, false) }; // [M :45-48 — +0x54 == 0 → 1]
        let (count, step) = (paths[pi].count, paths[pi].step);
        if count == 0 { return (false, false); } // [M :49 — the empty path returns 0]
        let mut posed = false;
        // Spline_Interp(t, framelist, 0, count, out, rec+0x1c, 2); param_3 == 0 → the pose lands
        // in obj+0x90 [M :50-55]. A NULL framelist writes nothing (the matrix stays) [M 0x1e1858:100-102].
        if let Some(keys) = &paths[pi].keys {
            if let Some(m) = door::interp(keys, 0, count, self.t, true) {
                self.matrix = m;
            }
        }
        self.t += step; // [M :53]
        posed = true; // SP_SetPosRot(*(rec+0x2c), obj+0x90) — the caller poses the SP [M :115]
        if (count as f32) < self.t {
            // The latch: a patrol copter (+0x74 == 100, the Mayhew value [L the level's params])
            // sets obj+0xFE bit 1 and still re-picks below [M :63-67].
            if self.arrival == 100 { self.landed = true; }
            // The pick: ratio = damage / (hp - hp*0.1); >= 0.5 → Rand_Rand(rec+0x78) [M disasm
            // 0x17d718/0x17d748], else (uint)(rec+0x78 * ratio + 0.5); clamp to rec+0x78 [M :71-80].
            let ratio = self.damage / (self.hp - self.hp * 0.1);
            let mut pick = if 0.5 <= ratio {
                rng.rand(self.group_paths as u32)
            } else {
                (self.group_paths as f32 * ratio + 0.5) as u32
            };
            if (self.group_paths as u32) < pick { pick = self.group_paths as u32; }
            // Walk CopterPaths from the head: the first entry in the current path's group whose
            // pick index matches and whose kind is not 2 [M :81-110].
            let group = paths[pi].group;
            let mut next = None;
            for (j, p) in paths.iter().enumerate() {
                if p.group != group { continue; }
                if p.pick as u32 == pick {
                    if p.kind != 2 { next = Some(j); break; }
                }
            }
            let dest = next.unwrap_or(pi); // [M :110 — none → keep the current path]
            self.t = paths[dest].reset_t as f32; // [M :113 — t = the (new) path's reset-t]
            return (true, posed); // [M :112 — uVar2 = 1: the path transitioned]
        }
        (false, posed)
    }

    /// `Copter_Update` 0x17dbb8 — one tick. Returns `(unhide, posed, music_combat, shots)`:
    /// - `unhide` — the gate's first ON tick: the caller runs `SP_HideObj(sp, 0)`
    ///   (`Scripts::unhide_sp`) — the game does it right here in the head, then zeroes +0x70 [M head];
    /// - `posed` — `SP_SetPosRot(sp, obj+0x90)` ran: the caller puts `self.matrix` on the SP
    ///   player [M Copter_Move :115];
    /// - `music_combat` — a case-2 tick raised `Music_Event(2,1)` [M :189]; the caller folds it;
    /// - `shots` — the tick's `Bullet_init` requests (the gun 0x17d828 / the missile 0x17d9f0);
    ///   the caller spawns the shared `Bullet`s and plays the gun sound.
    pub fn update(
        &mut self,
        paths: &[CopterPath],
        ch: &mut Channels,
        gas: &mut crate::gas::GasPool,
        frame: u32,
        explode_level: bool,
        rng: &mut crate::drone::Rng,
        ctx: &SpotCtx,
        parts: &[Option<PartPose>; 9],
    ) -> (bool, bool, bool, Vec<CopterShot>) {
        // The gate: a non-zero gate channel that is off stops everything [M 0x17dbb8 head]; on the
        // first ON tick the head unhides the SP and zeroes the field — the copter then updates
        // unconditionally, even if the channel turns off mid-flight [M `+0x70 = 0`].
        let mut unhide = false;
        if self.gate != 0 {
            if !ch.on(self.gate as u32) { return (false, false, false, Vec::new()); }
            self.gate = 0;
            unhide = true;
        }
        // The countdown pulse [M :98-103]: +0x76 counts down; hitting 0 writes the +0x74 channel.
        if self.countdown != 0 {
            self.countdown -= 1;
            if self.countdown == 0 && self.arrival != 0 { ch.set(self.arrival as u32, 1, frame); }
        }
        // The parts[6] recharge (gun B's +0xF4 → its +0xF0 bit 0x10) [M :104-113] — the port
        // keeps no part objects; not modelled.
        let mut posed = false;
        let mut music_combat = false;
        let mut shots: Vec<CopterShot> = Vec::new();
        match self.state {
            0 => {
                // Case 0 [M :115-137]: the CopterPaths scan for the record's own group.
                if !paths.is_empty() {
                    let my_group = self.path.map(|p| paths[p].group);
                    for (j, p) in paths.iter().enumerate() {
                        if Some(p.group) == my_group {
                            if p.kind == 2 { self.arrival_path = Some(j); }
                            else { self.group_paths += 1; }
                        }
                    }
                }
                self.state = 1;
                // Case 0 falls into the damage tail below (states 0-3 are death-eligible).
            }
            1 => {
                // Case 1 [M :138-187]: Copter_Move, then the spot check behind `rec+0x74 != 100`
                // (the Mayhew latch keeps the +0x74 == 100 patrols out entirely [M :140]): the
                // +0xF6 timer counts EVERY case-1 tick [M :141-143]; from
                // FRAME_RATE_INT·4/DiffMod on the cone+LOS test runs every tick until it passes.
                let (_, p) = self.move_tick(paths, rng);
                posed = p;
                if self.arrival != 100 {
                    self.timer = self.timer.wrapping_add(1); // [M :141-143]
                    if (240.0 / ctx.diff_mod) <= self.timer as f32
                        && ctx.player_present
                        && self.spot_visible(ctx)
                    {
                        self.timer = 0; // [M :181]
                        self.state = 2; // [M :182]
                        self.attack_acc = 0.0; // [M :183 — rec+0x6c]
                        self.attackmode = rng.rand(10) as u16; // [M :184 — Rand_Rand(10)]
                    }
                }
            }
            2 => {
                // Case 2 [M :188-243]: Music_Event(2,1) every tick (the caller folds the flag),
                // the move, the timer, the shot, then the exits.
                music_combat = true; // [M :189]
                let (_, p) = self.move_tick(paths, rng);
                posed = p;
                self.timer = self.timer.wrapping_add(1); // [M :192]
                if self.attackmode & 1 == 0 {
                    // Copter_FireMissile 0x17d9f0 [M :193-194]. The pod draw runs EVERY call
                    // [M :47]; the low bit picks slot 8 (rec+0x50, even) / slot 7 (rec+0x4c, odd)
                    // [M :49-54]; a missing pod returns after the draw [M :55-57]; then the
                    // 60·DiffMod fire gate [M :58-64]; on 0x07000008 the homing draw [M :65-76].
                    let pod_pick = rng.rand(0x86e6); // [M 0x17d9f0:47 — Rand_Rand(0x86e6)]
                    let slot = if pod_pick & 1 == 0 { 8 } else { 7 }; // [M :49-54]
                    if let Some(pod) = parts[slot] {
                        let gate = (60.0 * ctx.diff_mod) as u32; // [M :58 — fptoui(FRAME_RATE·DiffMod)]
                        if gate != 0 && ctx.frame % gate == 0 {
                            let dir = if ctx.level == 0x0700_0008
                                && rng.rand((8.0 / ctx.diff_mod) as u32) == 0
                            {
                                // [M :68-75 — dir = normalise(player+0x30 − pod+0xC0)]
                                let d = [
                                    ctx.player_pos[0] - pod.pos[0],
                                    ctx.player_pos[1] - pod.pos[1],
                                    ctx.player_pos[2] - pod.pos[2],
                                ];
                                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                                if l > 0.0 { [d[0] / l, d[1] / l, d[2] / l] } else { [0.0; 3] }
                            } else {
                                dir_of(&pod.matrix) // [M :78 — Mat_GetDir(pod+0x90)]
                            };
                            // [M :81-94 — muzzle = pod+0x80 + dir·(glist radius·1.5)]
                            shots.push(CopterShot {
                                muzzle: [
                                    pod.center[0] + dir[0] * pod.radius * 1.5,
                                    pod.center[1] + dir[1] * pod.radius * 1.5,
                                    pod.center[2] + dir[2] * pod.radius * 1.5,
                                ],
                                dir,
                                weapon: if ctx.level == 0x0700_0008 { 0x62 } else { 0x6d }, // [M :81-86]
                                sound: None,
                            });
                        }
                    }
                } else if let Some(gun) = parts[5] {
                    // Copter_FireGun 0x17d828 [M :196-197] — the 6-tick cadence and the muzzle
                    // live in fire_gun_tick; gun A is slot 5 (rec+0x44), gun B slot 6 (rec+0x48).
                    if let Some((muzzle, dir, weapon)) = self.fire_gun_tick(
                        parts[6].is_some(),
                        gun.pos,
                        gun.radius,
                        ctx.player_pos,
                        ctx.level == 0x0700_0008,
                    ) {
                        shots.push(CopterShot { muzzle, dir, weapon: weapon as u16, sound: Some(0x5df) }); // [M :85]
                    }
                }
                // The exits [M :199-242]: no player or the timer past FRAME_RATE_INT·5·DiffMod →
                // reset [M :200-202]; else with rec+0x6c ≤ 100 the spot test re-runs — a pass
                // STAYS in state 2 (the damage tail runs below, [M :236 → LAB_0017e180]), a fail
                // resets the timer [M :237]; rec+0x6c > 100 resets it too [M :238-239]. Every
                // non-staying exit lands in state 1 with rec+0x6c zeroed [M :241-242].
                let bail = !ctx.player_present
                    || (60.0 * 5.0) * ctx.diff_mod < self.timer as f32; // [M :200]
                let stay = !bail && self.attack_acc <= 100.0 && self.spot_visible(ctx); // [M :205-236]
                if !stay {
                    self.timer = 0; // [M :202 / :239 — LAB_0017e008]
                    self.state = 1; // [M :241]
                    self.attack_acc = 0.0; // [M :242]
                }
            }
            4 => {
                // Case 4 [M :244-254]: finish the current path, then adopt the arrival path with
                // t = 0 and write the arrival channel.
                let (r, p) = self.move_tick(paths, rng);
                posed = p;
                if r {
                    self.path = self.arrival_path;
                    self.t = 0.0;
                    if self.out != 0 { ch.set(self.out as u32, 1, frame); }
                    self.state = 5;
                }
            }
            5 => {
                // Case 5 [M :255-275]: fly the arrival path; 0 → plain return (no damage tail);
                // 1 → re-write the channel + obj+0xFE bit 1, every tick.
                let (r, p) = self.move_tick(paths, rng);
                if r {
                    if self.out != 0 { ch.set(self.out as u32, 1, frame); }
                    self.landed = true;
                    return (unhide, p, music_combat, shots);
                }
                return (unhide, p, music_combat, shots);
            }
            6 => {
                // Case 6 [M :265-275]: the channel, the explosion (not ported), the flag.
                if self.out != 0 { ch.set(self.out as u32, 1, frame); }
                self.landed = true;
                return (unhide, false, music_combat, shots);
            }
            _ => {}
        }
        // The damage tail [M :277-371]. The 9-part walk [M :277-320, jal FilterBullets
        // 0x17e1bc, the accumulate stores swc1 → +0x64/+0x6c at 0x17e23c/0x17e250]: per slot,
        // `Collide_FilterBullets` drops the bullet hits whose weapon class ANDs the slot's
        // Bits filter [M 0x1ece30:25-33], then every remaining entry with damage > 0 adds
        // `damage · Bits.scale` to BOTH rec+0x64 and rec+0x6c [M :293-307]. The port's
        // `part_hits` are this tick's contacts (update_player_bullets); the owner roll inside
        // the counted branch — the class-5 bullet whose BU+0x30 shooter is class 3 — is true
        // for every port player bullet, and sets the game's `PlrStat_LogShotHitEnemy(0)`
        // [M :294-304 + :314-316, jal 0x199840]: the caller folds it into the stats outbox.
        let mut shot_hit_enemy = false;
        for i in 0..9 {
            let (filter, scale) = PART_BITS[i];
            for h in self.part_hits.iter().filter(|h| h.slot as usize == i) {
                if h.class & filter != 0 { continue; } // [M 0x1ece30:26-33 — the hit is dropped]
                if h.damage > 0.0 {
                    self.damage += h.damage * scale; // [M :306]
                    self.attack_acc += h.damage * scale; // [M :307]
                    shot_hit_enemy = true;
                }
            }
        }
        self.part_hits.clear();
        self.shot_hit_enemy = shot_hit_enemy;
        // The smoke/gas branch [M :321-353]: past half hp, with the slot-3 emitter present
        // (`rec+0x3c != 0`), `Rand_Rand(10)` rolls every tick and the >= 8 arm spawns
        // `Gas_Init` at the emitter's obj+0x80 with amount `(1 - damage/hp) · 127` packing the
        // colour word [M :339-350]; the arm's own draws are `Rand_FRand(π)` [:341] and
        // `Rand_FRand(2.0)` [:346] (jal 0x1e3780 at 0x17e320/0x17e338). Gas_Init's internal
        // spin draws are kind-3's two [M Gas_Init 0x1aacc8:72-80 — the searchlight's
        // init_searchlight_smoke consumes the same pair]; the port's `init_copter_smoke`
        // replays them through the shared RNG in the game's order.
        if self.hp * 0.5 < self.damage {
            if let Some(p) = self.slot3_pos {
                if rng.rand(10) >= 8 {
                    let ratio = 1.0 - self.damage / self.hp;
                    let amount = (ratio * 127.0) as u8; // [M :339]
                    let yaw = rng.frand(std::f32::consts::PI); // [M :341]
                    let size = rng.frand(2.0) + 3.0; // [M :346-348]
                    gas.init_copter_smoke(rng, p, yaw, size, amount);
                }
            }
        }
        // The death: states 4/5 exempt (`sltiu(state-4, 2)` skips, disasm 0x17e390-0x17e39c);
        // every other state with hp < damage → 4, or 6 on 0x0700000b/0x07000012 [M :358-370];
        // the arrival channel writes either way [M disasm 0x17e3bc-0x17e3c8].
        let exempt = self.state == 4 || self.state == 5;
        if !exempt && self.hp < self.damage {
            self.state = if explode_level { 6 } else { 4 };
            if self.out != 0 { ch.set(self.out as u32, 1, frame); }
        }
        (unhide, posed, music_combat, shots)
    }

    /// The spot test's visible half [M 0x17dbb8:152-172]: the view cone
    /// (`Intersect_ConeSphere(COPT_COS, COPT_SIN, obj+0xC0, Mat_GetDir(obj+0x90), player+0x80)`)
    /// and `Collide_LineOfSight(obj+0xC0 + dir·4, player+0x80, …, 0x2e)`. The caller owns the
    /// player-presence and 1000-unit distance gates [M :144-151].
    fn spot_visible(&self, ctx: &SpotCtx) -> bool {
        let pos = self.pos();
        let dir = dir_of(&self.matrix); // Mat_GetDir [M :152]
        if !cone_sphere(COPT_COS, COPT_SIN, pos, dir, ctx.player_sph, ctx.player_radius) {
            return false; // [M :153-155]
        }
        // [M :156-169 — the probe pos + dir·4.0 (uStack_d0 = 0x40800000)]
        let probe = [
            pos[0] + dir[0] * 4.0,
            pos[1] + dir[1] * 4.0,
            pos[2] + dir[2] * 4.0,
        ];
        // [M :170-172 — build_FindCel + Collide_LineOfSight(probe, player+0x80, cel, obj,
        // player, 0x2e); the port's shared walk is the documented equivalent]
        ctx.world.line_of_sight(probe, ctx.player_sph, crate::world::LOS_COPTER_SPOT)
    }

    /// The copter's world position (the matrix row 3) — the viewer's draw/cull helpers.
    pub fn pos(&self) -> V3 { [self.matrix[3][0], self.matrix[3][1], self.matrix[3][2]] }

    /// `Copter_GetBody` 0x17e4b8:6-9 — the body entity, with the hide-gate folded in: a set gate
    /// whose channel is off means the body is `SP_HideObj`'d (Create 0x17d340:73-74 hides it,
    /// Update 0x17dbb8:95 unhides) and the accessor reads as absent. `gate_channel_on` is the
    /// caller's `switch_channels[gate] != 0` read.
    pub fn body(&self, gate_channel_on: bool) -> Option<u32> {
        if self.gate != 0 && !gate_channel_on {
            return None; // [M 0x17e4b8:6-8]
        }
        self.body_obj // [M 0x17e4b8:9 — *(rec+0x38)]
    }

    /// `Copter_FireGun` 0x17d828 — one attack tick. Only gun A (`rec+0x44`, slot 5) counts and
    /// fires [M :58-65]; gun B (`rec+0x48`) is only re-primed to 2 after a shot (with its +0xf0
    /// bit 0x10 cleared) when present [M :94-96]. `gun` is gun A's
    /// world position (the game reads it from the gun obj's +0xc0), `radius` the gun's cel-glist
    /// sphere radius (`*(glist)+0x38` [M :66-84]), `player` Bond's +0x30 with y+0.5 [M :55-59],
    /// `level_is_07000008` selects the weapon id (0x61 on 0700008, else 0x71 [M :86-89]).
    /// Returns Some((muzzle, dir, weapon_id)) on the tick the gun fires, else None.
    ///
    /// [M] the cadence: gun A's counter pre-increments and fires when the OLD value > 4 — every
    /// 6th call (:63-65). [M] after a fire gun A's counter resets to 0 and gun B's (when
    /// `gun_b_present`) is set to 2 (:93-96). No RNG here (the gun/missile pick is the caller's
    /// `Rand_Rand(10)` at 0x17dbb8:184).
    pub fn fire_gun_tick(
        &mut self,
        gun_b_present: bool,
        gun: V3,
        radius: f32,
        player: V3,
        level_is_07000008: bool,
    ) -> Option<(V3, V3, u32)> {
        let old = self.gun_counters[0] as i16; // signed short [M :63 `short sVar1`]
        self.gun_counters[0] = (old as u16).wrapping_add(1); // [M :63-65 pre-increment]
        if old <= 4 {
            return None; // [M :65 — fires only when the old value > 4]
        }
        let target = [player[0], player[1] + 0.5, player[2]]; // [M :55-59]
        let mut dir = [target[0] - gun[0], target[1] - gun[1], target[2] - gun[2]];
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if len > 0.0 {
            dir = [dir[0] / len, dir[1] / len, dir[2] / len]; // [M :66-72 normalise]
        }
        let muzzle = [gun[0] + dir[0] * radius, gun[1] + dir[1] * radius, gun[2] + dir[2] * radius];
        // [M :66-84 muzzle = gun pos + dir * cel-glist radius]
        let weapon = if level_is_07000008 { 0x61 } else { 0x71 }; // [M :86-89]
        self.gun_counters[0] = 0; // [M :93]
        if gun_b_present {
            self.gun_counters[1] = 2; // [M :94-95]
        }
        Some((muzzle, dir, weapon))
    }

    /// `Copter_Delete` 0x17e568:8-12 — remove the copter from the port's list (the game's
    /// `LList_Remove(0x2a3830, ext)` + the 0x7c zeroing; the SP part delete is the script
    /// machinery's). Returns true when a copter actually left. Live once the death path
    /// (Copter_Update 0x17dbb8:277-371) pokes the delete dispatch.
    pub fn delete(copters: &mut Vec<Copter>, index: usize) -> bool {
        if index < copters.len() {
            copters.remove(index);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::door::{interp, parse_keys};

    fn copter() -> Copter {
        Copter {
            placement: 0,
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            gate: 0,
            out: 0,
            arrival: 0,
            countdown: 0,
            scripts: [0; 2],
            path: None,
            arrival_path: None,
            t: 0.0,
            group_paths: 0,
            state: 0,
            timer: 0,
            damage: 0.0,
            hp: 299.0,
            landed: false,
            body_obj: Some(0x1234),
            gun_counters: [0; 2],
            attack_acc: 0.0,
            part_hits: Vec::new(),
            slot3_pos: None,
            shot_hit_enemy: false,
            attackmode: 0,
        }
    }

    /// A spot context for the machine tests: an empty world (every LOS clear), the player at
    /// `player_pos` (the sphere centre on it), d1 (DiffMod 1.0), 0x07000008.
    fn spot_ctx(world: &crate::world::CollisionWorld, player_pos: V3) -> SpotCtx<'_> {
        SpotCtx {
            world,
            player_pos,
            player_sph: player_pos,
            player_radius: crate::sensor::TARGET_RADIUS,
            player_present: true,
            diff_mod: 1.0,
            frame: 0,
            level: 0x0700_0008,
        }
    }

    /// A four-node straight-line framelist: x = 0, 600, 1200, 1800, identity quats (28-byte
    /// keys: pos xyz + quat xyzw — `door::parse_keys`).
    fn keys4() -> Vec<Key> { parse_keys(&{
        let mut b = Vec::new();
        for x in [0.0f32, 600.0, 1200.0, 1800.0] {
            b.extend_from_slice(&x.to_le_bytes()); b.extend_from_slice(&0.0f32.to_le_bytes()); b.extend_from_slice(&0.0f32.to_le_bytes()); // pos
            b.extend_from_slice(&0.0f32.to_le_bytes()); b.extend_from_slice(&0.0f32.to_le_bytes());
            b.extend_from_slice(&0.0f32.to_le_bytes()); b.extend_from_slice(&1.0f32.to_le_bytes()); // quat identity
        }
        b
    }) }

    fn path(group: u16, kind: u16, pick: u16, count: u16, reset: u16, keys: Option<Vec<Key>>) -> CopterPath {
        CopterPath { placement: 0, keys, step: 0.05, count, group, reset_t: reset, pick, kind }
    }

    // [M 0x17d628:45-49] the return values: a null path → 1, an empty path → 0.
    #[test]
    fn move_tick_returns_the_games_result() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        assert!(c.move_tick(&[], &mut rng).0, "+0x54 == 0 → 1");
        let paths = vec![path(1, 0, 0, 0, 0, Some(keys4()))];
        c.path = Some(0);
        assert!(!c.move_tick(&paths, &mut rng).0, "count == 0 → 0");
    }

    // [M 0x17d628:50-63] the flight: the pose lands in obj+0x90 and t advances by the step.
    #[test]
    fn move_tick_poses_the_matrix_and_advances_t() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        let paths = vec![path(1, 0, 0, 3, 0, Some(keys4()))];
        c.path = Some(0);
        assert!(!c.move_tick(&paths, &mut rng).0, "still flying → 0");
        assert!((c.t - 0.05).abs() < 1e-5, "t += step");
        // The pose: t≈0 sits at key 0 (the origin); after enough ticks the x grows along the line.
        assert!(c.matrix[3][0].abs() < 60.0, "t≈0 sits at key 0");
        for _ in 0..50 { let _ = c.move_tick(&paths, &mut rng); }
        assert!(c.matrix[3][0] > 100.0, "the copter moved along the path (x = {})", c.matrix[3][0]);
    }

    // [M 0x17d628:71-80] the pick: undamaged → deterministic 0; heavy damage → the shared RNG.
    #[test]
    fn move_tick_picks_the_next_path() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        let keys = keys4();
        // group 1: path 0 (pick 0, count 3), path 1 (pick 1), the arrival path kind 2 (pick 9).
        let paths = vec![
            path(1, 0, 0, 3, 0, Some(keys.clone())),
            path(1, 0, 1, 3, 0, Some(keys.clone())),
            path(1, 2, 9, 3, 0, Some(keys.clone())),
        ];
        c.path = Some(0);
        c.group_paths = 2; // case 0 counted the two kind-0 paths
        c.t = 3.5; // past count 3: the transition runs this tick
        assert!(c.move_tick(&paths, &mut rng).0, "the transition returns 1");
        assert_eq!(c.path, Some(0), "ratio 0 → pick (uint)(2·0+0.5) = 0 → the pick-0 path");
        assert!((c.t - 0.0).abs() < 1e-5, "t = the new path's reset-t");
        // A pick landing on an arrival path's index skips it (kind 2 is never adopted) [M :93-99].
        let paths2 = vec![
            path(1, 0, 0, 3, 0, Some(keys.clone())),
            path(1, 2, 1, 3, 0, Some(keys.clone())), // the only pick-1 entry is the arrival path
        ];
        c.path = Some(0);
        c.group_paths = 2;
        c.t = 3.5;
        assert!(c.move_tick(&paths2, &mut rng).0);
        assert_eq!(c.path, Some(0), "pick 1 finds only the kind-2 path → the walk keeps the current path");
        // The clamp [M :78-80].
        c.damage = 299.0; // ratio = 299/269.1 > 0.5 → the random arm
        c.path = Some(0);
        c.t = 3.5;
        let draws = rng.draws;
        let _ = c.move_tick(&paths2, &mut rng).0;
        assert_eq!(rng.draws, draws + 1, "the heavy-damage arm draws Rand_Rand(rec+0x78) [M disasm 0x17d748]");
    }

    // [M 0x17d340:18-45 + 0x1d1b68] the node build: params → fields, the step formula, the m_paths
    // resolution. [S] the placement below is synthetic; the ISO test pins real records.
    #[test]
    fn path_node_build_matches_the_record() {
        let mut gas = crate::gas::GasPool::default();
        let mut p = Placement {
            index: 0, hashcode: u32::MAX, flags: 0x2f,
            translation: [0.0; 3], rotation: [0.0; 3], params: Vec::new(), extra: [0; 9],
        };
        p.extra[7] = (30u32 << 16) | 121; // rate 30, frames 121
        p.extra[8] = 3; // m_paths[3]
        p.params = vec![(0, 0), (1, 7), (2, 2), (3, 10), (4, 1), (5, 2), (6, 5), (7, 0x4b), (8, 0), (9, 0)];
        let blocks: Vec<&[u8]> = vec![&[], &[], &[], &[0u8; 28 * 121]];
        let paths = CopterPath::create(std::slice::from_ref(&p), &blocks);
        assert_eq!(paths.len(), 1);
        let n = &paths[0];
        assert_eq!(n.count, 120, "frames - 1 [M 0x17d340:28-32]");
        assert_eq!(n.group, 7, "param 1");
        assert_eq!(n.pick, 2, "param 2");
        assert_eq!(n.reset_t, 10, "min(count, param 3) [M 0x17d340:34-38]");
        assert_eq!(n.kind, 0, "param 0");
        // step = count · (1/60) · (30/30) = 2.0
        assert!((n.step - 2.0).abs() < 1e-5, "step = count·REC_FRAME_RATE·(FRAME_RATE/scale) [M 0x17d340:33-41]");
        assert!(n.keys.is_some(), "m_paths[extra[8]] resolved");
        // A non-zero param 0 (an arrival path) still builds the node [M :42].
        p.params[0] = (0, 2);
        let paths2 = CopterPath::create(std::slice::from_ref(&p), &blocks);
        assert_eq!(paths2[0].kind, 2);
    }

    // [M 0x17dbb8:115-137] case 0: the group scan fills rec+0x58 and rec+0x78, then state 1.
    #[test]
    fn case_0_scans_the_group() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.path = Some(0);
        let keys = keys4();
        let paths = vec![
            path(7, 0, 0, 3, 0, Some(keys.clone())),
            path(9, 0, 0, 3, 0, Some(keys.clone())), // another group — ignored
            path(7, 0, 1, 3, 0, Some(keys.clone())),
            path(7, 2, 0, 3, 0, Some(keys.clone())), // the arrival path
        ];
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        c.update(&paths, &mut ch, &mut gas, 0, false, &mut rng, &ctx, &[None; 9]);
        assert_eq!(c.state, 1, "case 0 → 1");
        assert_eq!(c.arrival_path, Some(3), "the kind-2 group member → rec+0x58");
        assert_eq!(c.group_paths, 2, "the kind-0 group members counted into rec+0x78");
    }

    // [M 0x17dbb8 head, :244-254] the gate edge and case 4: the arrival path + the channel.
    #[test]
    fn gate_edge_then_case_4_writes_the_arrival() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.gate = 5;
        c.out = 0x4b;
        c.path = Some(0);
        let keys = keys4();
        let paths = vec![
            path(7, 0, 0, 1, 0, Some(keys.clone())),
            path(7, 2, 9, 1, 0, Some(keys.clone())),
        ];
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        // The gate off: nothing runs.
        let (unhide, posed, _, _) = c.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx, &[None; 9]);
        assert!(!unhide && !posed, "gated off → nothing");
        assert_eq!(c.state, 0, "still state 0 behind the gate");
        ch.set(5, 1, 0);
        let (unhide, _, _, _) = c.update(&paths, &mut ch, &mut gas, 2, false, &mut rng, &ctx, &[None; 9]);
        assert!(unhide, "the gate's first ON tick unhides [M head]");
        assert_eq!(c.gate, 0, "+0x70 zeroed [M head]");
        assert_eq!(c.state, 1, "case 0 ran this tick");
        assert_eq!(c.arrival_path, Some(1));
        assert_eq!(c.group_paths, 1);
        // Case 4: drive the current path to its end.
        c.state = 4;
        c.t = 3.5; // past count 1
        let mut fired = false;
        for f in 3..10 {
            let (_, _, _, _) = c.update(&paths, &mut ch, &mut gas, f, false, &mut rng, &ctx, &[None; 9]);
            if ch.on(0x4b) { fired = true; break; }
        }
        assert!(fired, "case 4's transition writes rec+0x72 [M :249-252]");
        assert_eq!(c.state, 5);
        assert_eq!(c.path, Some(1), "the arrival path adopted with t = 0");
        assert!((c.t - 0.0).abs() < 1e-5);
    }

    // [M :255-275 + disasm 0x17e390-0x17e39c] state 5 re-writes the channel on the path's end and
    // is exempt from the death; a state-1 copter with hp < damage goes to 4 (6 on 0b/12).
    #[test]
    fn state_5_holds_and_the_death_takes_the_other_states() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.out = 0x4b;
        c.path = Some(0);
        let keys = keys4();
        let paths = vec![path(7, 0, 0, 1, 0, Some(keys.clone()))];
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        // State 5, the path just ended: the channel re-writes and obj+0xFE sets, every tick.
        c.state = 5;
        c.t = 3.5;
        let (_, _, _, _) = c.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx, &[None; 9]);
        assert!(c.landed, "obj+0xFE bit 1 [M LAB_0017e16c]");
        assert!(ch.on(0x4b), "the channel re-write [M :260-263]");
        // The death: state 1 with hp < damage → 4 (07000005-like), 6 on 0b/12.
        let mut d = copter();
        d.path = Some(0);
        d.hp = 299.0;
        d.damage = 300.0;
        let (_, _, _, _) = d.update(&paths, &mut ch, &mut gas, 2, false, &mut rng, &ctx, &[None; 9]);
        assert_eq!(d.state, 4, "hp < damage outside states 4/5 → 4 [M disasm 0x17e3b8-0x17e3c0]");
        assert!(ch.on(0x4b), "the death writes the arrival channel [M disasm 0x17e3bc]");
        let mut e = copter();
        e.path = Some(0);
        e.hp = 299.0;
        e.damage = 300.0;
        let (_, _, _, _) = e.update(&paths, &mut ch, &mut gas, 3, true, &mut rng, &ctx, &[None; 9]);
        assert_eq!(e.state, 6, "0x0700000b/0x07000012 explode instead [M :360-363]");
        // States 4/5 are exempt (sltiu(state-4, 2) skips, disasm 0x17e398-0x17e39c).
        let mut g = copter();
        g.path = Some(0);
        g.state = 4;
        g.hp = 299.0;
        g.damage = 300.0;
        let (_, _, _, _) = g.update(&paths, &mut ch, &mut gas, 4, false, &mut rng, &ctx, &[None; 9]);
        assert_eq!(g.state, 4, "state 4 is exempt from the death");
    }

    // [M 0x17dbb8:98-103] the countdown pulse: +0x76 hits 0 → the +0x74 channel.
    #[test]
    fn countdown_pulses_its_channel() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.countdown = 2;
        c.arrival = 0x33;
        let paths = Vec::new();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        c.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx, &[None; 9]);
        assert!(!ch.on(0x33), "1 tick left: no pulse yet");
        c.update(&paths, &mut ch, &mut gas, 2, false, &mut rng, &ctx, &[None; 9]);
        assert!(ch.on(0x33), "the countdown hit 0 → ch +0x74 [M :99-103]");
        assert_eq!(c.countdown, 0);
    }

    // [M 0x17e4b8:6-9] the hide-gate: gate set + channel off → absent; channel on or gate 0 → body.
    #[test]
    fn body_respects_the_hide_gate() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.gate = 5;
        assert_eq!(c.body(false), None, "gated and off → SP_HideObj'd, absent");
        assert_eq!(c.body(true), Some(0x1234), "gated and on → the slot-2 body");
        c.gate = 0;
        assert_eq!(c.body(false), Some(0x1234), "no gate → always visible");
    }

    // [M 0x17d828:63-65] the 6-tick cadence: fires when the OLD counter value > 4.
    #[test]
    fn fire_gun_fires_every_sixth_call() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        let player = [100.0, 10.0, 0.0];
        let gun = [0.0, 0.0, 0.0];
        for i in 0..5 {
            assert!(c.fire_gun_tick(true, gun, 2.0, player, false).is_none(), "call {i} must not fire");
        }
        let shot = c.fire_gun_tick(true, gun, 2.0, player, false);
        assert!(shot.is_some(), "call 6 fires [M :65]");
    }

    // [M 0x17d828:86-89] the weapon id: 0x71 normally, 0x61 on 0700008.
    #[test]
    fn fire_gun_weapon_id_per_level() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        for _ in 0..5 { c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false); }
        assert_eq!(c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false).unwrap().2, 0x71);
        let mut c2 = copter();
        for _ in 0..5 { c2.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], true); }
        assert_eq!(c2.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], true).unwrap().2, 0x61);
    }

    // [M 0x17d828:93-96] after a fire: firing counter resets to 0, the sister's becomes 2.
    #[test]
    fn fire_gun_swaps_the_counters() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        for _ in 0..5 { c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false); }
        c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false);
        assert_eq!(c.gun_counters[0], 0);
        assert_eq!(c.gun_counters[1], 2);
        // gun B never fires itself: only gun A's counter moves on later calls [M :58-65].
        c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false);
        assert_eq!(c.gun_counters, [1, 2]);
        // sister: no gun B → its counter is left alone [M :94 the null test]
        let mut d = copter();
        for _ in 0..6 { d.fire_gun_tick(false, [0.0; 3], 1.0, [0.0; 3], false); }
        assert_eq!(d.gun_counters, [0, 0]);
    }

    // [M 0x17d828:66-84] the muzzle sits radius units along the aim direction from the gun.
    #[test]
    fn muzzle_is_radius_along_the_aim() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        for _ in 0..5 { c.fire_gun_tick(true, [0.0; 3], 1.0, [0.0; 3], false); }
        let (muzzle, dir, _) = c.fire_gun_tick(true, [0.0, 0.0, 0.0], 3.0, [4.0, 3.0, 0.0], false).unwrap();
        // dir = normalise(4, 3.5, 0); muzzle = dir * 3.
        let len = (4.0f32 * 4.0 + 3.5 * 3.5).sqrt();
        let want = [4.0 / len * 3.0, 3.5 / len * 3.0, 0.0];
        assert!((dir[0] - 4.0 / len).abs() < 1e-5 && (dir[1] - 3.5 / len).abs() < 1e-5);
        assert!((muzzle[0] - want[0]).abs() < 1e-5 && (muzzle[1] - want[1]).abs() < 1e-5);
        // [M :55-59] the target's y carries the +0.5.
        assert!(dir[1] > 0.0);
    }

    // [M 0x17e568:8-12] the delete removes the copter from the list.
    #[test]
    fn delete_removes_from_the_list() {
        let mut gas = crate::gas::GasPool::default();
        let mut v = vec![copter(), copter()];
        v[1].placement = 7;
        assert!(Copter::delete(&mut v, 1));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].placement, 0);
        assert!(!Copter::delete(&mut v, 5), "out-of-range delete is a no-op");
    }

    // The `interp` reuse: the same door spline answers the copter's frames (a straight line, an
    // identity rotation).
    #[test]
    fn interp_serves_the_framelist() {
        let mut gas = crate::gas::GasPool::default();
        let keys = keys4();
        let m = interp(&keys, 0, 3, 1.5, true).unwrap();
        assert!((m[3][0] - 900.0).abs() < 60.0, "the straight line at t=1.5 sits midway");
    }

    // copter-hitdata-damage [M Copter_Update 0x17dbb8:277-320; D the Bits table, ACTION.ELF .data 0x2a37e8]:
    // the walk scales each counted hit by its slot's Bits entry into rec+0x64 AND rec+0x6c, and
    // the slot-4 filter (0x10) drops the class-0x10 weapon hits [M Collide_FilterBullets
    // 0x1ece30:25-33].
    #[test]
    fn the_part_walk_accumulates_the_bits_scales_and_drops_the_filtered_class() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        // Slots 0/1/2 (scales 2/3/1), damage 10 each, the pistol's class 0x8 [L weapon_data
        // 0x2bf150+0x10 census]. Damage must stay below half hp so the gas branch draws nothing.
        c.part_hits = vec![
            PartHit { slot: 0, damage: 10.0, class: 0x8 },
            PartHit { slot: 1, damage: 10.0, class: 0x8 },
            PartHit { slot: 2, damage: 10.0, class: 0x8 },
        ];
        c.update(&[], &mut ch, &mut gas, 0, false, &mut rng, &ctx, &[None; 9]);
        assert!((c.damage - 60.0).abs() < 1e-4, "10·2 + 10·3 + 10·1 = 60, got {}", c.damage);
        assert!((c.attack_acc - 60.0).abs() < 1e-4, "the same sum lands in rec+0x6c [M :307]");
        assert!(c.shot_hit_enemy, "a counted player shot sets the PlrStat roll [M :314-316]");
        // Slot 4 (scale 100, filter 0x10): the class-0x10 hit (a turret/missile-class round,
        // [L the weapon census]) is DROPPED; the class-0x8 hit counts ×100.
        let mut c = copter();
        c.part_hits = vec![
            PartHit { slot: 4, damage: 1.0, class: 0x10 },
            PartHit { slot: 4, damage: 1.0, class: 0x8 },
        ];
        c.update(&[], &mut ch, &mut gas, 0, false, &mut rng, &ctx, &[None; 9]);
        assert!((c.damage - 100.0).abs() < 1e-4, "the 0x10 hit is dropped, the 0x8 hit ×100, got {}", c.damage);
        // Sister: a zero-damage contact (the game stores the hit before the damage write) counts
        // nothing and the walk stays silent on the empty slots.
        let mut c = copter();
        c.part_hits = vec![PartHit { slot: 5, damage: 0.0, class: 0x8 }, PartHit { slot: 3, damage: 4.0, class: 0x8 }];
        c.update(&[], &mut ch, &mut gas, 0, false, &mut rng, &ctx, &[None; 9]);
        assert!((c.damage - 8.0).abs() < 1e-4, "slot 3 ×2 counts, the 0-damage and scale-0 slots do not, got {}", c.damage);
        assert!(c.shot_hit_enemy, "the counted 4.0 slot-3 hit sets the roll (the 0-damage contact alone would not)");
    }

    // The death through the real walk: hits past hp take the state to 4 (the exempt tail's rule,
    // disasm 0x17e390-0x17e3c8) and write rec+0x72 [M :358-370]; the arrival then rides case 4.
    #[test]
    fn real_hits_past_hp_write_the_arrival_channel() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.out = 0x4b;
        c.hp = 299.0;
        c.state = 1;
        // 299.1 damage through slot 0's ×2: 150 hits of 1.0.
        c.part_hits = (0..150).map(|_| PartHit { slot: 0, damage: 1.0, class: 0x8 }).collect();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        // slot3_pos stays None: the gas branch's `rec+0x3c == 0` gate must consume no RNG [M :326].
        c.update(&[], &mut ch, &mut gas, 0, false, &mut rng, &ctx, &[None; 9]);
        assert!(c.hp < c.damage, "150·2 = 300 > 299");
        assert_eq!(c.state, 4, "hp < damage off the exempt states → 4 [M :358-359]");
        assert!(ch.on(0x4b), "the death writes rec+0x72 [M disasm 0x17e3bc-0x17e3c8]");
        // Sister: the same walk one hit short stays alive and writes nothing.
        let mut c = copter();
        c.out = 0x4b;
        c.state = 1;
        c.part_hits = (0..149).map(|_| PartHit { slot: 0, damage: 1.0, class: 0x8 }).collect();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // The player far outside the 1000-unit spot gate: the spot check stays idle, the
        // machine's other rules run unchanged.
        let ctx = spot_ctx(&world, [1.0e6; 3]);

        c.update(&[], &mut ch, &mut gas, 0, false, &mut rng, &ctx, &[None; 9]);
        assert!((c.damage - 298.0).abs() < 1e-4);
        assert_eq!(c.state, 1, "298 < 299: still flying");
        assert!(!ch.on(0x4b), "no arrival below hp");
    }

    // The machine's facing: a copter matrix whose row 2 (Mat_GetDir's direction) points +z.
    fn facing_z() -> Mat4 {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    // [M 0x17dbb8:140-187] the spot check: the timer counts case-1 ticks; at 240/DiffMod the
    // in-cone, in-range, on-sight player flips state 1 → 2 with the Rand_Rand(10) draw.
    #[test]
    fn the_spot_check_opens_the_attack_at_240() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.matrix = facing_z();
        let paths = Vec::new();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // 100 units along the facing, inside the 1000-unit gate and the 45° cone, LOS clear.
        let ctx = spot_ctx(&world, [0.0, 0.0, 100.0]);
        for _ in 0..240 {
            let (_, _, music, shots) = c.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx, &[None; 9]);
            assert!(!music && shots.is_empty());
            assert_eq!(c.state, 1, "the spot runs only from case-1 tick 240 (case 0 counts none)");
        }
        let draws = rng.draws;
        let (_, _, music, _) = c.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx, &[None; 9]);
        assert_eq!(c.state, 2, "tick 240 passes the spot → state 2 [M :179-184]");
        assert!(!music, "Music_Event is case 2's own beat, the entry tick already ran it");
        assert_eq!(rng.draws, draws + 1, "the entry draws attackmode = Rand_Rand(10) [M :184]");
        assert_eq!(c.timer, 0);
        assert!((c.attack_acc - 0.0).abs() < 1e-6);
    }

    // [M 0x17dbb8:152-155] the cone gates the spot: a player behind the copter never flips it.
    #[test]
    fn the_cone_gates_the_spot() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.matrix = facing_z();
        let paths = Vec::new();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        // Behind the facing (−z), 100 units: in range, outside the 45° half-angle cone.
        let ctx = spot_ctx(&world, [0.0, 0.0, -100.0]);
        for _ in 0..400 {
            c.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx, &[None; 9]);
        }
        assert_eq!(c.state, 1, "out-of-cone → the spot never passes [M :153-155]");
        assert_eq!(rng.draws, 0, "no RNG while the spot fails");
        // Sister: the apex inside the player's sphere auto-hits the cone [M 0x1ee100:36-37].
        let mut c2 = copter();
        c2.matrix = facing_z();
        let ctx2 = spot_ctx(&world, [0.0, 0.0, 0.5]);
        for _ in 0..241 {
            c2.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx2, &[None; 9]);
        }
        assert_eq!(c2.state, 2, "the apex inside the sphere → the cone auto-hits");
    }

    // [M 0x17dbb8:188-242] the attack pass: Music_Event every tick, the gun's 6-tick cadence,
    // the visible player keeps it in state 2, the 300·DiffMod timer bails to state 1.
    #[test]
    fn the_attack_pass_fires_and_bails_on_the_timer() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.matrix = facing_z();
        c.state = 2;
        c.attackmode = 1; // odd → Copter_FireGun [M :191-197]
        c.timer = 0;
        let paths = Vec::new();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        let ctx = spot_ctx(&world, [0.0, 0.0, 100.0]);
        // Gun A (slot 5) registered 2 units behind the muzzle math; gun B absent.
        let mut parts = [None; 9];
        parts[5] = Some(PartPose {
            pos: [0.0, 0.0, 0.0],
            center: [0.0, 0.0, 0.0],
            radius: 2.0,
            matrix: facing_z(),
        });
        let mut fired_at = Vec::new();
        for tick in 0..300u32 {
            let (_, _, music, shots) = c.update(&paths, &mut ch, &mut gas, tick, false, &mut rng, &ctx, &parts);
            assert!(music, "Music_Event(2,1) every case-2 tick [M :189]");
            if !shots.is_empty() {
                fired_at.push(tick);
                assert_eq!(shots[0].weapon, 0x61, "07000008's gun record [M 0x17d828:86-89]");
                assert_eq!(shots[0].sound, Some(0x5df), "the gun's Sound_Play3D [M 0x17d828:85]");
            }
            assert_eq!(c.state, 2, "visible stays in state 2 [M :236]");
        }
        assert_eq!(fired_at[0], 5, "fire_gun_tick's old>4 cadence: the 6th case-2 tick");
        assert_eq!(fired_at.len(), 50, "every 6th of the 300 pass ticks re-arms and fires");
        // Tick 301: 300·DiffMod < timer → the bail.
        let (_, _, _, _) = c.update(&paths, &mut ch, &mut gas, 300, false, &mut rng, &ctx, &[None; 9]);
        assert_eq!(c.state, 1, "the timer past 300·DiffMod bails [M :200-202, :241]");
        assert_eq!(c.timer, 0, "the bail resets the timer [M :202]");
        assert!((c.attack_acc - 0.0).abs() < 1e-6, "and rec+0x6c [M :242]");
    }

    // [M 0x17d9f0] the missile arm: the 0x86e6 draw every call, the 60·DiffMod fire gate, the
    // 0x62 record on 07000008, the muzzle radius·1.5 off pod+0x80.
    #[test]
    fn the_missile_draws_and_gates_like_the_decompile() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.matrix = facing_z();
        c.state = 2;
        c.attackmode = 0; // even → Copter_FireMissile [M :191-194]
        let paths = Vec::new();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        let ctx = spot_ctx(&world, [0.0, 0.0, 100.0]);
        // Both pod slots registered identically: whichever the 0x86e6 bit picks, one spawns.
        let mut parts = [None; 9];
        let pod = PartPose { pos: [0.0, 0.0, 0.0], center: [0.0, 0.0, 0.0], radius: 2.0, matrix: facing_z() };
        parts[7] = Some(pod);
        parts[8] = Some(pod);
        // Frame 1: the 60·DiffMod gate fails → only the 0x86e6 draw, no shot.
        let mut ctx = ctx;
        ctx.frame = 1;
        let draws = rng.draws;
        let (_, _, _, shots) = c.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx, &parts);
        assert!(shots.is_empty(), "frame 1 % 60 != 0 → no fire [M 0x17d9f0:58-64]");
        assert_eq!(rng.draws, draws + 1, "the pod draw runs EVERY call [M 0x17d9f0:47]");
        // Frame 60: the gate passes → the homing draw + the 0x62 shot.
        ctx.frame = 60;
        let draws = rng.draws;
        let (_, _, _, shots) = c.update(&paths, &mut ch, &mut gas, 60, false, &mut rng, &ctx, &parts);
        assert_eq!(shots.len(), 1, "the gate tick fires the pod [M :65-97]");
        assert_eq!(shots[0].weapon, 0x62, "07000008's missile record [M :84-86]");
        assert_eq!(rng.draws, draws + 2, "the pod draw + the homing draw Rand_Rand(8/DiffMod) [M :47, :66-67]");
        let m = shots[0].muzzle;
        let len = (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt();
        assert!((len - 3.0).abs() < 1e-4, "muzzle = pod+0x80 + dir·(radius·1.5) = 3 units out");
        // A missing pod: the draw still runs, no shot.
        ctx.frame = 120;
        let draws = rng.draws;
        let (_, _, _, shots) = c.update(&paths, &mut ch, &mut gas, 120, false, &mut rng, &ctx, &[None; 9]);
        assert!(shots.is_empty(), "no pod → no Bullet_init [M :55-57]");
        assert_eq!(rng.draws, draws + 1, "the pod draw precedes the null check [M :47-57]");
    }

    // [M 0x17dbb8:205, :238-242] rec+0x6c > 100 bails the pass even with the player visible.
    #[test]
    fn attack_acc_over_100_bails_the_pass() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.matrix = facing_z();
        c.state = 2;
        c.attackmode = 1;
        c.attack_acc = 101.0;
        let paths = Vec::new();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        let ctx = spot_ctx(&world, [0.0, 0.0, 100.0]);
        let (_, _, music, _) = c.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx, &[None; 9]);
        assert!(music);
        assert_eq!(c.state, 1, "rec+0x6c > 100 → the bail despite the sight [M :238-242]");
        assert!((c.attack_acc - 0.0).abs() < 1e-6);
        // Sister: exactly 100 stays [M :205 — `<= 100.0`].
        let mut c2 = copter();
        c2.matrix = facing_z();
        c2.state = 2;
        c2.attackmode = 1;
        c2.attack_acc = 100.0;
        c2.update(&paths, &mut ch, &mut gas, 2, false, &mut rng, &ctx, &[None; 9]);
        assert_eq!(c2.state, 2, "rec+0x6c == 100 keeps the pass [M :205]");
    }

    // [M 0x17dbb8:140] the Mayhew latch: a +0x74 == 100 record never counts the timer, never
    // draws, never attacks.
    #[test]
    fn the_mayhew_latch_never_spots() {
        let mut gas = crate::gas::GasPool::default();
        let mut c = copter();
        c.matrix = facing_z();
        c.arrival = 100;
        let paths = Vec::new();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        let world = crate::world::CollisionWorld::from_triangles_flags(Vec::new());
        let ctx = spot_ctx(&world, [0.0, 0.0, 100.0]);
        for _ in 0..500 {
            c.update(&paths, &mut ch, &mut gas, 1, false, &mut rng, &ctx, &[None; 9]);
        }
        assert_eq!(c.state, 1, "+0x74 == 100 keeps the copter in state 1 [M :140]");
        assert_eq!(c.timer, 0, "the timer never counts behind the latch");
        assert_eq!(rng.draws, 0, "and draws nothing");
    }
}
