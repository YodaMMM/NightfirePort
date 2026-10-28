//! Review of unported-guard-ai-other-3b (2026-10-21): the HostageKiller pairing and execution on
//! 07000002, the level that places them (placement mode param 4 → +0xC5 = 4 via
//! `NDrone2_GetDTYPENEW` 0x14c938's `switch(param_3)` case 4 [M]; census: k16 = 4 on 07000002 ×2 and
//! 07000003 ×2 only — not 07000012, whose dtype-4 placements carry mode param 0 [D drone-props]).
//! - Pairing `NDrone2_SetupHostageKiller` 0x144030: the nearest mode-5 drone with +0x2B0 == 0 within
//!   5.0, BOTH links set (killer +0x2B0 = hostage, hostage +0x2B0 = killer) [M :22-59].
//! - Execution: HostageKillerAttack 0x15c940 ENTER arms timer 10 at FRAME_RATE; case 10 calls the
//!   execution anim 0x58 with done-msg 5 (CallAnim a3 = 5 → +0x598 [disasm 0x15cab8; M 0x13ab58:100]);
//!   case 5 sends msg 0xb (delay 10) to the hostage [M :61-66], whose case 0xb enters HostageDie 0xb
//!   [M 0x15cb78], and the killer attacks (0x56).

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_killer_pairs_both_ways_and_his_execution_kills_the_hostage() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002"));
    w.wait(120);
    let (ki, hi) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let ki = sim.drones.list.iter().position(|d| d.mode == 4 && (d.pos[0] + 45.6).abs() < 0.5 && (d.pos[2] - 10.9).abs() < 0.5).expect("the 0x17 killer [D]");
        let hi = sim.drones.list.iter().position(|d| d.mode == 5 && (d.pos[0] + 45.5).abs() < 0.5 && (d.pos[2] - 12.2).abs() < 0.5).expect("geisha 1 [D]");
        (ki, hi)
    };
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.channels.set(0x17, 1, sim.drones.frame); }
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let k = &sim.drones.list[ki];
        assert_eq!(k.state, state::HOSTAGE_KILLER, "mode 4 starts in HostageKiller 8 [M 0x14cbd8 case 4]");
        assert_eq!(k.hostage_target, Some(hi), "the killer links the nearest mode-5 drone within 5.0 [M 0x144030]");
        assert_eq!(sim.drones.list[hi].captor, Some(ki), "and the hostage links back to HIM [M 0x144030:58]");
    }
    // The execution chain from HostageKillerAttack (the Global timer-10 route into state 9 is the
    // game's; entered directly here — the subject is the 9 → anim 0x58 → msg 5 → hostage 0xb chain).
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.list[ki].set_state(state::HOSTAGE_KILLER_ATTACK); }
    let mut died = None;
    for t in 0..600u32 {
        w.wait(1);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.list[hi].state == state::HOSTAGE_DIE || sim.drones.list[hi].state == state::HOSTAGE_DEAD { died = Some(t); break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(died.is_some(), "the execution anim's done-msg 5 sends msg 0xb → HostageDie — hostage state {:#x}, killer {:#x}",
        sim.drones.list[hi].state, sim.drones.list[ki].state);
    // Case 5 then sets Attack 0x56 (whose ENTER routes on at once — so the state LOG holds it).
    assert!(sim.drones.list[ki].log.iter().any(|(_, s)| *s == state::ATTACK), "case 5 then attacks [M 0x15c940 case 5]");
}

/// Sister: a mode-5 drone farther than 5.0 is never paired — the other killer (0x19's, at
/// (−11.2, −37.8)) takes geisha 2 only, never geisha 1 [M 0x144030 `fVar9 < 5.0`].
#[test]
fn a_hostage_beyond_5_units_is_not_paired() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002"));
    w.wait(120);
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.channels.set(0x19, 1, sim.drones.frame); }
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();
    let k2 = sim.drones.list.iter().position(|d| d.mode == 4 && (d.pos[0] + 11.2).abs() < 0.5).expect("the 0x19 killer [D]");
    let g1 = sim.drones.list.iter().position(|d| d.mode == 5 && (d.pos[0] + 45.5).abs() < 0.5).expect("geisha 1");
    assert_ne!(sim.drones.list[k2].hostage_target, Some(g1), "geisha 1 is ~60 units away — never his");
    assert!(sim.drones.list[k2].hostage_target.is_some(), "he pairs geisha 2 within 5.0");
}

