//! ninja-boss-cutscene: the Double-Cross pt. 4 (07000004) story cut-scene's third actor — the ninja
//! (skin 0x0500001e) — must exist, pose and draw.
//!
//! The ninja boss was missing from the level's last cut-scene. The cut-scene
//! is 0x06000082 (830 ticks), played by the main map's ScriptPlayer #2 right after the intro
//! 0x06000083 hands over (its stream-6 event 0xD writes switch_channels[56] = 1 at tick 500; #2 waits
//! on mask 1 / channel 56 — SP_Hit 0x1dc470:15-17 bit 1, SP_Update 0x1dbda8's health/MissionState==1
//! gate). Its op-0x0A streams name THREE actors [D 07000004.bin kind-7]: stream 1 skin 0x0500004f
//! (anim 0x06000508), stream 2 skin 0x05000009 — Bond (anim 0x06000509) — and stream 19 skin
//! 0x0500001e (anim 0x06000507), the level's assassin drone's own skin (placement param 0).
//!
//! The pre-fix failure: 0x06000507's only window is sequence 0x0400047a, whose record skeleton byte
//! is 0 (07F00004.bin) while the skin's is 8 (both 73 bones, translation mask 0x1/0/0) — shipped
//! cross-rig content the game plays (`AnimFrameCopy` 0x1c5668 walks the SEQUENCE's own record
//! skeleton, :52, and gates the per-bone writes on the SKIN's bone flags) but the port's
//! `script_pose_together` rejected (strict `seq.skeleton == skin.skeleton`) — no base window, pose
//! `None`, character not drawn. Evidence and the fix: docs/systems/ninja-boss-cutscene.md.
//!
//! Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The disc facts [D]: the actor lists of both cut-scene scripts, the ninja's anim script/sequence,
/// the skin/skeleton bytes. Numbers and ids only; every assertion has a sister that can fail.
#[test]
fn story_cutscene_actor_set_matches_the_disc() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let image = nf_formats::iso9660::IsoImage::open(&iso).expect("iso opens");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&image).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000004.bin")).expect("07000004.bin");
    let data = image.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("level bytes");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("level bin parses");
    let level = nf_formats::level::Level::parse(&data).expect("level parses");

    // The two ScriptPlayers on the main map: scripts, mode, trigger mask, channel.
    let m = level.main_map().expect("main map");
    let mut sps = level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0xd9);
    let p2 = sps.next().expect("SP #2 present");
    let p3 = sps.next().expect("SP #3 present");
    assert!(sps.next().is_none(), "exactly two ScriptPlayers");
    let param = |p: &nf_formats::placement::Placement, k: u32| p.params.iter().find(|(i, _)| *i == k).map(|(_, v)| *v);
    assert_eq!(param(p2, 0), Some(0x0600_0082), "SP #2's script A");
    assert_eq!(param(p2, 2), Some(0), "SP #2 waits (mode 0)");
    assert_eq!(param(p2, 5), Some(1), "SP #2's trigger mask = channel bit");
    assert_eq!(param(p2, 6), Some(56), "SP #2 waits on channel 56");
    assert_eq!(param(p3, 0), Some(0x0600_0083), "SP #3's script A");
    assert_eq!(param(p3, 2), Some(1), "SP #3 plays at once (mode 1)");
    // Sister: the intro must NOT also be the story scene's script (a re-pair fails here).
    assert_ne!(param(p2, 0), param(p3, 0), "the two SPs play different scripts");

    // 0x06000082's op-0x0A (stream, anim script, skin) triples; the ninja is stream 19.
    let anis = |h: u32| {
        let nis = level.nis.get(&h).unwrap_or_else(|| panic!("nis {h:08x} in the level archive"));
        let mut out = Vec::new();
        for (si, st) in nis.streams.iter().enumerate() {
            for c in st.decode() {
                if let nf_formats::nis::Cmd::Anim { script, skin, .. } = c { out.push((si as u32, script, skin)); }
            }
        }
        out
    };
    assert_eq!(anis(0x0600_0082), vec![
        (1, 0x0600_0508, 0x0500_004f),
        (2, 0x0600_0509, 0x0500_0009),
        (19, 0x0600_0507, 0x0500_001e),
    ], "the story scene's actor list — the ninja on stream 19");
    // Sister: the OTHER actors' pairing must not move (a skin swap between streams fails here).
    assert_eq!(anis(0x0600_0083), vec![(1, 0x0600_0506, 0x0500_004f), (2, 0x0600_0505, 0x0500_0009)],
        "the intro's actor list — two actors, no ninja");

    // The ninja's animation: 0x06000507's single opcode-0 window → 0x0400047a.
    let raw5 = lb.entries.iter().filter(|e| e.kind == 5)
        .map(|e| nf_formats::anim::Script::parse(lb.data(&data, e)).expect("kind-5 parses"))
        .find(|s| s.hashcode == 0x0600_0507).expect("anim script 0x06000507 in 07000004.bin");
    let wins: Vec<_> = raw5.sequences().collect();
    assert_eq!(wins, vec![(1u16, 831u16, 0x0400_047au32)], "0x06000507's windows");

    // The sequence: record skeleton byte 0 (cross-rig against the skin's 8), full body walk.
    let fe7 = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07F00004.bin")).expect("07F00004.bin");
    let d7 = image.read_bytes(bin.lba as u64 * 2048 + fe7.offset as u64, fe7.size as u64).expect("anim lib bytes");
    let lb7 = nf_formats::levelbin::LevelBin::parse(&d7).expect("anim lib parses");
    let e47a = lb7.entries.iter().filter(|e| e.kind == 4)
        .find(|e| u32::from_le_bytes(lb7.data(&d7, e)[0..4].try_into().unwrap()) == 0x0400_047a)
        .expect("sequence 0x0400047a in 07F00004.bin");
    let seq = nf_formats::anim::Sequence::parse(lb7.data(&d7, e47a)).expect("sequence parses");
    assert_eq!((seq.frames, seq.skeleton, seq.channels.len(), seq.flags), (831, 0, 222, 0x00));

    // The skin: skeleton byte 8, one skinned mesh that resolves in the level's own objects.
    let skin = level.skins.get(&0x0500_001e).expect("skin 0x0500001e in 07000004.bin's own archive");
    assert_eq!(skin.skeleton, 8);
    let mains: Vec<u32> = skin.main.iter().flatten().copied().collect();
    assert_eq!(mains, vec![0x0200_0292], "the ninja skin's single skinned mesh");
    assert!(level.by_hash.contains_key(&0x0200_0292), "the ninja mesh resolves in the level's own archive");
    // Sister: the skeleton facts that make the cross-rig pairing well-defined — a count or mask
    // change on either side fails here.
    let skel_of = |id: u8| lb.entries.iter().filter(|e| e.kind == 6)
        .map(|e| nf_formats::character::parse_skeleton(lb.data(&data, e)).expect("skeleton parses"))
        .find(|s| s.id as u8 == id).expect("skeleton present");
    let (s0, s8) = (skel_of(0), skel_of(8));
    assert_eq!((s0.bones, s8.bones), (73, 73), "skeletons 0 and 8 share the bone count");
    assert_eq!(s0.translation_mask, s8.translation_mask, "skeletons 0 and 8 share the translation mask");
}

