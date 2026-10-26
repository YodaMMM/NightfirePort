//! wiring-07000003: the level's objective wiring, loaded through the real level path — the AND/SEQ
//! chains, the locked secret-bedroom door, the ch-0x31 plate, and the Dragon Safe key's channel.
//!
//! The disc [D `nf-extract placements-all 07000003.bin`; the scripts inside `07000003.bin` via
//! `nf-extract nis`]:
//! - AND #435: out 0x52 = 0x4E AND 0x51 — the exit key gate (#451) AND obj2's enable [D p0/p1/p2; the
//!   mux wrapper shifts slot[k] = p(k−1), out = slot 1 = p0 [M 0x1b2c68; Trigger_Create 0x1b1720:61-91]].
//! - AND #436: out 0x51 = 0x4F AND 0x50 (obj1 "Rescue Mayhew's servants"; 0x4F/0x50 = the un-ported
//!   mission/dialogue layer).
//! - SEQ #472: out 0x68 = 0x83 then 0x84; SEQ #474: out 0x69 = 0x85 then 0x86 — each pairs a touch
//!   plate (#471 out 0x83 / #473 out 0x85) with a guard's death channel (0x84 = guard #399's, 0x86 =
//!   guard #390's [D p12]) → SoundTriggers #490/#491 (snd 333 once [D]).
//! - Door #4 `door_secret_bedroom`: p3 = 49 = the lock channel [D; +0x68 per Door_Create 0x136100,
//!   docs/fields/door.md] — locked until ch 0x31 rises. Plate #504 (master bedroom) out 0x31 mask 4.
//! - SS #502 `bedroom_triggerpicture`: mask = 1 (the don't-write bit — NOT E-usable), +0x10 = 49:
//!   `SS_Operate` 0x1a9df8 returns 1 once ch 0x31 is on, so the picture swings as the response.
//! - The Dragon Safe key: script 0x06000563 stream 1 spawns a kind-2 pickup with +0x28 = 0x4E [D];
//!   `Pickup_Handler` 0x18e650:330-333 writes it on take — obj0's completion channel.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_and_seq_chains_match_the_disc() {
    let scene = nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_ref().unwrap();
    let tr = |p: usize| sim.drones.triggers.iter().find(|t| t.placement == p)
        .unwrap_or_else(|| panic!("trigger #{p} missing"));
    let and = nf_game::triggers::kind::AND;
    let seq = nf_game::triggers::kind::SEQ_AND;
    // [D p0/p1/p2] per placement; [M 0x1b2c68] the port carries the wrapper's slot shift.
    let a435 = tr(435);
    assert_eq!(a435.kind, and, "#435 is a MultiplexIn");
    assert_eq!(a435.out, 0x52, "#435 out = the exit key gate / obj2 enable");
    assert_eq!(&a435.list[..2], &[0x4e, 0x51], "#435 = 0x4E (the safe key) AND 0x51 (obj1) — slots 3-7 unused [D]");
    let a436 = tr(436);
    assert_eq!(a436.kind, and);
    assert_eq!(a436.out, 0x51, "#436 out = obj1's watch channel");
    assert_eq!(&a436.list[..2], &[0x4f, 0x50], "#436 = 0x4F AND 0x50 (the mission layer's inputs)");
    let s472 = tr(472);
    assert_eq!(s472.kind, seq, "#472 is a MultiplexSIn");
    assert_eq!(s472.out, 0x68, "#472 out → SoundTrigger #490 (snd 333)");
    assert_eq!(&s472.list[..2], &[0x83, 0x84], "#472 = plate #471's 0x83 THEN guard #399's death 0x84");
    let s474 = tr(474);
    assert_eq!(s474.kind, seq);
    assert_eq!(s474.out, 0x69, "#474 out → SoundTrigger #491");
    assert_eq!(&s474.list[..2], &[0x85, 0x86], "#474 = plate #473's 0x85 THEN guard #390's death 0x86");
    // The touch plates the SEQs pair with, and the two special plates.
    assert_eq!(tr(471).out, 0x83, "plate #471");
    assert_eq!(tr(471).lock, 0x90, "plate #471 locks out once 0x90 (#489) fired");
    assert_eq!(tr(473).out, 0x85, "plate #473");
    assert_eq!(tr(504).out, 0x31, "plate #504 = the secret-bedroom unlock writer");
    assert_eq!(tr(504).mask & 8, 0, "#504 mask 4 — not the usual player-8 plate [D]");
    assert_eq!(tr(476).out, 0x4d, "plate #476 = the objective reveal (0x4D) at the start");
}

