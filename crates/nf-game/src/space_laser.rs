//! The 0700001b space laser — `Create_SpaceLaser` 0x1aa8b8 (class 0x50 → `SpaceLaser_Update`
//! 0x1aa3f8, docs/atlas/object.md:164) [M 001aa8b8].
//!
//! The game's only caller is `parsemap_create_dynamic_objects` 0x1d04f0 **case 0x38** (line 227,
//! the dynamic-object placement walk). The disc ships exactly one kind-0x38 placement: 0700001b
//! PL#108 — the station's laser hazard, armed by switch channel 19, firing at the player every
//! 10 s, killed by channel 14 (Drake's death) [D script-census.md:144; docs/levels/0700001b.md:71].
//! The sibling kind-0x39 SpaceMissile is already ported (`crate::space_missile`, the same walk).
//!
//! Param map [M 001aa8b8:14-37] — the placement record's param words (rec+0x2c = param 0, the
//! same convention `Create_SpaceMissile` 0x1a9ef0 uses):
//! - ext+0x20/+0x24/+0x28 = the raw f32 bits of params 0/1/2,
//! - ext+0x30/+0x34 = `(float)(int)` of params 3/4 (signed int → float),
//! - ext+0x38 = 10.0f (the constant 0x41200000 — the 10 s fire period),
//! - ext+0x40/+0x42/+0x44 = the low u16 of params 5/6/7,
//! - obj+0xff = 0x50 (class), obj+0xf4 = 0 (the update state).
//!
//! `SpaceLaser_Update` 0x1aa3f8 and `SpaceLaser_Register` 0x1aa9e8 are ported below (the
//! script lifecycle behind a `script_playing` input). Sound: `sound-coverage.md:95` lists class
//! 0x50 as MISSING (ids 0x46a / 0x5de here).

/// One kind-0x38 placement's laser object (the ext block at obj+0xE0 plus obj+0xf4 [M 001aa8b8]).
use crate::world::V3;

#[derive(Clone, Debug, PartialEq)]
pub struct SpaceLaser {
    pub placement: usize,
    /// ext+0x20: param 0 as raw f32 bits [M].
    pub a: f32,
    /// ext+0x24: param 1 as raw f32 bits [M].
    pub b: f32,
    /// ext+0x28: param 2 as raw f32 bits [M].
    pub c: f32,
    /// ext+0x30: param 3 as an UNSIGNED u32 → float (the negative arm halves with the sticky bit,
    /// converts, doubles — disasm 0x1aa934-0x1aa968; review 2026-10-21, the branch read it signed).
    pub p3: f32,
    /// ext+0x34: param 4, the same unsigned conversion [disasm 0x1aa970-0x1aa9a4].
    pub p4: f32,
    /// ext+0x40: param 5 low u16 [M].
    pub ch5: u16,
    /// ext+0x42: param 6 low u16 [M].
    pub ch6: u16,
    /// ext+0x44: param 7 low u16 [M].
    pub ch7: u16,
    /// ext+0x38: the 10.0f constant [M 001aa8b8:35].
    pub period: f32,
    /// obj+0xf4: the update state, 0 at creation [M 001aa8b8:38].
    pub state: u16,
    /// ext lane 0 (ext+0x00..0x0C): the fire origin — what `SpaceLaser_Register` 0x1aa9e8
    /// overwrites with the registered script object's position quad [M 0x1aa9e8:12-20].
    pub fire_origin: V3,
    /// ext lane 1 (ext+0x10..0x1C): the aim target — the player quad at track start, the ray
    /// impact point after a state-2 hit [M 0x1aa3f8:105-112, :153-160].
    pub target: V3,
    /// ext lane 3+0x00: the arm-delay clock [M 0x1aa3f8:78-80].
    pub arm_delay: f32,
    /// ext lane 3+0x08: the fire cooldown [M 0x1aa3f8:94-96].
    pub cooldown: f32,
    /// obj+0xFE bit 1, the delete flag state 4 raises [M 0x1aa3f8:189-195].
    pub deleted: bool,
    /// ext lane 3+0x0C as a bool: the whine handle being live (the port has no sound runtime).
    pub whine_on: bool,
    /// The loaded script's tick clock — the port's `Script_Update` stand-in (the game runs
    /// `Script_Update` every tick in states 1 and 3 [M 0x1aa3f8:116/:176]; `Script_IsPlaying`
    /// 0x1db630 reports the play head against the stream's End marker [M Script_StreamEnd
    /// 0x1db420 clears the play bit]). Advanced by the sim wiring after each `tick`.
    pub script_ticks: u32,
    /// The loaded script's [D] length (the lib record's `length` field; the wiring fills it at
    /// the `LoadScript` hand-off from `Data::scripts` — 360/360/10 for the 1b records
    /// [D docs/levels/0700001b-audit.md:606-612]). 0 = no script loaded (a missing script is
    /// the game's own `Script_Load` failure — `Script_IsPlaying(0)` reports 0 and state 1
    /// exits on the next tick).
    pub script_len: u32,
    /// The live whine handle (`Sound_Play(100.0, 0x5de, 0, 0)`'s return, stored at
    /// ext lane 3+0x0C [M 0x1aa3f8:119-121]; `Sound_Stop`ed on the shared tail [M :231-235]).
    /// 0 = none. Written by the sim wiring only.
    pub whine_handle: u32,
}

