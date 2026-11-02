//! door-scan-skip-bit-2: the player's `obj+0xF0` bit 0x10 through its RUNTIME life, on the real
//! 07000001 boot. The boot value is SET (obj+0xF0 = 0x890 [L ee_start_07000001.bin]; the
//! `Player_Init` 0x1910f8:115 → `AnimObjectNew` → `hashtable_set_object_to_entity_gfx` →
//! `Control_SetGList` 0x134960:12-13 glist copy), but the game's own runtime writers flip it:
//! `Player_Disable(…,0)` 0x195f70:17 sets it (script event 4 [M 0x1d94c0:119], the NIS handler
//! [M 0x205f10:105]) and `Player_Enable` 0x195498:53,77 clears it — so from the intro's hand-back
//! (`Script_Free` 0x1da880:44-48) to level end the game's Bond reads bit-CLEAR and the
//! `Door_Update` 0x1363c8 auto-close scan SEES him. The boot-only model (the previous task's)
//! kept him invisible for the whole level and stalled the escort at the interior shoji doors.
//! Opt-in: NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use walker::{Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The real 07000001 boot: the skip bit is SET from the first tick (the boot board 0x890 [L]),
/// and the intro's `Player_Enable` hand-back CLEARS it [M 0x195498:53,77] — after which it stays
/// clear in play (the game's `Player_ChangeSubState` 0x190630:35-40 clears it on every substate
/// change too; the port's one modelled site is the Cam2Mode hand-back).
#[test]
fn the_player_skip_bit_is_a_boot_value_the_intro_hand_back_clears() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    w.tick(Default::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.bond_door_skip,
            "07000001 boot: the player obj+0xF0 = 0x890, bit 0x10 SET [L ee_start_07000001.bin]");
    }
    // The intro cut-scene writes ch 0xB (the contact's enable channel) — then the tail's event 6
    // re-places Bond and `Script_Free` re-enables him (the fix_npc_doors boot sequence).
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    assert!(enabled_at.is_some(), "the intro must write ch 0xB (the contact's enable channel)");
    w.wait_enabled(6000, "the intro tail (event 6 re-places Bond)");
    w.wait(300);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.bond_door_skip,
            "the intro's `Player_Enable` 0x195498:53,77 must clear the skip bit — the game's \
             in-play Bond (bit CLEAR) is visible to the Door_Update 0x1363c8 auto-close scan");
    }
    // And it stays clear in play: nothing but a script disable sets it [M 0x195f70:17].
    w.wait(600);
    assert!(!w.scene.sim.as_ref().unwrap().drones.bond_door_skip,
        "the bit stays clear during normal play");
}
