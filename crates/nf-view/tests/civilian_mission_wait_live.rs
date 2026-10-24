//! civilian-mission-wait: the mission civilian's pause state (`CivilianMissionWait` 0x15dbb0,
//! `NDrone2_StateFuncs[0x17]` [L 0x29b300]).
//!
//! CivilianMission (0x16) case 0x22 parks the walker in 0x17: stand idle at alert < 0.66 (else the
//! look-around 0x21), timer C armed 2 s (+0xCC4 = GameState + FRAME_RATE_INT·2 [M 0x15dbb0:29]) —
//! `NDrone2_PreDroneControl` 0x148db0:97-111 fires msg 0xc when the clock expires and the state
//! re-arms it while Bond stands within 3.0 in the 0.525 rad view cone, else resumes 0x16 [M
//! 0x15dbb0:87-111; BondWithinAngle 0x154f98]. A second 0x22 while waiting re-arms the clock [M
//! 0x15dbb0:136-141]. The decompile holds no literal msg-0x22 sender (every send site checked,
//! 2026-10-16 — see the msg::WAIT_PAUSE doc in drone.rs); the tests drive the id through the real
//! state-machine entry (`Drone::send`), the path a data-driven sender feeds.
//!
//! State and msg ids are written raw (`0x16`, `0x17`, `0x22` …) so this file compiles — and the
//! assertions fail — against the pre-fix master. Opt-in via `NF_ISO`. Level 07000007: the party's
//! 35 dtype-10 guests carry prop 0x1D and six kind-8 mission paths sit under their placements
//! [D 07000007.bin (FileList entry 34, offset 0x8384800, size 0xf55000) main map = LevelBin entry
//! 526 "01000097": 0x1A block payload 0xf2e9a0 holds 35 dtype-0xa placements; 0x05 block payload
//! 0xe23a64 holds 9 paths — boundary 0 + two kind-0x2 + kind-0x8 at AiNetwork 3..8 = Network.paths
//! [2..7]].

use nf_view::scene::{Scene, Sim};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The walker: a dtype-10 guest with no switch channel (starts immediately) whose placement sits on
/// a kind-8 mission path node within the game's 0.3 acceptance [M AssignAIPath 0x154190: fVar11 = 0.3].
/// Cited [D]: the guest is main-map placement 402 (record 0xf382e0, params (3,0) switch /
/// (0xf,0xa) dtype 10), standing 0.017 from Network.paths[2] = AiNetwork path 3 "Party Patrol
/// Path_1" (record 0xe2ebd4) node 0 (record 0xe2ec80, pos at 0xe2ec90) — the only dtype-10
/// switch-0 guest inside the acceptance (next closest: placement 349 at 3.06; the path-4/5 node-0
/// guests, placements 425/424, carry switch 6 and are excluded by the channel gate).
fn find_walker(sim: &Sim) -> usize {
    let data = sim.data();
    let walker = sim.drones.list.iter().position(|dr| {
        dr.present && dr.dtype == 10 && dr.switch_channel == 0
            && data.nav.paths.iter().any(|p| p.kind & 8 != 0 && p.nodes.iter().any(|n| {
                (n.pos[1] - dr.feet()[1]).abs() <= 2.0
                    && (dr.pos[0] - n.pos[0]).powi(2) + (dr.pos[2] - n.pos[2]).powi(2) < 0.09
            }))
    }).expect("the walker is placement 402 on Network.paths[2] (Party Patrol Path_1) node 0 [D 07000007.bin: 0x1A record 0xf382e0, 0x05 path record 0xe2ebd4, node 0 record 0xe2ec80]");
    drop(data);
    walker
}