/// The headless frame's audio half (matrix.rs `sample`'s pattern — a device-less test cannot run
/// the windowed loop's `sound_tick`, whose driver callbacks need an output): advances the sound
/// engine's state machine so one-shot voices end and the drones' `voice_finished` refresh sees it
/// next tick. `&|_| false` = no device reports back; a one-shot ends after START_GRACE.
/// The mission-dialogue line ids' sfx set (`TalkToMissionObj` 0x14a8b8: `+0xC5` 4 → 0x29..0x2e,
/// 5 → 0x2f..0x32 [M :8-19]) — the census tracks only these: both drones also bark non-mission
/// lines, which the game's busy tests never sequence.
fn mission_set(sim: &nf_view::scene::Sim, lo: usize, hi: usize) -> std::collections::HashSet<u32> {
    let mut out = std::collections::HashSet::new();
    if let Some(v) = sim.voices.as_deref() {
        for l in v.iter().skip(lo).take(hi - lo + 1) { if l.sfx != 0 { out.insert(l.sfx); } }
    }
    out
}

fn audio_step(scene: &mut nf_view::scene::Scene) {
    let sim = scene.sim.as_mut().unwrap();
    let (pos, f) = (sim.bond.head(), sim.bond.look());
    let norm = { let l = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6); [f[2] / l, 0.0, -f[0] / l] };
    sim.drones.map_sounds.update(&mut sim.drones.audio, pos);
    sim.drones.audio.update(sim.sfx.as_ref(), (pos, norm, [0.0, 1.0, 0.0], f), &|_| false);
    let _ = sim.drones.audio.take_commands();
}

