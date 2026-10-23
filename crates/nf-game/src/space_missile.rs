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
//!   `ext+0x14 -= 512.0` (the constant at 0x31d0dc [M]) and stay while the result is >= 0
//!   (c.lt.s/bc1f 0x1aa200-208); on < 0: `MissileDeploy[p4−1] = 1` (0x1aa238, base 0x30cd70 at
//!   0x1aa218) and `obj+0xf4 = 1` (state 1, 0x1aa21c).
//! - **state 1** [M 0x1aa23c]: `Script_Update(ext[0])` per tick; when `script+0x9c != 0`
//!   (0x1aa244-24c) → state 2 the SAME tick. `Create_SpaceMissile` runs ONE `Script_Update` at
//!   create [M 0x1a9ef0:61], so state-1 tick k = script update k+1 (entry stream clock k); the
//!   exit Event-3 (stream op 0x12 kind 3, param 0 → `+0x9c = −1` [M Script_EventHandler
//!   0x1d94c0:97-101], nonzero) sits at Wait(L−1) [D decode: t3299/1799/2399 per script] and
//!   fires on the update whose entry clock is L−1 — **state-1 tick L−1** (L = the p0 header
//!   length [D]). The port plays the same stream as a tick counter with the [D]-decoded writes:
//!   the lengths and every in-stream channel write below are disc reads, and the flow pins are
//!   the [M] states — the observable behaviour (channel writes at ticks, the state exits) is the
//!   script's; its anim-entity/sound streams are visual/audio and unported like the rest of the
//!   object-side gfx.
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
//! waits on ch 191 (SP #199's out) whose own gate 190 has **no writer** in the level's 77 scripts
//! [D sweep] — missile 4's console chain therefore never runs from disc data alone. The p1
//! scripts (060007c6 for missiles 1/3/5/7, 060007e1 for 2/4/6/8 [D]) end at **1200** and write
//! ch 99 = 1 @t1000 [D]; the p2 scripts are per-missile (0600091c/06000920/06000921/06000922/
//! 0600091b/06000923/06000924/06000925 [D]) ending at **1710/1170/1230/1610/1710/1410/1610/1170**
//! with the per-missile writes in `P2_WRITES` [D]. The p1/p2 scripts carry no sub-scripts [D
//! decode]; their EVENT kind 0x0c (@t End−5) sets `SCRIPTINFO+0xaa8 |= 0x10` [M 0x1d94c0:159-161],
//! consumed only by the camera-fade path gated on `+0xaa8 & 0x40` [M 0x1da998:47-51] — camera
//! false on every missile script [D header] — so it is sim-invisible. Their ENTITY type-0x31
//! streams (the launch-bay meshes) and sounds 0x466/0x465/0x393 are visual/audio, unported like
//! the port's other object gfx.

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
/// The p2 scripts' in-stream channel writes per missile index 1..8 [D decode 2026-10-22].
pub const P2_WRITES: [&[(u32, u16)]; 8] = [
    &[(1, 81), (1000, 46), (1030, 83)],
    &[(1, 82), (490, 84)],
    &[(1, 83), (550, 85)],
    &[(1, 84), (930, 86), (990, 41)],
    &[(1, 85), (1030, 87), (1130, 42)],
    &[(1, 86), (730, 88), (830, 43)],
    &[(1, 87), (930, 89)],
    &[(1, 88), (490, 90)],
];
/// The state-0 countdown decrement, the float at 0x31d0dc [M].
pub const COUNTDOWN_STEP: f32 = 512.0;

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
    /// ext+0x14: the state-0 countdown (p3: 105/40/20/0/60/105/105/60 [D]).
    pub countdown: f32,
    /// obj+0xf4: the state 0..4 [M].
    pub state: u16,
    /// obj+0xf6: 0 = the disarm branch, 1 = the launch branch (set in state 2 [M 0x1aa2bc/0x1aa31c]).
    pub launch_flag: u16,
    /// The running script's stream clock, in ticks: state 2 seeds 0 (the state-2 tick is the
    /// branch script's FIRST update [M 0x1aa2b0/0x1aa30c — entry clock 0]) and state 3 advances
    /// one per tick [M Script_Update 0x1da998:33-43]; the End-marker time ends the wait
    /// [M 0x1db420 + 0x1db630]. State 1 reuses it as the p0's tick counter from the state-0 exit.
    pub timer: u32,
}