fn param(params: &[(u32, u32)], k: u32) -> u32 {
    params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v)
}

/// `Create_SpaceLaser` 0x1aa8b8's ext writes, from one kind-0x38 placement's params.
pub fn from_placement(placement: usize, params: &[(u32, u32)]) -> SpaceLaser {
    SpaceLaser {
        placement,
        a: f32::from_bits(param(params, 0)),
        b: f32::from_bits(param(params, 1)),
        c: f32::from_bits(param(params, 2)),
        p3: param(params, 3) as f32,
        p4: param(params, 4) as f32,
        ch5: param(params, 5) as u16,
        ch6: param(params, 6) as u16,
        ch7: param(params, 7) as u16,
        period: 10.0,
        state: 0,
        fire_origin: [0.0; 3],
        target: [0.0; 3],
        arm_delay: param(params, 3) as f32, // ext lane 3+0x00 = param 3 (165.0 on disc [D])
        // ext lane 3+0x08 = the creator's constant 0x41200000 = 10.0 s [M 001aa8b8:38] — the
        // first shot waits the full 10 s re-fire period, it does not fire on the first gate tick.
        cooldown: 10.0,
        deleted: false,
        whine_on: false,
        script_ticks: 0,
        script_len: 0,
        whine_handle: 0,
    }
}