/// hostage-killer-dialogue (2026-10-22): the mission-dialogue sequencer.
/// `NDrone2_MonitorTalkToMissionObj` 0x144340 runs at the top of HostageKiller's UPDATE
/// [M 0x15c728:34]: step 0 sends msg 0x1A to the captor's own obj [M :19-21]; step 1 waits on the
/// captor's own +0x4D0 (the busy test clears a finished line [M :10-24]) then sends 0x1A to the
/// hostage (+0x2B0) [M :26-29]; step 2 waits on the HOSTAGE's +0x4D0 (read through hostage
/// obj+0xE0, clearing a finished one [M :47-56]) then sends 0x1A to the captor [M :58-73] — the
/// pair alternate lines, each gated on the other's voice. The 0x1A handler is
/// `NDrone2_TalkToMissionObj` 0x14a8b8: +0xC5 4 → 0x29+Rand(6), 5 → 0x2F+Rand(4) [M :8-21].
/// Observation: the step-1 sends are observed as the step machine's 1 → 2 edge (the game
/// dispatches them the same tick [M 0x172128] and the port delivers post-walk); the step-2/step-0
/// self-sends ride `self_msg` (same-tick, not queued) — so the hostage-directed sends recur and
/// the alternation is pinned by the mission voice PERIODS, correlated to the sends. One proven
/// exception to strict alternation: the killer's pool line 0x2E carries sfxId 0 [D], SpeechSFX
/// no-ops it [M 0x141b00] and step 1 then advances at once [M 0x144340:10-24] — the HOSTAGE speaks
/// twice; accepted only across a gap where the killer's voice never appears.
#[test]
fn the_captor_and_hostage_alternate_mission_lines() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002"));
    let (ki, hi) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let ki = sim.drones.list.iter().position(|d| d.mode == 4 && (d.pos[0] + 45.6).abs() < 0.5 && (d.pos[2] - 10.9).abs() < 0.5).expect("the 0x17 killer [D]");
        let hi = sim.drones.list.iter().position(|d| d.mode == 5 && (d.pos[0] + 45.5).abs() < 0.5 && (d.pos[2] - 12.2).abs() < 0.5).expect("geisha 1 [D]");
        (ki, hi)
    };
    // The 0x17 channel brings the pair in (switch_channel 23 [D placement]; the existing pairing
    // test's scaffolding) — the killer sits disabled in state 1 until it fires.
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.channels.set(0x17, 1, sim.drones.frame); }
    let (vk, vh) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (mission_set(sim, 0x29, 0x2e), mission_set(sim, 0x2f, 0x32))
    };
    // The census tracks only MISSION voice periods — a period counts when an expectation is
    // pending for that drone: the step-1 send to the hostage (observed in the queues), the
    // step-2 self-send (the hostage's period just closed), or the opener (the step left 0).
    // Both drones also bark non-mission lines that can share a mission line's sfx sample.
    let mut hostage_sends = 0u32;
    let mut periods: Vec<(u32, u32, usize, bool)> = Vec::new(); // (start, end, drone, silent gap before it)
    let mut open: Option<(usize, u32, u32)> = None;       // (drone, start, handle)
    let mut expect: Option<(usize, u32)> = None;          // (drone, deadline)
    let mut overlap: Option<u32> = None;
    let mut paired = false;
    let mut last_handles: [Option<u32>; 2] = [None; 2];                     // last voice handle seen per drone
    let mut k_prev_step = 0u16;                                             // the killer's talk_step last tick
    // The captor's OWN turn observed: +0x4ce (talk_step) is written only by the monitor — case 0
    // self-sends 0x1A and advances 0→1 [M 0x144340 case 0 (`uVar1 != 0` → Drone_Message(*param_1,
    // 0x1a); the only other writer is DefaultInit's zero [M 0x14b300:80]] — so talk_step != 0
    // before anything spoke is exactly the captor's opener turn having run.
    let mut captor_turn = false;
    let mut open_gap = false;                                               // silent gap of the open period
    let mut gap_after: Option<usize> = None;                                // the drone whose period just closed
    let mut gap_killer_voice = false;                                       // the killer's voice seen in the gap
    for t in 0..2700u32 {
        w.wait(1);
        audio_step(&mut w.scene);
        let sim = w.scene.sim.as_ref().unwrap();
        if !paired {
            paired = sim.drones.list[ki].hostage_target == Some(hi)
                && sim.drones.list[ki].state == state::HOSTAGE_KILLER;
            if paired { continue; }
        }
        let (k, h) = (&sim.drones.list[ki], &sim.drones.list[hi]);
        // Step 1's send to the hostage: the game dispatches it the same tick [M 0x172128] and the
        // port delivers it post-walk, so the send is observed as the step machine's 1 → 2 edge
        // (the tick the captor hands the turn over).
        let sent = k_prev_step == 1 && k.talk_step == 2;
        k_prev_step = k.talk_step;
        if sent { hostage_sends += 1; expect = Some((hi, t + 4)); }
        if !captor_turn && periods.is_empty() && open.is_none() && k.talk_step != 0 { captor_turn = true; }
        // The gap between periods: the killer's turn is the only thing between two hostage
        // periods — record whether his voice appears there at all.
        if open.is_none() {
            if gap_after == Some(hi) { gap_killer_voice |= k.voice.is_some(); }
        }
        // Expect the killer's reply once the hostage's mission period closes [M 0x144340:42-64].
        if let Some((d, st, _)) = open { if d == hi && h.voice.is_none() { expect = Some((ki, t + 3)); let _ = st; } }
        // Close the open period when its voice clears (both drones first, then opens — the
        // killer's reply lands the same tick the hostage's finished voice is cleared).
        for (di, voice) in [(ki, k.voice), (hi, h.voice)] {
            if voice.is_none() && open.map_or(false, |(d, _, _)| d == di) {
                if let Some((d, st, _)) = open.take() {
                    periods.push((st, t, d, std::mem::take(&mut open_gap)));
                    gap_after = Some(d);
                    gap_killer_voice = false;
                }
            }
        }
        // Open a period: a mission-sfx voice with a fresh handle while an expectation is pending
        // (or the opener: the killer's step just left 0 and nothing has spoken yet).
        for (di, voice, set, last) in [(ki, k.voice, &vk, 0), (hi, h.voice, &vh, 1)] {
            if let Some(v) = voice {
                if set.contains(&v.1) && last_handles[last] != Some(v.0) {
                    if open.is_some() {
                        let opener = di == ki && k.talk_step != 0 && periods.is_empty();
                        let expected = matches!(expect, Some((d, dl)) if d == di && t <= dl);
                        if opener || expected { overlap = Some(t); break; } // a second mission voice while one is open
                    } else {
                        let opener = di == ki && k.talk_step != 0 && periods.is_empty();
                        let expected = matches!(expect, Some((d, dl)) if d == di && t <= dl);
                        if opener || expected {
                            open_gap = gap_after == Some(di) && !gap_killer_voice;
                            open = Some((di, t, v.0));
                            expect = None;
                            gap_after = None;
                        }
                    }
                }
                last_handles[last] = Some(v.0);
            }
        }
        if let Some((_, dl)) = expect { if t > dl { expect = None; } }
        if periods.len() >= 4 { break; } // two full alternations — enough to pin the sequencer
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let k = &sim.drones.list[ki];
        assert!(paired, "the killer pairs geisha 1 and stays HostageKiller [M 0x144030; 0x15c728:21-26]");
        assert!(overlap.is_none(), "mission voices overlapped — each waits on the other's +0x4D0 [M 0x144340:10-24, 47-56]");
        assert!(hostage_sends >= 2, "the step machine keeps sending 0x1A to the hostage (the bug: never any send); got {hostage_sends}");
        assert!(periods.len() >= 4, "both sides speak twice — the dialogue runs; periods {periods:?}");
        // The captor's TURN is first — step 0 sends 0x1A to his own obj and advances [M
        // 0x144340 case 0] — but one of his six lines is silent [M 0x14a8b8 0x29+Rand_Rand(6);
        // D line 0x2E carries sfxId 0; M 0x141b00 SpeechSFX returns before +0x4D0 is written;
        // M 0x144340:10-24 step 1's busy test then falls through at once], so the first
        // OBSERVED period may be the hostage's — accepted only across the captor's own turn
        // (talk_step != 0 before anything spoke; see the captor_turn note above).
        assert!(periods[0].2 == ki || (periods[0].2 == hi && captor_turn),
            "the opener is the captor's own line [M 0x144340:34-40]; a hostage first is valid only across his silent-0x2E roll [M 0x14a8b8 + D 0x2E.sfx=0 + M 0x141b00 + M 0x144340:10-24]; captor_turn {captor_turn}, periods {periods:?}");
        for i in 1..periods.len() {
            // Strict alternation with the one proven exception (the silent-0x2E roll — see the
            // header comment): a same-side repeat is accepted only across a gap where the
            // killer's voice never appeared.
            let repeat_ok = periods[i].3 && periods[i].2 == hi;
            assert!(repeat_ok || periods[i - 1].2 != periods[i].2,
                "period {i}: the mission lines alternate, except after the captor's silent-0x2E roll [M 0x14a8b8 Rand(6) + D 0x2E.sfx=0 + M 0x144340:10-24]; periods {periods:?}");
            assert!(periods[i].0 >= periods[i - 1].1, "period {i} starts only after the previous ended [M 0x144340 busy tests]");
        }
        let _ = (sim, k);
    }
}

