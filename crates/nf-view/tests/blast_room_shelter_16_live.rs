//! 07000016's launch pulses: how the player lives through them. The pack cinema 0x06000631 hands
//! Bond back on the pit floor (stream 8's single key [D pack_cinema_16_live]) and the launch
//! script 0x0600090a raises ch 0xf3 for ticks 1611..2170 and 6810..7320 [D stream 6/9], which arms
//! HurtPoly-ShuttleLaunch (placement 309: gate 0xf3, 1000 damage [D params]) — a 1000-damage
//! contact kills Bond in one hit [M Player_DealWithObjHit 0x194458:95-101 → Player_HandlePain
//! 0x1902c8]. The shelter is the pit's two BLAST ROOMS:
//! - the volume's six bands (y −24.08 … −21.32) are each the same pit-floor polygon, and that
//!   polygon leaves the two blast rooms out (west x ≈ −19..−13, east x ≈ 19..27, z ≈ −166..−175)
//!   [D the placement's collision tris; probe16_pitmap]; the contact is per tri — `Collide_Update`
//!   0x1ea398 runs `Collide_Intersect` on the volume's glist [M], the port's capsule-vs-tris;
//! - the rooms' door leaves are the launch script's entities 02000994..02000997 (streams 15..18):
//!   OPEN at 0..1000, CLOSED (the two leaves of a pair meet) over 1350..2230 and 6660..7360, open
//!   again from 2580 / 7710 [D keys] — each pulse lies inside a closed window. So the player has
//!   ~1000 ticks from the hand-back to get inside; the door shuts behind him, the pulse passes,
//!   the door reopens.
//! Opt-in via NF_ISO.

use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};
use nf_game::player::{Input, RamSave};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

/// The west room's interior and the open pit floor just outside its door (the walk's last
/// waypoint and the sister's stop).
const SHELTER: [f32; 2] = [-15.2, -170.4];
const OUTSIDE: [f32; 2] = [-9.0, -173.0];

/// [D] The door leaves and the pulse windows, straight from the disc's launch script.
#[test]
fn the_blast_doors_are_shut_through_both_pulses_and_open_at_the_hand_back() {
    let iso_path = iso();
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lb = LevelBin::parse(&data).unwrap();
    let e = lb.entries.iter().find(|e| e.kind == 7 && u32::from_str_radix(&e.name, 16) == Ok(0x0600090a)).unwrap();
    let n = Nis::parse(0x0600090a, lb.data(&data, e)).unwrap();
    // The pulse windows: stream 6 and stream 9's ch-243 writes.
    let writes = |si: usize| -> Vec<(u32, u32, u32)> {
        let mut t = 0u32;
        let mut out = Vec::new();
        for c in n.streams[si].decode() {
            match c {
                nf_formats::nis::Cmd::Wait(w) => t = w as u32,
                nf_formats::nis::Cmd::Event { kind: 0x0d, params } if params.len() >= 2 => out.push((t, params[0], params[1])),
                _ => {}
            }
        }
        out
    };
    let mut pulse: Vec<(u32, u32)> = writes(6).into_iter().chain(writes(9)).filter(|w| w.1 == 243).map(|w| (w.0, w.2)).collect();
    pulse.sort();
    assert_eq!(pulse, vec![(1611, 1), (2170, 0), (6810, 1), (7320, 0)], "the launch script's ch-243 pulses");
    // The leaf pairs: (stream 15, 16) the east door, (17, 18) the west door.
    let pose = |si: usize, at: f32| -> [f32; 3] {
        let s = &n.streams[si];
        let ks = &n.keys[s.first_key as usize..=s.last_key as usize];
        let k = ks.iter().rev().find(|k| k.time <= at + 0.5).unwrap_or(&ks[0]);
        [k.pos[0], k.pos[1], k.pos[2]]
    };
    let gap = |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    for (l, r, name) in [(15usize, 16usize, "east"), (17, 18, "west")] {
        assert_eq!(n.streams[l].last_key - n.streams[l].first_key, 9, "{name} leaf keys");
        // Open at the hand-back (the script starts one tick after it) and until 1000.
        for t in [0.0f32, 1000.0, 2580.0, 6310.0, 7710.0] {
            assert!(gap(pose(l, t), pose(r, t)) > 2.0, "the {name} door is open at {t} (gap {:.2})", gap(pose(l, t), pose(r, t)));
        }
        // Shut across each pulse: the leaves meet at 1350 and stay until 2230; 6660..7360.
        for t in [1350.0f32, 1611.0, 2170.0, 2230.0, 6660.0, 6810.0, 7320.0, 7360.0] {
            assert!(gap(pose(l, t), pose(r, t)) < 0.05, "the {name} door is shut at {t} (gap {:.2})", gap(pose(l, t), pose(r, t)));
        }
    }
    // Sister: the comparison can fail — at the open key the leaves are apart, so "shut" is false.
    assert!(gap(pose(17, 0.0), pose(18, 0.0)) >= 0.05);
}

