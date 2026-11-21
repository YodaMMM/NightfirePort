//! The driving missions' streamed sound: which mission-script event starts which music stream or
//! speech line, the two stream channels (`Music`, `Speech`), the speech / ambience fader that ducks
//! the effects and the music under a line, the "Bond moment" stinger, and the script's mix events.
//! docs/systems/drive-audio.md ("Streams") is the write-up. Headless: the outputs are requests and
//! volumes; nothing here plays audio.
//!
//! Evidence (all [M], DRIVING.ELF decompile / disassembly, EE addresses; [D] = disc data):
//! - `EStreamEvent` (action 0x14): creator `FUN_0015d248` copies argument words 1..6 into the
//!   event (`FUN_0015cfd0`); the handler `FUN_0015d000` runs only while `DAT_0032e678` is set and
//!   (word 5 = 0 or the player health fraction `FUN_00206b20(mgr, 1)` > 0). Word 1 = the channel
//!   name (`speech`, `nis`, else a stream channel by name), word 2 = the stream name, word 3 = an
//!   int read as float (`cvt.s.w` at 0x15d164), word 4 = loop, word 6 = prebuffer. A name on the
//!   invalid-stream list `DAT_00355dc4` is refused (that list's writer is not traced).
//!   `speech` → `FUN_002f9798(word3, SpeechVsAmbience, name, nis = 0, word6)` unless the mission
//!   manager's +0x3d0 and +0x3cc are both set (`FUN_00206788`); `nis` → the same call with
//!   word3 = 0 and nis = 1; any other channel → `FUN_002e9728(word3, channel, name, loop = word4,
//!   0, word6)`. All compares are case-insensitive (`FUN_002a9198`; the channel map `FUN_002ea2e0`
//!   uses it too).
//! - Channels: the audio init `FUN_002eb7a8` makes exactly two, `FUN_002e9578("Music",
//!   <speech base> minus its last 2 characters)` and `FUN_002e9578("Speech", <speech base>)`; the
//!   extension is `mus` for `music`, `spe` for `speech`/`nis` (0x3c33c0 / 0x3c37e0 / 0x3c37e8).
//!   The channel's mix group is its own name (`FUN_002e8a90`: `FUN_002f2d50(name)` into +0x84).
//! - Queue `FUN_002e9728`: a 0x4c-byte record — the name lower-cased with `/` → `\`, +0x38 the
//!   float, +0x3c loop, +0x40 nis, +0x44 prebuffer. Channel update `FUN_002e8c70`: volume =
//!   +0x6c × the group × the 3-D gain; while a stream plays and a record waits whose +0x38 >= 0 the
//!   current one fades out linearly over that many seconds (sim ticks / 60) and stops; at -1 it
//!   plays to its end. An idle channel pops the next record; a prebuffered one is opened held
//!   ("prebuffering stream"), released by `EReleaseStream` (`FUN_001560d8`: `DAT_00355db0` 1 → 2)
//!   or purged once it has waited longer than +0xb4 (= `FUN_00261170()` × 10, `FUN_002e8a90`). A
//!   looping stream re-queues itself before it runs dry.
//! - `SpeechVsAmbience` (`FUN_002f9688` → `FUN_002f8d58`): the Speech channel, `0:Fader2`,
//!   `0:Fade Effects`, `0:Fade Music`. Start `FUN_002f8e00`, per update `FUN_002f9180`, ambience
//!   name `FUN_002f9090` (from the region the player car is in, `FUN_00147e30`), its level
//!   `FUN_002f97f8`, the "city" hook `FUN_002f9120`, ambience on/off `FUN_002f9178`.
//! - Stinger: `EAddScoreAction` (`FUN_00147b80`) plays `bondmove` after the score: channel
//!   `DAT_00355e70` (initially "Speech") = `speech` → the NIS path, `music` → the Music channel
//!   with fade 0. `ESwitchStingerChannel` (`FUN_0015d760`) accepts only `music` / `speech`.
//! - Mix events: `ESetAudioMix` `FUN_00157308` (target = clamp((pct/100 + 1) × the group's +0x10),
//!   `FUN_002f33a0` over word 2 reads), `ERevertAudioMix` `FUN_00156a58` (`FUN_002f3410`),
//!   `EResetAudioMix` `FUN_001562c8` (`FUN_002f3450`); all three only while `DAT_0032e668` is set.
//! - `EPlaySound` `FUN_00154658`: `FUN_002f4aa0(bank name, sfx, "Power Ups", 2)` (2-D one-shot from
//!   the bank registered under that name, `FUN_002fd308`). `EPlaySoundPos` `FUN_00154798`: an
//!   `AOneShotSound` in `World Sounds` at the trigger's position, min = word 5, max = word 6;
//!   `EStopSoundPos` `FUN_0015cd90` stops the one at that trigger (`FUN_002ec9a8`).
//!
//! Unproven (stated plainly): what +0x48 of a channel does (set to 1.0 / the ambience level, turned
//! into a 0..24000 value for `FUN_00309ea8`; a filter or a rate); the unit of the prebuffer purge
//! clock (`FUN_00261038` / `FUN_00261170`); the writers of `DAT_00355e6c`, `DAT_003c82c8`,
//! `DAT_00355e88` (inputs here); the meaning of manager +0x3cc / +0x3d0 beyond "set by the camera /
//! letterbox events" (`FUN_002065d0`); the stream's real end (the port's caller reports it).

