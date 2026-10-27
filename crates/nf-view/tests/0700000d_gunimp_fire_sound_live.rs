//! The mounted laser's fire sound plays on the game's cadence (0700000d's mount #225).
//!
//! [M `GunImp_Update` 0x1320d8:247-334] the laser's `TurretGuns[0xb]+0x18` sound (1307
//! [D ACTION.ELF @0x26f520]) is a LOOP the imp holds: on a fire tick with no handle stored
//! (`ext+0x98 == 0`) it plays `Sound_Play3D(1307, obj+0x30)` once and stores the handle
//! (:284-291); while the trigger stays down the loop keeps playing across the 1.0 s cooldown
//! ticks — no restart, no second voice (:271-278's held path returns without stopping);
//! `Sound_Stop` + handle zero happens when the trigger releases, when the hot byte is set
//! (:261-269), and on dismount ([M `GunImp_Deactivate` 0x132c18:62-63]). The port previously
//! played a fresh `play3d` per shot — the fire audio stuttered (a re-attack every 1.0 s while
//! the old voice kept looping underneath).
//!
//! The test drives `gunimp_tick` (the mounted tick the view's step calls) and reads the sound
//! pool's active ids (the `Sounds::active_ids` test hook). Opt-in via NF_ISO.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const LASER_FIRE: u32 = 1307; // [D TurretGuns[0xb]+0x18]

fn count_active(sim: &nf_view::scene::Sim, id: u32) -> usize {
    sim.drones.audio.active_ids().iter().filter(|&&x| x == id).count()
}

/// Mount the deck gun (the aim test's walk).
fn mounted() -> Option<nf_view::scene::Scene> {
    let iso = iso()?;
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..2000 {
            sim.step_bond(Default::default(), false, false, false);
            if !sim.disabled { break; }
        }
        sim.bond.pos = [137.27, 9.33, -323.0];
        sim.bond.yaw = std::f32::consts::PI;
        sim.bond.pitch = 0.0;
        for _ in 0..6 {
            sim.step_bond(Default::default(), false, false, true);
            if sim.drones.props.gunimps.iter().any(|g| g.active) { break; }
        }
    }
    let ok = scene.sim.as_ref().unwrap().drones.props.gunimps.iter().any(|g| g.active);
    if !ok { return None; }
    Some(scene)
}

fn weapons() -> nf_formats::weapon::WeaponData {
    let iso = iso().expect("NF_ISO");
    let image = nf_formats::iso9660::IsoImage::open(&iso).expect("iso");
    let elf = image.read_file(image.find("/ACTION.ELF").expect("elf")).expect("elf");
    nf_formats::weapon::WeaponData::from_action_elf(&elf).expect("weapons")
}

#[test]
fn the_fire_sound_starts_once_holds_while_firing_and_stops_on_release() {
    let Some(mut scene) = mounted() else { eprintln!("NF_ISO not set or the mount is unreachable — skipping"); return };
    let weapons = weapons();
    let sim = scene.sim.as_mut().unwrap();
    // The first fire tick starts exactly ONE voice.
    sim.drones.gunimp_tick(&weapons, &sim.world, 0.0, 0.0, true, false, &mut sim.bond);
    assert_eq!(count_active(sim, LASER_FIRE), 1, "the first shot starts the 1307 loop once [M :284-291]");
    // Holding the trigger across two full cooldown cycles (the 1.0 s interval ≈ 60 ticks
    // [M TurretGuns[0xb]+0x14 = 1.0]): still exactly one voice — no restart, no stacking.
    for _ in 0..130 {
        sim.drones.gunimp_tick(&weapons, &sim.world, 0.0, 0.0, true, false, &mut sim.bond);
    }
    assert_eq!(count_active(sim, LASER_FIRE), 1,
        "a held trigger keeps ONE 1307 loop across the cooldown shots [M :271-278 keeps the handle; \
         :284-291 only plays while ext+0x98 == 0] — a restart here is the fire-audio stutter");
    // The handle is the game's ext+0x98: nonzero while held.
    assert!(sim.drones.props.gunimps[0].fire_handle != 0, "the stored handle [M :290]");
    // Releasing the trigger stops the loop [M :251-269].
    sim.drones.gunimp_tick(&weapons, &sim.world, 0.0, 0.0, false, false, &mut sim.bond);
    assert_eq!(count_active(sim, LASER_FIRE), 0, "releasing the trigger stops the loop [M :261-269]");
    assert_eq!(sim.drones.props.gunimps[0].fire_handle, 0, "the handle zeroes [M :267]");
    // A later press starts it again — once.
    sim.drones.props.gunimps[0].cooldown = 0.0;
    sim.drones.gunimp_tick(&weapons, &sim.world, 0.0, 0.0, true, false, &mut sim.bond);
    assert_eq!(count_active(sim, LASER_FIRE), 1, "a re-press starts the loop again, once [M :284-291]");
    // Dismounting while firing kills the loop with the mount [M 0x132c18:62-63].
    sim.drones.gunimp_tick(&weapons, &sim.world, 0.0, 0.0, true, true, &mut sim.bond);
    assert!(!sim.drones.props.gunimps[0].active, "the use key dismounts");
    assert_eq!(count_active(sim, LASER_FIRE), 0, "the dismount stops the loop [M 0x132c18:62-63]");
}

#[test]
fn the_overheat_stops_the_loop_while_the_trigger_stays_down() {
    let Some(mut scene) = mounted() else { eprintln!("NF_ISO not set or the mount is unreachable — skipping"); return };
    let weapons = weapons();
    let sim = scene.sim.as_mut().unwrap();
    // Drive the heat past the 200 saturation with the trigger held: the shot adds 2.0
    // [M :292-298] and the tick drains 1.0 [M :100-106], net +1.0 per held tick.
    for i in 0..230 {
        sim.drones.props.gunimps[0].cooldown = 0.0; // skip the interval wait, not the heat
        sim.drones.gunimp_tick(&weapons, &sim.world, 0.0, 0.0, true, false, &mut sim.bond);
        if sim.drones.props.gunimps[0].hot {
            // The tick AFTER the saturation goes through the game's pressed+hot stop path
            // [M :271-278 → :261-269]: the loop stops while the trigger is still down.
            sim.drones.gunimp_tick(&weapons, &sim.world, 0.0, 0.0, true, false, &mut sim.bond);
            assert_eq!(count_active(sim, LASER_FIRE), 0,
                "the hot byte stops the held loop [M :271-278] (shot {i})");
            assert_eq!(sim.drones.props.gunimps[0].fire_handle, 0, "the handle zeroes [M :267]");
            return;
        }
    }
    panic!("100 shots at 2.0 heat never saturated the hot byte [M :292-298]");
}
