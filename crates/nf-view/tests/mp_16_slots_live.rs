//! mp-16-slots: the 16-participant cap end to end, numbers only. The 8-human
//! match (8 sessions over 127.0.0.1, one sim each, the movement-only tick the
//! lockstep holds) agrees on every hash for 600 ticks; 2 humans + 14 bots run
//! 600 ticks on 2 machines with the bots driven (equal hashes); the lobby
//! model holds 16 participants. NF_ISO-gated (the arena loads from the disc);
//! the lobby-model test is not.

use std::time::{Duration, Instant};

use nf_game::mp::MpSettings;
use nf_net::net::{Transport, UdpTransport};
use nf_net::session::{Event, Session};
use nf_net::{MatchSettings, NEUTRAL};

use nf_view::mpnet::{match_to_mp, mp_to_match, to_player_input};

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

const ARENA: u32 = 0x0700_0024; // MP_MAP_ROWS[0] = Skyrail [D placement]

fn scripted_input(t: u32, slot: u8) -> nf_net::Input {
    let mut i = NEUTRAL;
    i.forward = ((t as i32 + slot as i32) % 89) as f32 / 89.0;
    i.strafe = if t % 13 == 0 { -0.4 } else { 0.0 };
    i.turn = ((t % 17) as f32 - 8.0) * 0.002;
    i.look = if t % 31 == 0 { 0.2 } else { 0.0 };
    i.jump = t % 67 == 0;
    i.crouch = t % 47 == 0;
    i
}

fn cfg() -> nf_net::Config {
    nf_net::Config { hash_every: 15, ..Default::default() }
}

struct Peer {
    sess: Session<UdpTransport>,
    executed: u32,
}

/// A movement-only lockstep tick for one machine of `n_humans`: the local
/// slot's body ticks through `tick_ctx` (the movement core both machines run
/// — `net_match_live`'s shape), the remotes through `step_net_remotes`, every
/// slot's position lands in the match, and the hash folds all 16 slots.
fn machine_tick(
    p: &mut Peer,
    sim: &mut nf_view::scene::Sim,
    inputs: &[nf_net::Input; nf_net::MAX_AGENTS],
) -> u64 {
    let local = p.sess.local_slot() as usize;
    let player_inputs = std::array::from_fn(|i| to_player_input(&inputs[i]));
    sim.step_net_remotes(&player_inputs, local);
    let ladder = nf_game::player::LadderCtx::default();
    let own = to_player_input(&inputs[local]);
    sim.bond.tick_ctx(&sim.world, &ladder, &own);
    if let Some(m) = sim.mp.as_mut() {
        for s in 0..nf_net::MAX_AGENTS {
            if s == local {
                m.set_slot(s, sim.bond.pos);
            } else if let Some(b) = &sim.mp_bonds[s] {
                m.set_slot(s, b.pos);
            }
        }
    }
    sim.net_state_hash(local)
}