use std::collections::VecDeque;

use nf_formats::trackstart::{MapScript, TriggerAction};

use super::audio::{speech_base, Mixer};

pub const E_STREAM: u32 = 0x14;
pub const E_PLAY_SOUND: u32 = 0x86;
pub const E_PLAY_SOUND_POS: u32 = 0xa4;
pub const E_STOP_SOUND_POS: u32 = 0xa6;
pub const E_SET_AUDIO_MIX: u32 = 0xf1;
pub const E_REVERT_AUDIO_MIX: u32 = 0xf4;
pub const E_RESET_AUDIO_MIX: u32 = 0x10a;
pub const E_SWITCH_STINGER: u32 = 0x110;
pub const E_RELEASE_STREAM: u32 = 0x111;

/// The stinger stream `FUN_00147b80` plays (0x36e618).
pub const STINGER: &str = "bondmove";
/// `DAT_00355e70`'s initial text [ELF data].
pub const STINGER_DEFAULT: &str = "Speech";

/// Where an `EStreamEvent` goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamTarget {
    /// `speech`: a line through the fader (ducks effects and music).
    Speech,
    /// `nis`: through the fader in the `NIS` group, no ducking, fade word forced to 0.
    Nis,
    /// Any other name: the channel of that name (`music` on every track [D]).
    Channel(String),
}

/// One decoded `EStreamEvent` (argument words 1..6).
#[derive(Clone, Debug, PartialEq)]
pub struct StreamRequest {
    pub target: StreamTarget,
    pub name: String,
    /// Word 3 as `cvt.s.w`: seconds to fade the current stream out before this one; -1 = wait
    /// for it to end.
    pub fade: f32,
    /// Word 4: loop (read by the channel path only; the speech path always queues loop = 0).
    pub looped: bool,
    /// Word 5: only while the player has health left.
    pub needs_health: bool,
    /// Word 6: open held until `EReleaseStream`.
    pub prebuffer: bool,
}

/// The sound side of one script action.
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptSound {
    Stream(StreamRequest),
    /// `EPlaySound`: a 2-D one-shot `sfx` from the bank registered as `bank`, group `Power Ups`.
    PlaySound { bank: String, sfx: String },
    /// `EPlaySoundPos`: a 3-D one-shot at a trigger, group `World Sounds`.
    PlaySoundPos { trigger: Option<usize>, bank: String, sfx: String, flag: bool, min: f32, max: f32 },
    StopSoundPos { trigger: Option<usize> },
    SetMix { group: String, steps: i32, percent: f32 },
    RevertMix { group: String, steps: i32 },
    ResetMix { steps: i32 },
    /// `ESwitchStingerChannel`: `Some` when the name is accepted (`music` / `speech`).
    Stinger(Option<String>),
    ReleaseStream,
}

fn text(m: &MapScript, w: u32) -> Option<String> {
    m.ref_path(w).map(|s| s.strip_prefix("CHAR::").unwrap_or(s).to_string())
}

/// The target `FUN_0015d000` picks for a channel name.
pub fn stream_target(channel: &str) -> StreamTarget {
    if channel.eq_ignore_ascii_case("speech") {
        StreamTarget::Speech
    } else if channel.eq_ignore_ascii_case("nis") {
        StreamTarget::Nis
    } else {
        StreamTarget::Channel(channel.to_string())
    }
}

/// Decodes the sound actions (the creators' argument words) [M creators above]. `None` for any
/// other action, or a stream event with a missing word 1/2 ("EStreamEvent called on incomplete
/// trigger").
pub fn script_sound(a: &TriggerAction, m: &MapScript) -> Option<ScriptSound> {
    let w = |i| a.word(i);
    Some(match a.id {
        E_STREAM => {
            let ch = text(m, w(1))?;
            ScriptSound::Stream(StreamRequest {
                target: stream_target(&ch),
                name: text(m, w(2))?,
                fade: w(3) as i32 as f32,
                looped: w(4) != 0,
                needs_health: w(5) != 0,
                prebuffer: w(6) != 0,
            })
        }
        E_PLAY_SOUND => ScriptSound::PlaySound { bank: text(m, w(1))?, sfx: text(m, w(2))? },
        // FUN_001549f0: event +4 = word 1 (position source), +8 bank, +0xc sfx, +0x10 word 4,
        // +0x14 / +0x18 = words 5 / 6 read as floats (`lwc1` of the ctor's float args).
        E_PLAY_SOUND_POS => ScriptSound::PlaySoundPos {
            trigger: m.trigger_ref(w(1)),
            bank: text(m, w(2))?,
            sfx: text(m, w(3))?,
            flag: w(4) != 0,
            min: a.float(5),
            max: a.float(6),
        },
        E_STOP_SOUND_POS => ScriptSound::StopSoundPos { trigger: m.trigger_ref(w(1)) },
        // FUN_001573e0 → FUN_001572e8: +4 group, +8 steps (int), +0xc percent (`lwc1` 0x157350).
        E_SET_AUDIO_MIX => ScriptSound::SetMix { group: text(m, w(1))?, steps: w(2) as i32, percent: a.float(3) },
        E_REVERT_AUDIO_MIX => ScriptSound::RevertMix { group: text(m, w(1))?, steps: w(2) as i32 },
        E_RESET_AUDIO_MIX => ScriptSound::ResetMix { steps: w(1) as i32 },
        E_SWITCH_STINGER => {
            let c = text(m, w(1))?;
            ScriptSound::Stinger((c.eq_ignore_ascii_case("music") || c.eq_ignore_ascii_case("speech")).then_some(c))
        }
        E_RELEASE_STREAM => ScriptSound::ReleaseStream,
        _ => return None,
    })
}

