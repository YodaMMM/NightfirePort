//! net-desync-detail: the 2-machine loopback desync diagnosis. Two app sims on loopback play a
//! scripted lockstep match through the real session layer (the `net_match_live` shape, the live
//! loop's `net_tick`); the desync watcher on each machine keeps the component-hash history (the
//! 7 named components) and, at the exchange cadence, the per-item breakdowns. A test-hook
//! perturbation of (a) one rocket's position, (b) a bot's goal, (c) a door on ONE machine makes
//! the hashes disagree; the machines exchange the components for the mismatched tick
//! (`PT_HASHCOMPS`) and the first diverging component's items (`PT_HASHDETAIL`) and each names
//! the right component and slot, and the report zip holds the input log since the match start.
//! The unperturbed sister match writes no report. NF_ISO-gated (the arenas load from the disc).

use nf_game::mp::MpSettings;
use nf_net::net::UdpTransport;
use nf_net::session::{Event, Session};
use nf_net::{MatchSettings, NEUTRAL};

use nf_view::desync::{Action, Detail, Watcher};
use nf_view::mpnet::to_player_input;
use nf_view::scene::{NET_HASH_COMPONENT_NAMES, NET_HASH_COMPONENTS};

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn scripted_input(t: u32, slot: u8) -> nf_net::Input {
    let mut i = NEUTRAL;
    i.forward = ((t as i32 + slot as i32) % 97) as f32 / 97.0;
    i.strafe = if t % 11 == 0 { -0.5 } else { 0.0 };
    i.turn = ((t % 13) as f32 - 6.0) * 0.002;
    i.look = if t % 29 == 0 { 0.25 } else { 0.0 };
    i.jump = t % 61 == 0;
    i.crouch = t % 43 == 0;
    i
}

fn cfg() -> nf_net::Config {
    nf_net::Config { hash_every: 15, ..Default::default() }
}

struct Peer {
    sess: Session<UdpTransport>,
}
fn host(port: u16) -> Peer {
    Peer { sess: Session::new_host(cfg(), UdpTransport::bind(&format!("127.0.0.1:{port}")).unwrap(), MatchSettings::default(), 0) }
}
fn client(port: u16) -> Peer {
    Peer { sess: Session::new_client(cfg(), UdpTransport::bind("127.0.0.1:0").unwrap(), format!("127.0.0.1:{port}").parse().unwrap()) }
}

fn settings(humans: u16, bots: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = humans as u32;
    s.player_count = humans;
    s.bot_count = bots;
    s.total_players = humans as u32 + bots;
    s.seed = 0x0bed_0bad;
    s.time_limit = 10 * 60; // the live host's 10 minutes (deathmatch()'s raw 10 ends in 10 s)
    s
}

/// The live match start's per-machine setup (`swap_in_scene`'s net block).
fn machine(path: &str, s: &MpSettings, slot: usize) -> nf_view::scene::Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

/// Pump until both sides hold tick `t` (bounded at 5 s). The polls ROUTE their events into the
/// machines' watchers — the exchange packets arrive between ticks (during the wait), exactly
/// like the live app's per-frame poll; dropping them here would drop the desync exchange.
fn pump_till(h: &mut Machine, c: &mut Machine, ms: &mut u64, t: u32) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        *ms += 5;
        h.drain(*ms);
        c.drain(*ms);
        if h.peer.sess.ready_inputs(t).is_some() && c.peer.sess.ready_inputs(t).is_some() {
            return;
        }
        assert!(std::time::Instant::now() < deadline, "tick {t}: both inputs arrive within 5 s");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// One machine in the diagnosis run: the session, the sim, the watcher, the sent-detail flag.
struct Machine {
    peer: Peer,
    scene: nf_view::scene::Scene,
    watch: Watcher,
    local: usize,
    desynced: bool,
}

impl Machine {
    /// Send the watcher's next packet (the driver's `send_desync_action`).
    fn send(&mut self, a: Action) {
        match a {
            Action::SendComps(tick, comps) => self.peer.sess.send_hash_comps(tick, comps),
            Action::SendDetail(tick, component, items) => self.peer.sess.send_hash_detail(tick, component, items),
        }
    }