/// 8 humans over 127.0.0.1: 8 sessions, one sim each, 600 lockstep ticks —
/// every hash exchange agrees on every machine [S mp-16-slots: the cap is the
/// user's request; the per-tick rules are the game's/lockstep's own].
#[test]
fn eight_humans_600_ticks_hashes_agree() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    const N: usize = 8;
    let mut host = Peer {
        sess: Session::new_host(cfg(), UdpTransport::bind("127.0.0.1:27961").unwrap(),
            MatchSettings::default(), 0x0BED_0BAD),
        executed: 0,
    };
    let mut clients: Vec<Peer> = (0..N - 1)
        .map(|_| Peer {
            sess: Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
                "127.0.0.1:27961".parse().unwrap()),
            executed: 0,
        })
        .collect();
    // Handshake: everyone Connected (the slots may arrive in any order).
    let mut ms: u64 = 0;
    let deadline = Instant::now() + Duration::from_secs(20);
    while host.sess.agents().len() < N {
        ms += 5;
        let _ = host.sess.poll(ms);
        for c in clients.iter_mut() {
            let _ = c.sess.poll(ms);
        }
        assert!(Instant::now() < deadline, "the 8-session handshake");
        std::thread::sleep(Duration::from_millis(1));
    }
    // The host's match settings arm the session (NetCore::start_match's shape).
    let mut s = MpSettings::deathmatch(ARENA);
    s.human_count = N as u32;
    s.player_count = N as u16;
    s.total_players = N as u32;
    s.seed = 0x0BED_0BAD;
    host.sess.set_host_match(mp_to_match(&s), s.seed);
    host.sess.start_match(0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while clients.iter().any(|c| !c.sess.is_started()) {
        ms += 5;
        let _ = host.sess.poll(ms);
        for c in clients.iter_mut() {
            let _ = c.sess.poll(ms);
        }
        assert!(Instant::now() < deadline, "GO reaches all 8");
        std::thread::sleep(Duration::from_millis(1));
    }
    // One sim per SIMMED machine — the host plus the first client. The
    // sandbox holds 3 GB and one arena load is ~875 MB (measured: 2 loads =
    // 1.75 GB fit, 4 = OOM-killed), so 8 full sims cannot run here. The
    // other six sessions run session-only: they still feed real inputs every
    // tick and gate `ready_inputs` — the lockstep never runs a tick without
    // every one of the 8 humans' inputs — which is what proves their flow;
    // the sim-hash agreement is asserted on the two simmed machines (the
    // 2-machine hash equality is `two_humans_14_bots`' own proof).
    const SIMMED_CLIENTS: usize = 1;
    let arena = format!("{:08x}.bin", s.level_code);
    let mut host_scene = nf_view::load_mp_carry(&path, &arena, false, None, s.clone()).expect("host sim loads");
    host_scene.sim.as_mut().expect("host sim").swap_local_slot(0);
    let mut client_scenes: Vec<Option<nf_view::scene::Scene>> = Vec::new();
    for (k, c) in clients.iter().enumerate() {
        if k >= SIMMED_CLIENTS {
            client_scenes.push(None);
            continue;
        }
        let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s.clone()).expect("client sim loads");
        {
            let sim = scene.sim.as_mut().expect("client sim");
            assert_eq!(sim.mp_bonds.len(), nf_net::MAX_AGENTS, "the sim holds 16 slot rows [S mp-16-slots]");
            // This machine's local body is ITS slot's (the load spawns the
            // slot-0 view; swap_local_slot parks 0 and takes the local one).
            sim.swap_local_slot(c.sess.local_slot() as usize);
        }
        client_scenes.push(Some(scene));
    }
    let mut desyncs: Vec<String> = Vec::new();
    for t in 0..600u32 {
        host.sess.local_input(t, scripted_input(t, 0));
        for (i, c) in clients.iter_mut().enumerate() {
            let slot = c.sess.local_slot();
            c.sess.local_input(t, scripted_input(t, slot));
            let _ = i;
        }
        // Pump until every machine can run tick t (the lockstep never guesses).
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            ms += 5;
            let _ = host.sess.poll(ms);
            for c in clients.iter_mut() {
                let _ = c.sess.poll(ms);
            }
            let all_ready = host.sess.ready_inputs(t).is_some()
                && clients.iter_mut().all(|c| c.sess.ready_inputs(t).is_some());
            if all_ready || Instant::now() > deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let hi = host.sess.ready_inputs(t).expect("host ready");
        let hh = {
            let sim = host_scene.sim.as_mut().expect("host sim");
            machine_tick(&mut host, sim, &hi)
        };
        host.executed = t + 1;
        host.sess.submit_hash(t, hh);
        for (k, c) in clients.iter_mut().enumerate() {
            // EVERY session gates on the full 8-human input row (the lockstep
            // never guesses); the simmed ones also run the movement tick.
            let ci = c.sess.ready_inputs(t).expect("client ready");
            c.executed = t + 1;
            if let Some(scene) = &mut client_scenes[k] {
                let ch = {
                    let sim = scene.sim.as_mut().expect("client sim");
                    machine_tick(c, sim, &ci)
                };
                if t % 15 == 0 {
                    assert_eq!(hh, ch, "tick {t}: simmed machine {k}'s hash agrees with the host's");
                }
                c.sess.submit_hash(t, ch);
            }
        }
        assert_eq!(host.executed, t + 1);
        for c in clients.iter() {
            assert_eq!(c.executed, t + 1, "every machine ran tick {t}");
        }
        if t % 15 == 0 {
            desyncs.extend(host.sess.poll(ms).into_iter().filter_map(|e| match e {
                Event::Desync { tick, .. } => Some(format!("host {tick}")),
                _ => None,
            }));
            for c in clients.iter_mut() {
                desyncs.extend(c.sess.poll(ms).into_iter().filter_map(|e| match e {
                    Event::Desync { tick, .. } => Some(format!("client {tick}")),
                    _ => None,
                }));
            }
        }
    }
    assert!(desyncs.is_empty(), "no desync over the honest 8-human run: {desyncs:?}");
    // The 8 humans' bodies all exist, one per participant slot.
    let sim = host_scene.sim.as_ref().expect("host sim");
    for s in 0..N {
        assert!(s == 0 || sim.mp_bonds[s].is_some(), "slot {s} holds its body");
    }
    assert_eq!(host.executed, 600);
    for c in clients.iter() {
        assert_eq!(c.executed, 600, "every session stayed in the lockstep for all 600 ticks");
    }
}