/// `ESetAudioMix` [M 0x157308]: the fade target, clamp((percent / 100 + 1) × base, 0, 1).
pub fn set_mix_target(base: f32, percent: f32) -> f32 {
    let v = (percent / 100.0 + 1.0) * base;
    if v > 1.0 { 1.0 } else if v < 0.0 { 0.0 } else { v }
}

/// Applies a mix event (the `DAT_0032e668` gate is the caller's). Returns false for other kinds.
pub fn apply_mix(ev: &ScriptSound, mix: &mut Mixer) -> bool {
    match ev {
        ScriptSound::SetMix { group, steps, percent } => {
            let i = mix.get_or_create(group);
            let t = set_mix_target(mix.groups[i].base, *percent);
            mix.fade_to(i, t, *steps);
        }
        ScriptSound::RevertMix { group, steps } => {
            let i = mix.get_or_create(group);
            mix.revert(i, *steps);
        }
        ScriptSound::ResetMix { steps } => mix.reset_all(*steps),
        _ => return false,
    }
    true
}

/// The archive entry a stream name opens: lower-cased, `/` → `\`, + `.asf` (`FUN_002e9728`'s copy
/// and `FUN_002f8e00`'s `"|%s.asf"` / `FUN_002e8c70`'s channel prefix).
pub fn stream_entry(name: &str) -> String {
    let mut s: String = name.chars().map(|c| if c == '/' { '\\' } else { c.to_ascii_lowercase() }).collect();
    s.push_str(".asf");
    s
}

/// The archive a target's channel reads [M 0x2eb7a8 + `FUN_002e9578`]: `Music` = the speech base
/// without its 2-character language + `.mus`; `Speech` / `nis` = `<base><lang>.spe` (with the
/// English fallback of `audio::speech_archive`). Upper-case on the disc (`MIS01.MUS`) [D].
pub fn stream_archive(target: &StreamTarget, track: &str, speech: &str) -> Option<String> {
    match target {
        StreamTarget::Speech | StreamTarget::Nis => Some(speech.to_string()),
        StreamTarget::Channel(c) if c.eq_ignore_ascii_case("music") => Some(format!("driving\\{}.mus", speech_base(track))),
        // FUN_002e9688 returns 0 for an unregistered name; the game would then crash in
        // FUN_002e9728. No track uses one [D].
        StreamTarget::Channel(_) => None,
    }
}

// ---------------------------------------------------------------------------------------------
// One stream channel (AStream, FUN_002e8a90 / FUN_002e9728 / FUN_002e8c70)
// ---------------------------------------------------------------------------------------------

/// A queued stream (the 0x4c-byte record).
#[derive(Clone, Debug, PartialEq)]
pub struct Queued {
    pub name: String,
    pub fade: f32,
    pub looped: bool,
    pub nis: bool,
    pub prebuffer: bool,
}

/// What a channel update asks the player to do.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StreamOut {
    /// Open and play this entry now (`stream_entry` form).
    pub start: Option<String>,
    /// Open this entry held (prebuffered).
    pub hold: Option<String>,
    /// Start the held stream ("releasing stream").
    pub release: bool,
    /// Stop the current stream.
    pub stop: bool,
    /// The channel volume this update (+0x50, before the 0..127 rounding).
    pub volume: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StreamChannel {
    /// The mix group (+0x84): the channel name at first; the fader switches the Speech channel
    /// between `Speech`, `NIS` and `Ambience`.
    pub group: String,
    /// +0x6c the channel's own volume.
    pub volume: f32,
    /// +0x48 (meaning unproven; 1.0 or the ambience level).
    pub aux: f32,
    pub queue: VecDeque<Queued>,
    /// The playing record (+0x5c >= 0), its loop (+0x7c), nis (+0x80), prebuffer (+0x84) flags.
    pub current: Option<Queued>,
    /// +0x4c the fade-out clock in seconds.
    pub fade_t: f32,
    /// Held prebuffer: the clock it was opened at (`DAT_00355db4`) and whether it was released.
    pub held_since: Option<i64>,
    pub released: bool,
    /// +0x70: nothing playing and nothing queued.
    pub idle: bool,
    /// +0x78: the last update's volume rounded to 0 of 127.
    pub silent: bool,
}

impl StreamChannel {
    pub fn new(name: &str) -> StreamChannel {
        StreamChannel { group: name.into(), volume: 1.0, aux: 1.0, queue: VecDeque::new(), current: None, fade_t: 0.0, held_since: None, released: false, idle: true, silent: false }
    }

    /// `FUN_002e9728`: queue a record and clear the idle flag (+0x70 = 0).
    pub fn push(&mut self, q: Queued) {
        self.queue.push_back(Queued { name: stream_entry(&q.name), ..q });
        self.idle = false;
    }

