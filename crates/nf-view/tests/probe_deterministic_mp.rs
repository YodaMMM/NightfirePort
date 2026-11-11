//! net row determinism (level-05-basin-floor-6): three MP sims of the Ravine arena driven
//! DETERMINISTICALLY — no sessions, no UDP, the same row every machine, the frag test's
//! scripted controls and kills. Pins the lockstep contract the net layer must preserve:
//! identical applied rows ⇒ bit-identical sims at every local slot (the per-tick state hash
//! agrees for 1000 ticks, kills included). This is the baseline the
//! net_match_end_clients_live desync hunt measures against: its three in-process machines
//! diverge (bodies + match components) while their applied rows fold identically, so the
//! divergence rides a session-driven side channel (spawn/join timing), not the rows and not
//! the sim's row handling — and NOT the per-tick hurt state in the CollisionWorld (each
//! machine loads and builds its own world [M load_impl → CollisionWorld::build_scoped per
//! call; the world's per-tick fields are plain RwLock/Mutex, no Arc], and this probe's three
//! sims — which share no world — stay clean).

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn settings() -> nf_game::mp::MpSettings {
    let mut s = nf_game::mp::MpSettings::deathmatch(0x0700_004b);
    s.human_count = 3;
    s.player_count = 3;
    s.bot_count = 0;
    s.total_players = 3;
    s.seed = 0x1a6c_6ab7;
    s.frag_limit = 2;
    s.time_limit = 600;
    s.started = true;
    s
}

fn machine(path: &str, s: &nf_game::mp::MpSettings, slot: usize) -> nf_view::scene::Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

/// The frag test's scripted controls (net_match_end_clients_live's scripted_input).
fn scripted_input(t: u32, slot: u8) -> nf_net::Input {
    let mut i = nf_net::NEUTRAL;
    i.forward = ((t as i32 + slot as i32) % 97) as f32 / 97.0;
    i.strafe = if t % 11 == 0 { -0.5 } else { 0.0 };
    i.turn = ((t % 13) as f32 - 6.0) * 0.002;
    i.look = if t % 29 == 0 { 0.25 } else { 0.0 };
    i.jump = t % 61 == 0;
    i.crouch = t % 43 == 0;
    i.fire = t % 23 == 0;
    i
}

#[test]
fn probe_deterministic_three_sims() {
    let path = iso();
    let s = settings();
    let mut scenes: Vec<nf_view::scene::Scene> = (0..3).map(|slot| machine(&path, &s, slot)).collect();
    let mut hashes = [0u64; 3];
    for t in 0..1000u32 {
        let row = std::array::from_fn(|i| scripted_input(t, i as u8));
        for (slot, sc) in scenes.iter_mut().enumerate() {
            {
                let sim = sc.sim.as_mut().unwrap();
                nf_view::net_tick(sim, &row, slot);
            }
            if t == 400 || t == 900 {
                let sim = sc.sim.as_mut().unwrap();
                let (mp, rng) = (&mut sim.mp, &mut sim.drones.rng);
                mp.as_mut().expect("the mp match").player_killed(1, Some(0), false, false, rng);
            }
        }
        for (slot, sc) in scenes.iter().enumerate() {
            hashes[slot] = sc.sim.as_ref().unwrap().net_state_hash(slot);
        }
        if !(hashes[0] == hashes[1] && hashes[0] == hashes[2]) {
            eprintln!("[probe] DESYNC at t{t}: {:016x} {:016x} {:016x}", hashes[0], hashes[1], hashes[2]);
            let comps: Vec<[u64; nf_view::scene::NET_HASH_COMPONENTS]> = (0..3)
                .map(|slot| scenes[slot].sim.as_ref().unwrap().net_component_hashes(slot))
                .collect();
            for c in 0..nf_view::scene::NET_HASH_COMPONENTS {
                if !(comps[0][c] == comps[1][c] && comps[0][c] == comps[2][c]) {
                    eprintln!("[probe]   component {} ({}) diverges: {:016x} {:016x} {:016x}",
                        c, nf_view::scene::NET_HASH_COMPONENT_NAMES[c], comps[0][c], comps[1][c], comps[2][c]);
                }
            }
            for (slot, sc) in scenes.iter().enumerate() {
                let sim = sc.sim.as_ref().unwrap();
                let b = |i: usize| if i == slot { Some(&sim.bond) } else { sim.mp_bonds[i].as_ref() };
                eprintln!("[probe]   sim{slot}: slot0 {:?} slot1 {:?} slot2 {:?}",
                    b(0).map(|b| (b.pos, b.yaw, b.pitch, b.substate, b.crouched)),
                    b(1).map(|b| (b.pos, b.yaw, b.pitch, b.substate, b.crouched)),
                    b(2).map(|b| (b.pos, b.yaw, b.pitch, b.substate, b.crouched)));
            }
            return; // the first divergence is the evidence
        }
    }
    eprintln!("[probe] 1000 ticks, no desync");
}

#[test]
fn probe_arena_hurts() {
    let path = iso();
    let s = settings();
    let arena = format!("{:08x}.bin", s.level_code);
    let scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[probe] arena hurts: {}", sim.hurts.len());
    for (k, h) in sim.hurts.iter().enumerate() {
        eprintln!("[probe] hurt[{k}] plc {} gate {} disabled {} fired {} flying {} one_shot {} shape_key {} damage {} tris {}",
            h.placement, h.gate, h.disabled, h.fired, h.flying, h.one_shot, h.shape_key, h.damage, h.tris.len());
    }
}
