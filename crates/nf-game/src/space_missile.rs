//! The 0700001b SpaceMissiles (`Create_SpaceMissile` 0x1a9ef0 / `SpaceMissile_Update` 0x1aa118,
//! update id 0x4f [M 0x1a9ef0:18]).
//!
//! Eight kind-0x39 placements (#129, #131–137 [D]): each carries p0 = the launch-anim script
//! (record +0x2c), p1 = the launched-away script (ext+0x04), p2 = the disarmed script (ext+0x08),
//! p3 = the state-0 countdown seed (ext+0x14, f32), p4 = the missile index 1..8 (ext+0x0c),
//! p5 = the abort channel 199 (ext+0x0e), p6 = the per-missile channel 21..28 (ext+0x10),
//! p7 = the launch-order gate 19 (ext+0x12) [D placement dump 2026-10-17; M 0x1a9ef0 param map].
//!
//! State machine (the jump table `PTR_LAB_002fcc30` at 0x2fcc30, five entries [M disasm]:
//! 0→0x1aa1c8, 1→0x1aa23c, 2→0x1aa25c, 3→0x1aa33c, 4→0x1aa3e0):
//! - **state 0** [M 0x1aa1c8]: return while `switch_channels[ext+0x12]` (p7 = 19) is clear; then
//!   `ext+0x14 -= REC_FRAME_RATE` and stay while the result is >= 0 (c.olt.s/bc1f 0x1aa200-208);
//!   on < 0: `MissileDeploy[p4−1] = 1` (0x1aa238, base 0x30cd70 at 0x1aa218) and `obj+0xf4 = 1`
//!   (state 1, 0x1aa21c). The step is **`REC_FRAME_RATE` = 1/60.0**: the disc disassembly of the
//!   state-0 body loads the constant with `lui $at,0x31; lwc1 $f1,-0x2f24($at)` =
//!   0x310000−0x2f24 = **0x30d0dc** (symtab `0030d0dc` = `REC_FRAME_RATE`; disassembled
//!   2026-11-06: 0x1aa1f0 `lui $at,0x31` / 0x1aa1f4 `lwc1 $f1,-0x2f24($at)` / 0x1aa1fc
//!   `sub.s $f0,$f0,$f1` / 0x1aa200 `c.olt.s $f0,$f0,0` / 0x1aa208 `bc1f`), and the p3 seeds
//!   are SECONDS [D placement params, re-dumped 2026-11-06: #129 105 · #131 105 · #132 40 ·
//!   #133 20 · #134 0 · #135 60 · #136 105 · #137 60 — i.e. by missile INDEX 1..8:
//!   **105/105/40/20/0/60/105/60**; the older "105/40/20/0/60/105/105/60" list in this note was
//!   mis-ordered against the p4 indices — the placement dump and the measured state-0 arming
//!   under the held gate (m5 at the first gated tick, m4 at +1200, m3 at +2400) both read the
//!   index map above] — the countdown runs seed·60 gated ticks, so the eight missiles leave
//!   state 0 STAGGERED (missile 5's 0 s seed expires on the first gated tick; 4 at +1200,
//!   3 at +2400, 6/8 at +3600, 1/2/7 at +6300). The tree shipped 512.0/tick (every seed
//!   spent in one gated tick, all eight state-0 exits the same frame) until aim-route-4; the
//!   1/60 step re-paces the 0700001b mission flow (console deadlines move to
//!   launch + seed·60 + P0_LEN − 1 — the walkreal_0700001b walks were re-ordered onto the
//!   deadline order in the same task).
//! - **state 1** [M 0x1aa23c]: `Script_Update(ext[0])` per tick; when `script+0x9c != 0`
//!   (0x1aa244-24c) → state 2 the SAME tick. `Create_SpaceMissile` runs ONE `Script_Update` at
//!   create [M 0x1a9ef0:61], so state-1 tick k = script update k+1 (entry stream clock k); the
//!   exit Event-3 (stream op 0x12 kind 3, param 0 → `+0x9c = −1` [M Script_EventHandler
//!   0x1d94c0:97-101], nonzero) sits at Wait(L−1) [D decode: t3299/1799/2399 per script] and
//!   fires on the update whose entry clock is L−1 — **state-1 tick L−1** (L = the p0 header
//!   length [D]). The sim plays the real scripts through the script runtime as creator-owned
//!   SCRIPTINFOs (`Scripts::owned_*`, via `MissileScripts`): their entity, light and sound
//!   streams run, and the channel writes and the exit hold come from the streams themselves.
//!   The [D] tables below are the decode those streams carry; the unit tests replay them.
//! - **state 2** [M 0x1aa25c]: `switch_channels[p4]` (the console channel 1..8) set → free the p0
//!   script, load+play+`Script_Update` the **p2** script (ext+0x08 — its FIRST update runs on the
//!   state-2 tick [M 0x1aa2b0]), `obj+0xf6 = 0`, `MissileDeploy[p4−1]=2`, state 3; not set → the
//!   **p1** script (ext+0x04, same first update [M 0x1aa30c]), `obj+0xf6 = 1`,
//!   `MissileDeploy[p4−1]=3`, state 3 (0x1aa278-338).
//! - **state 3** [M 0x1aa33c]: `Script_Update` + `Script_IsPlaying` per tick; when the script
//!   ends — the End-marker (op 04) time [M Script_StreamEnd 0x1db420 clears the play bit;
//!   `Script_IsPlaying` 0x1db630:8-17 reports it] — `obj+0xf6 == 0` → `switch_channels[p6]`
//!   (21..28) = 1, else → `switch_channels[p5]` (199) = 1, both with
//!   `switch_channels_time[ch] = frame` (0x1aa354-3d0); state 4, ext[0] freed (0x1aa3d4). The
//!   branch script's update 1 ran on the state-2 tick, so its End time E elapses on **state-3
//!   tick E** (entry clock E−1…: update k has entry clock k−1, the marker fires at entry clock
//!   E = update E+1 = state-3 tick E). ch 21..28 feed AND #139 (21 & 22 & 27 → ch 9, the
//!   "Disable the missiles" objective [D #139]).
//!
//! **The launch branch's mission outcome** — the fail lands on ch 99, NOT on ch 199: the p1
//! scripts write `ch 99 = 1` @t1000 [D 060007c6/060007e1 stream 14], and ch 99 is the game's
//! mission-fail channel — `Mission_Update` 0x182c60:23-33 (internal 1) reads it → internal 2
//! (MissionState 2, LevelToEndTo 0x7000048, the type-3 fail msg, TimeOut 300) → internal 7
//! (GS_PausePlayer — the world keeps ticking) → internal 4 → the 0x40000042 fail menu
//! [M 0x182c60:45-57, 123-130, 77-81]; Restart warm-reloads the level [M 0x216618:19-21]. The
//! monitor's ch-199 arm (`Mission_MonitorObjectives` 0x1827d8:113-115, FailLabel 0x0200_005a)
//! is unreachable on this path: the monitor's ONLY caller is Mission_Update internal 1
//! [M 0x182c60:22; the decompile caller sweep], and ch 99 @t1000 always precedes the missile's
//! state-3 ch 199 @t1200 [D END 1200 + M 0x1aa360-3c4] — FailLabel stays the init sentinel 0
//! [M 0x182270:27]. `Trigger_Activate` 0x1b2318:64-66 only goes inert on ch 99 (no trigger,
//! NIS or LOAD_LEVEL reads it — the decompile sweep; the level's only ch-99 param is the
//! kind-0x38 SpaceLaser #130's own fail channel p5 [D]).
//! - **state 4**: the shared epilogue only — the object is inert (the outer guard also stops the
//!   per-tick `Script_Update` block in states 0 and 4 [M 0x1aa118:13]).
//!
//! `MissileDeploy` is the 8 bytes at 0x30cd70 [M 0x1aa218/0x1aa2c8/0x1aa324], zeroed per
//! `Create_SpaceMissile` [M 0x1a9ef0:62], read by `HUD_UpdateSpacePane` 0x19d3b8 (the eight lamps:
//! 1 → the flashing "in flight" tex, 3 → the dim launched tex [M hud.rs]).
//!
//! The in-stream channel writes are [D] (script decode 2026-10-17/22, `Nis::parse` +
//! `Stream::decode`): #129 0600092a (3300 ticks) → ch 31 @t359 · #131 0600092b (3300) → 32 ·
//! #132 06000929 (1800) → 33 · #133 06000a30 (1800) → ch 192 @t1, no gate write · #134 06000927
//! (1800) → 35 · #135 0600092c (2400) → 36 · #136 0600092e (3300) → 37 · #137 0600092d (2400)
//! → 38. Every p0 but missile 4's also writes ch 181..188 (= 180+p4) @t10 and ch 171..178
//! (= 170+p4) @t(L−60) [D decode 2026-10-22, re-derived with probe_980_missile_scripts
//! 2026-10-23 — 06000a30 has 2 streams: ch 192 @t1 + the exit only]. Gate 34's writer is
//! 06000928 @t359 (SP #200's script) [D]; SP #200
//! waits on ch 191 (also SP #199's out), which the arrival script 0x0600085c writes itself (stream
//! 6, with SP #199's gate ch 190 on stream 9 [D]) — missile 4's console chain runs from the level
//! start (the older "gate 190 has no writer" sweep missed 0x0600085c). The p1
//! scripts (060007c6 for missiles 1/3/5/7, 060007e1 for 2/4/6/8 [D]) end at **1200** and write
//! ch 99 = 1 @t1000 [D]; the p2 scripts are per-missile (0600091c/06000920/06000921/06000922/
//! 0600091b/06000923/06000924/06000925 [D]) ending at **1710/1170/1230/1610/1710/1410/1610/1170**
//! with the per-missile writes in `P2_WRITES` [D]. The p2 scripts run op-0x1b sub-scripts
//! (0600091b: 0x0600091a @t120, 0x060009f2 @t1000, 0x0600096e @t1120/t1132 [D decode]); their
//! EVENT kind 0x0c (@t End−5) sets `SCRIPTINFO+0xaa8 |= 0x10` [M 0x1d94c0:159-161], the keep-
//! objects bit, so the end leaves the meshes standing. Their ENTITY streams (the launch-bay
//! meshes; the p1 flame 0x20007e1 is obj type 0xE4, which `Script_CreateEntity` 0x1d8968 has
//! no arm for — a plain class-0 object, no hurt volume [M disasm 0x1d8990-0x1d8a18]) and sounds
//! 0x466/0x465/0x393 now play.

