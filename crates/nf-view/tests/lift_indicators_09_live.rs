//! lift-indicators-09: 07000009's garage-lift floor indicators and call buttons — the port now
//! creates and draws them where the game does.
//!
//! The pieces [D level.nis keys + probe_lift_09.rs; M the interp chain]: the two lift cab
//! ScriptPlayers (placements #20, class 0xD9, mode 8, scripts 0x060007D/0x060007E) preload at
//! level start, run two ticks (`SP_LoadScript` 0x1dbc08's mode-8 arm) and hold on a stream
//! Event-3 negative delay until the cab activation; their keyed matrices persist through the
//! hold [M Script::update's delay gate, nis.rs]. Their SUB scripts place the visible chrome:
//! 0x0600064E s0 creates the floor-indicator plates 0x020006B7/B8/B9/BA (keys zero — positioned
//! by the SUB parent chain [M Script_Interp case 7]) and 0x0600064F s2 the call-button plates
//! 0x020006B2/B3 (keys constant (−4.62,−375.52,25.91) [D]). Each cab starts each sub three
//! times (`Script_SubScriptStart` 0x1db050 ops on streams 13-18 [D]), so the sim holds SIX
//! indicator plates and SIX button plates — three per shaft, the two shafts at x ≈ +1.25 and
//! −2.77 [D the composed positions]. The game draws them all at boot [L a fresh-boot
//! PCSX2 frame at --bond -3.45,-375.98,20.61,-6.124,-0.083 shows "L" above each lift door].
//!
//! The port's defect (fixed here): the draw slots are per-(script,stream,hash) with the copy
//! count taken from placed SPs whose OWN scripts contain the hash — every SUB hash read 1 — and
//! `Sim::script_entity_k` serves only the newest match, so one cab's plates drew and the other
//! cab's never did. The fix counts each SUB-op start across the script walk (scene.rs, the
//! `sub_copies` map), giving six slots per sub-entity hash.

#[path = "common/walker.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_and_run(ticks: u32) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(&iso(), "07000009.bin", false).expect("load 07000009");
    for _ in 0..ticks { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    scene
}

/// The sim's live entities of one gfx hash, with their matrices' translations.
fn live_plates(sim: &nf_view::scene::Sim, hash: u32) -> Vec<[f32; 3]> {
    sim.drones.scripts.objects.list.iter().flatten().filter_map(|o| match o {
        nf_game::nis::Obj::Entity { hash: h, matrix, .. } if *h == hash =>
            Some([matrix[3][0], matrix[3][1], matrix[3][2]]),
        _ => None,
    }).collect()
}

fn close(a: [f32; 3], b: [f32; 3], tol: f32) -> bool {
    (a[0] - b[0]).abs() < tol && (a[1] - b[1]).abs() < tol && (a[2] - b[2]).abs() < tol
}