/// The game's ext block is five 16-byte lanes (`pauVar3[k] = ext + k·16` [M 0x1aa3f8:74]):
/// lane 0 = the fire origin (what `SpaceLaser_Register` fills), lane 1 = the aim target,
/// lane 2 = the script block (+0x00/+0x04/+0x08 = p0/p1/p2, +0x0C the SCRIPTINFO handle),
/// lane 3 = the timers (+0x00 arm delay, +0x04 re-raise, +0x08 cooldown, +0x0C sound handle),
/// lane 4 = the channels (p5 write / p6 kill / p7 arm).
///
/// What one `SpaceLaser_Update` 0x1aa3f8 tick does to the outside world. The script lifecycle
/// is abstracted: `SpaceLaserUpdate::script_playing` is fed by the caller each tick (the port's
/// script-player wiring for these 0x060007c0/0x060007c1/0x060007e3 scripts is the same pending
/// integration `space_missile.rs` documents — its pinned-decode approach applies here too).
#[derive(Debug, PartialEq)]
pub enum LaserOut {
    /// `Script_Load(id, 0x2d6200, 0x2d6200, 0,0,0, [callback], ext)` + `Script_Play(mode)` +
    /// the first `Script_Update` [M 0x1aa3f8:95-112 (tracking, mode 0, callback 0x1aa9e8) /
    /// :87-94 (dead, mode 1, no callback)]. `register` = the callback is installed.
    LoadScript { id: u32, play_mode: u8, register: bool },
    /// The state-1 whine: `Sound_Play(100.0, 0x5de, 0, 0)` while the handle is free [M :115-121].
    WhineStart,
    /// Any state's `Sound_Stop` + handle clear (state 0/2/3 shared tail, 4 always) [M :89-92,
    /// :231-235, :190-195].
    SoundStop,
    /// The state-2 shot: `Bullet_init(-1, obj, 0, 0x2c6154, origin, dir, 0xffff)` — weapon_data
    /// base 0x2bf150 + 107·0x10c = record 107 [M :172-175, disasm 0x1aa7c8-0x1aa7d8].
    FireBullet { dir: V3 },
    /// The state-2 beam draw: class-0x10 object at the origin, mesh 0x20005cd, brightness
    /// `Rand_Rand(0x80)+0x7f`, stretched origin→target (`PositionBeam`) [M :176-188] — draw
    /// hand-off (hash + endpoints + alpha; the port's renderer consumes this).
    Beam { alpha: u8 },
    /// `Emitter_Create(0,0,0xc000010, target, 0x2d6210, 0,0,0)` at the hit point [M :189].
    ImpactEmitter,
}

/// The state-2 fire's beam object (`control_create_object(0x40, …)` then `obj+0xff = 0x10`
/// [M 0x1aa3f8:159-165 — control_funcs row 0x10 = `Debris_Update` 0x1aac00, object.md:110]):
/// ext+0x30 = 5 (the life counter), ext+0x32 = 0 (the alpha decay rate), the mesh 0x20005cd,
/// obj+0x106 = `0x7f + Rand(0x80)` (the brightness byte; rate 0 → it never decays)
/// [M 0x1aa3f8:163-167], the stretch origin→target set once by `PositionBeam` [M :188].
/// `Debris_Update` decrements ext+0x30 per tick and raises the delete flag when it drops
/// below 1 — the beam lives 5 update ticks [M 001aac00:20-28]. The port models exactly that
/// as the renderer's draw hand-off (endpoints + alpha per tick; the draw is done separately).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaserBeam {
    /// The fire origin (the stretch start).
    pub from: V3,
    /// The aim lane at the fire tick — the ray impact point when the ray hit [M :153-160].
    pub to: V3,
    /// obj+0x106: `0x7f + Rand(0x80)` [M :166-167].
    pub alpha: u8,
    /// ext+0x30: 5 at creation, −1 per update tick, deleted at < 1 [M 001aac00:20-28].
    pub life: i16,
}

/// The per-tick inputs: the live channels, GameState+0x52, the player position quad
/// (glb_players+0x30), whether the loaded script is still playing.
pub struct SpaceLaserTick<'a> {
    pub ch: &'a mut crate::triggers::Channels,
    pub frame: u32,
    pub player: V3,
    pub script_playing: bool,
}