    /// Poll the session and feed every desync-layer event into the watcher.
    fn drain(&mut self, ms: u64) {
        for ev in self.peer.sess.poll(ms) {
            match ev {
                Event::Desync { tick, .. } => {
                    self.desynced = true;
                    for a in self.watch.on_desync(tick) {
                        self.send(a);
                    }
                }
                Event::HashComps { tick, comps } => {
                    for a in self.watch.on_peer_comps(tick, comps) {
                        self.send(a);
                    }
                }
                Event::HashDetail { tick, component, items } => {
                    for a in self.watch.on_peer_detail(tick, component, items) {
                        self.send(a);
                    }
                }
                _ => {}
            }
        }
    }
}

/// The perturbation the run applies (the test hook: after the tick's step, before its hash).
type Poke = Box<dyn Fn(&mut nf_view::scene::Sim)>;

/// The two-machine run: `ticks` scripted lockstep ticks through `net_tick` (the live loop's
/// shape), the watcher fed every tick + the per-item rows every `hash_every` (15), the input log
/// recorded on both sessions. `arm` runs once on both machines before the poke tick (a shared
/// test-hook state, e.g. a live rocket both machines hold); `poke` runs on the CLIENT only after
/// the poke tick's step, before its hash. Returns both machines once both diagnose (or at the
/// tick limit — the caller asserts).
fn run_pair(path: &str, s: &MpSettings, port: u16, ticks: u32, poke_tick: u32, arm: Option<&dyn Fn(&mut nf_view::scene::Sim)>, poke: Option<&Poke>) -> (Machine, Machine, u32) {
    let mut h = Peer { sess: host(port).sess };
    let mut c = Peer { sess: client(port).sess };
    let mut ms: u64 = 0;
    // Handshake + GO (the net_match_live shape).
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !(h.sess.agents().len() == 2 && c.sess.local_slot() != 0) {
        ms += 5;
        let _ = h.sess.poll(ms);
        let _ = c.sess.poll(ms);
        assert!(std::time::Instant::now() < deadline, "handshake");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    h.sess.set_host_match(nf_view::mpnet::mp_to_match(s), s.seed);
    h.sess.start_match(0).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !c.sess.is_started() {
        ms += 5;
        let _ = h.sess.poll(ms);
        let _ = c.sess.poll(ms);
        assert!(std::time::Instant::now() < deadline, "GO");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let mut hm = Machine { peer: host(0), scene: machine(path, s, 0), watch: Watcher::default(), local: 0, desynced: false };
    hm.peer.sess = h.sess;
    let mut cm = Machine { peer: client(0), scene: machine(path, s, 1), watch: Watcher::default(), local: 1, desynced: false };
    cm.peer.sess = c.sess;

    for t in 0..ticks {
        hm.peer.sess.local_input(t, scripted_input(t, 0));
        cm.peer.sess.local_input(t, scripted_input(t, 1));
        pump_till(&mut hm, &mut cm, &mut ms, t);
        let hi = hm.peer.sess.ready_inputs(t).unwrap();
        let ci = cm.peer.sess.ready_inputs(t).unwrap();
        // The live loop's tick on each machine (the full local path; every machine reaches the
        // same component hashes).
        let comps_h = nf_view::net_tick_comps(hm.scene.sim.as_mut().unwrap(), &hi, 0);
        let comps_c = nf_view::net_tick_comps(cm.scene.sim.as_mut().unwrap(), &ci, 1);
        // The shared test-hook state (both machines, identical values).
        if t == 5 {
            if let Some(f) = arm {
                f(hm.scene.sim.as_mut().unwrap());
                f(cm.scene.sim.as_mut().unwrap());
            }
        }
        // The perturbation: after the tick's step, before its hash (the live machine's bug).
        let poked = t == poke_tick && poke.is_some();
        if poked {
            poke.unwrap()(cm.scene.sim.as_mut().unwrap());
        }
        // The watcher feed (the app's per-tick shape: components + the fold, the detail rows at
        // the exchange cadence) — recomputed after the poke so the hash sees it.
        let (ch, cc) = if poked {
            (hm.scene.sim.as_ref().unwrap().net_component_hashes(0), cm.scene.sim.as_ref().unwrap().net_component_hashes(1))
        } else {
            (comps_h, comps_c)
        };
        if t < poke_tick {
            assert_eq!(ch, cc, "tick {t}: the honest machines agree (the perturbation is the only source)");
        }
        hm.watch.on_tick(t, ch);
        cm.watch.on_tick(t, cc);
        hm.peer.sess.record_run_tick(t, hi);
        cm.peer.sess.record_run_tick(t, ci);
        // The combined hash every tick (the session gates the wire sends on its cadence).
        hm.peer.sess.submit_hash(t, nf_view::scene::fold_component_hashes(&ch));
        cm.peer.sess.submit_hash(t, nf_view::scene::fold_component_hashes(&cc));
        if t % 15 == 0 {
            let ih: Detail = hm.scene.sim.as_ref().unwrap().net_component_items(0);
            let ic: Detail = cm.scene.sim.as_ref().unwrap().net_component_items(1);
            hm.watch.on_detail_tick(t, ih);
            cm.watch.on_detail_tick(t, ic);
        }
        hm.drain(ms);
        cm.drain(ms);
        if hm.watch.diagnosis.is_some() && cm.watch.diagnosis.is_some() {
            return (hm, cm, t);
        }
    }
    (hm, cm, ticks)
}

/// The report zip from a diagnosed machine (the app's `finish_desync_report`, minus the save +
/// open: the tests assert on the contents).
fn report_files(m: &Machine, s: &MpSettings) -> Vec<(String, Vec<u8>)> {
    let sim = m.scene.sim.as_ref().unwrap();
    let d = &sim.drones;
    let channels = d.triggers.len() + d.doors.list.len() * 2 + d.props.destroy.len() + d.pickups.len() + d.sss.list.len() + sim.hurts.len();
    let counts = [d.pickups.len(), d.doors.list.len(), d.triggers.len(), channels, d.props.destroy.len(), d.breaks.list.len(), d.sss.list.len(), sim.hurts.len()];
    let payload = nf_view::mpnet::encode_match_start(&nf_view::mpnet::mp_to_match(s), s.seed);
    let (base, log) = m.peer.sess.run_log();
    let diag = m.watch.diagnosis.as_ref().unwrap();
    nf_view::desync::desync_report_files(diag, sim.level, &payload, base, log, counts, &m.watch.hist.to_text())
}

/// The unperturbed sister: an honest 2-machine match never desyncs, never diagnoses, and no
/// report is built.
#[test]
fn unperturbed_match_writes_no_report() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings(2, 0);
    let (hm, cm, last) = run_pair(&path, &s, 27961, 45, u32::MAX, None, None);
    assert_eq!(last, 45, "the run went the distance with no diagnosis");
    assert!(!hm.desynced && !cm.desynced, "no desync over the honest run");
    assert!(hm.watch.diagnosis.is_none() && cm.watch.diagnosis.is_none(), "no diagnosis — no report");
    // The watcher kept the history (the report's raw material) on both machines.
    assert!(hm.watch.hist.comps_at(30).is_some() && cm.watch.hist.comps_at(30).is_some(), "the component history covers the match");
}

/// (a) A rocket's position poked on one machine: the diagnosis names `combat` and the rocket's
/// item (the rounds-count item, then one item per round), and the report holds the input log.
#[test]
fn rocket_perturbation_names_combat() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings(2, 0);
    // A live rocket both machines hold from tick 5 (a slow harmless round — the shared state).
    let arm: &dyn Fn(&mut nf_view::scene::Sim) = &|sim: &mut nf_view::scene::Sim| {
        sim.mp_combat.rounds.push(nf_game::mp::combat::Round {
            owner: 0, weapon: 0x2d, pos: [1.0, 2.0, 3.0], dir: [0.0, 0.0, 1.0],
            speed: 0.0, full_speed: 0.0, ramp: false, travelled: 0.0, range: 1.0e9,
            damage: 0.0, first: false, gravity: false, homing: false, guided: false,
            target: None, contact_blast: false, radius: 0.0, dead: false, born: 0,
        });
    };
    let poke: Poke = Box::new(|sim: &mut nf_view::scene::Sim| {
        sim.mp_combat.rounds[0].pos[0] += 4.0; // one rocket's position, one machine only
    });
    let (hm, cm, _) = run_pair(&path, &s, 27962, 60, 30, Some(arm), Some(&poke));
    for (m, name) in [(&hm, "host"), (&cm, "client")] {
        let d = m.watch.diagnosis.as_ref().unwrap_or_else(|| panic!("{name} diagnosed"));
        assert_eq!(d.component_name(), "combat", "{name}: the combat component is the first diverging one");
        assert_eq!(d.item, Some(1), "{name}: item 0 is the rounds count, item 1 is round 0 (the rocket)");
        assert_eq!(d.tick, 30, "{name}: the desync tick is the poke tick (the first exchange after)");
    }
    // The report zip holds the input log since the match start + the settings + the history.
    let files = report_files(&cm, &s);
    let get = |n: &str| files.iter().find(|(f, _)| f == n).map(|(_, b)| b.clone());
    let (base, log) = nf_view::desync::decode_match_log(&get("match-log.bin").expect("match-log.bin in the zip")).expect("decodes");
    assert_eq!(base, 0);
    assert!(log.len() >= 31, "the log covers the match from tick 0 past the desync (got {})", log.len());
    let report = String::from_utf8(get("report.txt").unwrap()).unwrap();
    assert!(report.contains("kind: desync") && report.contains("first_component: combat") && report.contains("first_item: 1"), "{report}");
    assert!(report.contains(&format!("level: {:08x}", s.level_code)) && report.contains("seed"), "{report}");
    let (settings2, seed2) = nf_view::mpnet::decode_match_start(&get("match-settings.bin").unwrap()).expect("the settings payload round-trips");
    assert_eq!(seed2, s.seed);
    assert_eq!(settings2.level_code, s.level_code);
    assert!(get("hash-history.txt").is_some());
}

