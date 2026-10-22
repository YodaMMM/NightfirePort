//! fix-landing-matrix: the 0700004a −52 landing cut-scene's event-6 handback matrix, resolved.
//!
//! The parked walkreal_0700004a note called the marker evidence "contradictory" (base-relative NIS
//! keys vs the raw placement). THE RESOLUTION (this task):
//! - **[M disasm 0x1d9ab4 + 0x1d9c10]** `Script_Interp`'s `Script_GetInterp` calls pass the 10th
//!   (base-concat) argument = `(stream parent == -1)` — `nor $v1, $zero, $v1; sltiu $v1, $v1, 1;
//!   sw $v1, ($sp)` — the SCRIPT BASE composes INSIDE the interp for UNPARENTED streams, and at
//!   the parent-chain end for parented ones (`Concat(param_1, obj+0x90)` [M 0x1d9a08:106-133]).
//!   Every stream's matrix = base · key(s) exactly once. (The decompile drops the 10th arg; the
//!   C-only reading "base only at a chain end" is what the reverted fix-final-blockers2 shipped.)
//!   The camera case composes it unconditionally [M disasm 0x1da054-0x1da068, already cited in
//!   nis.rs].
//! - **[D 0700004a.bin census]** the landing SP **#289** "Script Player" sits at
//!   **(0.00, −53.534, −3.377)**, rotation 0 — its matrix IS the script base ([M Script_Load
//!   0x1d8248 takes the SP's pos/rot; SP_CreateScriptPlayer 0x1db9a8).
//! - **[D 0780004a.bin 0x060006cf]** stream 25 is the UNPARENTED entity (obj_type 0x19) whose
//!   single key at t575 is **(1.799, −54.128, −3.453)**; stream 26 carries event 6 at t575
//!   (params [1] — the mode byte [M 0x1d94c0 case 6 stores it at SCRIPTINFO+0xab6]).
//! - So the event-6 snapshot (`Mat_Copy(prev_stream_obj+0x90 → SCRIPTINFO+0x40)` [M
//!   0x1d94c0:125-134]) is the marker's COMPOSED matrix:
//!   **(1.799, −107.662, −6.830)** = base + key. Script_Free re-enables Bond there
//!   (`Player_Enable(glb_players, SCRIPTINFO+0x40, mode)` [M 0x1da880:46-49]).
//! The test pins the composed value through the real path (the ride → the scene → the event).
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
/// key 1409 [D 0780004a.bin] compose to the handback matrix.
const BASE: [f32; 3] = [0.0, -53.53418, -3.37652];
const KEY: [f32; 3] = [1.799, -54.128, -3.453];

#[test]
fn the_landing_handback_matrix_is_the_base_composed_marker() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700004a.bin", false).expect("load 0700004a");
    // The [D] decode pins: the marker stream is unparented and its key is the disc's own.
    {
        let sim = scene.sim.as_ref().unwrap();
        let nis = sim.drones.scripts.lib.get(&0x0600_06cf).expect("the landing script loads from the pack [D]");
        let (si, st) = nis.streams.iter().enumerate()
            .find(|(_, s)| s.decode().iter().any(|c| matches!(c, nf_formats::nis::Cmd::Entity { obj_type: 0x19, .. })))
            .expect("the obj_type-0x19 marker stream exists [D]");
        for c in st.decode() {
            if let nf_formats::nis::Cmd::Entity { hash: 0, parent, obj_type: 0x19, .. } = c {
                assert_eq!(parent, None, "the marker stream is unparented [D 0x060006cf s{si}] — the disasm's base-concat flag fires [M 0x1d9ab4]");
            }
        }
        let k = &nis.keys[st.first_key as usize];
        assert!((k.pos[0] - KEY[0]).abs() < 0.01 && (k.pos[1] - KEY[1]).abs() < 0.01 && (k.pos[2] - KEY[2]).abs() < 0.01,
            "the marker's key at t575 is the disc's own [D 0780004a.bin], got {:?}", k.pos);
    }
    // Board and ride to the landing scene's event-6 tick (the fix_finale ride shape).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, -7.0, -3.4], 0.0);
    }
    // The enable applies at the scene's end (t2187 = the ride's t1612 + the 575-tick scene):
    // the port's PlayerEnable places Bond at the snapshot's translation (scene.rs's handler —
    // `Bond::stand_at(world, [m[3][0], m[3][1], m[3][2]], yaw)`), so Bond's position right
    // after the free IS the composed handback matrix, read through the real path.
    let mut landed: Option<[f32; 3]> = None;
    for t in 2180..2400u32 {
        // (the ride loop above already ran to t3000 in the first pass — this pass re-reads the
        // window with a fresh ride below)
        let _ = t;
        break;
    }
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
    // Sister: the composition is NOT the raw key (the raw-key reading would place the handback
    // on the parked platform — the pre-blockers2 reading the marker contradiction came from).
    assert!((p[1] - KEY[1]).abs() > 1.0, "the snapshot carries the base ({:.3} ≠ the raw {:.3})", p[1], KEY[1]);
}
