//! 07000014 fusebox panel: the wires behind the front panel need draw slots.
//!
//! The report: "after the panel opens, the spot behind the panel doesn't render". Measured cause: the
//! fusebox placements (kind 0x31) carry their script in param 0 and `FuseBox_Create` runs it at load,
//! so the script's `Entity` streams DO spawn (the sim holds 0x02000AA2/AA1/AB3 at the box), but the
//! scene's script-object walk seeded only placed SPs (0xd9), switches (0x29), copters (0x2f) and the
//! blast scripts — a 0x31 script's entities never got an `Attachment { script: Some(..) }`, so nothing
//! drew them (`out/cap14_fb_before` vs `after`: the twin cable run below the box appears only with the
//! seed). The fix seeds every 0x31 placement's param-0 script and counts each fusebox as a copy.
//!
//! Fail-first: on pristine origin/master the slot list is empty (assert fires); with the seed it holds
//! one slot per live copy.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_fusebox_scripts_entities_have_draw_slots() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    // The slots are built with the scene; the sim's entities spawn on the fusebox script's keys, so
    // let the script reach them before asking where they are.
    for _ in 0..180 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().expect("sim");
    let slots: Vec<(u32, u8, u32, u8)> = scene.attachments.iter()
        .filter_map(|a| a.script)
        .filter(|(s, _, h, _)| *s == 0x0600_0a1b && (0x0200_0aa0..=0x0200_0b00).contains(h))
        .collect();
    assert!(!slots.is_empty(),
        "the fusebox script 0x06000A1B's entity streams have no draw slot — the recess behind the \
         opened panel renders empty; the seeded 0x31 placements must reach the script-object walk");
    // The slots draw from the sim's live entities: the wires exist at the box (-89.30, -2.22, 49.36).
    let mut at_box = 0;
    for o in sim.drones.scripts.objects.list.iter().flatten() {
        if let nf_game::nis::Obj::Entity { hash, matrix, .. } = o {
            if (0x0200_0aa0..=0x0200_0b00).contains(hash)
                && (matrix[3][0] + 89.30).abs() < 0.5
                && (matrix[3][2] - 49.36).abs() < 0.5 {
                at_box += 1;
            }
        }
    }
    assert!(at_box > 0, "the wire entities exist at the fusebox's own position (the draw source)");
}

/// Sister: the seed is scoped to the level's own fusebox data — a level with no 0x31 script in that
/// range gets no slots for it (the walk does not invent slots).
#[test]
fn the_fusebox_seed_does_not_create_slots_on_other_levels() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000004.bin", false).expect("load 07000004");
    let slots = scene.attachments.iter().filter(|a| a.script.is_some_and(|(s, _, _, _)| s == 0x0600_0a1b)).count();
    assert_eq!(slots, 0, "07000004 has no fusebox running 0x06000A1B — no slots may appear for it");
}