use crate::triggers::Channels;

/// The p0 launch-script lengths per missile index 1..8, in ticks [D decode 2026-10-17].
pub const P0_LEN: [u32; 8] = [3300, 3300, 1800, 1800, 1800, 2400, 3300, 2400];
/// The gate channel each p0 script writes at t359 (0 = none) [D].
pub const P0_GATE: [u16; 8] = [31, 32, 33, 0, 35, 36, 37, 38];
/// #133's 06000a30 writes ch 192 (0xc0) at t1 instead of a gate [D].
pub const P0_ALT_WRITE: u16 = 192;
/// Each p0's t10 write: ch 181..188 (= 180 + missile index) [D decode 2026-10-22, re-derived
/// 2026-10-23 probe]. Missile 4's 06000a30 has NO t10 write (2 streams only) — 0 = none.
pub const P0_SYNC_AT: u32 = 10;
pub const P0_SYNC_CH: [u16; 8] = [181, 182, 183, 0, 185, 186, 187, 188];
/// Each p0's write at t(L−60): ch 171..178 (= 170 + missile index) [D decode 2026-10-22;
/// t3240/1740/2340 per script]. Missile 4's 06000a30 has NO t(L−60) write — 0 = none.
pub const P0_PRE_END: u32 = 60;
pub const P0_PRE_END_CH: [u16; 8] = [171, 172, 173, 0, 175, 176, 177, 178];
/// The p1 (launched-away) scripts' End-marker time, state-3 ticks [D 060007c6 / 060007e1:
/// header/End 1200]. The old stand-in 1001 was the last EVENT's time, not the script end.
pub const P1_LEN: u32 = 1200;
/// The p2 (disarmed) scripts' End-marker times per missile index 1..8, state-3 ticks
/// [D 0600091c 1710 · 06000920 1170 · 06000921 1230 · 06000922 1610 · 0600091b 1710 ·
/// 06000923 1410 · 06000924 1610 · 06000925 1170]. The old stand-in 1131 was missile 5's
/// last-EVENT time, not the script end, and one value for all eight.
pub const P2_LEN: [u32; 8] = [1710, 1170, 1230, 1610, 1710, 1410, 1610, 1170];
/// The p1 scripts' in-stream channel writes (stream tick, channel) [D decode 2026-10-22:
/// ch 99 = 1 @t1000 in both scripts].
pub const P1_WRITES: &[(u32, u16)] = &[(1000, 99)];
/// The p2 scripts' in-stream channel writes per missile index 1..8 [D decode 2026-10-22]: the
/// later column is 0x83..0x8a, the Apocalypse placements' latch channels.
pub const P2_WRITES: [&[(u32, u16)]; 8] = [
    &[(1, 81), (1000, 46), (1030, 131)],
    &[(1, 82), (490, 132)],
    &[(1, 83), (550, 133)],
    &[(1, 84), (930, 134), (990, 41)],
    &[(1, 85), (1030, 135), (1130, 42)],
    &[(1, 86), (730, 136), (830, 43)],
    &[(1, 87), (930, 137)],
    &[(1, 88), (490, 138)],
];
/// The state-0 countdown decrement: `REC_FRAME_RATE` = 1/60.0 per gated tick [M the state-0
/// body 0x1aa1f0-0x1aa208 loads 0x30d0dc with `lui $at,0x31; lwc1 $f1,-0x2f24($at)`; symtab
/// `0030d0dc` = REC_FRAME_RATE; the compare/branch is c.olt.s/bc1f 0x1aa200-208]. The p3 seeds
/// are seconds [D placement params re-dumped 2026-11-06: by missile index 1..8
/// 105/105/40/20/0/60/105/60 — the older "105/40/20/0/60/105/105/60" note was mis-ordered
/// against the p4 indices; the dump and the measured arming agree], so a seed spends seed·60
/// gated ticks in state 0.
pub const COUNTDOWN_STEP: f32 = 1.0 / 60.0;