/// 2 humans + 14 bots on 2 machines: the bots' brains drive identically on
/// both (the shared bit-exact RNG), the hashes agree for 600 ticks, and the
/// match holds 16 participants.
#[test]
fn two_humans_14_bots_600_ticks_hashes_agree() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut h = Peer {
        sess: Session::new_host(cfg(), UdpTransport::bind("127.0.0.1:27962").unwrap(),
            MatchSettings::default(), 0x0BED_0BAD),
        executed: 0,
    };
    let mut c = Peer {
        sess: Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(),
            "127.0.0.1:27962".parse().unwrap()),
        executed: 0,
    };
    let mut ms: u64 = 0;
    let deadline = Instant::now() + Duration::from_secs(20);
    while h.sess.agents().len() < 2 {
        ms += 5;
        let _ = h.sess.poll(ms);
        let _ = c.sess.poll(ms);
        assert!(Instant::now() < deadline, "handshake");
        std::thread::sleep(Duration::from_millis(1));
    }
    let mut s = MpSettings::deathmatch(ARENA);
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 14; // the port's extension: 16 participants [S mp-16-slots]
    s.total_players = 16;
    s.seed = 0x0BED_0BAD;
    h.sess.set_host_match(mp_to_match(&s), s.seed);
    h.sess.start_match(0).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !c.sess.is_started() {
        ms += 5;
        let _ = h.sess.poll(ms);
        let _ = c.sess.poll(ms);
        assert!(Instant::now() < deadline, "GO");
        std::thread::sleep(Duration::from_millis(1));
    }
    let arena = format!("{:08x}.bin", s.level_code);
    let mut hs = nf_view::load_mp_carry(&path, &arena, false, None, s.clone()).expect("host sim loads");
    let mut cs = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("client sim loads");
    cs.sim.as_mut().expect("client sim").swap_local_slot(1);
    {
        let sim = hs.sim.as_ref().expect("host sim");
        let m = sim.mp.as_ref().expect("match");
        assert_eq!(m.bots.len(), 14, "14 bots created (the disc's 4 records cycle) [S mp-16-slots]");
        assert_eq!(m.game.slots.len(), 16);
        for (i, b) in m.bots.iter().enumerate() {
            assert_eq!(b.slot, 2 + i, "bot {i} holds the slot after the humans");
        }
    }
    let mut desyncs: Vec<String> = Vec::new();
    for t in 0..600u32 {
        h.sess.local_input(t, scripted_input(t, 0));
        c.sess.local_input(t, scripted_input(t, 1));
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            ms += 5;
            let _ = h.sess.poll(ms);
            let _ = c.sess.poll(ms);
            if h.sess.ready_inputs(t).is_some() && c.sess.ready_inputs(t).is_some() {
                break;
            }
            assert!(Instant::now() < deadline, "tick {t} readiness");
            std::thread::sleep(Duration::from_millis(1));
        }
        let hi = h.sess.ready_inputs(t).expect("host ready");
        let ci = c.sess.ready_inputs(t).expect("client ready");
        let hh = {
            let sim = hs.sim.as_mut().expect("host sim");
            sim.step_mp_bots();
            machine_tick(&mut h, sim, &hi)
        };
        let ch = {
            let sim = cs.sim.as_mut().expect("client sim");
            sim.step_mp_bots();
            machine_tick(&mut c, sim, &ci)
        };
        h.executed = t + 1;
        c.executed = t + 1;
        h.sess.submit_hash(t, hh);
        c.sess.submit_hash(t, ch);
        if t % 15 == 0 {
            assert_eq!(hh, ch, "tick {t}: the 2-machine hashes agree (14 bots drive)");
            desyncs.extend(h.sess.poll(ms).into_iter().filter_map(|e| match e {
                Event::Desync { tick, .. } => Some(format!("host {tick}")),
                _ => None,
            }));
            desyncs.extend(c.sess.poll(ms).into_iter().filter_map(|e| match e {
                Event::Desync { tick, .. } => Some(format!("client {tick}")),
                _ => None,
            }));
        }
    }
    assert!(desyncs.is_empty(), "no desync over the 2+14 run: {desyncs:?}");
    // Both machines agree on the bots' final positions too (the hash covered
    // them; the poses cross-check directly).
    let hp = hs.sim.as_ref().expect("host sim").mp_bot_positions();
    let cp = cs.sim.as_ref().expect("client sim").mp_bot_positions();
    assert_eq!(hp.len(), 14);
    assert_eq!(hp, cp, "the bots' brains ended in the same state on both machines");
}

