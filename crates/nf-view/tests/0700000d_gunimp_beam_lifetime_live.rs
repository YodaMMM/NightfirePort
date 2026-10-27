//! 0700000d mounted laser: the beam lives only while the gunner holds the trigger — and dies with the mount.
//!
//! [M] `GunImp_Update` 0x1320d8 creates and positions the beam object per shot; the port's `GunImp.beam` carries the pair the
//! draw stretches. Real play showed it hanging in the room after release and after dismount (F12 004/005), because the sim's
//! clear sat in the non-firing branch that returns early once no imp is active, and the view drew ANY imp's beam.
//!
//! Fail-first on the round-2 patch: the dismount case leaves `beam` set.
//! Opt-in via NF_ISO.

#[test]
fn the_mounted_laser_beam_dies_with_the_trigger_and_the_mount() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    let image = nf_formats::iso9660::IsoImage::open(&iso).expect("iso");
    let elf = image.read_file(image.find("/ACTION.ELF").expect("elf")).expect("elf");
    let weapons = nf_formats::weapon::WeaponData::from_action_elf(&elf).expect("weapons");
    let sim = scene.sim.as_mut().unwrap();
    let gi = sim.drones.props.gunimps.iter().position(|g| g.turret == 0x0b).expect("the laser GunImp");
    sim.drones.gunimp_activate(gi);
    let mut bond = sim.bond.clone();
    let beam = |s: &nf_view::scene::Sim| s.drones.props.gunimps[gi].beam;
    sim.drones.gunimp_tick(&weapons, 0.0, 0.0, true, false, &mut bond);
    assert!(beam(sim).is_some(), "a shot sets the beam (the quad draws this tick)");
    sim.drones.gunimp_tick(&weapons, 0.0, 0.0, false, false, &mut bond);
    assert!(beam(sim).is_none(), "releasing the trigger clears the beam [M: the beam object stops with the fire]");
    sim.drones.gunimp_tick(&weapons, 0.0, 0.0, true, false, &mut bond);
    assert!(beam(sim).is_some(), "firing again re-arms it");
    sim.drones.gunimp_tick(&weapons, 0.0, 0.0, false, true, &mut bond);
    assert!(!sim.drones.props.gunimps[gi].active, "the use key dismounts");
    assert!(beam(sim).is_none(), "dismounting clears the beam with the mount (no stale quad in the room)");
}