#[test]
fn the_secret_bedroom_door_locks_until_ch_0x31() {
    let scene = nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_ref().unwrap();
    // [D #4 p3=49 / #5 p3=0]; [M Door_Create 0x136100 → ext+0x68 (docs/fields/door.md)].
    let d = |p: usize| sim.drones.doors.list.iter().find(|d| d.placement == p)
        .unwrap_or_else(|| panic!("door #{p} missing"));
    assert_eq!(d(4).ch_lock, 49, "door_secret_bedroom is the one locked door");
    assert_eq!(d(5).ch_lock, 0, "door_secret_library has no lock channel");
    assert_eq!(d(4).kind, 2, "#4 is a proximity slider [D p0=2]");
    // SS #502: the ch-0x31-driven picture, not a channel writer and not E-usable.
    // [D #502 p2=1 → ext+0x1E = 1 (SS_Create 0x1a9c58:33): bit 0 = don't write, bit 2 (E-use) CLEAR;
    // p3=49 → ext+0x10 = the operate gate [M SS_Operate 0x1a9df8:47-48 reads switch_channels[+0x10]].
    let ss = sim.drones.sss.list.iter().find(|s| s.placement == 502).expect("SS #502");
    assert_eq!(ss.channel, 49, "#502's operate gate is ch 0x31");
    assert_eq!(ss.mask & 4, 0, "#502 is not E-usable (mask 1)");
    assert_eq!(ss.state, 0x40, "#502 sits in the sound-then-animate state [D p0=64]");
}

#[test]
fn the_mission_record_agrees_with_the_chains() {
    let scene = nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_ref().unwrap();
    // The mission record [L ee_start_05.bin @ 0x2a3f18] and the AND chains must agree.
    let def = nf_game::mission::LEVELS.iter().find(|d| d.level == 0x0700_0003).unwrap();
    assert_eq!(def.objectives[0].channel, 0x4e, "obj0 watches 0x4E = the safe key's take channel");
    assert_eq!(def.objectives[1].channel, 0x51, "obj1 watches 0x51 = AND #436's out");
    assert_eq!(def.objectives[2].enable, 0x52, "obj2 enables on 0x52 = AND #435's out");
    assert_eq!(def.objectives[0].enable, 0x4d, "obj0 reveals on 0x4D = plate #476's out");
    let _ = sim;
}

#[test]
fn the_spawned_key_params_reach_the_port() {
    // The port parses 06000563 from the level archive it loads; its stream-1 Entity op carries the
    // pickup params [D]. Assert through the real load path: the level's NIS library holds 06000563
    // and its stream-1 entity params decode kind 2 / channel 0x4e.
    let scene = nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_ref().unwrap();
    let lib = &sim.drones.scripts.lib;
    let script = lib.get(&0x0600_0563).expect("06000563 in the level's script library");
    let found = script.streams.iter().flat_map(|s| s.decode()).find_map(|c| match c {
        nf_formats::nis::Cmd::Entity { hash, obj_type: 0xf0, params, .. } => Some((hash, params)),
        _ => None }).expect("stream 1 spawns the kind-2 key pickup");
    assert_eq!(found.0, 0x0200_05f1, "the key's object hash [D]");
    // [M Pickup_Create 0x18dda0 arg map: p0 kind, p1 +0x24, p2 +0x26, p3 +0x28, p4 +0x2A, p5 +0x2C].
    assert_eq!(found.1[0], 2, "kind 2 = key");
    assert_eq!(found.1[3], 0x4e, "+0x28 = 0x4E — the take writes obj0's watch channel");
    assert_eq!(found.1[4], 0xffff, "no take sound [D]");
    assert_eq!(found.1[5], 0, "respawn 0 = delete on take");
}
