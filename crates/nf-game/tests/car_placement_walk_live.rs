//! placement-kind-census: the 0xDA create arm — `parsemap_create_dynamic_objects` 0x1d04f0
//! case 0xda → `Car_Create` 0x1305b0, walked over the real archives.
//!
//! Red on the old code: the port had the Car data model (`Car::create`'s gates, the state
//! machine) but NO placement walk and an SP-wrong gate — `Car::create` returned None whenever
//! `MPSettings+0x384 == 0`, while the game's off-gate is `MP-enabled && vehicles-Off`
//! ([M disasm 0x1305dc `beqz v1` skips the vehicles check when MPSettings+0x384 is 0;
//! 0x1305e8 `beql v0, zero → return 0`]). On a non-MP load of an archive that carries 0xDA
//! placements (07000046 carries exactly one [D census]), the game creates a vehicle (the
//! option word 0 falls through the variant pick to the helicopter byte — the decomp's goto),
//! the old gate created nothing.
//!
//! Run: NF_ISO=<iso> cargo test --offline --release -p nf-game --test car_placement_walk_live

use nf_game::drone::Rng;
use nf_formats::{disc, iso9660::IsoImage, level::Level};
use nf_game::car::{CarSystem, VEHICLES_OFF, VEHICLES_TANKS};

fn load(code: u32) -> Level {
    let path = std::env::var_os("NF_ISO").expect("NF_ISO not set");
    let iso = IsoImage::open(path.to_str().unwrap()).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let name = format!("{code:08x}.bin");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name))
        .unwrap_or_else(|| panic!("archive {name} missing"));
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).unwrap()
}

/// 07000046 carries exactly one kind-0xDA placement [D census 2026-11-08]. A non-MP load
/// (MPSettings+0x384 == 0) with the option word 0 must create ONE vehicle, the helicopter
/// variant (option 0 falls through `Car_Create` 0x1305b0's variant pick to +0x112 = 1), posed
/// at the placement, slot 0.
#[test]
fn sp_load_creates_the_07000046_vehicle() {
    let level = load(0x0700_0046);
    let m = level.main_map().unwrap();
    let (n, pos, rot) = {
        let cars: Vec<_> = level.maps[m].placements.iter().enumerate()
            .filter(|(_, p)| p.flags & 0xff == 0xda).collect();
        assert_eq!(cars.len(), 1, "07000046: one 0xDA placement [D census]");
        (cars.len(), cars[0].1.translation, cars[0].1.rotation)
    };
    let _ = n;
    let mut rng = Rng::new(0x12345678);
    let mut sys = CarSystem::init();
    // MPSettings+0x384 == 0 (no match), MPSettings+0x460 == 0 (the option word as it stands).
    sys.create_walk(&level, false, VEHICLES_OFF, 0x0700_0046, &mut rng);
    assert_eq!(sys.num_tanks, 1, "the game creates the 07000046 vehicle on a non-MP load [M 0x1305b0:13-16]");
    let car = sys.tanks[0].as_ref().unwrap();
    assert!(car.heli, "option 0 falls through the variant pick to the helicopter byte [M 0x1305b0:29-43]");
    assert_eq!(car.slot, 0, "CAR_INFO+0x10e = NumTanks at create [M 0x1305b0:72]");
    assert_eq!(car.state, nf_game::car::CAR_IDLE, "state 0 at create");
    // The pose: View_RotTransMatrix of the placement — the translation column is the placement's
    // position, the same build every placement-pose creator here uses.
    let want = nf_formats::placement::view_rot_trans_matrix(rot, pos);
    assert_eq!(car.matrix, want, "obj+0x90 built at the placement pose [M 0x1305b0:27 Control_CreateObjEx]");
    assert_eq!(car.spawn_matrix, want, "TankSpawns[slot] backs the fresh pose up [M 0x1305b0:74]");
    // Nothing was drawn from the shared stream (no Random roll on option 0): a second walk with
    // a DIFFERENT seed must produce the identical car.
    let mut rng2 = Rng::new(0xdead_beef);
    let mut sys2 = CarSystem::init();
    sys2.create_walk(&level, false, VEHICLES_OFF, 0x0700_0046, &mut rng2);
    assert_eq!(sys.tanks[0], sys2.tanks[0], "option 0 draws nothing from the shared stream [M 0x1305b0:36-43]");
}

/// An MP match with the Enviro-Mods option Off creates NO cars (the gate fires: MP enabled &&
/// option Off) [M 0x1305b0:13-16]; Tanks creates all four arena placements as tanks, slots 0..4;
/// Random draws per created placement in placement order.
#[test]
fn mp_match_gates_and_counts() {
    // Skyrail 07000024: four 0xDA placements [D mp_map_objects_live probe].
    let level = load(0x0700_0024);
    let mut rng = Rng::new(7);
    let mut off = CarSystem::init();
    off.create_walk(&level, true, VEHICLES_OFF, 0x0700_0024, &mut rng);
    assert_eq!(off.num_tanks, 0, "MP enabled + vehicles Off: Car_Create returns 0 [M 0x1305b0:13-16]");
    // The gated-out walk draws nothing (the gate returns before the roll).
    let mut rng_a = Rng::new(99);
    let mut rng_b = Rng::new(99);
    let mut tanks = CarSystem::init();
    tanks.create_walk(&level, true, VEHICLES_TANKS, 0x0700_0024, &mut rng_a);
    assert_eq!(tanks.num_tanks, 4, "all four Skyrail placements create [D mp_map_objects_live]");
    for (i, slot) in tanks.tanks.iter().take(4).enumerate() {
        let car = slot.as_ref().unwrap();
        assert!(!car.heli, "Tanks option: the variant byte is 0 [M 0x1305b0:41]");
        assert_eq!(car.slot, i as u16, "slots follow the created order [M 0x1305b0:72]");
    }
    // Ravine is blocked outright (GameState+0x12 == 0x700004b → return 0 [M 0x1305b0:19-22]);
    // its four placements draw nothing either.
    let ravine = load(0x0700_004b);
    let mut sys = CarSystem::init();
    sys.create_walk(&ravine, true, VEHICLES_TANKS, 0x0700_004b, &mut rng_b);
    assert_eq!(sys.num_tanks, 0, "Ravine creates no cars [M 0x1305b0:19-22]");
    assert_eq!(rng_a.draws, rng_b.draws,
        "a blocked Ravine walk draws nothing from the shared stream [M the gate precedes the roll]");
}
