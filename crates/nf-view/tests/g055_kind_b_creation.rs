//! G-055: `NDrone2_GetDroneTypeAttackTypeFriend` 0x14cbd8's voice-set-0xB arm. With the
//! first-block alertness `+0x50C >= 1.0` and `+0xD8 == 0xB` (skin 0x0500001E, the ninja
//! [M NDrone2_DefaultInit 0x14b300:158-162]) the function calls `NDrone2_ChangeToAttackMode`
//! 0x14a648 and returns before the `+0x5A2` alert fold [M 0x14cbd8:119-128], so the start state
//! stays `DroneTypeSettings[13]`'s 0xA6 NinjaStand [D docs/fields/dronetype-tables.md row 13]
//! instead of the folded 0xA7. NinjaStand's ENTER then sets the engage range `+0xF0 = 3.0` and the
//! combat range `+0xFC = 5.0` before SetState(0xA7) [M 0x16b990 case 1].
fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn data(sim: &nf_view::scene::Sim) -> nf_game::drone::Data<'_> {
    nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty, ai_points: &sim.ai_points,
    }
}

#[test]
fn voice_set_b_ninja_starts_in_ninja_stand() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut seen = 0;
    for lv in ["07000004.bin", "07000016.bin"] {
        let scene = nf_view::load(&iso, lv, false).expect("load");
        let sim = scene.sim.as_ref().unwrap();
        let d = data(sim);
        for (i, dr) in sim.drones.list.iter().enumerate().filter(|(_, dr)| dr.skin == 0x0500_001e) {
            seen += 1;
            assert_eq!(dr.kind, 0xb, "{lv} #{i}: voice set 0xB [M 0x14b300:161]");
            assert!(dr.alert >= 1.0, "{lv} #{i}: first-block alertness 1.0 [D census]");
            assert_eq!(dr.start_state(&d), 0xa6, "{lv} #{i}: the 0xB arm returns before the fold — NinjaStand [M 0x14cbd8:119-128]");
            assert_eq!(dr.alt_state, 0, "{lv} #{i}: +0x5A4 stays 0 (the :157 fold is skipped) [M 0x14cbd8:124-127]");
            assert_eq!(dr.second_tag, 0, "{lv} #{i}: +0x13A stays 0 [M 0x14cbd8:205-209 skipped]");
            assert!(!dr.using_2nd, "{lv} #{i}: no alternate state — ChangeToAttackMode leaves the first block [M 0x14a648:11]");
        }
    }
    assert_eq!(seen, 3, "07000004 #1 + 07000016 #105/#126 [D g055_behaviour_census]");
}

#[test]
fn ninja_stand_enter_sets_the_close_ranges() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000016.bin", false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    let g = sim.drones.list.iter().position(|d| d.present && d.enabled && !d.corpse && d.health > 0.0)
        .expect("a live drone on 07000016");
    sim.drones.list[g].engage_range = 12.0;
    sim.drones.list[g].combat_range = 20.0;
    sim.drones.list[g].set_state(0xa6);
    sim.step_bond(Default::default(), false, false, false);
    let dr = &sim.drones.list[g];
    assert_eq!(dr.engage_range, 3.0, "+0xF0 = 3.0 [M 0x16b990 case 1]");
    assert_eq!(dr.combat_range, 5.0, "+0xFC = 5.0 [M 0x16b990 case 1]");
    assert!(dr.bounded && dr.vision_on, "+0x4F8 |= 0x80000000 | 0x10000 [M 0x16b990 case 1]");
}