impl SpaceLaser {
    /// `SpaceLaser_Update` 0x1aa3f8. Draw notes: the only RNG is state 0's `Rand_Random()&1`
    /// script pick and state 2's beam-alpha `Rand_Rand(0x80)` — both consumed only on ticks
    /// that reach them.
    pub fn tick(&mut self, t: &mut SpaceLaserTick, rng: &mut crate::drone::Rng, out: &mut Vec<LaserOut>) {
        let gate_open = self.ch7 == 0 || t.ch.on(self.ch7 as u32); // [M :75-77 ext+0x44 lane 4+4]
        if gate_open {
            self.arm_delay -= REC_FRAME_RATE; // [M :78-80 lane 3+0x00 — runs only while the arm gate is open]
        }
        let mut state = self.state;
        if state != 4 && self.ch6 != 0 && t.ch.on(self.ch6 as u32) {
            self.state = 4; // [M :86-90 the kill gate jumps from any state but 4]
            state = 4;
        }
        match state {
            0 => {
                if gate_open {
                    if self.whine_on {
                        out.push(LaserOut::SoundStop); // [M :89-92]
                        self.whine_on = false;
                    }
                    let old = self.arm_delay;
                    self.cooldown -= REC_FRAME_RATE; // [M :94-96 lane 3+0x08]
                    if old < 0.0 {
                        out.push(LaserOut::LoadScript { id: self.c.to_bits(), play_mode: 1, register: false });
                        self.state = 3; // [M :87-94 dead script]
                    } else if self.cooldown < 0.0 {
                        let pick = rng.random() & 1; // [M :97-99 the p0/p1 pick]
                        let id = if pick == 0 { self.a.to_bits() } else { self.b.to_bits() };
                        out.push(LaserOut::LoadScript { id, play_mode: 0, register: true });
                        self.target = t.player; // [M :105-112 the player quad → lane 1]
                        self.state = 1;
                    }
                }
            }
            1 => {
                // [M :115-126 — Script_Update, the whine gate, the IsPlaying exit. The whine's
                // volume compare `(u16)(SI+0xab0)>>1 < *(f32*)(SI+0xa0)` is script-internal;
                // the port starts it on the first tick the handle is free.]
                if !self.whine_on {
                    out.push(LaserOut::WhineStart);
                    self.whine_on = true;
                }
                if !t.script_playing {
                    self.state = 2; // [M :122-126]
                    self.whine_on = false;
                }
            }
            2 => {
                let mut dir = [self.target[0] - self.fire_origin[0], self.target[1] - self.fire_origin[1],
                    self.target[2] - self.fire_origin[2]]; // [M :128-134]
                let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                if len > 0.0 {
                    dir = [dir[0] / len, dir[1] / len, dir[2] / len]; // [M Vec_Normalise]
                }
                // [M :136-152 — the 1000-unit ray (0x447a0000); a hit rewrites the target to
                // hits[0]+0x40 (the impact point) [M :153-160]. The ray needs world collision —
                // the port's caller supplies the final target (the beam endpoint + emitter).]
                out.push(LaserOut::FireBullet { dir }); // [M :172-175]
                out.push(LaserOut::Beam { alpha: rng.rand(0x80) as u8 + 0x7f }); // [M :183-185]
                out.push(LaserOut::ImpactEmitter); // [M :189]
                self.cooldown = 10.0; // [M :190 0x41200000]
                self.state = 0; // [M :191]
                if self.whine_on {
                    out.push(LaserOut::SoundStop); // [M shared tail :231-235]
                    self.whine_on = false;
                }
            }
            3 => {
                self.p4 -= REC_FRAME_RATE; // [M :196-199 lane 3+0x04, the re-raise timer]
                if self.p4 < 0.0 && self.ch5 != 0 {
                    t.ch.set(self.ch5 as u32, 1, t.frame); // [M :200-204 ch[p5] every tick]
                }
                if self.whine_on {
                    out.push(LaserOut::SoundStop);
                    self.whine_on = false;
                }
            }
            _ => {
                if self.whine_on {
                    out.push(LaserOut::SoundStop);
                    self.whine_on = false;
                }
                self.deleted = true; // [M :189-195 obj+0xFE |= 1 — the delete pass picks it up]
            }
        }
    }

    /// `SpaceLaser_Register` 0x1aa9e8 (the tracking scripts' stream callback, 24 bytes): the
    /// registered object's position quad `obj+0xC0..0xCC` becomes the fire origin [M :12-20].
    /// The port's existing "copies the laser's position" note (space-laser-update.md) is
    /// imprecise — it is the SCRIPT OBJECT's position, the last one the script registered.
    pub fn register(&mut self, script_object_pos: V3) {
        self.fire_origin = script_object_pos;
    }
}

const REC_FRAME_RATE: f32 = 1.0 / 60.0; // [M 0x30d0dc]

#[cfg(test)]
mod tests {
    use super::*;

