//! bathhouse-armed-girl-02: one of the bathhouse girls appeared to have a gun and shoot at Bond
//! (07000002, seen at --bond -43.11,6.46,10.37 frame 4206).
//!
//! The placement [D — the 07000002 census, `census_07000002_drones.rs`]: the bathhouse girls are
//! the three skin-0x05000034 drone placements #464 (-37.5,6.5,4.4), #465 (-45.5,5.8,12.2 — the
//! girl in view, 2.9 units from the camera) and #466 (-12.0,6.6,-39.1): kind 8, dtype 0xE
//! (param 15) → team 3 — the civilian team [M NDrone2_GetDroneTypeAttackTypeFriend 0x14cbd8's
//! team switch; `NDrone2_IsCivilian` 0x14aae0], weapon parameter 9 = 1 (the fists) and
//! weapon-swap parameter 10 = 1 [D]. The game's weapon store is `NDrone2_DefaultInit` 0x14b300:
//! parameter 9, default 1, 0 only for skin 0x0500001E [M 0x14b300:94-99,264]; the swap id is
//! `*(char *)(iVar3 + 0x45) = (char)piVar19[0x17]` = parameter 10 [M 0x14b300:266].
//!
//! Why the port armed her: the mode-7 girl (#464, `GetDTYPENEW` dtype 0xE block 3 → 7
//! [M 0x14c938:111-117], EnemyRunToPoint) walks her run course into Attack; `DroneFunc_
//! AttackNoWeapon` 0x14af80 returns the next state for an unarmed attacker — CivilianScared
//! (0x12) unless the swap id `+0x45` > 1, then DrawWeapon (0x66) — and `NDrone2_DSTATE_
//! DrawWeapon` 0x166000 re-gates the same way (case 1: `+0x45` < 2 → SetState(0x12); case 2's
//! `ChangeWeapon(6)` fires only after the draw anim, [M :41-43]). The port sent EVERY unarmed
//! attacker to DrawWeapon and its EXIT hardcoded the case-2 sidearm (weapon 6, 16 rounds) —
//! the girl drew a gun in combat and shot. The port also never parsed `+0x45` (0 for everyone).
//!
//! The fix loads `+0x45` from parameter 10 and adds the game's two gates. Opt-in via NF_ISO.
use nf_game::player::Input;
use nf_game::drone::state;

fn load_and_skip() -> Option<nf_view::scene::Scene> {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return None };
    let mut scene = match nf_view::load(&iso, "07000002.bin", false) {
        Ok(s) => s,
        Err(e) => { eprintln!("load: {e}"); return None }
    };
    let mut n = 0u32;
    let sim = scene.sim.as_mut().unwrap();
    while n < 400 {
        if sim.drones.scripts.players.iter().any(|p| p.state == 2 && p.scripts.contains(&0x0600_0084)) {
            sim.drones.scripts.skip_request = true; // the game's skip action [M SP_Update 0x1dbda8]
        }
        sim.step_bond(Input::default(), false, false, false);
        n += 1;
    }
    drop(sim);
    Some(scene)
}

/// The bathhouse girls and the start civilian: every 07000002 civilian placement [D census
/// #463-466] spawns with the placement's own weapon byte — parameter 9 = 1, the fists
/// [M DefaultInit 0x14b300:94-99,264] — and the swap id `+0x45` = parameter 10 = 1
/// [M 0x14b300:266]: no gun, and no drawable one. The sister: the guards beside them
/// (#442-#462) carry real guns (parameter 9 = 0x12/0xe/0x1c/0x1e) and a drawable swap id (0xE).
#[test]
fn every_02_civilian_spawns_unarmed_with_no_drawable_swap_id() {
    let Some(mut scene) = load_and_skip() else { return };
    let sim = scene.sim.as_ref().unwrap();
    let mut girls = 0;
    let mut guards = 0;
    let mut named: Option<u32> = None;
    for d in &sim.drones.list {
        match d.skin {
            0x0500_0034 | 0x0500_004f => {
                girls += 1;
                assert_eq!(d.weapon, 1, "civilian placement {} (skin {:08x}) spawns with weapon 1, the fists [M 0x14b300:264]", d.placement, d.skin);
                assert_eq!(d.weapon_swap, 1, "civilian placement {} swap id +0x45 = parameter 10 = 1 [D census; M 0x14b300:266]", d.placement);
                assert_eq!(d.team, if d.skin == 0x0500_0034 { 3 } else { 2 }, "the girls are team 3 civilians [M 0x14cbd8's team switch]");
            }
            0x0500_0036..=0x0500_003a => {
                guards += 1;
                assert!((2..=0x46).contains(&d.weapon), "guard placement {} spawns armed (weapon {:#x}) [D census]", d.placement, d.weapon);
                assert!(d.weapon_swap > 1, "guard placement {} has a drawable swap id ({:#x})", d.placement, d.weapon_swap);
                if d.placement == 448 { named = Some(d.weapon); }
            }
            _ => {}
        }
    }
    assert_eq!(girls, 4, "the census's four civilian placements (#463-466) are in the level [D]");
    assert!(guards >= 15, "the census's guard block (#442-#462) backs the sister ({guards} found)");
    assert_eq!(named, Some(0x1c), "the in-view guard #448 (skin 0x05000036) holds weapon 0x1c [D census #448]");
}