fn dist(a: &[f32; 3], b: &[f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

/// The `Data` the msg path needs, built from disjoint field borrows so the send can take
/// `&mut sim.drones.rng` alongside it (the explode_live pattern).
fn send_wait(scene: &mut Scene, walker: usize) {
    let sim = scene.sim.as_mut().unwrap();
    let difficulty = sim.drones.difficulty;
    // Disjoint field borrows only (no `sim.drones` in the Data) so `&mut sim.drones.rng` coexists.
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
        ai_points: &sim.ai_points,
    };
    let frame = sim.drones.frame;
    sim.drones.list[walker].send(&data, &mut sim.drones.rng, frame, 0x22);
}

/// The wait request parks a walking mission guest in CivilianMissionWait 0x17: she freezes (the
/// state has no route half), holds the ~2 s timer-C wait with Bond far away, then resumes 0x16 and
/// walks again. Pre-fix the port has no msg-0x22 arm: the message drops through the GLOBAL fallback
/// and the guest walks straight through — the audit's symptom [M 0x15d860 case 0x22; 0x15dbb0].
#[test]
fn a_mission_guest_pauses_on_the_wait_message_and_resumes() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let walker = find_walker(sim);

    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    {
        let dr = &sim.drones.list[walker];
        assert_eq!(dr.state, 0x16, "the guest runs CivilianMission [M 0x15d860] — got {:#x}", dr.state);
        assert!(dr.route.is_some(), "the guest walks an assigned mission route");
    }

    // The wait request — Bond parked far away so the timer expiry takes the resume arm.
    let guest_pos = sim.drones.list[walker].pos;
    let wait_frame = sim.drones.frame;
    send_wait(&mut scene, walker);
    let sim = scene.sim.as_mut().unwrap();
    {
        let dr = &sim.drones.list[walker];
        assert_eq!(dr.state, 0x17, "msg 0x22 parks the walker in CivilianMissionWait [M 0x15d860:130-136] — got {:#x}", dr.state);
        assert!(dr.timer_c.is_some(), "the wait clock is armed [M 0x15dbb0:29-32]");
        let idle = dr.anim.state == 0x24 || dr.anim.call.map_or(false, |c| c.0 == 0x24);
        assert!(idle, "alert {:.2} < 0.66 stands idle 0x24 [M 0x15dbb0:23-25] — anim {:#x}", dr.alert, dr.anim.state);
    }

    // She freezes: the wait state has no route half — 100 ticks of standing, not walking.
    for _ in 0..100 { sim.bond.pos = [guest_pos[0] + 40.0, guest_pos[1], guest_pos[2]]; sim.step_bond(Default::default(), false, false, false); }
    {
        let dr = &sim.drones.list[walker];
        assert_eq!(dr.state, 0x17, "still waiting inside the 2 s clock (timer fires at {}+120)", wait_frame);
        let moved = dist(&dr.pos, &guest_pos);
        assert!(moved < 0.3, "the guest pauses at the wait node (moved {moved:.2} in 100 ticks — the port walked straight through pre-fix)");
    }

    // The clock expires with Bond far: back to CivilianMission and walking again [M 0x15dbb0:87-110].
    let mut resumed = false;
    for _ in 0..60 {
        sim.bond.pos = [guest_pos[0] + 40.0, guest_pos[1], guest_pos[2]];
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.list[walker].state == 0x16 { resumed = true; break; }
    }
    assert!(resumed, "the expired wait resumes CivilianMission 0x16 [M 0x15dbb0 case 0xc → SetState(0x16)]");
    let after = sim.drones.list[walker].pos;
    for _ in 0..150 { sim.bond.pos = [guest_pos[0] + 40.0, guest_pos[1], guest_pos[2]]; sim.step_bond(Default::default(), false, false, false); }
    let dr = &sim.drones.list[walker];
    let moved = dist(&dr.pos, &after);
    assert!(moved > 0.2, "the guest walks again after the resume (moved {moved:.2} in 150 ticks)");
}

/// Sister: the wait EXTENDS while Bond stands close ahead — the expired clock's case-0xc check
/// re-arms the 2 s wait while Bond is inside the 0.525 rad cone within 3.0 (2D) [M 0x15dbb0:87-106;
/// BondWithinAngle 0x154f98], and a second wait request re-arms too [M 0x15dbb0:136-141]. With Bond
/// parked 2.0 ahead of her facing the guest is still waiting 2.5 s later; the same run with Bond far
/// resumed inside 2 s (the first test's arm).
#[test]
fn the_wait_holds_while_bond_stands_close_ahead() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let walker = find_walker(sim);

    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    assert_eq!(sim.drones.list[walker].state, 0x16, "the guest walks first");
    let guest_pos = sim.drones.list[walker].pos;
    let yaw = sim.drones.list[walker].yaw;
    send_wait(&mut scene, walker);
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.drones.list[walker].state, 0x17, "the wait state entered");

    // Bond 2.0 units ahead of her facing (forward = [sin yaw, 0, cos yaw]), inside cone + radius.
    let ahead = [guest_pos[0] + yaw.sin() * 2.0, guest_pos[1], guest_pos[2] + yaw.cos() * 2.0];
    for _ in 0..150 {
        sim.bond.pos = ahead;
        sim.step_bond(Default::default(), false, false, false);
    }
    {
        let dr = &sim.drones.list[walker];
        assert_eq!(dr.state, 0x17,
            "Bond 2.0 ahead in the 0.525 cone keeps the wait alive past the first 2 s clock [M 0x15dbb0:88-106] — got {:#x}", dr.state);
        assert!(dr.timer_c.is_some(), "the clock re-armed for another 2 s");
        let moved = dist(&dr.pos, &guest_pos);
        // 0.5 not 0.3: the walk danim's blend-out root motion adds ~0.3 of drift while the stand-idle
        // takes over; walking the same 150 ticks covers ≥ 1.0 (the first test's ≥ 2.0/300 ticks).
        assert!(moved < 0.5, "she still has not moved (moved {moved:.2})");
    }

    // A second wait request while waiting re-arms the clock [M 0x15dbb0:136-141].
    send_wait(&mut scene, walker);
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..100 {
        sim.bond.pos = [guest_pos[0] + 40.0, guest_pos[1], guest_pos[2]];
        sim.step_bond(Default::default(), false, false, false);
    }
    assert_eq!(sim.drones.list[walker].state, 0x17,
        "the re-sent wait request holds the pause past the original deadline [M 0x15dbb0:136-141]");
    // Bond leaves: the next expiry resumes the route.
    let mut resumed = false;
    for _ in 0..60 {
        sim.bond.pos = [guest_pos[0] + 40.0, guest_pos[1], guest_pos[2]];
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.list[walker].state == 0x16 { resumed = true; break; }
    }
    assert!(resumed, "Bond gone → the wait ends and the route resumes [M 0x15dbb0 case 0xc else-arm]");
}