    // [M 001aa8b8:14-38] the full param map with the DISC values of 0700001b PL#108
    // [D docs/levels/0700001b.md:73]. Correction 2026-10-26: the old synthetic data labelled
    // param 5 "the arm channel" — the decompile disagrees: ext+0x44 (param 7) is the arm gate
    // [M 0x1aa3f8:75-77], ext+0x40 (param 5) is the state-3 re-raise channel [M 0x1aa3f8:200-204].
    #[test]
    fn the_param_map_matches_the_creator() {
        let ps: Vec<(u32, u32)> = vec![
            (0, 0x060007c0), // tracking script A [D]
            (1, 0x060007c1), // tracking script B [D]
            (2, 0x060007e3), // dead/idle script [D]
            (3, 165),        // arm-delay clock seconds [D]
            (4, 65),         // state-3 re-raise timer seconds [D]
            (5, 99),         // state-3 write channel [D]
            (6, 14),         // kill gate (Drake's death) [D]
            (7, 19),         // arm gate [D — never written by any disc script]
        ];
        let l = from_placement(4, &ps);
        assert_eq!(l.placement, 4);
        assert_eq!((l.a.to_bits(), l.b.to_bits(), l.c.to_bits()), (0x060007c0, 0x060007c1, 0x060007e3));
        assert_eq!((l.p3, l.p4), (165.0, 65.0));
        assert_eq!((l.ch5, l.ch6, l.ch7), (99, 14, 19));
        assert_eq!(l.period, 10.0);
        assert_eq!(l.state, 0);
        // Missing params read 0 (the game's record always carries them; the port defaults).
        let empty = from_placement(0, &[]);
        assert_eq!((empty.a, empty.p3, empty.ch5), (0.0, 0.0, 0));
    }

    use crate::drone::Rng;
    use crate::triggers::Channels;

    fn tick(l: &mut SpaceLaser, ch: &mut Channels, frame: u32, script_playing: bool) -> Vec<LaserOut> {
        let mut out = Vec::new();
        let mut t = SpaceLaserTick { ch, frame, player: [1.0, 2.0, 3.0], script_playing };
        let mut rng = Rng::new(0);
        l.tick(&mut t, &mut rng, &mut out);
        out
    }

    /// [M 0x1aa3f8:75-80] with the arm gate closed (ch 19 off — its disc condition: NO script
    /// ever writes it [D docs/levels/0700001b.md:71]), nothing ticks: no cooldown decrement, no
    /// script load, no draw. The laser is inert on retail data.
    #[test]
    fn inert_while_the_arm_gate_is_closed() {
        let mut l = from_placement(0, &[(5, 99), (6, 14), (7, 19)]);
        let mut ch = Channels::default();
        let mut rng_draws = Rng::new(0);
        for f in 0..600u32 {
            let mut out = Vec::new();
            let mut t = SpaceLaserTick { ch: &mut ch, frame: f, player: [1.0, 2.0, 3.0], script_playing: false };
            l.tick(&mut t, &mut rng_draws, &mut out);
            assert!(out.is_empty());
            assert_eq!(l.state, 0);
        }
        assert_eq!(rng_draws.draws, 0, "no RNG reaches the stream while gated [M :75-99]");
        assert_eq!(l.arm_delay, 0.0, "the arm clock only runs while the gate is open [M :78-80]");
    }

    /// [M :86-90] the kill gate (ch 14, Drake's death [D]) jumps to state 4 from any state but
    /// 4 and raises the delete flag [M :189-195].
    #[test]
    fn kill_gate_deletes() {
        let mut l = from_placement(0, &[(6, 14)]);
        let mut ch = Channels::default();
        ch.set(14, 1, 0);
        let out = tick(&mut l, &mut ch, 1, false);
        assert_eq!(l.state, 4);
        assert!(l.deleted);
        assert!(out.iter().all(|o| !matches!(o, LaserOut::LoadScript { .. })));
    }