/// The repro shot: the terrace pads wake the bathhouse block (the girls' and their guards'
/// switch channel, placement parameter 3 = 22/23/25 [D census; the wake is the pads' out-channel
/// write — WAIT_SWITCH UPDATE `channel_on` → start state]). With Bond at the repro spot, the
/// mode-7 girl #464 runs her course and her chain reaches Attack; the game answers an unarmed
/// attacker with no swap id with CivilianScared [M AttackNoWeapon 0x14af80], so she never plays
/// the draw anim, never takes the case-2 sidearm and never fires. The armed guard on the same
/// channels (#448, weapon 0x1c) still opens fire — the sister that can fail. (Pre-fix this test
/// fails on the girl's weapon: the port armed her with the DrawWeapon-exit sidearm at ~tick 700.)
#[test]
fn the_woken_girl_never_draws_or_fires_and_the_guard_still_fires() {
    let Some(mut scene) = load_and_skip() else { return };
    let girls: Vec<usize>;
    {
        let sim = scene.sim.as_mut().unwrap();
        girls = sim.drones.list.iter().enumerate()
            .filter(|(_, d)| matches!(d.skin, 0x0500_0034 | 0x0500_004f))
            .map(|(i, _)| i).collect();
        sim.drones.set_channel(22);
        sim.drones.set_channel(23);
        sim.drones.set_channel(25);
        sim.bond.pos = [-43.11, 6.46, 10.37]; // the repro camera
        sim.bond.prev_pos = sim.bond.pos;
    }
    let mut ever_fired = false;
    let mut guard_fired = false;
    for _ in 0..1800u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        for g in &girls {
            let g = &sim.drones.list[*g];
            assert_eq!(g.weapon, 1, "the girl {} stays with the fists the whole run", g.placement);
            assert!(g.state != state::DRAW_WEAPON, "the girl {} never walks into DrawWeapon", g.placement);
        }
        for b in &sim.drones.bullets {
            if b.state == 2 { continue; }
            ever_fired = true;
            let o = &sim.drones.list[b.owner];
            assert!(!matches!(o.skin, 0x0500_0034 | 0x0500_004f),
                "a civilian (placement {}, skin {:08x}) fired a bullet [M AttackNoWeapon 0x14af80 sends them to CivilianScared instead]", o.placement, o.skin);
            if o.team == 1 { guard_fired = true; }
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    let girl_log = |pl: usize| sim.drones.list.iter().find(|d| d.placement == pl)
        .map(|d| d.log.iter().map(|&(_, s)| s).collect::<Vec<u8>>()).unwrap_or_default();
    for pl in [464usize, 465, 466] {
        let log = girl_log(pl);
        assert!(!log.contains(&state::DRAW_WEAPON), "girl #{pl}'s state log never holds DrawWeapon (0x66): {log:x?}");
    }
    assert!(girl_log(464).contains(&state::CIVILIAN_SCARED),
        "the mode-7 girl's chain answered her unarmed attack with CivilianScared [M 0x14af80]: {:x?}", girl_log(464));
    assert!(ever_fired && guard_fired, "the sister: an armed guard on the same channels opened fire");
}