/// What `SpaceMissile_Update` asks of the script runtime — the creator-owned SCRIPTINFO calls
/// [M 0x1a9ef0:46-73, 0x1aa23c-0x1aa350]. The sim wires it to `Scripts::owned_*`; the unit tests
/// below drive a stand-in built from the [D] decode tables above.
pub trait MissileScripts {
    /// `Script_Load(hash, obj+0x30, obj+0x50, 0,0,0,0)` + `Script_Play(si, 0)` [M 0x1aa280-0x1aa2a8,
    /// 0x1a9ef0:46-48] — `matrix` is the object's placement transform. `None` = the NULL load.
    fn load(&mut self, hash: u32, matrix: Mat4) -> Option<usize>;
    /// `Script_Update(si)` [M 0x1da998].
    fn update(&mut self, h: usize, ch: &mut Channels, frame: u32);
    /// `SCRIPTINFO+0x9c` — event 3's hold (`p0 − 1`) [M 0x1d94c0:97-101].
    fn delay(&self, h: usize) -> i32;
    /// `Script_IsPlaying(si)` [M 0x1db630].
    fn playing(&self, h: usize) -> bool;
    /// `Script_Free(si)` [M 0x1da880].
    fn free(&mut self, h: usize);
}

pub type Mat4 = [[f32; 4]; 4];

