//! script-event-channel-writes: the per-event unit tests on REAL disc scripts. The cited
//! [D] census rows (`census_channel_events.rs`) are decoded from the ISO's own archives and
//! then driven through the port's real `Script::update` path (the `Script_EventHandler`
//! 0x1d94c0 walk) with the corrected 0xD/0x11 rule: the game returns on a zero value BEFORE
//! the store [M 0x1d94c0 case 0xd disasm 0x1d9758 `beqz $v1`, case 0x11 disasm 0x1d9798
//! `beqz $a0`; raw words 10600034 a0440000 / 10800024 a0620000 — the decompiled C drops both
//! returns], so a script event can never CLEAR a channel. Opt-in: NF_ISO.
use nf_formats::iso9660::IsoImage;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis::{Cmd, Nis};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// One level archive's parsed script `Nis` by hash.
fn disc_script(level: &str, hash: u32) -> Nis {
    let img = IsoImage::open(iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&format!("{level}.bin"))).expect("level bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = LevelBin::parse(&data).expect("levelbin");
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        if se.hash() == hash {
            return Nis::parse(hash, lb.data(&data, se)).expect("nis");
        }
    }
    panic!("script {hash:08x} not in {level}.bin");
}

/// The stream's events in the Wait timeline (op 5 arms the wait [M Script_Run 0x1d8548 case 5]).
fn timed_events(s: &nf_formats::nis::Stream) -> Vec<(u32, u8, Vec<u32>)> {
    let mut out = Vec::new();
    let mut wait = 0u32;
    for c in s.decode() {
        if let Cmd::Wait(t) = c { wait = t as u32; }
        if let Cmd::Event { kind, params } = c { out.push((wait, kind, params)); }
    }
    out
}

/// The real disc stream's commands run through the port's Script::update; returns the
/// script-path (source 1) writes (channel, value, frame).
fn run_disc_stream(level: &str, hash: u32, stream: usize, frames: u32) -> Vec<(u8, u8, u32)> {
    let nis = disc_script(level, hash);
    // The cited stream's own bytes, alone in the script (the other streams' Entity/Anim ops
    // need the level's object/anim world; the event behaviour under test is the stream's).
    let one = Nis {
        hashcode: nis.hashcode, camera: nis.camera, loop_start: nis.loop_start, length: nis.length,
        streams: vec![nis.streams[stream].clone()], keys: nis.keys.clone(),
    };
    let ident: nf_game::door::Mat4 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    struct DiscWorld;
    impl nf_game::nis::World for DiscWorld {
        fn anim_length(&self, _script: u32) -> Option<u16> { Some(100) }
        fn discrete0(&self, _skin: u32, _script: u32, _frame: f32) -> Option<nf_game::door::Mat4> { None }
    }
    let mut script = nf_game::nis::Script::load(std::sync::Arc::new(one), ident, 1.0);
    script.start(false);
    // The game un-holds an event-3 delay through SP_Activate 0x1dc338:84 / SP_UnPause 0x1dca58:13
    // (both zero SCRIPTINFO+0x9c [M]); the isolated-stream test starts un-held the same way.
    script.delay = 0;
    let mut objs = nf_game::nis::Objects::default();
    let mut ch = nf_game::triggers::Channels::default();
    let world = DiscWorld;
    let mut out = Vec::new();
    let mut fired = Vec::new();
    for frame in 0..frames {
        let mut subs = Vec::new();
        // The game un-holds an event-3 delay through SP_Activate 0x1dc338:84 / SP_UnPause
        // 0x1dca58:13 (both zero SCRIPTINFO+0x9c [M]); the isolated-stream run applies that
        // un-hold each frame (06000530's stream 8 carries its own event-3 holds at t1/74/99/134
        // [D decode test], which the real level clears through its SP triggers).
        script.delay = 0;
        ch.writes = Some(Vec::new());
        script.update(&mut subs, &Default::default(), &mut objs, &mut ch, &world, frame, &mut out);
        for (c, v, f, src) in ch.writes.take().unwrap_or_default() {
            if src == 1 { fired.push((c, v, f)); }
        }
    }
    fired
}