/// Through the real load: after the intro hands over (tick 500), SP #2 plays 0x06000082 and the
/// ninja's Obj::Anim runs on anim script 0x06000507; `script_pose_together` poses it (the fix —
/// `None` before), the root moves, and the other two actors pose unchanged (the sister).
#[test]
fn ninja_poses_in_the_port_cutscene() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000004.bin", false).expect("07000004 loads");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..700 { sim.step_bond(Input::default(), false, false, false); }
    }
    // The story scene is playing (the intro's ch-56 hand-over fired it).
    assert!(scene.sim.as_ref().unwrap().drones.scripts.players.iter().any(|p| p.scripts.contains(&0x0600_0082) && p.state == 2),
        "SP #2 plays 0x06000082 after the intro's channel-56 hand-over");
    let mut sim = scene.sim.take().expect("sim");
    let script_of = |sim: &nf_view::scene::Sim, src: (u32, u8), skin: u32| sim.script_anim(src, skin)
        .unwrap_or_else(|| panic!("the cut-scene runs but Obj::Anim {:08x}/{} is gone", src.0, src.1));
    let pose_of = |sim: &nf_view::scene::Sim, c: &nf_view::scene::Character, frame: f32, script: u32| {
        nf_game::anim::script_pose_together(&sim.scripts, &sim.seqs, &c.skeleton, &c.skin, script, frame)
    };
    fn find<'a>(scene: &'a nf_view::scene::Scene, stream: u8, skin: u32) -> &'a nf_view::scene::Character {
        scene.characters.iter()
            .find(|c| c.script == Some((0x0600_0082, stream)) && c.skin.hashcode == skin)
            .unwrap_or_else(|| panic!("no character for stream {stream} skin {skin:08x}"))
    }
    // The ninja (stream 19): the sim's object advances and the pose exists with a moving root.
    let ninja = find(&scene, 19, 0x0500_001e);
    let (script_a, frame_a) = script_of(&sim, (0x0600_0082, 19), 0x0500_001e);
    assert_eq!(script_a, 0x0600_0507, "the ninja's Obj::Anim runs anim script 0x06000507");
    assert!(frame_a >= 190.0, "the ninja's frame advances with the scene (got {frame_a})");
    let pose_a = pose_of(&sim, ninja, frame_a, script_a).expect("the ninja poses (None before the fix)");
    assert!(pose_a.iter().all(|(t, q)| t.iter().all(|v| v.is_finite()) && q.iter().all(|v| v.is_finite()) && q.iter().any(|v| *v != 0.0)),
        "the ninja's pose is finite");
    // Sister: the OTHER two actors pose exactly as before the fix — the new arm must not have
    // disturbed the matching path (their roots here are the pre-fix probe's values at the same
    // frame; the sim is deterministic, so frame_a is the probe's 200).
    for (stream, skin, script, root) in [
        (1u8, 0x0500_004fu32, 0x0600_0508u32, [-1.922_239_9f32, 5.884_119_5, 19.776_71]),
        (2, 0x0500_0009, 0x0600_0509, [-1.296_018_7, 5.893_909_5, 20.263_088]),
    ] {
        let c = find(&scene, stream, skin);
        let (s, f) = script_of(&sim, (0x0600_0082, stream), skin);
        assert_eq!(s, script, "stream {stream}'s anim script unchanged");
        assert!((f - frame_a).abs() < 1.0, "stream {stream}'s frame tracks the ninja's ({f} vs {frame_a})");
        let p = pose_of(&sim, c, f, s).unwrap_or_else(|| panic!("stream {stream}'s actor stopped posing"));
        assert!(p[0].0.iter().zip(root.iter()).all(|(a, b)| (a - b).abs() < 0.01),
            "stream {stream}'s root moved: {:?} vs the pre-fix {:?}", p[0].0, root);
    }
    // Then the scene advances: the ninja's frame and root move with it.
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    let (_, frame_b) = script_of(&sim, (0x0600_0082, 19), 0x0500_001e);
    let pose_b = pose_of(&sim, ninja, frame_b, script_a).expect("the ninja still poses a half-second later");
    assert!(frame_b > frame_a, "the ninja's frame advances ({frame_a} → {frame_b})");
    assert!(pose_a[0].0.iter().zip(pose_b[0].0.iter()).any(|(a, b)| (a - b).abs() > 0.01),
        "the ninja's root moves between the two ticks (the sequence's root motion)");
    scene.sim = Some(sim);
    // And the ninja is not hidden while the scene runs.
    let ci = scene.characters.iter().position(|c| c.script == Some((0x0600_0082, 19))).unwrap();
    assert!(!scene.char_hidden(ci, Default::default()), "the ninja's character is drawn while the scene runs");
}
