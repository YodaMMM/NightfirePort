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
/// Observation: the step-1 sends ride the port's message queues (visible); the step-2/step-0
/// self-sends ride `self_msg` (same-tick, not queued) — so the hostage-directed sends recur and
/// the alternation is pinned by the mission voice PERIODS, correlated to the sends.
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
    let mut periods: Vec<(u32, u32, usize)> = Vec::new(); // (start, end, drone)
    let mut open: Option<(usize, u32, u32)> = None;       // (drone, start, handle)
    let mut expect: Option<(usize, u32)> = None;          // (drone, deadline)
    let mut overlap: Option<u32> = None;
    let mut paired = false;
    let mut last_handles: [Option<u32>; 2] = [None; 2];                     // last voice handle seen per drone
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
        // Step 1's send to the hostage: pushed during the killer's UPDATE, it sits in his outbox
        // until the next tick's drain — one post-tick sighting per send.
        let in_outbox = k.msg_out.iter().any(|(tt, m, _)| *m == nf_game::drone::msg::MISSION_TALK && *tt == hi);
        if in_outbox { hostage_sends += 1; expect = Some((hi, t + 4)); }
        // Expect the killer's reply once the hostage's mission period closes [M 0x144340:58-73].
        if let Some((d, st, _)) = open { if d == hi && h.voice.is_none() { expect = Some((ki, t + 3)); let _ = st; } }
        // Close the open period when its voice clears (both drones first, then opens — the
        // killer's reply lands the same tick the hostage's finished voice is cleared).
        for (di, voice) in [(ki, k.voice), (hi, h.voice)] {
            if voice.is_none() && open.map_or(false, |(d, _, _)| d == di) {
                if let Some((d, st, _)) = open.take() { periods.push((st, t, d)); }
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
                            open = Some((di, t, v.0));
                            expect = None;
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
        assert_eq!(periods[0].2, ki, "the opener is the captor's own line [M 0x144340:19-21]");
        for i in 1..periods.len() {
            assert_ne!(periods[i - 1].2, periods[i].2, "period {i}: the mission lines strictly alternate [M 0x144340 step machine 0/1/2]");
            assert!(periods[i].0 >= periods[i - 1].1, "period {i} starts only after the previous ended [M 0x144340 busy tests]");
        }
        let _ = (sim, k);
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
    let (vk, vh) = (mission_set(sim, 0x29, 0x2e), mission_set(sim, 0x2f, 0x32));
    let hi = sim.drones.list.iter().position(|d| (d.pos[0] + 45.5).abs() < 0.5 && (d.pos[2] - 12.2).abs() < 0.5).expect("geisha 1 by position [D]");
    assert!(!sim.drones.list[hi].voice.map_or(false, |v| vk.contains(&v.1) || vh.contains(&v.1)),
        "the unpaired hostage never speaks a mission line");
    assert!(!k.voice.map_or(false, |v| vk.contains(&v.1) || vh.contains(&v.1)),
        "the failed captor never speaks a mission line");
}