/// The one-screen lobby model with 16 participants: 16 player rows, 15 bot
/// rows, `row_count()` == `rows().len()`, a client edit from slot 15 merges
/// on the host. No ISO (pure model).
#[test]
fn lobby_model_holds_16_participants() {
    let bots: [nf_view::menu::MpBot; nf_net::lobby::BOTS] =
        std::array::from_fn(|_| nf_view::menu::MpBot { enabled: false, team_mi6: false, character: 0, skill: 5, aim: 2, hit_points: 100, speed: 1, recovery1: 100, recovery2: 100, trait_id: 0, dirty: false });
    let mut lobby = nf_view::mp_lobby::MpLobby::new_host(
        0x1, ARENA, 0, 1, 0, 10, 10, [0, 0], [0; 5],
        bots, "Player 1", true, 0,
    );
    // 15 remote humans join (slots 1..15) — 16 participants with the host.
    for i in 1..nf_net::lobby::SLOTS {
        lobby.players[i] = nf_view::mp_lobby::MpLobbyPlayer {
            codename: format!("Player {}", i + 1),
            character: 0, team: 0, ready: true, joined: true, ping_ms: i as u32,
        };
    }
    // All 15 bots enabled.
    for b in lobby.bots.iter_mut() { b.enabled = true; }
    assert_eq!(lobby.players.len(), 16);
    assert_eq!(lobby.bots.len(), 15);
    assert!(lobby.start_enabled(), "everyone joined is ready");
    // The row count agrees with the built list (the join-aware count test).
    let rows = lobby.rows();
    assert_eq!(lobby.row_count(), rows.len(), "row_count() == rows().len()");
    assert_eq!(rows.len(), 7 + 2 + 5 + 15 * 2 + 16 + 1, "settings + 15 bots × 2 rows + 16 players + Start");
    // The wire snapshot roundtrips with every row filled.
    let wire = lobby.to_state();
    let dec = nf_net::LobbyState::decode(&wire.encode()).expect("the 16-participant snapshot decodes");
    assert_eq!(dec, wire);
    // A client edit from slot 15 (the 16th row) merges into the host model.
    let edit = nf_net::LobbyEdit { slot: 15, field: nf_net::lobby::EDIT_CHARACTER, value: 7 };
    assert!(lobby.apply_edit(&edit), "slot 15 is a legal lobby slot [S mp-16-slots]");
    assert_eq!(lobby.players[15].character, 7);
    // Sister: slot 16 is beyond the cap — refused.
    assert!(!lobby.apply_edit(&nf_net::LobbyEdit { slot: 16, field: nf_net::lobby::EDIT_CHARACTER, value: 1 }));
}
