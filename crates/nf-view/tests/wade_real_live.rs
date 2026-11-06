//! wade_real_live: the shallow-water wade behaviours on the REAL 07000002 shallows (-79, 4.68, 28)
//! [L PCSX2: the feet contact byte 0xE, anim+0x60 bits 8|0x10, steps 0x28]. Pins, from real input:
//! the wade walks at the dry speed (the game reads no water state in the walk-speed path [M
//! Player_SSWalk 0x192310, Player_ResetWalkAnim 0x195a88 — no BLData+0x50/+0x60 read]), the wading
//! steps spawn the Env_WaterRipple ring [M Player_DealWithObjHit 0x194458:92-95 → Env_WaterRipple
//! 0x17c118:9's (mat&0x3f)−0xd<2 gate], the water surfaces spawn NO foot decals (the disc's 0xE
//! EffectRecord foot_decal_4/8 = 0 [D]; the game's Effect_Player reads the same table [M
//! 0x1cec48:84-89, 0 = none]), and the eye stays at the surfaced 0.7 target [M Player_PositionCamera
//! 0x18f5d8:114-128 — no wade term; Player_GetHeadPos 0x195aa0].
use nf_game::player::Input;
use nf_game::world_fx::RIPPLE_MESH;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_02() -> nf_view::scene::Scene {
    let mut scene = nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..2000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    scene
}

/// The mean horizontal speed over `ticks` of held-forward walking.
fn walk_speed(sim: &mut nf_view::scene::Sim, ticks: usize) -> f32 {
    let mut steps = 0.0f32;
    for _ in 0..ticks {
        let p0 = sim.bond.pos;
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        steps += ((sim.bond.pos[0] - p0[0]).powi(2) + (sim.bond.pos[2] - p0[2]).powi(2)).sqrt();
    }
    steps / ticks as f32
}

#[test]
fn the_02_shallows_wade_walks_at_the_dry_speed() {
    let mut scene = load_02();
    let dry = { let sim = scene.sim.as_mut().unwrap(); walk_speed(sim, 60) };
    assert!(dry > 0.05, "the dry walk must move: {dry}");
    let sim = scene.sim.as_mut().unwrap();
    // The PCSX2-measured shallows spot [L]: standing there grounds on the submerged floor with the
    // water plane (0xE) between root and root − (cc + 0.6) — the feet byte the landing reads.
    sim.bond.pos = [-79.0, 4.68, 28.0];
    sim.bond.prev_pos = sim.bond.pos;
    sim.bond.vel = [0.0; 3];
    for _ in 0..40 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.bond.grounded && sim.bond.substate == 0, "Bond stands in the shallows: {:?} sub {}",
        sim.bond.pos, sim.bond.substate);
    let feet = [sim.bond.pos[0], sim.bond.pos[1] - (sim.bond.cc + 0.6), sim.bond.pos[2]];
    let byte = sim.world.segment_surface(sim.bond.pos, feet, false).map(|h| h.2 & 0x3f);
    assert_eq!(byte, Some(0xE), "the shallows spot reads the water plane: {byte:?}");
    // Walk WITHIN the water line: the wade strip is ~3 units deep (z 28->31 dry, z 24 deep), so
    // pace north/south with a test-side re-aim between the legs. The movement itself is real input.
    let mut dist = 0.0f32;
    let mut wade_ticks = 0u32;
    for leg in 0..4 {
        if leg % 2 == 1 { sim.bond.yaw += std::f32::consts::PI; }
        for _ in 0..24 {
            let p0 = sim.bond.pos;
            let byte = sim.world.segment_surface(sim.bond.pos,
                [sim.bond.pos[0], sim.bond.pos[1] - (sim.bond.cc + 0.6), sim.bond.pos[2]], false)
                .map(|h| h.2 & 0x3f);
            sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
            if byte == Some(0xE) {
                wade_ticks += 1;
                dist += ((sim.bond.pos[0] - p0[0]).powi(2) + (sim.bond.pos[2] - p0[2]).powi(2)).sqrt();
            }
        }
    }
    assert!(wade_ticks >= 40, "the pacing stayed in the water long enough: {wade_ticks}");
    assert!(sim.bond.substate == 0, "wading, not swimming: sub {}", sim.bond.substate);
    let wet = dist / wade_ticks as f32;
    assert!(wet > dry * 0.9 && wet < dry * 1.1,
        "the game does not slow the wade [M 0x192310 — no water read]: dry {dry:.4} wet {wet:.4}");
}

#[test]
fn wading_steps_spawn_ripples_no_foot_decals_and_keep_the_eye() {
    let mut scene = load_02();
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [-79.0, 4.68, 28.0];
    sim.bond.prev_pos = sim.bond.pos;
    sim.bond.vel = [0.0; 3];
    for _ in 0..40 { sim.step_bond(Default::default(), false, false, false); }
    // The disc's own record: the water surfaces carry NO foot decal meshes [D] — the game's
    // Effect_Player reads the same EffectInfo and spawns none [M 0x1cec48:84-89].
    let rec = sim.surfaces.as_ref()
        .and_then(|s| s.effects.get(0xE)).expect("the 0xE EffectRecord exists [D]");
    assert_eq!((rec.foot_decal_4, rec.foot_decal_8), (0, 0), "0xE has no foot decal meshes [D]");
    let decals0 = sim.drones.effects.made;
    let mut made_ripple = 0u32;
    let mut wade_ticks = 0u32;
    for leg in 0..4 {
        if leg % 2 == 1 { sim.bond.yaw += std::f32::consts::PI; }
        for _ in 0..24 {
            let before = sim.drones.gas.puffs.iter().filter(|g| g.mesh == RIPPLE_MESH).count();
            let byte = sim.world.segment_surface(sim.bond.pos,
                [sim.bond.pos[0], sim.bond.pos[1] - (sim.bond.cc + 0.6), sim.bond.pos[2]], false)
                .map(|h| h.2 & 0x3f);
            sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
            if byte == Some(0xE) {
                wade_ticks += 1;
                let after = sim.drones.gas.puffs.iter().filter(|g| g.mesh == RIPPLE_MESH).count();
                if after > before { made_ripple += 1; }
            }
        }
    }
    assert!(wade_ticks >= 40, "the pacing stayed in the water long enough: {wade_ticks}");
    assert!(made_ripple >= 1, "wading steps spawn the Env_WaterRipple ring [M 0x194458:92-95]: \
        {} spawn ticks over {wade_ticks} wade ticks", made_ripple);
    // No foot decal may sit ON the water plane: the 0xE record carries no decal meshes [D], and the
    // steps that left the water legitimately decal the dry surface they landed on. Check every decal
    // the pacing spawned against the surface UNDER IT.
    let made_now = sim.drones.effects.made;
    let new = sim.drones.effects.decals.iter().skip(decals0 as usize).collect::<Vec<_>>().clone();
    assert_eq!(new.len() as u32, made_now - decals0, "the spawned decals are the tail of the list");
    for d in new {
        let byte = sim.world.segment_surface([d.pos[0], d.pos[1] + 0.2, d.pos[2]],
            [d.pos[0], d.pos[1] - 2.5, d.pos[2]], false).map(|h| h.2 & 0x3f);
        assert!(byte != Some(0xE), "a foot decal sits on the water plane at {:?}: byte {byte:?}", d.pos);
    }
    assert!((sim.bond.eye - 0.7).abs() < 1e-4, "the eye stays at the surfaced target [M 0x18f5d8:114-128]: {}",
        sim.bond.eye);
}
