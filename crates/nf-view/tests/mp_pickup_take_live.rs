//! audit-pickup-calls pins on the real MP arenas: the take message's duration and label arm
//! (`Text_AddMsg` 180 SP / 90 MP [M Pickup_Handler 0x18e650:324-327]; the MP weapon name from
//! `weapon_data[raw]+0x3C` bound bare [M :249-256]; the MP ammo format "%d x %s" [D 0x30cc28]),
//! and the take SOUND for every taker — the shared tail plays `+0x2A` for bots too (the
//! placement's own word — the bot arm jumps straight to the tail [M :137 → :334-337]).
//! Driven through the real MP load (`nf_view::load_mp_carry`) and the real take path
//! (`Sim::step_mp_pickups`' touch walk — the same walk `Pickup_Update` 0x18e148 runs).

use nf_formats::iso9660::IsoImage;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(img: &IsoImage, archive: &str) -> Vec<u8> {
    let (entries, bin) = nf_formats::disc::files_bin_entries(img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn pos_of(scene: &nf_view::scene::Scene, k: usize) -> [f32; 3] {
    let sim = scene.sim.as_ref().unwrap();
    let p = &sim.drones.pickups[k];
    [p.m[3][0], p.m[3][1] + 0.4, p.m[3][2]]
}

fn step(scene: &mut nf_view::scene::Scene, n: u32) {
    for _ in 0..n {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
    }
}

/// The local take on a real arena: the HUD info message carries the MP duration (90 ticks) and
/// the MP weapon label (bound bare from +0x3C — no 0x200000D wrapper), and the take sound 0xF5
/// lands in the hearing list.
#[test]
fn mp_take_message_is_ninety_ticks_with_the_mp_label_and_the_sound_plays() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = IsoImage::open(&iso).unwrap();
    let archive = "07000023.bin";
    let level = nf_formats::level::Level::parse(&read_archive(&img, archive)).expect("the arena parses");
    let _ = level;
    let mut ms = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    ms.bot_count = 1;
    ms.total_players = 2;
    let mut scene = nf_view::load_mp_carry(&iso, archive, false, None, ms).expect("the arena loads");
    step(&mut scene, 10);

    // The nearest alive weapon-set pickup to Bond's spawn (the real touch walk takes it).
    let target = {
        let sim = scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        sim.drones.pickups.iter().enumerate()
            .filter(|(_, p)| !p.gone && p.state != 2 && p.kind == nf_game::pickup::kind::WEAPON)
            .min_by(|a, b| {
                let d = |p: &nf_game::pickup::Pickup| {
                    let dx = p.m[3][0] - bp[0]; let dy = p.m[3][1] - bp[1]; let dz = p.m[3][2] - bp[2];
                    dx * dx + dy * dy + dz * dz
                };
                d(a.1).total_cmp(&d(b.1))
            })
            .map(|(k, _)| k).expect("the arena's set pickups exist")
    };
    let at = pos_of(&scene, target);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [at[0], at[1], at[2]], 0.0);
    }
    // local-match-bot-pickups: the take pass now runs at the TICK'S HEAD (the MP driver, right
    // after the bots), so the take sound's hearing-list copy is consumed by the SAME tick's AI
    // hearing pass and never survives a step boundary — the old per-tick `drones.sounds` scan
    // read the old order's accident, not the law. The law (the shared tail plays the placement's
    // word at the pickup [M 0x18e650:334-337]) is read off a SECOND take driven straight into
    // the pass, before any further tick consumes the hearing list.
    let mut heard = false;
    for _ in 0..40 {
        step(&mut scene, 1);
        let sim = scene.sim.as_ref().unwrap();
        if !sim.drones.taken.is_empty() { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.taken.is_empty(), "the pickup is taken through the real MP touch walk");
    let taken_first = sim.drones.taken.len() - 1; // the driven take's entry (the HUD drain ran in-tick)
    let (second, at2) = {
        let sim = scene.sim.as_ref().unwrap();
        let b = sim.drones.pickups.iter().enumerate()
            .filter(|(k, p)| *k != target && !p.gone && p.state != 2 && p.kind == nf_game::pickup::kind::WEAPON)
            .map(|(k, _)| k).next().expect("a second weapon pickup");
        let p = &sim.drones.pickups[b];
        (b, [p.m[3][0], p.m[3][1], p.m[3][2]])
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [at2[0], at2[1], at2[2]], 0.0);
        sim.step_mp_pickups();
    }
    let sim = scene.sim.as_ref().unwrap();
    if sim.drones.sounds.iter().any(|s| s.id == 0xf5
        && (s.pos[0] - at2[0]).abs() < 1.0 && (s.pos[2] - at2[2]).abs() < 1.0) { heard = true; }
    // The take sound [M 0x18e650:290 + :336-337]: kinds 0/1 force 0xF5 into +0x2A and the tail
    // plays it.
    assert!(heard, "the take plays 0xF5 at the pickup (heard within the 40 ticks)");
    // The message [M 0x18e650:249-256 + :279-288 + :324-328]: the MP arm binds
    // weapon_data[raw]+0x3C bare (weapon) or formats "%d x %s" (ammo), and Text_AddMsg carries
    // 90 ticks (180 is the SP value).
    // The driven take's own entry — the second (direct-drive) take's message has no HUD drain
    // yet (no step ran after it), so the pane assert reads the first take's taken record.
    let (_, t) = &sim.drones.taken[taken_first];
    let lang = sim.lang.as_ref().expect("the arena's text table");
    let expected = match t {
        nf_game::pickup::Taken::Weapon { weapon } => {
            lang.bind(sim.weapons.mp_name_label(*weapon as usize)).unwrap_or("").to_string()
        }
        nf_game::pickup::Taken::Ammo { weapon, rounds } => {
            let at = sim.weapons.ammo_type(*weapon as usize);
            let name = lang.bind(sim.weapons.ammo_name_label(at)).unwrap_or("");
            format!("{} x {}", rounds, name)
        }
        _ => String::new(),
    };
    assert!(!expected.is_empty(), "the take is a weapon or an ammo take, got {:?}", t);
    assert!(sim.hud.info_msgs.iter().any(|m| m.text == expected && m.time > 80.0 && m.time <= 90.0),
        "the MP take message = the +0x3C label (or the x format) at the game's 90-tick duration (the pane decays it per tick): expected [{expected}] vs info {:?} msgs {:?}",
        sim.hud.info_msgs.iter().map(|m| (&m.text, m.time)).collect::<Vec<_>>(),
        sim.hud.msgs.iter().map(|m| (&m.text, m.time)).collect::<Vec<_>>());
}

