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
//!   - case 1 (fly): `Copter_Move`, then the spot check (the 240/DiffMod timer, the 1000-unit
//!     cone/LOS test, `attackmode_159 = Rand_Rand(10)`, all behind `rec+0x74 != 100` — the
//!     Mayhew latch, [M :138-187]) — NOT ported (no cone/LOS wiring here yet): the port's
//!     copters stay in state 1 and never draw RNG.
//!   - case 2 (attack): `Music_Event(2,1)`, the missile/gun pick, the bail timers [M :188-243] —
//!     unreachable while the spot check is not ported; `Copter_FireGun`'s cadence stays ported
//!     as `fire_gun_tick`.
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
    /// `rec+0x64`: the accumulated damage [M 0x17dbb8:306]. The port has no part hit chains —
    /// only a caller poke moves it (documented gap).
    pub damage: f32,
    /// `obj+0x68` = 299.0 x the difficulty multiplier [M `Copter_Create` 0x17d340:76-94]:
    /// d1 x1.0 (the decomp's fall-through default), d2 x1.5, d3 x2.0, d4 x4.0.
    pub hp: f32,
    /// obj+0xFE bit 1 — set by the +0x74 == 100 latch (Copter_Move) and cases 5/6 [M].
    pub landed: bool,
    /// `*(rec+0x38)` — part slot 2, the body entity id (`Copter_Register` 0x17e530:7-8 fills the
    /// slots from the SP script entities; the port captures the body when the assembly script
    /// spawns it). None until captured.
    pub body_obj: Option<u32>,
    /// The gun parts' fire counters — the game keeps `*(short*)(gun+0xf4)` per gun obj
    /// (slots 5/6) [M `Copter_FireGun` 0x17d828:63-65, 93-96]. [0] = gun A (rec+0x44),
    /// [1] = gun B (rec+0x48).
    pub gun_counters: [u16; 2],
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
                body_obj: None,
                gun_counters: [0; 2],
            });
        }
        out
    }

    /// `Copter_Move` 0x17d628 — one flight tick. Advances the spline t along the current path,
    /// poses obj+0x90 (`self.matrix`) from the interpolation, and on a path's end picks the next
    /// one (the damage-weighted rule). Returns the game's result: true = the path pointer is
    /// null or the path just transitioned, false = still flying (or an empty path) [M :36-117].
    pub fn move_tick(&mut self, paths: &[CopterPath], rng: &mut crate::drone::Rng) -> bool {
        let Some(pi) = self.path else { return true }; // [M :45-48 — +0x54 == 0 → 1]
        let (count, step) = (paths[pi].count, paths[pi].step);
        if count == 0 { return false; } // [M :49 — the empty path returns 0]
        // Spline_Interp(t, framelist, 0, count, out, rec+0x1c, 2); param_3 == 0 → the pose lands
        // in obj+0x90 [M :50-55]. A NULL framelist writes nothing (the matrix stays) [M 0x1e1858:100-102].
        if let Some(keys) = &paths[pi].keys {
            if let Some(m) = door::interp(keys, 0, count, self.t, true) {
                self.matrix = m;
            }
        }
        self.t += step; // [M :53]
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
            return true; // [M :112 — uVar2 = 1: the path transitioned]
        }
        // SP_SetPosRot(*(rec+0x2c), obj+0x90) — the caller poses the SP [M :115].
        false
    }

    /// `Copter_Update` 0x17dbb8 — one tick. Returns `(unhide, posed)`:
    /// - `unhide` — the gate's first ON tick: the caller runs `SP_HideObj(sp, 0)`
    ///   (`Scripts::unhide_sp`) — the game does it right here in the head, then zeroes +0x70 [M head];
    /// - `posed` — `SP_SetPosRot(sp, obj+0x90)` ran: the caller puts `self.matrix` on the SP
    ///   player [M Copter_Move :115].
    pub fn update(
        &mut self,
        paths: &[CopterPath],
        ch: &mut Channels,
        frame: u32,
        explode_level: bool,
        rng: &mut crate::drone::Rng,
    ) -> (bool, bool) {
        // The gate: a non-zero gate channel that is off stops everything [M 0x17dbb8 head]; on the
        // first ON tick the head unhides the SP and zeroes the field — the copter then updates
        // unconditionally, even if the channel turns off mid-flight [M `+0x70 = 0`].
        let mut unhide = false;
        if self.gate != 0 {
            if !ch.on(self.gate as u32) { return (false, false); }
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
                // Case 1 [M :138-187]: Copter_Move, then the spot check (not ported — no cone/LOS
                // wiring; the copter stays in state 1 and draws no RNG). The Mayhew latch keeps
                // +0x74 != 100 copters out of the spot test entirely [M :140].
                self.move_tick(paths, rng);
            }
            2 => {
                // Case 2 [M :188-243] (Music_Event, the missile/gun pick, the bail timers) is
                // unreachable while the spot check is not ported; fly the move anyway if a poke
                // puts the copter here.
                self.move_tick(paths, rng);
            }
            4 => {
                // Case 4 [M :244-254]: finish the current path, then adopt the arrival path with
                // t = 0 and write the arrival channel.
                if self.move_tick(paths, rng) {
                    self.path = self.arrival_path;
                    self.t = 0.0;
                    if self.out != 0 { ch.set(self.out as u32, 1, frame); }
                    self.state = 5;
                }
            }
            5 => {
                // Case 5 [M :255-275]: fly the arrival path; 0 → plain return (no damage tail);
                // 1 → re-write the channel + obj+0xFE bit 1, every tick.
                if self.move_tick(paths, rng) {
                    if self.out != 0 { ch.set(self.out as u32, 1, frame); }
                    self.landed = true;
                    return (unhide, true);
                }
                return (unhide, true);
            }
            6 => {
                // Case 6 [M :265-275]: the channel, the explosion (not ported), the flag.
                if self.out != 0 { ch.set(self.out as u32, 1, frame); }
                self.landed = true;
                return (unhide, false);
            }
            _ => {}
        }
        // The damage tail [M :277-371]. The 9-part hit walk that accumulates `damage` is not
        // ported (no part hit chains — `damage` only moves by a caller poke). The death:
        // states 4/5 exempt (`sltiu(state-4, 2)` skips, disasm 0x17e390-0x17e39c); every other
        // state with hp < damage → 4, or 6 on 0x0700000b/0x07000012 [M :358-370]; the arrival
        // channel writes either way [M disasm 0x17e3bc-0x17e3c8].
        let exempt = self.state == 4 || self.state == 5;
        if !exempt && self.hp < self.damage {
            self.state = if explode_level { 6 } else { 4 };
            if self.out != 0 { ch.set(self.out as u32, 1, frame); }
        }
        (unhide, false)
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
        let mut c = copter();
        let mut rng = crate::drone::Rng::default();
        assert!(c.move_tick(&[], &mut rng), "+0x54 == 0 → 1");
        let paths = vec![path(1, 0, 0, 0, 0, Some(keys4()))];
        c.path = Some(0);
        assert!(!c.move_tick(&paths, &mut rng), "count == 0 → 0");
    }

    // [M 0x17d628:50-63] the flight: the pose lands in obj+0x90 and t advances by the step.
    #[test]
    fn move_tick_poses_the_matrix_and_advances_t() {
        let mut c = copter();
        let mut rng = crate::drone::Rng::default();
        let paths = vec![path(1, 0, 0, 3, 0, Some(keys4()))];
        c.path = Some(0);
        assert!(!c.move_tick(&paths, &mut rng), "still flying → 0");
        assert!((c.t - 0.05).abs() < 1e-5, "t += step");
        // The pose: t≈0 sits at key 0 (the origin); after enough ticks the x grows along the line.
        assert!(c.matrix[3][0].abs() < 60.0, "t≈0 sits at key 0");
        for _ in 0..50 { let _ = c.move_tick(&paths, &mut rng); }
        assert!(c.matrix[3][0] > 100.0, "the copter moved along the path (x = {})", c.matrix[3][0]);
    }

    // [M 0x17d628:71-80] the pick: undamaged → deterministic 0; heavy damage → the shared RNG.
    #[test]
    fn move_tick_picks_the_next_path() {
        let mut c = copter();
        let mut rng = crate::drone::Rng::default();
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
        assert!(c.move_tick(&paths, &mut rng), "the transition returns 1");
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
        assert!(c.move_tick(&paths2, &mut rng));
        assert_eq!(c.path, Some(0), "pick 1 finds only the kind-2 path → the walk keeps the current path");
        // The clamp [M :78-80].
        c.damage = 299.0; // ratio = 299/269.1 > 0.5 → the random arm
        c.path = Some(0);
        c.t = 3.5;
        let draws = rng.draws;
        let _ = c.move_tick(&paths2, &mut rng);
        assert_eq!(rng.draws, draws + 1, "the heavy-damage arm draws Rand_Rand(rec+0x78) [M disasm 0x17d748]");
    }

    // [M 0x17d340:18-45 + 0x1d1b68] the node build: params → fields, the step formula, the m_paths
    // resolution. [S] the placement below is synthetic; the ISO test pins real records.
    #[test]
    fn path_node_build_matches_the_record() {
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
        c.update(&paths, &mut ch, 0, false, &mut rng);
        assert_eq!(c.state, 1, "case 0 → 1");
        assert_eq!(c.arrival_path, Some(3), "the kind-2 group member → rec+0x58");
        assert_eq!(c.group_paths, 2, "the kind-0 group members counted into rec+0x78");
    }

    // [M 0x17dbb8 head, :244-254] the gate edge and case 4: the arrival path + the channel.
    #[test]
    fn gate_edge_then_case_4_writes_the_arrival() {
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
        // The gate off: nothing runs.
        let (unhide, posed) = c.update(&paths, &mut ch, 1, false, &mut rng);
        assert!(!unhide && !posed, "gated off → nothing");
        assert_eq!(c.state, 0, "still state 0 behind the gate");
        ch.set(5, 1, 0);
        let (unhide, _) = c.update(&paths, &mut ch, 2, false, &mut rng);
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
            let (_, _) = c.update(&paths, &mut ch, f, false, &mut rng);
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
        let mut c = copter();
        c.out = 0x4b;
        c.path = Some(0);
        let keys = keys4();
        let paths = vec![path(7, 0, 0, 1, 0, Some(keys.clone()))];
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        // State 5, the path just ended: the channel re-writes and obj+0xFE sets, every tick.
        c.state = 5;
        c.t = 3.5;
        let (_, _) = c.update(&paths, &mut ch, 1, false, &mut rng);
        assert!(c.landed, "obj+0xFE bit 1 [M LAB_0017e16c]");
        assert!(ch.on(0x4b), "the channel re-write [M :260-263]");
        // The death: state 1 with hp < damage → 4 (07000005-like), 6 on 0b/12.
        let mut d = copter();
        d.path = Some(0);
        d.hp = 299.0;
        d.damage = 300.0;
        let (_, _) = d.update(&paths, &mut ch, 2, false, &mut rng);
        assert_eq!(d.state, 4, "hp < damage outside states 4/5 → 4 [M disasm 0x17e3b8-0x17e3c0]");
        assert!(ch.on(0x4b), "the death writes the arrival channel [M disasm 0x17e3bc]");
        let mut e = copter();
        e.path = Some(0);
        e.hp = 299.0;
        e.damage = 300.0;
        let (_, _) = e.update(&paths, &mut ch, 3, true, &mut rng);
        assert_eq!(e.state, 6, "0x0700000b/0x07000012 explode instead [M :360-363]");
        // States 4/5 are exempt (sltiu(state-4, 2) skips, disasm 0x17e398-0x17e39c).
        let mut g = copter();
        g.path = Some(0);
        g.state = 4;
        g.hp = 299.0;
        g.damage = 300.0;
        let (_, _) = g.update(&paths, &mut ch, 4, false, &mut rng);
        assert_eq!(g.state, 4, "state 4 is exempt from the death");
    }

    // [M 0x17dbb8:98-103] the countdown pulse: +0x76 hits 0 → the +0x74 channel.
    #[test]
    fn countdown_pulses_its_channel() {
        let mut c = copter();
        c.countdown = 2;
        c.arrival = 0x33;
        let paths = Vec::new();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::default();
        c.update(&paths, &mut ch, 1, false, &mut rng);
        assert!(!ch.on(0x33), "1 tick left: no pulse yet");
        c.update(&paths, &mut ch, 2, false, &mut rng);
        assert!(ch.on(0x33), "the countdown hit 0 → ch +0x74 [M :99-103]");
        assert_eq!(c.countdown, 0);
    }

    // [M 0x17e4b8:6-9] the hide-gate: gate set + channel off → absent; channel on or gate 0 → body.
    #[test]
    fn body_respects_the_hide_gate() {
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
        let keys = keys4();
        let m = interp(&keys, 0, 3, 1.5, true).unwrap();
        assert!((m[3][0] - 900.0).abs() < 60.0, "the straight line at t=1.5 sits midway");
    }
}