/// bathhouse-conversation-loop (2026-10-22): the conversation LOOPS. The game's monitor has no
/// terminal step — step 2 sends 0x1A to the captor and steps back to **1** [M 0x144340:42-64],
/// step 1 re-sends to the hostage once his own voice is done [M :10-29], and the 0x1A handler
/// re-rolls a fresh random line every turn (+0xC5 4 → 0x29+Rand(6), 5 → 0x2F+Rand(4) [M
/// 0x14a8b8:8-21]) — so the pair alternates lines indefinitely while the killer stays in
/// HostageKiller (its UPDATE runs the monitor every tick before EnemyLookForOpponent [M
/// 0x15c728 case 3]; an alert 0x11/0x12/0x14/0x15/0x16/0x1e or the execution is what ends it).
/// The port's `monitor_talk_to_mission_obj` wraps 2 → 1 the same way (drone.rs), so this pins the
/// LOOP itself on the real path, on BOTH placed pairs — the ch-0x17 bathhouse-floor pair and the
/// ch-0x19 tunnel-pond pair [D placements #447/#465 and #458/#466] — with per-pair period censuses
/// (the same correlation rules as the alternation test above): ≥ 4 full alternations each, strict
/// alternation, no overlap, the killer opens, the step machine is seen cycling through both 1 and
/// 2, and the hostage-directed sends keep recurring past the first exchange. A same-side repeat is
/// accepted only across a provably silent captor turn (his pool's 0x2E carries sfxId 0 [D]; see the
/// in-test comment).
#[test]
fn the_captor_and_hostage_conversations_loop_on_both_pairs() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002"));
    let (k1, h1, k2, h2) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let find = |mode: u8, x: f32, z: f32| {
            sim.drones.list.iter().position(|d| d.mode == mode && (d.pos[0] - x).abs() < 0.5 && (d.pos[2] - z).abs() < 0.5)
                .unwrap_or_else(|| panic!("drone mode {mode} at ({x},{z}) [D placements]"))
        };
        (find(4, -45.6, 10.9), find(5, -45.5, 12.2), find(4, -11.2, -37.8), find(5, -12.0, -39.1))
    };
    // Both channels bring the pairs in [D placements: param 3 = switch channel].
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.channels.set(0x17, 1, sim.drones.frame); }
    w.wait(1);
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.channels.set(0x19, 1, sim.drones.frame); }
    let (vk, vh) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (mission_set(sim, 0x29, 0x2e), mission_set(sim, 0x2f, 0x32))
    };
    struct Pair {
        ki: usize, hi: usize, paired: bool, hostage_sends: u32, overlap: Option<u32>,
        /// (start, end, speaker, silent_gap) — silent_gap: between this and the PREVIOUS period the
        /// other side's turn produced no voice at all (the game's silent-0x2E roll, see below).
        periods: Vec<(u32, u32, usize, bool)>,
        open: Option<(usize, u32, u32)>, expect: Option<(usize, u32)>,
        last_handles: [Option<u32>; 2], steps_seen: std::collections::BTreeSet<u16>, k_prev_step: u16,
        /// The silent_gap computed when the CURRENT open period started — attached to its record
        /// when it closes (records are appended at close time).
        open_gap: bool,
        /// Gap tracking: the drone whose period just closed, whether the OTHER drone showed a voice
        /// in the gap, and the killer's step values seen in the gap.
        gap_after: Option<usize>, gap_killer_voice: bool, gap_steps: std::collections::BTreeSet<u16>,
    }
    impl Pair {
        fn step(&mut self, sim: &nf_view::scene::Sim, t: u32, vk: &std::collections::HashSet<u32>, vh: &std::collections::HashSet<u32>) {
            let (k, h) = (&sim.drones.list[self.ki], &sim.drones.list[self.hi]);
            if !self.paired {
                self.paired = k.hostage_target == Some(self.hi) && k.state == nf_game::drone::state::HOSTAGE_KILLER;
                if !self.paired { return; }
            }
            self.steps_seen.insert(k.talk_step);
            if self.open.is_none() {
                if let Some(d) = self.gap_after { if d != self.ki { self.gap_killer_voice |= k.voice.is_some(); } }
                self.gap_steps.insert(k.talk_step);
            }
            // Step 1's send to the hostage, observed as the step machine's 1 → 2 edge (the port
            // delivers it the same tick — the game's delay-0 dispatch [M 0x172128] — so there is
            // no outbox window to catch).
            let sent = self.k_prev_step == 1 && k.talk_step == 2;
            self.k_prev_step = k.talk_step;
            if sent { self.hostage_sends += 1; self.expect = Some((self.hi, t + 4)); }
            if let Some((d, _, _)) = self.open { if d == self.hi && h.voice.is_none() { self.expect = Some((self.ki, t + 3)); } }
            for (di, voice) in [(self.ki, k.voice), (self.hi, h.voice)] {
                if voice.is_none() && self.open.map_or(false, |(d, _, _)| d == di) {
                    if let Some((d, st, _)) = self.open.take() {
                        self.periods.push((st, t, d, std::mem::take(&mut self.open_gap)));
                        self.gap_after = Some(d);
                        self.gap_killer_voice = false;
                        self.gap_steps.clear();
                    }
                }
            }
            for (di, voice, set, last) in [(self.ki, k.voice, vk, 0u8), (self.hi, h.voice, vh, 1u8)] {
                if let Some(v) = voice {
                    if set.contains(&v.1) && self.last_handles[last as usize] != Some(v.0) {
                        let opener = di == self.ki && k.talk_step != 0 && self.periods.is_empty();
                        let expected = matches!(self.expect, Some((d, dl)) if d == di && t <= dl);
                        if self.open.is_some() {
                            if opener || expected { self.overlap = Some(t); break; }
                        } else if opener || expected {
                            let silent_gap = self.gap_after == Some(di) && !self.gap_killer_voice;
                            self.open = Some((di, t, v.0));
                            self.expect = None;
                            self.open_gap = silent_gap;
                            self.gap_after = None;
                        }
                    }
                    self.last_handles[last as usize] = Some(v.0);
                }
            }
            if let Some((_, dl)) = self.expect { if t > dl { self.expect = None; } }
        }
        fn done(&self) -> bool { self.periods.len() >= 8 }
    }
    let mut p1 = Pair { ki: k1, hi: h1, paired: false, hostage_sends: 0, overlap: None, periods: Vec::new(), open: None, expect: None, last_handles: [None; 2], steps_seen: Default::default(), k_prev_step: 0, open_gap: false, gap_after: None, gap_killer_voice: false, gap_steps: Default::default() };
    let mut p2 = Pair { ki: k2, hi: h2, paired: false, hostage_sends: 0, overlap: None, periods: Vec::new(), open: None, expect: None, last_handles: [None; 2], steps_seen: Default::default(), k_prev_step: 0, open_gap: false, gap_after: None, gap_killer_voice: false, gap_steps: Default::default() };
    for t in 0..6000u32 {
        w.wait(1);
        audio_step(&mut w.scene);
        let sim = w.scene.sim.as_ref().unwrap();
        p1.step(sim, t, &vk, &vh);
        p2.step(sim, t, &vk, &vh);
        if std::env::var_os("NF_DIALOGUE_TRACE").is_some() && t < 200 {
            let (k, h) = (&sim.drones.list[p2.ki], &sim.drones.list[p2.hi]);
            eprintln!("t {t:3} step {} kv {:?} hv {:?} ktalks {} out {} p1n {} p2n {}",
                k.talk_step, k.voice.map(|v| v.0), h.voice.map(|v| v.0), k.talks.len(),
                k.msg_out.iter().filter(|(tt, m, _)| *m == nf_game::drone::msg::MISSION_TALK && *tt == p2.hi).count(),
                p1.periods.len(), p2.periods.len());
        }
        if p1.done() && p2.done() { break; }
    }
    for (p, name, x, z) in [(&p1, "the ch-0x17 pair", -45.6, 10.9), (&p2, "the ch-0x19 pair", -11.2, -37.8)] {
        let sim = w.scene.sim.as_ref().unwrap();
        let k = &sim.drones.list[p.ki];
        assert!(p.paired, "{name}: the killer pairs his geisha and stays HostageKiller [M 0x144030; 0x15c728:21-26]");
        assert!(p.overlap.is_none(), "{name}: mission voices overlapped [M 0x144340 busy tests]");
        assert!(p.hostage_sends >= 4, "{name}: the loop keeps sending 0x1A to the hostage past the first exchange; got {}",
            p.hostage_sends);
        assert!(p.periods.len() >= 8, "{name}: the conversation loops — 4 full alternations expected, got {} periods {periods:?}",
            p.periods.len(), periods = p.periods);
        assert_eq!(p.periods[0].2, p.ki, "{name}: the opener is the captor's own line [M 0x144340:34-40]");
        for i in 1..p.periods.len() {
            // Strict alternation, with one game-proven exception: a 1-in-6 killer turn rolls the
            // pool's silent line 0x2E (sfxId 0 [D Drone_SfxInfo]; SpeechSFX no-ops an sfx-0 entry
            // [M 0x141b00]), the monitor's step-1 +0x4D0==0 test then advances at once [M
            // 0x144340:10-24] and the HOSTAGE speaks again. Such a repeat is accepted only when the
            // gap shows the killer's full 2→1→2 wrap with NO killer voice at any census tick (the
            // dialogue lines' repeat delays are all 0 [D], so no other refusal exists; the
            // hostage's own pool has no silent line [D 0x2F..0x32.sfx ≠ 0], so the killer can never
            // legitimately repeat).
            let repeat_ok = p.periods[i].3 && p.periods[i].2 == p.hi;
            assert!(repeat_ok || p.periods[i - 1].2 != p.periods[i].2,
                "{name} period {i}: the lines alternate, except after the captor's silent-0x2E roll [M 0x14a8b8 Rand(6) + D 0x2E.sfx=0 + M 0x144340 step-1 advance]; periods {periods:?}",
                periods = p.periods);
            assert!(p.periods[i].0 >= p.periods[i - 1].1, "{name} period {i} starts only after the previous ended [M 0x144340 busy tests]");
        }
        assert!(p.steps_seen.contains(&1) && p.steps_seen.contains(&2),
            "{name}: the step machine wrapped 2 → 1 (the loop itself [M 0x144340:42-64]); seen {steps:?}",
            steps = p.steps_seen);
        let _ = (name, x, z, k, sim);
    }
}