/// The gated plane's SOLIDNESS law (level-05-basin-floor-3's port item 2). The game's gated hurt
/// object is skipped by the collision pick while its gate channel reads low and picked the tick
/// the channel rises: `Hurt_Create` 0x17e870's tail sets obj+0xF0 bit 0x20 at create when the
/// gate reads low (the channels start zeroed, so every gated volume loads disabled), `Hurt_Update`
/// 0x17e5d0:26-37 maintains that bit EVERY tick from the channel (low → set, high → cleared — the
/// same read the round-2 report pinned), and the pick skips bit-0x20 objects [M `Collide_Pick`
/// 0x1eb320:140's `& 0x20` arm; `Collide_PickObj` 0x1eae80:26-30 drops obj+0xF0 & 0x20]. So
/// HurtPoly-ShuttleLaunch #309 (gate 0xf3 = 243, 1000 damage [D params]) is a WALL the tick ch 243
/// rises and a ghost the tick it falls. The port holds the plane in the per-tick dynamic collision
/// set exactly while the channel reads high (`script_parts`'s hurt band, id bit 43) — the doors'
/// own per-tick set_dynamic precedent — and this test pins both directions on the world ray.
/// Probe point (−9.0, −173.0): the sister above proves the pit floor there lies inside the
/// volume's polygon (the pulse's contact kills a player standing on it), so a vertical ray
/// through that spot crosses the volume's own bands (y −24.08 … −21.32 [D the placement's tris]).
#[test]
fn the_shuttle_launch_plane_is_solid_only_while_ch_243_reads_high() {
    let mut scene = nf_view::load(&iso(), "07000016.bin", false).unwrap();
    let (band_lo, band_hi) = {
        let sim = scene.sim.as_ref().unwrap();
        let h = sim.hurts.iter().find(|h| h.gate == 0xf3).expect("HurtPoly-ShuttleLaunch #309 [D params]");
        assert!(h.disabled, "the volume loads with the 0x20 bit set — ch 243 starts low [M 0x17e870 tail]");
        (h.lo[1], h.hi[1])
    };
    eprintln!("[shelter solid] the volume's y band {band_lo:.2}..{band_hi:.2}");
    // The probe: a vertical world ray through the proven-inside spot. The pit floor there sits at
    // y −23.08 [L the old-code low run's stop, INSIDE the volume's own band range] — so the
    // distinguishing window runs from −20 (above the top band −21.32) down to −22.8, ABOVE the
    // floor: the volume's top band is the only thing that can stop it.
    let probe = |scene: &nf_view::scene::Scene| -> Option<f32> {
        let sim = scene.sim.as_ref().unwrap();
        sim.world.segment([-9.0, -20.0, -173.0], [-9.0, -22.8, -173.0]).map(|f| -20.0 - f * 2.8)
    };
    // LOW (the load state): the plane is a ghost — the ray flies through to the window's end.
    let low_stop = probe(&scene);
    eprintln!("[shelter solid] ch 243 low: ray stop at {low_stop:?}");
    assert!(low_stop.is_none(),
        "ch 243 low: the plane must not stop the ray above the pit floor (stop {low_stop:?}, band {band_lo:.2}..{band_hi:.2}) [M 0x17e870 tail + 0x1eb320:140]");
    // HIGH: the same ray stops inside the volume's band — the plane is solid.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(243, 1, sim.drones.frame);
    }
    // The hurt pass maintains `disabled` from the channel every tick [M 0x17e5d0:26-37]; run one
    // step so the bit clears and the set_dynamic rebuild takes the plane (the doors' order: the
    // hurt pass runs before the drones' per-tick set_dynamic).
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    let high_stop = probe(&scene);
    eprintln!("[shelter solid] ch 243 high: ray stop at {high_stop:?}");
    let high_y = high_stop.unwrap_or(f32::MIN);
    assert!(high_y >= band_lo - 0.1 && high_y <= band_hi + 0.1,
        "ch 243 high: the plane must stop the ray inside its band (stop {high_stop:?}, band {band_lo:.2}..{band_hi:.2}) [M Hurt_Update 0x17e5d0:26-37 clears the 0x20 bit; Collide_Pick 0x1eb320:140]");
    // LOW again: the falling gate re-parks the bit the same way [M 0x17e5d0:26-37 — the every-tick
    // level has no latch], the plane is a ghost again.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(243, 0, sim.drones.frame);
    }
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    {
        let sim = scene.sim.as_ref().unwrap();
        eprintln!("[shelter solid] after the low step: ch243 on={} disabled={} simdisabled={} hurts={}", sim.drones.channels.on(243), sim.hurts.iter().find(|h| h.gate == 0xf3).unwrap().disabled, sim.disabled, sim.hurts.len());
        eprintln!("[shelter solid] all 0xf3 hurts: {:?}", sim.hurts.iter().enumerate().filter(|(_,h)| h.gate==0xf3).map(|(k,h)| (k,h.disabled,h.shape_key,h.lo,h.hi)).collect::<Vec<_>>());
        eprintln!("[shelter solid] enabled drones {} total drones {} frame {}", sim.drones.list.iter().filter(|x| x.enabled).count(), sim.drones.list.len(), sim.drones.frame);
    }
    let low2_stop = probe(&scene);
    eprintln!("[shelter solid] ch 243 low again: ray stop at {low2_stop:?}");
    assert!(low2_stop.map_or(true, |y| y < band_lo - 0.1),
        "ch 243 low again: the plane must stop being solid [M 0x17e5d0:26-37 the every-tick law], stop {low2_stop:?}");
}

