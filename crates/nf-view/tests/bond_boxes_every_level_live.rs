//! bond-boxes-every-level: Bond's 16 hit boxes exist on EVERY story level.
//!
//! `Player_Init` 0x1910f8:94-108 resolves the player's skin hash off the level id and hands it to
//! `AnimObjectNew` — the hashtable always holds it, so the game's Bond is never skinless. The
//! disc carries each level's own player skin in its own archive (0x05000009 in 01-04, 0x050000d
//! in 05-08, 0x0500007c in 16/1b/43, 0x05000007 in the rest [D nf-extract bin-list census, kind-3
//! heads; every 0x05000007 carrier's record is byte-identical, sha1 04709295…]), and the loader
//! merges the one missing record where an archive lacks it (the 07000048 front-end case, carried
//! by the session per [M AnimLoadFile 0x1bdf40 — nothing frees skins]). This test walks the whole
//! story list and asserts the boxes exist everywhere, through the real `nf_view::load` path.
//!
//! The part ids are the disc's `HumanCollisionBones` 0x2c6be8 table order [D] (bone a of the 16
//! entries) — `Collide_Jointy` 0x1ec180 walks them in this order.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_game::mission;

/// The disc's part ids in `HumanCollisionBones` 0x2c6be8 table order [D 0x2c6be8, ACTION.ELF file
/// offset 0x1c0ae8; bone a of each of the 16 entries, the terminator is entry 16].
const DISC_PARTS: [i16; 16] = [5, 4, 53, 54, 55, 56, 49, 50, 51, 52, 20, 21, 23, 32, 35, 39];

#[test]
fn bond_has_16_boxes_on_every_story_level() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut checked = 0usize;
    let mut skipped: Vec<(String, String)> = Vec::new();
    for def in mission::LEVELS {
        let archive = format!("{:08x}.bin", def.level);
        let mut scene = match nf_view::load(&iso, &archive, false) {
            Ok(s) => s,
            Err(e) => { skipped.push((archive.clone(), format!("load: {e}"))); continue; }
        };
        let Some(sim) = scene.sim.as_mut() else {
            skipped.push((archive.clone(), "no sim (no playable bond start)".into())); continue;
        };
        let boxes = sim.bond_hit_boxes();
        let sk = nf_game::damage::player_skin_hash(def.level);
        assert_eq!(boxes.len(), 16,
            "{archive}: Bond's 16 body boxes must exist (player skin {sk:08x} [M Player_Init 0x1910f8:94-108], got {} boxes — the skin table has {})", boxes.len(), sim.skins.len());
        let parts: Vec<i16> = boxes.iter().map(|b| b.part).collect();
        assert_eq!(parts, DISC_PARTS,
            "{archive}: the box order is the disc's HumanCollisionBones table [D 0x2c6be8]");
        checked += 1;
    }
    eprintln!("bond boxes: {checked} story levels asserted, {} skipped: {:?}", skipped.len(), skipped);
    assert!(checked >= 20, "the story walk must cover the campaign levels, got {checked}");
    // No silent skips: every skip is a level the loader cannot enter as Bond (the driving
    // sections' car-only starts), printed above — none may be a skin miss.
    for (name, why) in &skipped {
        assert!(!why.contains("skin"), "{name}: a skip must never be a missing player skin ({why})");
    }
}