/// The draw list carries six slots per sub-entity hash (three starts × two cab SPs [D]) and the
/// cab parts one slot per placed SP; every live plate is served by one slot at its keyed spot.
#[test]
fn the_indicator_and_button_plates_draw_at_the_game_positions() {
    let scene = load_and_run(30);
    let sim = scene.sim.as_ref().expect("sim");

    // The slot counts (scene.attachments, the script tuples (script, stream, hash, copy)).
    let slots = |hash: u32| -> Vec<u8> {
        scene.attachments.iter().filter_map(|a| a.script)
            .filter(|&(_, _, h, _)| h == hash).map(|(_, _, _, k)| k).collect()
    };
    let b9 = slots(0x0200_06B9);
    let b2 = slots(0x0200_06B2);
    assert_eq!(b9.len(), 6, "six 0x020006B9 indicator slots (3 sub starts × 2 cab SPs [D]), got {b9:?}");
    assert_eq!(b2.len(), 6, "six 0x020006B2 button slots, got {b2:?}");
    assert_eq!(b9, [0, 1, 2, 3, 4, 5], "the copy indices cover every live plate");
    assert_eq!(b2, [0, 1, 2, 3, 4, 5], "the copy indices cover every live plate");

    // The sim holds six of each (three per shaft [D the composed positions]).
    let indicators = live_plates(sim, 0x0200_06B9);
    let buttons = live_plates(sim, 0x0200_06B2);
    assert_eq!(indicators.len(), 6, "six live indicator plates [D 3 starts × 2 cabs], got {indicators:?}");
    assert_eq!(buttons.len(), 6, "six live button plates, got {buttons:?}");

    // The keyed spots [D the 0x060007D/E stream keys composed through the SUB parent chains;
    // the two shafts sit at x ≈ +1.25 and −2.77]: per cab one indicator above the lobby doors
    // (y −366.57), one above the garage doors (y −374.39), one at the lobby's door top
    // (y −367.02); per cab the call button on the garage floor beside the shaft (y −375.88,
    // z 25.91 [D 0x0600064F s2 keys]) and the two lobby-side plates (y −368.0).
    let expect_indicators: [[f32; 3]; 6] = [
        [1.22, -367.02, 26.71], [1.27, -374.39, 26.14], [1.27, -366.57, 26.14],
        [-2.79, -367.02, 26.71], [-2.75, -374.39, 26.14], [-2.74, -366.57, 26.14],
    ];
    for want in &expect_indicators {
        assert!(indicators.iter().any(|p| close(*p, *want, 0.05)),
            "an indicator plate at {want:?} (the keyed spot [D]), live at {indicators:?}");
    }
    let expect_buttons: [[f32; 3]; 6] = [
        [3.06, -375.88, 25.91], [3.09, -368.03, 25.91], [0.07, -368.04, 26.54],
        [-4.62, -375.88, 25.91], [-4.59, -368.04, 25.91], [-3.95, -368.04, 26.54],
    ];
    for want in &expect_buttons {
        assert!(buttons.iter().any(|p| close(*p, *want, 0.05)),
            "a call-button plate at {want:?} (the keyed spot [D 0x0600064F s2 keys]), live at {buttons:?}");
    }

    // The render serves EVERY live plate: for each sim object some slot k returns its matrix.
    let served = |hash: u32| -> bool {
        live_plates(sim, hash).iter().all(|want| (0..6usize).any(|k| {
            sim.script_entity_k((0x0600_064E, 0), hash, k).or_else(|| sim.script_entity_k((0x0600_064F, 2), hash, k))
                .map(|m| close([m[3][0], m[3][1], m[3][2]], *want, 0.001))
                .unwrap_or(false)
        }))
    };
    assert!(served(0x0200_06B9), "every live indicator plate has a draw slot at its matrix");
    assert!(served(0x0200_06B2), "every live button plate has a draw slot at its matrix");
}

/// Sister: what the game never creates at boot stays absent — the activation-FX panel
/// 0x020005EB exists only from SP #64's script t=16 on the E press [D the 0x0600059B nis dump:
/// Entity 0x020005EA @t0, 0x020005EB @t16, EntityEnd @t25], so before activation neither the
/// sim nor any draw slot's answer holds it.
#[test]
fn the_activation_fx_stays_absent_until_the_panel_press() {
    let scene = load_and_run(30);
    let sim = scene.sim.as_ref().expect("sim");
    assert!(!sim.drones.scripts.objects.list.iter().flatten()
        .any(|o| matches!(o, nf_game::nis::Obj::Entity { hash: 0x0200_05EB, .. })),
        "0x020005EB is created only at script t=16 [D the 0x0600059B nis dump]");
    assert!(sim.script_entity_k((0x0600_059B, 0), 0x0200_05EB, 0).is_none(),
        "no draw answer for the un-created FX entity");
    // And the button panel 0x020005EA IS live from load (t=0 [D]) at its baked-art spot
    // (identity matrix — the art is world-baked [D gfx bounds (−18.38,−368.53,24.83)..]).
    assert!(sim.drones.scripts.objects.list.iter().flatten()
        .any(|o| matches!(o, nf_game::nis::Obj::Entity { hash: 0x0200_05EA, .. })),
        "the panel entity exists from load [D t=0]");
    let m = sim.script_entity_k((0x0600_059B, 0), 0x0200_05EA, 0).expect("the panel draws");
    assert!(close([m[3][0], m[3][1], m[3][2]], [0.0, 0.0, 0.0], 0.001),
        "the panel's object matrix is identity (the art carries the world position [D])");
}