    /// `FUN_002e99d0`: stop the current stream; idle when the queue is empty too.
    pub fn stop(&mut self) {
        self.current = None;
        self.held_since = None;
        if self.queue.is_empty() { self.idle = true; }
    }

    /// `FUN_002e9bc8`: drop the queue (`FUN_002eb4d8`) and stop.
    pub fn flush(&mut self) {
        self.queue.clear();
        self.stop();
    }

    /// `FUN_002e8c70` [M], once per audio update. `group` = the effective volume of `self.group`,
    /// `gain` the 3-D gain (1 for these 2-D channels), `ticks` the sim ticks since the last update,
    /// `ended` = the playing stream ran out (the IOP status +0x28 read 0), `now` the game clock and
    /// `purge` the +0xb4 limit in that clock.
    pub fn update(&mut self, group: f32, gain: f32, ticks: u32, ended: bool, now: i64, purge: i64) -> StreamOut {
        let mut out = StreamOut::default();
        let mut vol = self.volume * group * gain;
        if self.current.is_some() {
            if let Some(next) = self.queue.front() {
                if next.fade >= 0.0 {
                    self.fade_t += ticks as f32 * 0.016666668;
                    if next.fade <= self.fade_t {
                        self.current = None;
                        self.held_since = None;
                        out.stop = true;
                    } else {
                        vol *= 1.0 - self.fade_t / next.fade;
                    }
                }
            }
        }
        // the 0..127 rounding that feeds +0x78
        self.silent = !(vol >= 0.0 && ((if vol > 1.0 { 127 } else { (vol * 127.0 + 0.5) as i32 }) != 0));
        out.volume = vol;
        match self.current.clone() {
            None => {
                let Some(q) = self.queue.pop_front() else {
                    self.idle = true;
                    return out;
                };
                self.fade_t = 0.0;
                if q.prebuffer {
                    out.hold = Some(q.name.clone());
                    self.held_since = Some(now);
                    self.released = false;
                } else {
                    out.start = Some(q.name.clone());
                }
                let looped = q.looped;
                self.current = Some(q);
                // a looping stream with another record behind it is let go at once
                if looped && !self.queue.is_empty() {
                    self.current = None;
                    self.held_since = None;
                }
            }
            Some(cur) => {
                if let Some(t0) = self.held_since {
                    if self.released {
                        out.release = true;
                        self.held_since = None;
                    } else if now - t0 > purge {
                        // "purging stream [%s] after %d ticks"
                        self.stop();
                        out.stop = true;
                    }
                } else if ended {
                    if cur.looped {
                        out.start = Some(cur.name.clone());
                    } else {
                        self.current = None;
                        out.stop = true;
                    }
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------------------------
// SpeechVsAmbience (FUN_002f8d58 / FUN_002f8e00 / FUN_002f9180 / FUN_002f9090 / FUN_002f9120)
// ---------------------------------------------------------------------------------------------

/// The fader object (0x44 bytes). Field names follow their use; offsets in the docs.
#[derive(Clone, Debug, PartialEq)]
pub struct SpeechFader {
    /// +0 the ambience level 0..1 (also the duck amount: 0 = fully ducked).
    pub level: f32,
    /// +4 the current ambience stream name.
    pub ambience: String,
    /// +0x24 the ambience volume (`FUN_002f97f8`).
    pub ambience_volume: f32,
    /// +0x28 a line or an ambience change is in progress (ctor 1).
    pub busy: bool,
    /// +0x2c the ducking is lifted (Fader2 / Fader0 / Music held at 1).
    pub unducked: bool,
    /// +0x30 fade the channel out before the next stream.
    pub fade_out: bool,
    /// +0x34 the "city" hook ran.
    pub city: bool,
    /// +0x38 ambience changes accepted (ctor 1).
    pub enabled: bool,
    /// +0x3c an ambience is wanted (its name is not `Off`).
    pub ambience_on: bool,
}

impl Default for SpeechFader {
    /// `FUN_002f8d58` [M].
    fn default() -> Self {
        SpeechFader { level: 0.0, ambience: String::new(), ambience_volume: 1.0, busy: true, unducked: false, fade_out: false, city: false, enabled: true, ambience_on: false }
    }
}

/// The globals the fader reads that this port does not own.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FaderGlobals {
    /// `DAT_00355e88` (non-zero: Fader0 fades out at 0.25/s, the music channel update is skipped).
    pub e88: bool,
    /// `DAT_00355e6c < 0`: stop the channel.
    pub e6c_negative: bool,
    /// `DAT_003c82c8` (0 at init).
    pub c82c8: bool,
}

impl SpeechFader {
    /// `FUN_002f8e00` [M]: start a speech (`nis` false) or NIS line on the Speech channel.
    /// `fade` = the event's word 3 (0 for NIS), `exists` = the archive has `<name>.asf`.
    /// Returns whether a record was queued.
    pub fn start(&mut self, ch: &mut StreamChannel, music: &mut StreamChannel, mix: &mut Mixer, fade: f32, name: &str, nis: bool, prebuffer: bool, exists: bool) -> bool {
        if mix.volume_of("Speech") == 0.0 || mix.volume_of("NIS") == 0.0 || !exists {
            return false;
        }
        let cur = ch.current.as_ref();
        let (looping, cur_nis) = (cur.is_some_and(|c| c.looped), cur.is_some_and(|c| c.nis));
        // an NIS line waits for nothing but is dropped while a plain speech line plays
        if nis && !ch.idle && !looping && !cur_nis {
            return false;
        }
        if ch.idle || 0.0 <= fade {
            ch.group = if nis { "NIS" } else { "Speech" }.into();
            ch.volume = 1.0;
            ch.aux = 1.0;
            if !nis {
                let fe = mix.volume_of("0:Fade Effects");
                mix.store("0:Fader2", fe);
                music.volume = mix.volume_of("0:Fade Music");
            }
            self.level = 0.0;
        } else {
            self.fade_out = true;
        }
        ch.push(Queued { name: name.into(), fade, looped: false, nis, prebuffer });
        self.unducked = false;
        self.busy = true;
        true
    }

    /// `FUN_002f9090` [M]: the ambience region's stream name (`Off` = none).
    pub fn set_ambience(&mut self, name: &str) {
        if !self.enabled || self.ambience == name {
            return;
        }
        self.ambience_on = name != "Off";
        self.ambience = name.into();
        if !self.busy {
            self.fade_out = true;
            self.unducked = true;
        }
        self.busy = true;
    }

    /// `FUN_002f9120` [M]: in the `city` ambience, fade it out and lift the ducking.
    pub fn city_hook(&mut self) {
        if self.ambience.eq_ignore_ascii_case("city") {
            if !self.busy {
                self.fade_out = true;
                self.unducked = true;
            }
            self.city = true;
            self.busy = true;
        }
    }

    /// `FUN_002f9180` [M], once per audio update (`ticks` = sim ticks since the last one).
    pub fn update(&mut self, ch: &mut StreamChannel, music: &mut StreamChannel, mix: &mut Mixer, ticks: u32, g: FaderGlobals) {
        let dt = ticks as f32 * 0.016666668;
        if ch.idle {
            if !self.busy {
                if self.ambience_on {
                    ch.group = "Ambience".into();
                    ch.push(Queued { name: self.ambience.clone(), fade: 0.0, looped: true, nis: false, prebuffer: false });
                    self.level = 0.001;
                }
            } else {
                ch.volume = 0.0;
                if self.enabled {
                    self.busy = false;
                }
            }
        }
        if !self.busy {
            self.level += dt + dt;
            if 1.0 < self.level {
                self.level = 1.0;
                self.unducked = false;
            }
            ch.volume = self.level;
            ch.aux = self.ambience_volume;
        } else if !self.fade_out {
            if g.e6c_negative {
                ch.flush();
            } else if !ch.silent {
                ch.volume = 1.0;
                ch.aux = 1.0;
            } else if !g.c82c8 {
                ch.flush();
            }
        } else {
            self.level -= dt + dt;
            if self.level <= 0.0 {
                self.level = 0.0;
                let (looping, nis) = ch.current.as_ref().map_or((false, false), |c| (c.looped, c.nis));
                if looping || self.unducked {
                    ch.stop();
                }
                // FUN_002e99b0 reads the playing record's nis flag (+0x80)
                ch.group = if nis { "NIS" } else { "Speech" }.into();
                ch.volume = 1.0;
                self.fade_out = false;
            } else {
                ch.volume = self.level;
                ch.aux = self.ambience_volume;
            }
        }
        let cur_nis = ch.current.as_ref().is_some_and(|c| c.nis);
        if !g.e88 {
            if !self.unducked {
                let fe = mix.volume_of("0:Fade Effects");
                let fm = mix.volume_of("0:Fade Music");
                mix.store("0:Fader0", 1.0);
                if !cur_nis {
                    mix.store("0:Fader2", self.level * (1.0 - fe) + fe);
                    music.volume = self.level * (1.0 - fm) + fm;
                }
            } else {
                mix.store("0:Fader2", 1.0);
                mix.store("0:Fader0", 1.0);
                music.volume = 1.0;
            }
        } else {
            let fe = mix.volume_of("0:Fade Effects");
            mix.store("0:Fader2", self.level * (1.0 - fe) + fe);
            let i = mix.get_or_create("0:Fader0");
            let v = mix.groups[i].volume - dt * 0.25;
            mix.store("0:Fader0", if 0.0 < v { v } else { 0.0 });
        }
    }
}

/// `FUN_00147e30` [M]: the ambience level from the region's band (+0x14 top, +0x18 depth) and the
/// listener height y (listener +0x44): 0 below top - depth, 1 above top, else (y + top + depth) ×
/// 0.1. Checked on the disassembly 0x148088-0x1480f8: the argument register starts 0 (`mtc1
/// zero`), `c.olt.s y, top - depth` → 0; `c.olt.s top, y` → 1.0 (a `mov.s` of the saved 1.0 in
/// the delay slot); else `add.s y, top`, `add.s + depth`, `mul.s × 0x3dcccccd`. All signed floats, all additions.
/// On the disc [D, `drive_ambience_disc`] every `Envi` region has depth 10 and top -1000, except
/// `nearvilla` (MIS4 / RACE, top 0). With top 0 the band is a clean 0..1 ramp over y in [-10, 0];
/// with top -1000 the sum is negative only for y in [-1010, -1000] and 1.0 above it.
pub fn ambience_level(y: f32, top: f32, depth: f32) -> f32 {
    if y < top - depth {
        0.0
    } else if top < y {
        1.0
    } else {
        (y + top + depth) * 0.1
    }
}

/// The gate inputs of `FUN_0015d000`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StreamGate {
    /// `DAT_0032e678`.
    pub sound_on: bool,
    /// `FUN_00206b20(mgr, 1)`: manager +0x378 / +0x37c, the player health fraction (`FUN_00206ec0`).
    pub health: f32,
    /// `FUN_00206788`: manager +0x3d0 and +0x3cc both set (speech lines are skipped).
    pub cutscene: bool,
}

/// Both channels, the fader and the stinger channel.
#[derive(Clone, Debug, PartialEq)]
pub struct DriveStreams {
    pub music: StreamChannel,
    pub speech: StreamChannel,
    pub fader: SpeechFader,
    /// `DAT_00355e70`.
    pub stinger: String,
}

impl Default for DriveStreams {
    fn default() -> Self {
        DriveStreams { music: StreamChannel::new("Music"), speech: StreamChannel::new("Speech"), fader: SpeechFader::default(), stinger: STINGER_DEFAULT.into() }
    }
}

impl DriveStreams {
    /// `FUN_0015d000` [M]. `exists(entry)` = the Speech archive has the entry (the `|%s.asf`
    /// check `FUN_00253ba8`); the Music path does not check. Returns whether something was queued.
    pub fn stream_event(&mut self, r: &StreamRequest, g: StreamGate, mix: &mut Mixer, exists: impl Fn(&str) -> bool) -> bool {
        if !g.sound_on || (r.needs_health && !(0.0 < g.health)) {
            return false;
        }
        match &r.target {
            StreamTarget::Speech => {
                if g.cutscene {
                    return false;
                }
                let e = exists(&stream_entry(&r.name));
                self.fader.start(&mut self.speech, &mut self.music, mix, r.fade, &r.name, false, r.prebuffer, e)
            }
            StreamTarget::Nis => {
                let e = exists(&stream_entry(&r.name));
                self.fader.start(&mut self.speech, &mut self.music, mix, 0.0, &r.name, true, r.prebuffer, e)
            }
            StreamTarget::Channel(c) if c.eq_ignore_ascii_case("music") => {
                self.music.push(Queued { name: r.name.clone(), fade: r.fade, looped: r.looped, nis: false, prebuffer: r.prebuffer });
                true
            }
            StreamTarget::Channel(_) => false,
        }
    }

    /// `FUN_00147b80` [M]: the stinger after a score action.
    pub fn stinger(&mut self, mix: &mut Mixer, exists: impl Fn(&str) -> bool) -> bool {
        if self.stinger.eq_ignore_ascii_case("speech") {
            let e = exists(&stream_entry(STINGER));
            self.fader.start(&mut self.speech, &mut self.music, mix, 0.0, STINGER, true, false, e)
        } else if self.stinger.eq_ignore_ascii_case("music") {
            self.music.push(Queued { name: STINGER.into(), fade: 0.0, looped: false, nis: false, prebuffer: false });
            true
        } else {
            false
        }
    }

    /// `FUN_001560d8` [M]: release the held (prebuffered) stream, if any.
    pub fn release(&mut self) {
        for ch in [&mut self.music, &mut self.speech] {
            if ch.held_since.is_some() {
                ch.released = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::easnd::MixLine;

    fn mixer() -> Mixer {
        let ini = [
            MixLine { name: "0:Fade Effects".into(), volume: 0.4, param: 0 },
            MixLine { name: "0:Fade Music".into(), volume: 0.5, param: 0 },
            MixLine { name: "NIS".into(), volume: 1.0, param: 0 },
            MixLine { name: "Music".into(), volume: 0.8, param: 0 },
            MixLine { name: "Player Car".into(), volume: 0.6, param: 0 },
        ];
        Mixer::driving(&ini, 100, 100)
    }

    #[test]
    fn targets_and_archives() {
        assert_eq!(stream_target("CHAR"), StreamTarget::Channel("CHAR".into()));
        assert_eq!(stream_target("NIS"), StreamTarget::Nis);
        assert_eq!(stream_target("Speech"), StreamTarget::Speech);
        assert_eq!(stream_entry("PAR_R_DOM_6aB"), "par_r_dom_6ab.asf");
        assert_eq!(stream_entry("a/B"), "a\\b.asf");
        let m = StreamTarget::Channel("Music".into());
        assert_eq!(stream_archive(&m, "paris_mis01", "driving\\mis01en.spe").as_deref(), Some("driving\\mis01.mus"));
        assert_eq!(stream_archive(&m, "snow2a_race", "driving\\mis4fr.spe").as_deref(), Some("driving\\mis4.mus"));
        assert_eq!(stream_archive(&StreamTarget::Nis, "x", "driving\\mis3en.spe").as_deref(), Some("driving\\mis3en.spe"));
        assert_eq!(stream_archive(&StreamTarget::Channel("fx".into()), "x", "y"), None);
    }

    #[test]
    fn set_audio_mix_scales_the_ini_volume() {
        // MIS01 T614: ("Player Car", 1 step, 110 %) on an ini volume of 0.6 -> min(2.1 * 0.6, 1)
        assert_eq!(set_mix_target(0.6, 110.0), 1.0);
        assert!((set_mix_target(0.6, -50.0) - 0.3).abs() < 1e-6);
        assert_eq!(set_mix_target(0.6, -200.0), 0.0);
        let mut mix = mixer();
        apply_mix(&ScriptSound::SetMix { group: "Player Car".into(), steps: 4, percent: -50.0 }, &mut mix);
        let i = mix.find("Player Car").unwrap();
        assert!((mix.groups[i].step - (0.3 - 0.6) / 4.0).abs() < 1e-6);
        // revert goes back to +8 (0.6, saved by the setter)
        apply_mix(&ScriptSound::RevertMix { group: "Player Car".into(), steps: 2 }, &mut mix);
        assert_eq!(mix.groups[i].target, 0.6);
        // reset: a group with no ini line (base -1) fades to 0
        apply_mix(&ScriptSound::ResetMix { steps: 1 }, &mut mix);
        let s = mix.find("Speech").unwrap();
        assert_eq!(mix.groups[s].target, 0.0);
        assert_eq!(mix.groups[i].target, 0.6);
    }

    #[test]
    fn channel_fades_the_current_stream_out_over_the_fade_word() {
        let mut ch = StreamChannel::new("Music");
        ch.push(Queued { name: "snipeintro".into(), fade: -1.0, looped: false, nis: false, prebuffer: false });
        let o = ch.update(1.0, 1.0, 1, false, 0, 600);
        assert_eq!(o.start.as_deref(), Some("snipeintro.asf"));
        // fade -1: waits
        ch.push(Queued { name: "snipeloop".into(), fade: -1.0, looped: true, nis: false, prebuffer: false });
        assert_eq!(ch.update(1.0, 1.0, 60, false, 0, 600).volume, 1.0);
        assert!(ch.current.as_ref().is_some_and(|c| c.name == "snipeintro.asf"));
        // the intro ends -> the loop starts next update
        assert!(ch.update(1.0, 1.0, 1, true, 0, 600).stop);
        assert_eq!(ch.update(1.0, 1.0, 1, false, 0, 600).start.as_deref(), Some("snipeloop.asf"));
        // a 2 s fade: after 60 ticks (1 s) the volume is halved, after 120 it stops
        ch.push(Queued { name: "all".into(), fade: 2.0, looped: true, nis: false, prebuffer: false });
        let o = ch.update(0.8, 1.0, 60, false, 0, 600);
        assert!((o.volume - 0.4).abs() < 1e-5, "{}", o.volume);
        // the stop and the next record's start happen in the same update (0x2e8d64 then 0x2e8f8c)
        let o = ch.update(0.8, 1.0, 60, false, 0, 600);
        assert!(o.stop);
        assert_eq!(o.start.as_deref(), Some("all.asf"));
        // a looping stream restarts when it runs out
        assert_eq!(ch.update(0.8, 1.0, 1, true, 0, 600).start.as_deref(), Some("all.asf"));
        // fade 0 (the `all` music cue, the NIS path): cut on the next update
        ch.push(Queued { name: "silence".into(), fade: 0.0, looped: false, nis: false, prebuffer: false });
        let o = ch.update(0.8, 1.0, 1, false, 0, 600);
        assert!(o.stop);
        assert_eq!(o.start.as_deref(), Some("silence.asf"));
    }

    #[test]
    fn prebuffered_streams_wait_for_release_or_purge() {
        let mut s = DriveStreams::default();
        let mut mix = mixer();
        let g = StreamGate { sound_on: true, health: 1.0, cutscene: false };
        let r = StreamRequest { target: StreamTarget::Nis, name: "construction".into(), fade: -1.0, looped: false, needs_health: false, prebuffer: true };
        assert!(s.stream_event(&r, g, &mut mix, |_| true));
        let o = s.speech.update(1.0, 1.0, 1, false, 100, 600);
        assert_eq!(o.hold.as_deref(), Some("construction.asf"));
        assert!(!s.speech.update(1.0, 1.0, 1, false, 200, 600).release);
        s.release();
        assert!(s.speech.update(1.0, 1.0, 1, false, 300, 600).release);
        // a second one is purged after the limit
        let r2 = StreamRequest { name: "cornerstraight".into(), ..r };
        s.speech.flush();
        assert!(s.stream_event(&r2, g, &mut mix, |_| true));
        s.speech.update(1.0, 1.0, 1, false, 0, 600);
        assert!(!s.speech.update(1.0, 1.0, 1, false, 600, 600).stop);
        assert!(s.speech.update(1.0, 1.0, 1, false, 601, 600).stop);
    }

    #[test]
    fn a_speech_line_ducks_effects_and_music_then_ramps_back() {
        let mut s = DriveStreams::default();
        let mut mix = mixer();
        let g = StreamGate { sound_on: true, health: 1.0, cutscene: false };
        let line = StreamRequest { target: StreamTarget::Speech, name: "PAR_R_BOND_4".into(), fade: -1.0, looped: false, needs_health: false, prebuffer: false };
        // gates: health, cutscene, sound off, missing file
        assert!(!s.stream_event(&StreamRequest { needs_health: true, ..line.clone() }, StreamGate { health: 0.0, ..g }, &mut mix, |_| true));
        assert!(!s.stream_event(&line, StreamGate { cutscene: true, ..g }, &mut mix, |_| true));
        assert!(!s.stream_event(&line, StreamGate { sound_on: false, ..g }, &mut mix, |_| true));
        assert!(!s.stream_event(&line, g, &mut mix, |_| false));
        assert!(s.stream_event(&line, g, &mut mix, |e| e == "par_r_bond_4.asf"));
        // ducked at once: Fader2 = Fade Effects (0.4), music = Fade Music (0.5)
        assert_eq!(mix.groups[mix.find("0:Fader2").unwrap()].volume, 0.4);
        assert_eq!(s.music.volume, 0.5);
        assert_eq!(s.speech.group, "Speech");
        let fg = FaderGlobals::default();
        s.speech.update(1.0, 1.0, 1, false, 0, 600);
        s.fader.update(&mut s.speech, &mut s.music, &mut mix, 1, fg);
        assert_eq!(s.fader.level, 0.0);
        assert_eq!(s.speech.volume, 1.0);
        // the line ends: the channel goes idle, busy clears, the level ramps 2/60 per tick
        s.speech.update(1.0, 1.0, 1, true, 0, 600);
        s.speech.update(1.0, 1.0, 1, false, 0, 600);
        assert!(s.speech.idle);
        s.fader.update(&mut s.speech, &mut s.music, &mut mix, 3, fg);
        assert!(!s.fader.busy);
        assert!((s.fader.level - 0.1).abs() < 1e-6);
        let f2 = mix.groups[mix.find("0:Fader2").unwrap()].volume;
        assert!((f2 - (0.1 * 0.6 + 0.4)).abs() < 1e-6, "{f2}");
        assert!((s.music.volume - (0.1 * 0.5 + 0.5)).abs() < 1e-6);
        // 0.5 s later fully back
        s.fader.update(&mut s.speech, &mut s.music, &mut mix, 30, fg);
        assert_eq!(s.fader.level, 1.0);
        assert_eq!(mix.groups[mix.find("0:Fader2").unwrap()].volume, 1.0);
    }

    #[test]
    fn nis_lines_do_not_duck_and_yield_to_a_playing_line() {
        let mut s = DriveStreams::default();
        let mut mix = mixer();
        let g = StreamGate { sound_on: true, health: 1.0, cutscene: true };
        let nis = StreamRequest { target: StreamTarget::Nis, name: "nukedeath".into(), fade: -1.0, looped: false, needs_health: false, prebuffer: false };
        // NIS plays during a cutscene, groups NIS, no duck
        assert!(s.stream_event(&nis, g, &mut mix, |_| true));
        assert_eq!(s.speech.group, "NIS");
        assert_eq!(mix.groups[mix.find("0:Fader2").unwrap()].volume, 1.0);
        assert_eq!(s.music.volume, 1.0);
        // a speech line playing (not looped, not NIS) drops a new NIS line
        let mut s = DriveStreams::default();
        let line = StreamRequest { target: StreamTarget::Speech, name: "a".into(), ..nis.clone() };
        s.stream_event(&line, StreamGate { cutscene: false, ..g }, &mut mix, |_| true);
        s.speech.update(1.0, 1.0, 1, false, 0, 600);
        assert!(!s.stream_event(&nis, g, &mut mix, |_| true));
        // the stinger: default channel Speech -> NIS path; `music` -> Music queue
        let mut s = DriveStreams::default();
        assert!(s.stinger(&mut mix, |e| e == "bondmove.asf"));
        assert_eq!(s.speech.queue[0].name, "bondmove.asf");
        assert!(s.speech.queue[0].nis);
        s.stinger = "music".into();
        assert!(s.stinger(&mut mix, |_| false));
        assert_eq!(s.music.queue[0], Queued { name: "bondmove.asf".into(), fade: 0.0, looped: false, nis: false, prebuffer: false });
    }

    #[test]
    fn ambience_loops_when_the_speech_channel_is_free() {
        let mut s = DriveStreams::default();
        let mut mix = mixer();
        let fg = FaderGlobals::default();
        s.fader.set_ambience("city");
        assert!(s.fader.ambience_on && s.fader.busy);
        // first update: idle + busy -> volume 0, busy clears (enabled)
        s.fader.update(&mut s.speech, &mut s.music, &mut mix, 1, fg);
        assert!(!s.fader.busy);
        // next: idle + free -> the ambience is queued looping in group Ambience at level 0.001
        s.fader.update(&mut s.speech, &mut s.music, &mut mix, 1, fg);
        assert_eq!(s.speech.group, "Ambience");
        assert_eq!(s.speech.queue[0], Queued { name: "city.asf".into(), fade: 0.0, looped: true, nis: false, prebuffer: false });
        assert!((s.fader.level - (0.001 + 2.0 / 60.0)).abs() < 1e-6);
        s.fader.set_ambience("Off");
        assert!(!s.fader.ambience_on);
        assert_eq!(ambience_level(-200.0, -100.0, 50.0), 0.0);
        assert_eq!(ambience_level(0.0, -100.0, 50.0), 1.0);
        assert!((ambience_level(-120.0, -100.0, 50.0) - (-17.0)).abs() < 1e-4);
    }
}