/// Runs the chain from the bridge trigger (the placed setup the timing sister uses — Bond put on
/// trigger #255, the touch fires ch 0x81), then from the hand-back steers Bond with Input only to
/// `target` through `via`. Returns (hp at the end of pulse 1 or at death, the volume's contact
/// count, Bond's position at the pulse's rise).
fn run_to(target: [f32; 2], via: &[[f32; 2]], delay: u32) -> (f32, u32, [f32; 3], bool) {
    let mut scene = nf_view::load_with_carry(&iso(), "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.pos = [2.4, 1.2, -175.5];
    }
    let mut back = false;
    let mut wp = 0usize;
    let mut rise_pos = None;
    let mut risen = false;
    let mut alive_inside_before_shut = false;
    let mut t_back = 0u32;
    for t in 0..4400u32 {
        let (p, yaw, disabled, on243) = {
            let s = scene.sim.as_ref().unwrap();
            (s.bond.pos, s.bond.yaw, s.disabled, s.drones.channels.on(243))
        };
        if !back && !disabled && p[1] < -20.0 { back = true; t_back = t; }
        if on243 && !risen { risen = true; rise_pos = Some(p); }
        if risen && !on243 { break; }
        if back && t == t_back + 1000 { alive_inside_before_shut = p[0] < -13.0; }
        let mut inp = Input::default();
        if back && t >= t_back + delay {
            let goal = if wp < via.len() { via[wp] } else { target };
            let (dx, dz) = (goal[0] - p[0], goal[1] - p[2]);
            if wp < via.len() && dx.hypot(dz) < 0.8 { wp += 1; }
            if dx.hypot(dz) > 0.4 {
                let mut d = dx.atan2(dz) - yaw;
                while d > std::f32::consts::PI { d -= 2.0 * std::f32::consts::PI; }
                while d < -std::f32::consts::PI { d += 2.0 * std::f32::consts::PI; }
                inp.turn = d.clamp(-0.08, 0.08);
                inp.forward = if d.abs() < 0.6 { 1.0 } else { 0.2 };
            }
        }
        scene.sim.as_mut().unwrap().step_bond(inp, false, false, false);
        if scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let hurt = sim.hurts.iter().find(|h| h.gate == 0xf3).expect("HurtPoly-ShuttleLaunch");
    (sim.drones.health.health, hurt.hits, rise_pos.expect("ch 243 rose"), alive_inside_before_shut)
}

/// The shelter, end to end on Input only: from the hand-back Bond runs west into the blast room
/// before its door shuts (script 1000..1350 [D]) and stands the whole pulse out untouched — the
/// volume never makes contact (its polygon excludes the room [D]). SISTERS: the same run stopping
/// on the open pit floor just outside the door dies in the pulse's first ticks, and a run that
/// starts too late finds the door shut.
#[test]
fn the_west_blast_room_shelters_bond_through_the_pulse_and_the_open_floor_does_not() {
    let via = [[-8.5f32, -173.5], [-11.0, -170.0]];
    let (hp, hits, at, inside) = run_to(SHELTER, &via, 0);
    eprintln!("[shelter] hp {hp} hits {hits} at the rise ({:.2},{:.2},{:.2}) inside-by-1000 {inside}", at[0], at[1], at[2]);
    assert!(inside, "Bond is inside the room before the door shuts (the hand-back + 1000)");
    assert!(at[0] < -13.0, "Bond stands inside the west room when ch 243 rises (x {:.2})", at[0]);
    assert_eq!(hits, 0, "the volume never touches Bond in the room — its polygon excludes it [D]");
    assert_eq!(hp, 100.0, "Bond lives through the pulse in the shelter");
    // SISTER: the open floor outside the door is inside the polygon — the same pulse kills him.
    let (hp2, hits2, at2, _) = run_to(OUTSIDE, &via[..1], 0);
    eprintln!("[shelter sister] hp {hp2} hits {hits2} at the rise ({:.2},{:.2},{:.2})", at2[0], at2[1], at2[2]);
    assert!(hits2 > 0 && hp2 <= 0.0, "the open pit floor outside the room is lethal in the pulse (hp {hp2}, hits {hits2})");
    // SISTER 2: the door really shuts. The same run started 1150 ticks after the hand-back
    // reaches the doorway after the leaves met (script 1350 [D]): the shut door (the leaf
    // entities' collision) keeps Bond out on the pit side of the door line (x ≈ −12.71 [D keys])
    // and the pulse kills him there.
    let (hp3, hits3, at3, _) = run_to(SHELTER, &via, 1150);
    eprintln!("[shelter late] hp {hp3} hits {hits3} at the rise ({:.2},{:.2},{:.2})", at3[0], at3[1], at3[2]);
    assert!(at3[0] > -12.8, "the shut door keeps a late Bond out (x {:.2})", at3[0]);
    assert!(hits3 > 0 && hp3 <= 0.0, "shut out, the pulse kills him (hp {hp3}, hits {hits3})");
}