    /// [M :94-148] gate open, armed: after the creator's 10.0 s cooldown spends [M 001aa8b8:38
    /// — ext+0x38 = 0x41200000 at create; the old 0.0 seed here contradicted the creator and
    /// fired on the first gate tick — the TEST was wrong, fixed 2026-10-26 with the address],
    /// cooldown < 0 → the p0/p1 pick draw, the tracking script load (mode 0, REGISTER),
    /// target = player, state 1; the script's end → state 2 fires the bullet (record 107) +
    /// beam + emitter and re-arms the 10 s cooldown [M :128-191].
    #[test]
    fn the_armed_fire_path() {
        let mut l = from_placement(0, &[(0, 0x060007c0), (1, 0x060007c1), (3, 165), (5, 99), (7, 19)]);
        let mut ch = Channels::default();
        ch.set(19, 1, 0); // the arm gate — only ever poked in a live check, never on disc [D]
        // The 10.0 s creator seed spends at ~tick 600 (10.0 at −1/60 per tick with the float
        // drift; the decrement then the check [M :94-99]) → the tracking branch. The unit
        // harness feeds script_playing explicitly, so the 360-tick tracking window is driven
        // by hand after the load.
        let load_out = {
            let mut load = None;
            let mut script_left = 0u32;
            for f in 1..=601u32 {
                let playing = script_left > 0;
                let out = tick(&mut l, &mut ch, f, playing);
                if load.is_none() && out.iter().any(|o| matches!(o, LaserOut::LoadScript { .. })) {
                    load = Some(out);
                    script_left = 360;
                } else if script_left > 0 { script_left -= 1; }
            }
            load.expect("the tracking script loads once the 10 s seed spends [M 001aa8b8:38]")
        };
        let LaserOut::LoadScript { id, play_mode, register } = &load_out[0] else { panic!("{load_out:?}") };
        let (id, play_mode, register) = (*id, *play_mode, *register);
        assert_eq!((id, play_mode, register), (0x060007c0, 0, true), "mode 0 + the Register callback [M :100-108]");
        assert_eq!(l.target, [1.0, 2.0, 3.0], "the player quad → lane 1 [M :105-112]");
        assert_eq!(l.state, 1);
        l.register([7.0, 8.0, 9.0]); // the script's registered object parks the fire origin [M 0x1aa9e8]
        assert_eq!(l.fire_origin, [7.0, 8.0, 9.0]);
        // The loop's last playing tick already latched the whine [M :115-121 — the port's
        // first-free-handle start]; state 1 holds while the script plays, ends into 2.
        let _ = tick(&mut l, &mut ch, 2, true);
        assert!(l.whine_on, "the whine latched in state 1 [M :119-121]");
        assert_eq!(l.state, 1);
        let out = tick(&mut l, &mut ch, 3, false);
        assert_eq!(l.state, 2);
        // tick 4: the shot.
        let out = tick(&mut l, &mut ch, 4, false);
        assert!(matches!(out[0], LaserOut::FireBullet { .. }), "{out:?}");
        assert!(out.iter().any(|o| matches!(o, LaserOut::Beam { .. })), "the 0x20005cd beam [M :176-188]");
        assert!(out.iter().any(|o| matches!(o, LaserOut::ImpactEmitter)));
        assert_eq!(l.cooldown, 10.0, "the 10 s re-arm [M :190]");
        assert_eq!(l.state, 0);
    }

    /// [M :196-204] state 3 re-raises ch[p5] every tick once the re-raise timer is spent.
    #[test]
    fn dead_state_reraises_its_channel() {
        let mut l = from_placement(0, &[(4, 2), (5, 99)]); // p4 = 65.0 on disc [D]
        l.state = 3;
        let mut ch = Channels::default();
        for f in 1..=100u32 {
            tick(&mut l, &mut ch, f, false);
        }
        assert!(!ch.on(99), "p4 = 2 s: 100 ticks = 1.67 s, not yet [M :196-199]");
        for f in 101..5000u32 {
            tick(&mut l, &mut ch, f, false);
        }
        assert!(ch.on(99), "after the timer the channel holds [M :200-204]");
    }
}
