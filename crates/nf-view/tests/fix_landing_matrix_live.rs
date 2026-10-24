//! fix-landing-matrix: the 0700004a −52 landing cut-scene's event-6 handback matrix, resolved —
//! TWICE. The first reading (base-composed, the commit history) was REFUTED by the live audit:
//!
//! - **[L fresh boot, PCSX2]** the re-enable lands Bond at the marker key's OWN x/z
//!   **(1.7994, −3.4534)** with y on the held platform (the ride's root −54.13) — NOT the
//!   composed (1.799, −107.662, −6.830), which is a void on every map's collision [D all-map
//!   scan, level-4a-lift-3].
//! - **[D 0780004a.bin 0x060006cf]** the marker stream s25 is the UNPARENTED entity
//!   **hash 00000000, obj_type 0x19** whose single key at t575 is (1.799, −54.128, −3.453).
//! - **[D census, probe_entities_l4]** the hash-0 kind-0x19 class is the marker class: the only
//!   hash-0 entities on 07000001/0700004a's own scripts are kind 0x31 (which keep the compose —
//!   07000001's pack script 0x06000087 s11 drives the exit_aim walk). The port skips the script
//!   base for hash-0 kind-0x19 entities only (nis.rs interp case 1).
//! - **[M disasm 0x1d9ab4 + 0x1d9c10]** the base-concat 10th arg = `(stream parent == -1)` — the
//!   compose DOES fire for unparented streams in the decompile's reading; the hash-0 marker
//!   class's raw-key behavior is the [L]-pinned exception (the game-side discriminator — the
//!   hash, the kind, or the SP mode — is the named live check).
//!
//! The test pins the RAW key through the real path (the ride → the scene → the enable).
//! Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The platform entity's keyed y (the ride harness's helper, fix_finale_live).
fn platform_y(sim: &nf_view::scene::Sim) -> Option<f32> {
    sim.drones.scripts.objects.list.iter().flatten().find_map(|o| match o {
        nf_game::nis::Obj::Entity { hash, matrix, .. } if *hash == 0x0200_05b4 => Some(matrix[3][1]),
        _ => None,
    })
}

/// The disc's own numbers: the SP #289 base (the placement translation [D]) and the marker's
/// key 1409 [D 0780004a.bin]. The COMPOSED sum is what the live run REFUTED.
const BASE: [f32; 3] = [0.0, -53.53418, -3.37652];
const KEY: [f32; 3] = [1.799, -54.128, -3.453];

#[test]
fn the_landing_handback_matrix_is_the_raw_marker_key() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700004a.bin", false).expect("load 0700004a");
    // The [D] decode pins: the marker stream is unparented, hash 0, kind 0x19, its key the disc's own.
    {
        let sim = scene.sim.as_ref().unwrap();
        let nis = sim.drones.scripts.lib.get(&0x0600_06cf).expect("the landing script loads from the pack [D]");
        let (si, st) = nis.streams.iter().enumerate()
            .find(|(_, s)| s.decode().iter().any(|c| matches!(c, nf_formats::nis::Cmd::Entity { obj_type: 0x19, .. })))
            .expect("the obj_type-0x19 marker stream exists [D]");
        for c in st.decode() {
            if let nf_formats::nis::Cmd::Entity { hash: 0, parent, obj_type: 0x19, .. } = c {
                assert_eq!(parent, None, "the marker stream is unparented [D 0x060006cf s{si}]");
            }
        }
        let k = &nis.keys[st.first_key as usize];
        assert!((k.pos[0] - KEY[0]).abs() < 0.01 && (k.pos[1] - KEY[1]).abs() < 0.01 && (k.pos[2] - KEY[2]).abs() < 0.01,
            "the marker's key at t575 is the disc's own [D 0780004a.bin], got {:?}", k.pos);
    }
    // Board and ride to the landing scene's event-6 tick (the fix_finale ride shape).
    let mut landed: Option<[f32; 3]> = None;
    {
        let mut scene2 = nf_view::load(&iso, "0700004a.bin", false).expect("load 0700004a");
        {
            let sim = scene2.sim.as_mut().unwrap();
            sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, -7.0, -3.4], 0.0);
        }
        for t in 0..2600u32 {
            scene2.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            {
                let sim = scene2.sim.as_mut().unwrap();
                sim.drones.health.health = 100.0;
                if t % 30 == 0 && t < 1600 {
                    if let Some(y) = platform_y(sim) { sim.bond.pos = [3.0, y + 1.2, -3.4]; }
                }
            }
            // The enable lands between t2187 (the scene's end) and the next re-seat-less tick:
            // once the ride's ch-0x40 window has passed (t1612) the re-seat stops, so Bond's
            // position after t2187 is the enable's own placement.
            if t >= 2190 && landed.is_none() {
                let p = scene2.sim.as_ref().unwrap().bond.pos;
                if p[0] < 2.0 { landed = Some(p); }
            }
            if landed.is_some() { break; }
        }
    }
    let p = landed.expect("the enable places Bond at the handback matrix right after the scene's end (t2187)");
    // The [L] pin: the RAW key's x/z (the enable re-arms every tick while the scene's snapshot
    // stands, so the ride's enable-tick position reads the marker key directly). The composed
    // value (base + key = (1.799, −107.66, −6.83)) is the pre-live reading — refuted [L].
    assert!((p[0] - KEY[0]).abs() < 0.2 && (p[2] - KEY[2]).abs() < 0.2,
        "the handback is the RAW marker key [L fresh boot: x/z (1.7994, −3.4534)], got {p:?}");
    assert!((p[1] - (BASE[1] + KEY[1])).abs() > 1.0,
        "the snapshot does NOT carry the SP base ({:.3} vs the composed {:.3}) [L; D all-map scan voids it]",
        p[1], BASE[1] + KEY[1]);
}