/// The driver's cited rows decode from the disc's own bytes [D census_channel_events.rs]:
/// 07000012 06000530 s8 t520 0xD [3c,3c]; 06000a25 s2 t2 [89,1] + s3 t4 [89,0];
/// 07000005 06000952 s8 t93 [62,1]; 07000005 06000a23 s1 t300 0x11 [98,98,1d,1].
#[test]
fn the_cited_rows_decode_from_the_disc() {
    let n12 = disc_script("07000012", 0x0600_0530);
    let evs = timed_events(&nis12_stream(&n12, 8));
    assert_eq!(evs, vec![(1, 3, vec![0]), (74, 3, vec![0]), (99, 3, vec![0]), (134, 3, vec![0]), (520, 0xd, vec![0x3c, 0x3c])],
        "07000012 06000530 stream 8: four event-3 holds [M 0x1d94c0 case 3 — p0=0 → delay −1, the SP_Activate hold] then the 0xD [D census]");

    let n25 = disc_script("07000012", 0x0600_0a25);
    let s2 = timed_events(&n25.streams[2]);
    let s3 = timed_events(&n25.streams[3]);
    assert_eq!(s2, vec![(2, 0xd, vec![0x89, 1])], "06000a25 stream 2 [D]");
    assert_eq!(s3, vec![(4, 0xd, vec![0x89, 0])], "06000a25 stream 3 — the ZERO-value event [D]");

    let n05 = disc_script("07000005", 0x0600_0952);
    let evs = timed_events(&n05.streams[8]);
    assert!(evs.contains(&(93, 0xd, vec![0x62, 1])), "07000005 06000952 stream 8 [D]: {evs:?}");

    let n23 = disc_script("07000005", 0x0600_0a23);
    let evs = timed_events(&n23.streams[1]);
    assert!(evs.contains(&(300, 0x11, vec![0x98, 0x98, 0x1d, 1])), "07000005 06000a23 stream 1, the disc's ONE kind-0x11 row [D]: {evs:?}");
}

fn nis12_stream(n: &Nis, i: usize) -> &nf_formats::nis::Stream { &n.streams[i] }

/// The real [89,1] then [89,0] pair through the port's event path: the set lands (ch 0x89 = 1,
/// stamped at its frame) and the ZERO-value event leaves the channel SET — the game's
/// 0xD returns before the store [M 0x1d9758], so a script can never clear a channel.
#[test]
fn the_real_set_then_zero_pair_leaves_the_channel_set() {
    let fired = run_disc_stream("07000012", 0x0600_0a25, 2, 12);
    assert_eq!(fired, vec![(0x89, 1, 2)], "stream 2's [89,1] set ch 0x89 at its tick [D+M 0x1d94c0 case 0xd]");
    let fired3 = run_disc_stream("07000012", 0x0600_0a25, 3, 12);
    assert_eq!(fired3, Vec::<(u8, u8, u32)>::new(),
        "stream 3's [89,0] produced NO write at all — the zero value returns before the store [M 0x1d9758 beqz]");
}

/// The real [62,1] (07000005 06000952 s8 t93): ch 0x62 = 1 with the frame stamp.
#[test]
fn the_real_07000005_handback_set_stamps_its_frame() {
    let fired = run_disc_stream("07000005", 0x0600_0952, 8, 140);
    assert_eq!(fired, vec![(0x62, 1, 93)], "ch 0x62 = 1 at the stream's t93 [D census] with the event's frame stamp [M 0x1d97a4 join]");
}

/// The real 0x11 row (07000005 06000a23 s1 t300 [98,98,1d,1]): the test channel 0x98 reads
/// zero on this path → dest = p2 = 0x1d → ch 0x1d = 1, stamped [M 0x1d94c0 case 0x11
/// :179-192 + disasm 0x1d9768-0x1d97bc].
#[test]
fn the_real_0x11_selects_the_zero_dest_and_stamps() {
    let fired = run_disc_stream("07000005", 0x0600_0a23, 1, 360);
    assert_eq!(fired, vec![(0x1d, 1, 300)],
        "the EndLevel 0x11 read test ch 0x98 (zero) → dest p2 = 0x1d = 1 [D census; M case 0x11]");
}

/// The real value-carrying set [3c,3c] (07000012 06000530 s8 t520): ch 0x3c = 0x3c, stamped.
#[test]
fn the_real_value_0x3c_write_lands_with_its_value() {
    let fired = run_disc_stream("07000012", 0x0600_0530, 8, 560);
    assert_eq!(fired, vec![(0x3c, 0x3c, 520)], "ch 0x3c = 0x3c at t520 [D census; M case 0xd — (byte)p1 truncation]");
}
