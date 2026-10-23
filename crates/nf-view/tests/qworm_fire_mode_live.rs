//! qworm-firemode-softlock-0a: the fire-mode press on a gadget with no second mode moves nothing,
//! and switching away from the Q-Worm completes — the game's shape for an observed soft-lock.
//!
//! Game behavior [M `Player_Weapon` 0x1a24c8:188-231]: the mode press (`Input_Action(slot, 0xc, 4)`;
//! the watch pair 0x54/0x55 ORs the fire edge :189-192) with the view idle (:198) runs:
//! - the twin branch when `weapon_data[w]+0x05 != 0` (s8): `new = w + twin`, gated by
//!   `Player_WeaponHasAmmo` 0x1a16c0, written to `obj+0x63` (the switch request) [:209-228];
//! - the mode-byte branch otherwise, ONLY when the record's count `+0x2E != 1` [:210-215].
//! The Q-Worm 0x58 record carries `+0x05 = 0`, `+0x2E = 1`, `+0xB8 = 0`, `+0xBC = 0`
//! [L ee_start_0700000a.bin @ 0x2bf150 + 0x58·0x10C] — the press writes nothing: no request, no
//! mode byte, and the mode-anim block (0x1a24c8:113-151) never runs. `Player_WeaponChange`
//! 0x1a6f80's refusals — the view states 1/3/5-8/9/0xb/0xd-0x10 (:25-47) and the aim bit0 of
//! `BL+0x60` (:62-64) — are not set by such a press, so the switch completes.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn tick(scene: &mut nf_view::scene::Scene, n: u32) {
    if let Some(sim) = scene.sim.as_mut() {
        for _ in 0..n { sim.step_bond(Default::default(), false, false, false); }
    }
}

fn load(iso: &str) -> nf_view::scene::Scene {
    nf_view::load_with_carry(iso, "0700000a.bin", false, None).expect("load 0700000a")
}

fn equip_qworm(scene: &mut nf_view::scene::Scene) {
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.inventory.owned[0x58] = true;
        sim.drones.weapon_switch_request = Some(0x58);
    }
    tick(scene, 300);
}

/// The Q-Worm's record has no second mode for the mode arm to act on, and the P2K (the sister)
/// has one: the sister test below can land [L ee_start_0700000a.bin @ 0x2bf150 + w·0x10C].
#[test]
fn the_q_worm_record_has_no_second_mode_and_the_p2k_has_one() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load_with_carry(&iso, "0700000a.bin", false, None).expect("load 0700000a");
    let sim = scene.sim.as_ref().unwrap();
    let qw = sim.weapons.records.get(0x58).unwrap();
    assert_eq!(qw[5], 0, "Q-Worm +0x05 (s8 twin) = 0: the mode arm takes the mode-byte branch [L]");
    assert_eq!(qw[0x2e], 1, "Q-Worm +0x2E (mode count) = 1: the count-1 guard makes the press a no-op [M 0x1a24c8:210-215]");
    assert_eq!(u32::from_le_bytes(qw[0xb8..0xbc].try_into().unwrap()), 0, "Q-Worm +0xB8 = 0: the mode arm's anim block never runs [M 0x1a24c8:117-119]");
    assert_eq!(u32::from_le_bytes(qw[0xbc..0xc0].try_into().unwrap()), 0, "Q-Worm +0xBC = 0: no reverse anim either [L]");
    let p2k = sim.weapons.records.get(0x02).unwrap();
    assert_eq!(p2k[5] as i8, 1, "P2K +0x05 (s8 twin) = +1: a real second mode exists for the sister [L]");
}

/// The repro sequence: the Q-Worm out, the fire-mode press, then switching away completes.
/// Sister within: the P2K's mode press still toggles onto its twin (a real second mode) [M].
#[test]
fn mode_press_on_the_q_worm_moves_nothing_and_the_switch_completes() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso);
    tick(&mut scene, 30);
    equip_qworm(&mut scene);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.gun.weapon, 0x58, "the Q-Worm equipped");
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input { mode_switch: true, ..Default::default() }, false, false, false);
        assert_eq!(sim.gun.switch_request, 0x58, "no twin: no switch request [M 0x1a24c8:209-228's twin branch needs +0x05 != 0]");
        assert_eq!(sim.drones.inventory.fire_mode[0x58], 0, "count 1: the mode byte does not move [M 0x1a24c8:210-215]");
    }
    tick(&mut scene, 120);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.gun.weapon, 0x58, "still the Q-Worm after the press");
        assert_eq!(sim.gun.switch_state, 0, "the switch machine never left idle");
    }
    // Switch away through the request lane (obj+0x63), then back through the number-key lane.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.weapon_switch_request = Some(6);
    }
    tick(&mut scene, 300);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.gun.weapon, 6, "the switch away completes after the Q-Worm mode press");
        assert_eq!(sim.gun.switch_state, 0, "and settles idle");
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.weapon_switch_request = Some(0x58);
    }
    tick(&mut scene, 300);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input { switch_to: Some(0x43), ..Default::default() }, false, false, false);
    }
    tick(&mut scene, 300);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.gun.weapon, 0x43, "the number-key switch away completes too");
    }
}

/// Sister: a weapon with a real second mode still toggles — the P2K's press requests the
/// silenced twin 0x03 and lands on it [M 0x1a24c8:209-228 with +0x05 = 1; 0x1a16c0's shared
/// ammo type passes].
#[test]
fn a_weapon_with_a_real_second_mode_still_toggles() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso);
    tick(&mut scene, 30);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.gun.weapon = 2;
        sim.gun.switch_request = 2;
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
    }
    tick(&mut scene, 200);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.gun.switch_state, 0, "the forced P2K settled idle before the press");
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input { mode_switch: true, ..Default::default() }, false, false, false);
        assert_eq!(sim.gun.switch_request, 0x03, "the P2K's press requests the silenced twin 0x03 [M 0x1a24c8:209-228]");
    }
    tick(&mut scene, 200);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.gun.weapon, 0x03, "the twin switch completes");
        assert_eq!(sim.gun.switch_state, 0, "and settles idle");
    }
}