/// Sister: with no eligible hostage the pairing fail arm leaves HostageKiller for Idle 4 before
/// any UPDATE [M 0x15c728:27-29], so the monitor never runs and no 0x1A is ever sent (the game's
/// +0x2B0==0 broadcast [M 0x13a678:14-17] is unreachable — SetupHostageKiller's success gates the
/// state [M :21-29]). Poking the geisha's mode byte (+0xC5 [M]) off 5 makes the search find
/// nobody [M 0x144030:22-55].
#[test]
fn a_captor_with_no_hostage_link_sends_nothing() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002"));
    let ki = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().position(|d| d.mode == 4 && (d.pos[0] + 45.6).abs() < 0.5 && (d.pos[2] - 10.9).abs() < 0.5).expect("the 0x17 killer [D]")
    };
    { let sim = w.scene.sim.as_mut().unwrap(); sim.drones.channels.set(0x17, 1, sim.drones.frame); }
    w.wait(1);
    {
        // She is in state HOSTAGE by now; the mode byte (+0xC5 [M]) off 5 makes the killer's
        // search (next tick — the spawn/enable tick is skipped by the walk) find nobody
        // [M 0x144030:22-55] without disturbing her own state machine.
        let sim = w.scene.sim.as_mut().unwrap();
        let hi = sim.drones.list.iter().position(|d| d.mode == 5 && (d.pos[0] + 45.5).abs() < 0.5 && (d.pos[2] - 12.2).abs() < 0.5).expect("geisha 1 [D]");
        sim.drones.list[hi].mode = 0;
    }
    w.wait(300);
    let sim = w.scene.sim.as_ref().unwrap();
    let k = &sim.drones.list[ki];
    // He passes through the fail arm's Idle 4 [M 0x15c728:27-29]; IDLE's own follow-ups may then
    // move him on (the port's idle wander), so the LOG is the honest witness.
    assert!(k.log.iter().any(|(_, s)| *s == state::IDLE), "no hostage → the fail arm's Idle 4 [M 0x15c728:27-29]; log {:?}",
        k.log.iter().map(|(_, s)| *s).collect::<Vec<_>>());
    assert!(k.hostage_target.is_none(), "no link was ever bound [M 0x144030]");
    assert_eq!(k.talk_step, 0, "the monitor never engaged (no link → no UPDATE runs it [M 0x15c728:21-34])");
    assert!(sim.drones.delayed_msgs.iter().all(|(_, m, _)| *m != nf_game::drone::msg::MISSION_TALK),
        "no 0x1A was queued anywhere");
    assert!(k.msg_now.is_empty() && k.msg_out.is_empty(), "no 0x1A was ever sent (the monitor never engaged)");
    let (vk, vh) = (mission_set(sim, 0x29, 0x2e), mission_set(sim, 0x2f, 0x32));
    let hi = sim.drones.list.iter().position(|d| (d.pos[0] + 45.5).abs() < 0.5 && (d.pos[2] - 12.2).abs() < 0.5).expect("geisha 1 by position [D]");
    assert!(!sim.drones.list[hi].voice.map_or(false, |v| vk.contains(&v.1) || vh.contains(&v.1)),
        "the unpaired hostage never speaks a mission line");
    assert!(!k.voice.map_or(false, |v| vk.contains(&v.1) || vh.contains(&v.1)),
        "the failed captor never speaks a mission line");
}