/// (b) A bot's goal poked on one machine: the diagnosis names `bots` and the bot's slot.
#[test]
fn bot_goal_perturbation_names_bots() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings(2, 1);
    let poke: Poke = Box::new(|sim: &mut nf_view::scene::Sim| {
        sim.mp.as_mut().unwrap().bots[0].brain.vars.goals[1].target ^= 1;
    });
    let (hm, cm, _) = run_pair(&path, &s, 27963, 60, 30, None, Some(&poke));
    for (m, name) in [(&hm, "host"), (&cm, "client")] {
        let d = m.watch.diagnosis.as_ref().unwrap_or_else(|| panic!("{name} diagnosed"));
        assert_eq!(d.component_name(), "bots", "{name}: the bots component is the first diverging one");
        assert_eq!(d.item, Some(0), "{name}: bot 0's brain is the first differing bot item");
        assert_eq!(d.tick, 30, "{name}: the desync tick is the poke tick");
    }
}

/// (c) A door poked on one machine: the diagnosis names `world` and the door's item (after the
/// pickups); `--bug-replay` of the report replays the transcript and names the same component.
#[test]
fn door_perturbation_names_world_and_the_replay_names_it_too() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    // Sub Pen (07000025): the arena whose trigger plates open doors (the world-sync run pins
    // doors_moved > 0 there; 07000023 carries no doors).
    let mut s = settings(2, 0);
    s.level_code = 0x0700_0025;
    let poke: Poke = Box::new(|sim: &mut nf_view::scene::Sim| {
        sim.drones.doors.list[0].pos01 += 0.5; // a door, one machine only
    });
    let (hm, cm, _) = run_pair(&path, &s, 27964, 60, 30, None, Some(&poke));
    let pickups = cm.scene.sim.as_ref().unwrap().drones.pickups.len();
    for (m, name) in [(&hm, "host"), (&cm, "client")] {
        let d = m.watch.diagnosis.as_ref().unwrap_or_else(|| panic!("{name} diagnosed"));
        assert_eq!(d.component_name(), "world", "{name}: the world component is the first diverging one");
        assert_eq!(d.item, Some(pickups), "{name}: door 0's item sits right after the {pickups} pickups");
        assert_eq!(d.tick, 30, "{name}: the desync tick is the poke tick");
    }
    // --bug-replay of the report: the transcript rebuilds the match and names the first
    // tick/component where the honest replay leaves the reporter's recorded hashes.
    let files = report_files(&cm, &s);
    let zip = std::env::temp_dir().join(format!("nf-desync-test-{}.zip", std::process::id()));
    std::fs::write(&zip, nf_view::bugreport::zip_store(&files)).expect("write the report zip");
    let lines = nf_view::bugreport::bug_replay(&path, &zip, None).expect("the replay runs");
    let _ = std::fs::remove_file(&zip);
    let first = lines.iter().find(|l| l.contains("FIRST DIFFERENCE")).expect("the replay names the first difference");
    eprintln!("{lines:?}");
    assert!(first.contains("component 6 (world)"), "{first}");
    assert!(first.contains("tick 30"), "{first}");
    // The diagnosis and the replay agree on the component.
    assert_eq!(cm.watch.diagnosis.as_ref().unwrap().component, NET_HASH_COMPONENTS - 1, "world is the last component");
    assert_eq!(NET_HASH_COMPONENT_NAMES[6], "world");
}