/// One kind-0x39 placement's missile object (the ext block at obj+0xE0 plus obj+0xf4/0xf6 [M]).
#[derive(Clone, Debug, PartialEq)]
pub struct SpaceMissile {
    pub placement: usize,
    /// ext+0x0c: the missile index 1..8 (p4) — the console channel AND the MissileDeploy slot [M].
    pub index: u16,
    /// ext+0x0e: the abort channel (p5 = 199 on all eight [D]).
    pub fail_channel: u16,
    /// ext+0x10: the per-missile done channel (p6 = 21..28 [D]).
    pub done_channel: u16,
    /// ext+0x12: the launch-order gate (p7 = 19 on all eight [D]).
    pub gate_channel: u16,
    /// ext+0x14: the state-0 countdown (p3: by index 1..8 105/105/40/20/0/60/105/60 [D
    /// placement params 2026-11-06]).
    pub countdown: f32,
    /// obj+0xf4: the state 0..4 [M].
    pub state: u16,
    /// obj+0xf6: 0 = the disarm branch, 1 = the launch branch (set in state 2 [M 0x1aa2bc/0x1aa31c]).
    pub launch_flag: u16,
    /// The object's placement transform — `Control_CreateObjEx`'s obj+0x30/+0x50, the two
    /// VECTORs every `Script_Load` of this object passes [M 0x1a9ef0:46, disasm 0x1aa284/0x1aa29c].
    pub matrix: Mat4,
    /// The script hashes: [0] the launch anim (record +0x2c), [1] ext+0x04 the launched-away
    /// script (+0x30), [2] ext+0x08 the disarmed script (+0x34) [M 0x1a9ef0:21-23].
    pub scripts: [u32; 3],
    /// Records +0x4c..+0x58: up to four side scripts, loaded and played at create, updated in
    /// states 1-3 [M 0x1a9ef0:49-72, 0x1aa118:13-38]. All four are 0 on the disc [D params 8..11].
    pub side_scripts: [u32; 4],
    /// ext+0x00: the running script's handle (`MissileScripts`); `None` = NULL — the whole
    /// update is skipped then [M 0x1aa118:12]. State 3's end clears it without a free
    /// [M disasm 0x1aa3dc `sw $zero,($s0)`].
    pub info: Option<usize>,
    /// ext+0x18..+0x24: the side scripts' handles [M 0x1a9ef0:52-71].
    pub side_info: [Option<usize>; 4],
    /// The create beat ran (`Create_SpaceMissile`'s loads and its one `Script_Update`, which the
    /// game runs at level load; the sim runs it on the first tick, before the first update).
    pub created: bool,
}