/// A bot take plays the PLACEMENT's own sound word — the bot arm skips the override block
/// [M 0x18e650:137] and the shared tail plays +0x2A [M :334-337]. The arena's set pickups are
/// created with +0x2A = 0xF5 [M Pickup_CreateFromSet], so the bot take is audible too.
#[test]
fn bot_take_plays_the_placements_own_sound_word() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = IsoImage::open(&iso).unwrap();
    let archive = "07000023.bin";
    let mut ms = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    ms.bot_count = 1;
    ms.total_players = 2;
    let mut scene = nf_view::load_mp_carry(&iso, archive, false, None, ms).expect("the arena loads");
    step(&mut scene, 10);
    if scene.sim.as_ref().unwrap().mp.as_ref().map_or(0, |m| m.bots.len()) == 0 { eprintln!("no bots spawned — skipping"); return; }

    // Teleport the bot onto a weapon pickup it does not own (its spawn gun is 6) via the bot's
    // own teleport (`BotBrain::fell_out_map` — the goal machine wipes to BotIdle), then tick a
    // few steps: the shared touch walk (`step_mp_pickups`' bot loop) takes it on the first tick.
    let target = {
        let sim = scene.sim.as_ref().unwrap();
        let m = sim.mp.as_ref().unwrap();
        let held = &m.bots[0].brain.vars.arsenal.held;
        sim.drones.pickups.iter().enumerate()
            .filter(|(_, p)| !p.gone && p.state != 2 && p.kind == nf_game::pickup::kind::WEAPON
                && (p.weapon as usize) < held.len() && !held[p.weapon as usize])
            .map(|(k, _)| k).next().expect("a weapon pickup the bot does not hold")
    };
    let at = pos_of(&scene, target);
    let place = { let sim = scene.sim.as_ref().unwrap(); let m = sim.mp.as_ref().unwrap(); m.bots[0].brain.body.pos };
    {
        let sim = scene.sim.as_mut().unwrap();
        // Bond far away — the touch walk tests each taker separately.
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [at[0], at[1], at[2] + 60.0], 0.0);
        let m = sim.mp.as_mut().unwrap();
        m.bots[0].brain.fell_out_map([at[0], at[1], at[2]], 0.0);
    }
    let _ = place;
    // `step_mp_pickups` has no caller in the current tree (the audit finding — the local human's
    // touch runs through `update_pickups` alone), so the test drives the real shared pass
    // directly, the way the MP tick driver will once wired.
    scene.sim.as_mut().unwrap().step_mp_pickups();
    // The pushed take sound is consumed by the next tick's AI hearing pass — read it now.
    let heard = scene.sim.as_ref().unwrap().drones.sounds.iter().any(|s| s.id == 0xf5
        && (s.pos[0] - at[0]).abs() < 2.0 && (s.pos[2] - at[2]).abs() < 2.0);
    for _ in 0..5 {
        step(&mut scene, 1);
    }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[diag] pickup state {} gone {} taken {}", sim.drones.pickups[target].state, sim.drones.pickups[target].gone, sim.drones.taken.len());
    eprintln!("[diag] bot pos {:?} target at {:?}", sim.mp.as_ref().unwrap().bots[0].brain.body.pos, at);
    assert!(heard, "a bot take plays the placement's +0x2A (0xF5) at the pickup — the game's shared tail runs for bots too");
}