impl SpaceMissile {
    /// `SpaceMissile_Update` 0x1aa118, one tick. `deploy` is `MissileDeploy` (8 bytes, 0x30cd70).
    pub fn tick(&mut self, ch: &mut Channels, deploy: &mut [u8; 8], frame: u32) {
        match self.state {
            0 => {
                // [M 0x1aa1c8-1e4]: the gate first — no countdown while switch_channels[p7] is clear.
                if self.gate_channel != 0 && !ch.on(self.gate_channel as u32) {
                    return;
                }
                // [M 0x1aa1ec-20c]: countdown -= 512.0; stay while >= 0 (c.lt.s f0,f2 + bc1f).
                self.countdown -= COUNTDOWN_STEP;
                if self.countdown >= 0.0 {
                    return;
                }
                // [M 0x1aa210-238]: MissileDeploy[p4−1] = 1; obj+0xf4 = 1.
                if let Some(slot) = self.deploy_slot() {
                    deploy[slot] = 1;
                }
                self.state = 1;
                self.timer = 0;
            }
            1 => {
                // [M 0x1aa23c-258]: the p0 script plays; done → state 2. The in-stream writes are [D].
                self.timer += 1;
                let idx = (self.index.clamp(1, 8) - 1) as usize;
                if self.timer == P0_SYNC_AT && P0_SYNC_CH[idx] != 0 {
                    ch.set(P0_SYNC_CH[idx] as u32, 1, frame); // [D t10 → ch 181..188; m4: none]
                }
                if self.timer == 1 && self.index == 4 {
                    ch.set(P0_ALT_WRITE as u32, 1, frame); // [D 06000a30 t1 → ch 192]
                }
                let gate = P0_GATE[idx];
                if self.timer == 359 && gate != 0 {
                    ch.set(gate as u32, 1, frame); // [D t359 → ch 31..38]
                }
                if self.timer + P0_PRE_END == P0_LEN[idx] && P0_PRE_END_CH[idx] != 0 {
                    ch.set(P0_PRE_END_CH[idx] as u32, 1, frame); // [D t(L−60) → ch 171..178; m4: none]
                }
                // The exit Event-3 sits at Wait(L−1) [D] and fires on the update whose entry
                // clock is L−1 — state-1 tick L−1 (create ran update 1 [M 0x1a9ef0:61]);
                // `+0x9c != 0` reads the same tick [M 0x1aa244-24c].
                if self.timer >= P0_LEN[idx] - 1 {
                    self.state = 2; // [M 0x1aa250-258]
                }
            }
            2 => {
                // [M 0x1aa25c-338]: the console channel decides the branch.
                if ch.on(self.index as u32) {
                    self.launch_flag = 0; // [M 0x1aa2bc] — the p2 (disarmed) script
                    if let Some(slot) = self.deploy_slot() {
                        deploy[slot] = 2; // [M 0x1aa2cc-338, a0 = 2]
                    }
                } else {
                    self.launch_flag = 1; // [M 0x1aa31c] — the p1 (launched-away) script
                    if let Some(slot) = self.deploy_slot() {
                        deploy[slot] = 3; // [M a0 = 3]
                    }
                }
                // The state-2 tick runs the branch script's FIRST Script_Update [M 0x1aa2b0 /
                // 0x1aa30c] — the clock seeds 0 and state 3 counts from there.
                self.timer = 0;
                self.state = 3; // [M 0x1aa2c0/0x1aa328]
            }
            3 => {
                // [M 0x1aa33c-3d0]: the branch script plays; on end the channel write + state 4.
                self.timer += 1;
                let idx = (self.index.clamp(1, 8) - 1) as usize;
                let writes = if self.launch_flag == 0 { P2_WRITES[idx] } else { P1_WRITES };
                for &(t, c) in writes {
                    if self.timer == t {
                        ch.set(c as u32, 1, frame); // [D decode — the scripts' own kind-0x0d events]
                    }
                }
                if self.timer >= self.wait_len() {
                    if self.launch_flag == 0 {
                        ch.set(self.done_channel as u32, 1, frame); // [M 0x1aa354-398: ch[p6]]
                    } else {
                        ch.set(self.fail_channel as u32, 1, frame); // [M 0x1aa360-3c4: ch[p5] = 199]
                    }
                    self.state = 4; // [M 0x1aa3d4-3d8]
                }
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

    fn wait_len(&self) -> u32 {
        if self.launch_flag == 0 { P2_LEN[(self.index.clamp(1, 8) - 1) as usize] } else { P1_LEN }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triggers::Channels;

    /// A missile past its gate and countdown, in state 1.
    fn launch(index: u16) -> (SpaceMissile, Channels, [u8; 8]) {
        let mut m = SpaceMissile {
            placement: 0,
            index,
            fail_channel: 199,
            done_channel: 20 + index,
            gate_channel: 19,
            countdown: 0.0,
            state: 0,
            launch_flag: 0,
            timer: 0,
        };
        let mut ch = Channels::default();
        let mut deploy = [0u8; 8];
        ch.set(19, 1, 0);
        m.countdown = 105.0;
        m.tick(&mut ch, &mut deploy, 1); // state 0 exits on the first gated tick [M 0x1aa1ec-238]
        assert_eq!(m.state, 1);
        (m, ch, deploy)
    }

    /// The state-1 exit lands on tick L−1 (the create-time update eats clock 0 [M 0x1a9ef0:61];
    /// the exit Event-3 at Wait(L−1) [D 0600092a t3299]). Fails on the pre-task tree, which ran
    /// to the full L.
    #[test]
    fn state1_exits_at_header_length_minus_one() {
        let (mut m, mut ch, mut deploy) = launch(1);
        for t in 1..=3298u32 { m.tick(&mut ch, &mut deploy, t); }
        assert_eq!(m.state, 1, "still tracking the p0 script on tick 3298 [D t3299]");
        m.tick(&mut ch, &mut deploy, 3299);
        assert_eq!(m.state, 2, "the exit Event-3 fires on state-1 tick 3299 = L−1 [M 0x1a9ef0:61 + D t3299]");
    }

    /// Sister: the gate and sync writes still land at their [D] ticks while state 1 runs.
    #[test]
    fn state1_writes_gates_and_sync_channels() {
        let (mut m, mut ch, mut deploy) = launch(1);
        let mut fired = [false; 3];
        for t in 1..=3299u32 {
            m.tick(&mut ch, &mut deploy, t);
            if t == 10 { fired[0] = ch.on(181); } // [D t10 → ch 181..188]
            if t == 359 { fired[1] = ch.on(31); } // [D t359 → ch 31..38]
            if t == 3240 { fired[2] = ch.on(171); } // [D t(L−60) → ch 171..178]
        }
        assert!(fired.iter().all(|&f| f), "the p0's t10/t359/t(L−60) writes all landed [D]");
        assert!(!ch.on(182) && !ch.on(172), "sister: missile 1 writes only ITS lanes [D 0600092a]");
    }

    /// Missile 4's 06000a30 carries ONLY ch 192 @t1 and the exit @t1799 — no t10 sync, no
    /// t(L−60) lane, no gate [D probe: 2 streams]. Fails on a tree that writes ch 184/174 for it.
    #[test]
    fn missile4_p0_writes_only_its_alt_channel() {
        let (mut m, mut ch, mut deploy) = launch(4);
        for t in 1..=1799u32 {
            m.tick(&mut ch, &mut deploy, t);
            if t == 1 { assert!(ch.on(192), "06000a30's ch 192 @t1 [D]"); }
            assert!(!ch.on(184) && !ch.on(174) && !ch.on(34), "no sync/pre-end/gate lanes for missile 4 [D 06000a30]");
        }
        assert_eq!(m.state, 2, "the exit fires at 1799 = L−1 like the other p0s [D t1799]");
    }

    /// The disarm branch: state 2 seeds the clock at 0 (the state-2 tick is the branch script's
    /// first update [M 0x1aa2b0]) and the done channel fires on state-3 tick END = 1710 for
    /// missile 1 [D 0600091c], with the script's own writes along the way. Fails on the pre-task
    /// tree, which seeded timer = LEN−1 and fired on the FIRST state-3 tick.
    #[test]
    fn disarm_branch_runs_the_p2_wait_and_writes() {
        let (mut m, mut ch, mut deploy) = launch(1);
        for t in 1..=3299u32 { m.tick(&mut ch, &mut deploy, t); }
        ch.set(1, 1, 4000); // the console channel latches [D 060007e2 t682]
        m.tick(&mut ch, &mut deploy, 3300);
        assert_eq!(m.state, 3, "state 2 ran on the latched console channel [M 0x1aa25c]");
        assert_eq!(m.launch_flag, 0, "the disarm branch [M 0x1aa2bc]");
        assert_eq!(deploy[0], 2, "MissileDeploy = 2 [M 0x1aa2cc-338]");
        assert!(!ch.on(21), "sister: the done channel does not fire while the p2 script runs [D END 1710]");
        for t in 1..=1709u32 { m.tick(&mut ch, &mut deploy, 3300 + t); }
        assert!(ch.on(81) && ch.on(46) && ch.on(83), "the p2's own writes @t1/t1000/t1030 [D 0600091c]");
        assert!(!ch.on(21), "the p2 wait elapses 1709 ticks [D END 1710]");
        m.tick(&mut ch, &mut deploy, 5010);
        assert!(ch.on(21), "the done channel fires on state-3 tick 1710 [M 0x1aa354-398 + D END]");
        assert_eq!(m.state, 4, "the epilogue [M 0x1aa3d4-3d8]");
    }

    /// Sister for the launch branch: missile 4 (no console writer on disc [D]) runs the p1's
    /// 1200-tick End and aborts through ch 199 [M 0x1aa360-3c4].
    #[test]
    fn launch_branch_runs_the_p1_wait_then_aborts() {
        let (mut m, mut ch, mut deploy) = launch(4);
        for t in 1..=1799u32 { m.tick(&mut ch, &mut deploy, t); } // p0 L = 1800 [D 06000a30]
        m.tick(&mut ch, &mut deploy, 1800); // state 2: ch 4 clear → launch [M 0x1aa31c]
        assert_eq!(m.launch_flag, 1, "the launch branch [M 0x1aa31c]");
        assert_eq!(deploy[3], 3, "MissileDeploy = 3 [M a0 = 3]");
        for t in 1..=1199u32 { m.tick(&mut ch, &mut deploy, 1800 + t); }
        assert!(ch.on(99), "the p1's ch 99 = 1 @t1000 [D 060007c6/060007e1]");
        assert!(!ch.on(199), "sister: the abort waits out the 1200-tick p1 End [D]");
        m.tick(&mut ch, &mut deploy, 3000);
        assert!(ch.on(199), "ch 199 fires on state-3 tick 1200 [M 0x1aa360-3c4 + D END 1200]");
        assert_eq!(m.state, 4, "the epilogue [M 0x1aa3d4-3d8]");
    }
}