impl SpaceMissile {
    /// `Create_SpaceMissile` 0x1a9ef0's script beat: the p0 load + `Script_Play(_, 0)`, the side
    /// scripts' load + play, then ONE `Script_Update` of p0 [M 0x1a9ef0:46-73].
    pub fn create(&mut self, host: &mut dyn MissileScripts, ch: &mut Channels, frame: u32) {
        self.created = true;
        self.info = host.load(self.scripts[0], self.matrix);
        for k in 0..4 {
            if self.side_scripts[k] != 0 {
                self.side_info[k] = host.load(self.side_scripts[k], self.matrix);
            }
        }
        if let Some(h) = self.info {
            host.update(h, ch, frame);
        }
    }

    /// `SpaceMissile_Update` 0x1aa118, one tick. `deploy` is `MissileDeploy` (8 bytes, 0x30cd70).
    pub fn tick(&mut self, host: &mut dyn MissileScripts, ch: &mut Channels, deploy: &mut [u8; 8], frame: u32) {
        // [M 0x1aa118:12] a NULL ext[0] skips everything.
        let Some(h) = self.info else { return };
        // [M 0x1aa118:13-38] the side scripts update in states 1..3.
        if self.state != 0 && self.state != 4 {
            for s in self.side_info.iter().flatten() {
                host.update(*s, ch, frame);
            }
        }
        match self.state {
            0 => {
                // [M 0x1aa1c8-1e4]: the gate first — no countdown while switch_channels[p7] is clear.
                if self.gate_channel != 0 && !ch.on(self.gate_channel as u32) {
                    return;
                }
                // [M 0x1aa1ec-20c]: countdown -= REC_FRAME_RATE; stay while >= 0.
                self.countdown -= COUNTDOWN_STEP;
                if self.countdown >= 0.0 {
                    return;
                }
                // [M 0x1aa210-238]: MissileDeploy[p4−1] = 1; obj+0xf4 = 1.
                if let Some(slot) = self.deploy_slot() {
                    deploy[slot] = 1;
                }
                self.state = 1;
            }
            1 => {
                // [M 0x1aa23c-258]: Script_Update(ext[0]); +0x9c != 0 (the exit event 3's hold)
                // → state 2 the same tick.
                host.update(h, ch, frame);
                if host.delay(h) != 0 {
                    self.state = 2;
                }
            }
            2 => {
                // [M 0x1aa25c-338]: Script_Free(ext[0]); the console channel picks the branch
                // script; Script_Load + Script_Play(_, 0) + Script_Update.
                host.free(h);
                let disarmed = ch.on(self.index as u32);
                let (script, flag, lamp) = if disarmed { (self.scripts[2], 0, 2) } else { (self.scripts[1], 1, 3) };
                self.info = host.load(script, self.matrix);
                if let Some(n) = self.info {
                    host.update(n, ch, frame);
                }
                self.launch_flag = flag; // [M 0x1aa2bc / 0x1aa31c]
                self.state = 3; // [M 0x1aa2c0 / 0x1aa328]
                if let Some(slot) = self.deploy_slot() {
                    deploy[slot] = lamp; // [M 0x1aa2cc-338: a0 = 2 / 3]
                }
            }
            3 => {
                // [M 0x1aa33c-3d0]: Script_Update + Script_IsPlaying; on the end the channel
                // write, state 4, ext[0] = 0 (no free [M disasm 0x1aa3dc]).
                host.update(h, ch, frame);
                if host.playing(h) {
                    return;
                }
                let c = if self.launch_flag == 0 { self.done_channel } else { self.fail_channel };
                if c != 0 {
                    ch.set(c as u32, 1, frame); // [M 0x1aa354-3d0: ch[p6] / ch[p5] + its time]
                }
                self.state = 4;
                self.info = None;
            }
            _ => {} // [M 0x1aa3e0]: state 4 is the epilogue — inert.
        }
    }