/// Sister: the [D] citation itself, pinned byte-exact on both sides so the placement↔node pairing
/// cannot drift. Everything here is read straight off the disc image through nf_formats (the
/// archive, entry and offsets were walked by the probe this test cites; the sim side only supplies
/// the runtime pairing):
///
/// - 07000007.bin = FileList entry 34, offset 0x8384800 into FILES.BIN, size 0xf55000; main map =
///   LevelBin entry 526 "01000097" (kind 1, data 0x722120..0xf3ef00 in the archive).
/// - 0x05 block (entry-data offset 0x701940, size 0xc5c4): payload 0xe23a64..0xe30024, version 8,
///   9 paths. AiNetwork path 3 "Party Patrol Path_1" record 0xe2ebd4: kind word 0x8 at 0xe2ec54,
///   node count 5 at 0xe2ec78, extra length 0 at 0xe2ec7c, link count 4 at 0xe2edc0. Path 0 is the
///   only boundary (kind 0x1), paths 1–2 are kind 0x2, so the game's load order puts this path at
///   the port's `Network.paths[2]`. Its node 0 record 0xe2ec80 carries the position at +0x10 =
///   0xe2ec90: x `d3 e6 2b 42`, y `03 00 2c 42`, z `e9 7f 6d c2`; the loader adds 0.4 to y
///   [M AIPath_Parse 0x1d2598: node+0x14 += 0.4].
/// - 0x1A block (entry-data offset 0x80c87c, size 0xeee0): payload 0xf2e9a0, 513 placements.
///   Placement 402 record 0xf382e0 (0x11c bytes = 0x4c + 26 params): object index field 272,
///   hashcode 0xffffffff (by-index), flags 0x0f (dynamic drone), translation bits 0x422bda27 /
///   0x42305681 / 0xc26d8c1d, params (2,1) difficulty, (3,0) switch channel, (0xf,0xa) dtype 10.
///
/// The pairing: placement 402 stands 0.017 (2D) from node 0 — inside the game's 0.3 acceptance
/// [M AssignAIPath 0x154190: fVar11 = 0.3] — and is the ONLY dtype-10 switch-0 guest inside it
/// [D: next closest placement 349 at 3.06; the path-4/5 node-0 guests, placements 425/424 at
/// 0.062/0.075, carry switch 6]. Fails if the placement record or the node record moves, or if a
/// different node index became the closest.
#[test]
fn the_walker_is_placement_402_on_party_patrol_path_1_node_0() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };

    // --- the disc side, byte-exact [D] ---
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let (fl_idx, fe) = entries.iter().enumerate().find(|(_, e)| e.name.eq_ignore_ascii_case("07000007.bin"))
        .expect("07000007.bin in the FileList");
    assert_eq!(fl_idx, 34, "[D FileList index of 07000007.bin]");
    assert_eq!((fe.offset, fe.size), (0x8384800, 0xf55000), "[D FileList record: offset into FILES.BIN, size]");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read archive");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("LevelBin");
    let (mi, me) = lb.entries.iter().enumerate().find(|(_, e)| e.kind == 1).expect("main map entry");
    assert_eq!((mi, me.name.as_str()), (526, "01000097"), "[D main-map LevelBin entry]");
    assert_eq!(me.offset, 0x722120, "[D main-map entry data offset]");
    let raw = lb.data(&data, me);
    let mf = nf_formats::map::MapFile::parse(raw).expect("map file");

    let path_blk = mf.first(0x05).expect("0x05 AI-path block");
    let net = nf_formats::aipath::AiNetwork::parse(path_blk.payload(raw)).expect("AI paths");
    assert_eq!(net.paths.len(), 9, "[D path-count word 9 at 0xe23a68]");
    assert!(net.paths[0].is_boundary(), "[D path 0 Boundry kind 0x1 — the only boundary, so AiNetwork 3 → Network.paths[2]]");
    let p = &net.paths[3];
    assert_eq!(p.name, "Party Patrol Path_1", "[D name bytes at 0xe2ebd4]");
    assert_eq!(p.kind, 0x8, "[D kind word at 0xe2ec54]");
    assert_eq!(p.nodes.len(), 5, "[D node-count word at 0xe2ec78]");
    let n0 = p.nodes[0].pos;
    assert_eq!(n0[0].to_bits(), 0x422b_e6d3, "node 0 x [D 0xe2ec90: d3 e6 2b 42]");
    assert_eq!(n0[2].to_bits(), 0xc26d_7fe9, "node 0 z [D 0xe2ec98: e9 7f 6d c2]");
    assert_eq!(n0[1].to_bits(), 0x422c_0003, "node 0 y, disc bytes [D 0xe2ec94: 03 00 2c 42 — AiNetwork::parse keeps the raw disc value]");

    let plc_blk = mf.first(0x1a).expect("0x1A placement block");
    let pls = nf_formats::placement::parse(plc_blk.payload(raw)).expect("placements");
    assert_eq!(pls.len(), 513, "[D 0x1A record count]");
    let pl = &pls[402];
    assert_eq!((pl.hashcode, pl.flags), (0xffffffff, 0xf), "[D placement 402: by-index object, dynamic-drone flags]");
    assert_eq!(pl.translation.map(|f| f.to_bits()), [0x422b_da27, 0x4230_5681, 0xc26d_8c1d],
        "[D placement 402 translation bytes at 0xf382ec]");
    let param = |k: u32| pl.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
    assert_eq!((param(2), param(3), param(0xf)), (1, 0, 0xa),
        "[D placement 402 params: difficulty 1, switch channel 0, dtype 10]");
    drop(img);

    // --- the runtime pairing: the spawned drone stands on that node ---
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let d = sim.data();
    let rn = &d.nav.paths[2];
    assert_eq!(rn.name, "Party Patrol Path_1", "the port's Network.paths[2] is the cited path");
    let rn0 = rn.nodes[0].pos;
    assert_eq!((rn0[0].to_bits(), rn0[2].to_bits()), (0x422b_e6d3, 0xc26d_7fe9), "…node 0 carried through the load");
    assert_eq!(rn0[1].to_bits(), nf_formats::ps2float::add(f32::from_bits(0x422c_0003), 0.4).to_bits(),
        "the runtime node 0 y carries the loader's +0.4 [M AIPath_Parse 0x1d2598: node+0x14 += 0.4]");
    let w = sim.drones.list.iter().position(|dr| dr.placement == 402)
        .expect("placement 402 spawns a drone");
    let dr = &sim.drones.list[w];
    assert_eq!((dr.dtype, dr.switch_channel), (10, 0), "[D placement 402 params 0xf/3]");
    let dist = ((dr.pos[0] - rn0[0]).powi(2) + (dr.pos[2] - rn0[2]).powi(2)).sqrt();
    assert!(dist < 0.3,
        "placement 402 stands on node 0 within the game's 0.3 acceptance [M AssignAIPath 0x154190: fVar11 = 0.3] — got {dist:.3}");
    // Uniqueness: no other dtype-10 switch-0 guest is inside the acceptance of any kind-8 node
    // [D: next closest placement 349 at 3.06; the path-4/5 node-0 guests carry switch 6].
    let other = sim.drones.list.iter().any(|dr| dr.placement != 402 && dr.dtype == 10 && dr.switch_channel == 0
        && d.nav.paths.iter().any(|p| p.kind & 8 != 0 && p.nodes.iter().any(|n|
            (n.pos[1] - dr.feet()[1]).abs() <= 2.0
                && (dr.pos[0] - n.pos[0]).powi(2) + (dr.pos[2] - n.pos[2]).powi(2) < 0.09)));
    assert!(!other, "placement 402 is the only dtype-10 switch-0 guest on a kind-8 node [D]");
}