    fn deploy_slot(&self) -> Option<usize> {
        if (1..=8).contains(&self.index) {
            Some(self.index as usize - 1)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triggers::Channels;

    /// A stand-in script runtime from the [D] decode tables: each loaded script is (hash, update
    /// count, playing); update k runs at stream clock k−1 and fires the table's writes at their
    /// clock; a p0 holds (event 3) after its clock-(L−1) update; a p1/p2 stops on the update that
    /// reaches its End time.
    #[derive(Default)]
    struct Disc { scripts: Vec<Option<(u32, u32, bool)>>, freed: Vec<u32> }
    const P0: [u32; 8] = [0x0600092a, 0x0600092b, 0x06000929, 0x06000a30, 0x06000927, 0x0600092c, 0x0600092e, 0x0600092d];
    const P2: [u32; 8] = [0x0600091c, 0x06000920, 0x06000921, 0x06000922, 0x0600091b, 0x06000923, 0x06000924, 0x06000925];
    fn writes(hash: u32, clock: u32) -> Vec<u16> {
        let mut out = Vec::new();
        if let Some(k) = P0.iter().position(|&h| h == hash) {
            if clock == P0_SYNC_AT && P0_SYNC_CH[k] != 0 { out.push(P0_SYNC_CH[k]); }
            if clock == 1 && k == 3 { out.push(P0_ALT_WRITE); }
            if clock == 359 && P0_GATE[k] != 0 { out.push(P0_GATE[k]); }
            if clock + P0_PRE_END == P0_LEN[k] && P0_PRE_END_CH[k] != 0 { out.push(P0_PRE_END_CH[k]); }
        } else if let Some(k) = P2.iter().position(|&h| h == hash) {
            out.extend(P2_WRITES[k].iter().filter(|(t, _)| *t == clock).map(|(_, c)| *c));
        } else {
            out.extend(P1_WRITES.iter().filter(|(t, _)| *t == clock).map(|(_, c)| *c));
        }
        out
    }
    impl Disc {
        fn held(hash: u32, n: u32) -> bool {
            P0.iter().position(|&x| x == hash).is_some_and(|k| n >= P0_LEN[k])
        }
    }
    impl MissileScripts for Disc {
        fn load(&mut self, hash: u32, _m: Mat4) -> Option<usize> {
            self.scripts.push(Some((hash, 0, true)));
            Some(self.scripts.len() - 1)
        }
        fn update(&mut self, h: usize, ch: &mut Channels, frame: u32) {
            let Some(Some((hash, n, playing))) = self.scripts.get_mut(h) else { return };
            if !*playing || Disc::held(*hash, *n) { return; }
            let clock = *n;
            *n += 1;
            for c in writes(*hash, clock) { ch.set(c as u32, 1, frame); }
            if let Some(k) = P2.iter().position(|&x| x == *hash) {
                if clock >= P2_LEN[k] { *playing = false; }
            } else if !P0.contains(hash) && clock >= P1_LEN {
                *playing = false;
            }
        }
        fn delay(&self, h: usize) -> i32 { self.scripts[h].map_or(0, |(hash, n, _)| if Disc::held(hash, n) { -1 } else { 0 }) }
        fn playing(&self, h: usize) -> bool { self.scripts[h].is_some_and(|s| s.2) }
        fn free(&mut self, h: usize) { if let Some(s) = self.scripts[h].take() { self.freed.push(s.0); } }
    }

    fn missile(index: u16) -> SpaceMissile {
        let k = (index - 1) as usize;
        SpaceMissile {
            placement: 0, index, fail_channel: 199, done_channel: 20 + index, gate_channel: 19,
            countdown: 0.0, state: 0, launch_flag: 0, matrix: [[0.0; 4]; 4],
            scripts: [P0[k], if index % 2 == 1 { 0x060007c6 } else { 0x060007e1 }, P2[k]],
            side_scripts: [0; 4], info: None, side_info: [None; 4], created: false,
        }
    }

    /// A created missile past its gate and countdown, in state 1 (countdown 0.0 exits state 0
    /// on the first gated tick: 0 − 1/60 < 0 [M 0x1aa1ec-238]).
    fn launch(index: u16) -> (SpaceMissile, Disc, Channels, [u8; 8]) {
        let mut m = missile(index);
        let mut d = Disc::default();
        let mut ch = Channels::default();
        let mut deploy = [0u8; 8];
        m.create(&mut d, &mut ch, 0); // the create beat's one update ran clock 0 [M 0x1a9ef0:73]
        ch.set(19, 1, 0);
        m.tick(&mut d, &mut ch, &mut deploy, 1);
        assert_eq!(m.state, 1);
        (m, d, ch, deploy)
    }

    /// Sister: a gated-out missile never counts down [M 0x1aa1c8-1e4] and its p0 never runs
    /// past the create beat (state 0 runs no Script_Update).
    #[test]
    fn state0_holds_while_the_gate_is_clear() {
        let mut m = missile(4);
        let (mut d, mut ch, mut deploy) = (Disc::default(), Channels::default(), [0u8; 8]);
        m.create(&mut d, &mut ch, 0);
        for _ in 0..600 { m.tick(&mut d, &mut ch, &mut deploy, 1); }
        assert_eq!(m.state, 0, "gate 19 clear — no countdown, no deploy [M 0x1aa1c8-1e4]");
        assert_eq!(deploy[3], 0);
        assert_eq!(d.scripts[0].unwrap().1, 1, "only the create beat's update ran [M 0x1a9ef0:73]");
    }

    /// A NULL p0 load leaves the whole object inert [M 0x1aa118:12].
    #[test]
    fn a_missing_launch_script_leaves_the_missile_inert() {
        struct Miss;
        impl MissileScripts for Miss {
            fn load(&mut self, _: u32, _: Mat4) -> Option<usize> { None }
            fn update(&mut self, _: usize, _: &mut Channels, _: u32) {}
            fn delay(&self, _: usize) -> i32 { 0 }
            fn playing(&self, _: usize) -> bool { false }
            fn free(&mut self, _: usize) {}
        }
        let mut m = missile(5);
        let (mut ch, mut deploy) = (Channels::default(), [0u8; 8]);
        m.create(&mut Miss, &mut ch, 0);
        ch.set(19, 1, 0);
        for f in 0..10 { m.tick(&mut Miss, &mut ch, &mut deploy, f); }
        assert_eq!((m.state, deploy[4]), (0, 0));
    }

    /// The state-1 exit lands on tick L−1: the create-time update ate clock 0 [M 0x1a9ef0:73],
    /// the exit event 3 sits at Wait(L−1) [D 0600092a t3299].
    #[test]
    fn state1_exits_at_header_length_minus_one() {
        let (mut m, mut d, mut ch, mut deploy) = launch(1);
        for t in 1..=3298u32 { m.tick(&mut d, &mut ch, &mut deploy, t); }
        assert_eq!(m.state, 1, "still tracking the p0 script on tick 3298 [D t3299]");
        m.tick(&mut d, &mut ch, &mut deploy, 3299);
        assert_eq!(m.state, 2, "the exit event 3 holds the script on state-1 tick 3299 = L−1");
    }

    /// Sister: the p0's writes land at their [D] ticks while state 1 runs.
    #[test]
    fn state1_writes_gates_and_sync_channels() {
        let (mut m, mut d, mut ch, mut deploy) = launch(1);
        let mut fired = [false; 3];
        for t in 1..=3299u32 {
            m.tick(&mut d, &mut ch, &mut deploy, t);
            if t == 10 { fired[0] = ch.on(181); }
            if t == 359 { fired[1] = ch.on(31); }
            if t == 3240 { fired[2] = ch.on(171); }
        }
        assert!(fired.iter().all(|&f| f), "the p0's t10/t359/t(L−60) writes all landed [D]");
        assert!(!ch.on(182) && !ch.on(172), "sister: missile 1 writes only ITS lanes [D 0600092a]");
    }

    /// The disarm branch: state 2 frees the p0, loads the p2 and runs its first update; the done
    /// channel fires on state-3 tick END = 1710 for missile 1 [D 0600091c].
    #[test]
    fn disarm_branch_runs_the_p2_wait_and_writes() {
        let (mut m, mut d, mut ch, mut deploy) = launch(1);
        for t in 1..=3299u32 { m.tick(&mut d, &mut ch, &mut deploy, t); }
        ch.set(1, 1, 4000);
        m.tick(&mut d, &mut ch, &mut deploy, 3300);
        assert_eq!((m.state, m.launch_flag, deploy[0]), (3, 0, 2), "the disarm branch [M 0x1aa25c-338]");
        assert_eq!(d.freed, vec![0x0600092a], "Script_Free(ext[0]) on the p0 [M 0x1aa278]");
        assert!(!ch.on(21), "sister: the done channel does not fire while the p2 script runs [D END 1710]");
        for t in 1..=1709u32 { m.tick(&mut d, &mut ch, &mut deploy, 3300 + t); }
        assert!(ch.on(81) && ch.on(46) && ch.on(131), "the p2's own writes @t1/t1000/t1030 [D 0600091c]");
        assert!(!ch.on(21), "the p2 wait elapses 1709 ticks [D END 1710]");
        m.tick(&mut d, &mut ch, &mut deploy, 5010);
        assert!(ch.on(21), "the done channel fires on state-3 tick 1710 [M 0x1aa354-398 + D END]");
        assert_eq!((m.state, m.info), (4, None), "the epilogue clears ext[0] [M 0x1aa3d4-3dc]");
        assert_eq!(d.freed.len(), 1, "no free at the end [M 0x1aa3dc]");
    }

    /// Sister for the launch branch: missile 4 runs the p1's 1200-tick End and aborts through
    /// ch 199 [M 0x1aa360-3c4].
    #[test]
    fn launch_branch_runs_the_p1_wait_then_aborts() {
        let (mut m, mut d, mut ch, mut deploy) = launch(4);
        for t in 1..=1799u32 { m.tick(&mut d, &mut ch, &mut deploy, t); }
        m.tick(&mut d, &mut ch, &mut deploy, 1800);
        assert_eq!((m.launch_flag, deploy[3]), (1, 3), "the launch branch [M 0x1aa31c]");
        for t in 1..=1199u32 { m.tick(&mut d, &mut ch, &mut deploy, 1800 + t); }
        assert!(ch.on(99), "the p1's ch 99 = 1 @t1000 [D 060007c6/060007e1]");
        assert!(!ch.on(199), "sister: the abort waits out the 1200-tick p1 End [D]");
        m.tick(&mut d, &mut ch, &mut deploy, 3000);
        assert!(ch.on(199), "ch 199 fires on state-3 tick 1200 [M 0x1aa360-3c4 + D END 1200]");
        assert_eq!(m.state, 4);
    }
}
