//! audit-mp-panes — the MPGame+slot·0x30+0x26 status bits the port's rules write, against
//! `MP_setPlayerStatus` 0x18a518's writers (headless, numbers only). The bit map:
//! bit 0x1 the CTF flag carrier [M MP_FlagUpdate 0x1861a8:178-179], bit 0x2 the blueprint
//! carrier [M MP_BluePrintUpdate 0x1882d8:126-129], bits 0x4/0x8 the GoldenEye kind-6/7 item
//! carriers [M MP_GoldenEyeUpdate 0x187568:231-236], bit 0x10 on the KOTH hill [M
//! MP_KOHUpdate 0x186ae0:113-116]. Every assertion pairs a sister that fails if the bit were
//! the wrong one (the pre-fix port set the FLAG bit 0x1 on a blueprint touch).

use nf_game::drone::Rng;
use nf_game::mp::{scenario, MpMatch, MpSettings};

fn make(scenario: u32) -> MpMatch {
    let mut s = MpSettings::deathmatch(0x07000011);
    s.scenario = scenario;
    let mut m = MpMatch::init(s);
    m.start(&mut Rng::new(1));
    m
}

fn join(m: &mut MpMatch, slot: usize, team: u32) {
    m.settings.players[slot].team = team;
    m.set_slot(slot, [10.0 + 10.0 * slot as f32, 0.0, 0.0]);
}

/// The blueprint touch writes the BLUEPRINT bit 0x2 [M 0x1882d8:126-129 — MP_setPlayerStatus
/// (carrier, 2, 0x32, .., clear=0)], not the flag's 0x1. Sister: the flag bit stays clear and
/// the other slot carries nothing.
#[test]
fn blueprint_touch_sets_the_blueprint_bit() {
    let mut rng = Rng::new(7);
    let mut m = make(scenario::BLUEPRINT);
    join(&mut m, 0, 0);
    join(&mut m, 1, 1);
    m.register_mp_object([40.0, 1.0, 40.0], [0.0, 0.0, 1.0], 5, [5, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], 4, [4, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([41.0, 1.0, 41.0], [0.0, 0.0, 1.0], 4, [4, 1, 0, 0, 0, 0, 0], 0);
    m.create_mode_objects(&mut rng);
    assert!(m.objects_rt.blueprint.is_some());
    m.touch_blueprint(0);
    assert_eq!(m.game.slots[0].status_bits & 0x2, 0x2, "bit 0x2 sets [M 0x1882d8:126-129]");
    assert_eq!(m.game.slots[0].status_bits & 0x1, 0, "sister: the FLAG bit 0x1 stays clear");
    assert_eq!(m.game.slots[1].status_bits, 0, "sister: the other slot carries nothing");
}

/// The carrier's death drops the item and clears its own bit off the victim's word [M
/// 0x1882d8:81-84 msg 0x33 through MP_setPlayerStatus 0x18a518:88-96]. Sister: a death with
/// nothing carried leaves the word alone.
#[test]
fn blueprint_drop_clears_the_carriers_bit() {
    let mut rng = Rng::new(7);
    let mut m = make(scenario::BLUEPRINT);
    join(&mut m, 0, 0);
    join(&mut m, 1, 1);
    m.register_mp_object([40.0, 1.0, 40.0], [0.0, 0.0, 1.0], 5, [5, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], 4, [4, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([41.0, 1.0, 41.0], [0.0, 0.0, 1.0], 4, [4, 1, 0, 0, 0, 0, 0], 0);
    m.create_mode_objects(&mut rng);
    m.touch_blueprint(0);
    assert_eq!(m.game.slots[0].status_bits & 0x2, 0x2);
    m.player_killed(0, Some(1), false, false, &mut rng);
    assert_eq!(m.game.slots[0].status_bits & 0x2, 0, "the drop clears bit 0x2 [M 0x1882d8:81-84]");
    assert_eq!(m.game.slots[0].status_bits & 0x1, 0, "sister: the flag bit never lit");
    // sister 2: a kill with nothing carried changes no bits
    let mut m2 = make(scenario::BLUEPRINT);
    join(&mut m2, 0, 0);
    join(&mut m2, 1, 1);
    m2.register_mp_object([40.0, 1.0, 40.0], [0.0, 0.0, 1.0], 5, [5, 0, 0, 0, 0, 0, 0], 0);
    m2.register_mp_object([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], 4, [4, 0, 0, 0, 0, 0, 0], 0);
    m2.register_mp_object([41.0, 1.0, 41.0], [0.0, 0.0, 1.0], 4, [4, 1, 0, 0, 0, 0, 0], 0);
    m2.create_mode_objects(&mut rng);
    m2.player_killed(0, Some(1), false, false, &mut rng);
    assert_eq!(m2.game.slots[0].status_bits, 0);
}

/// The GoldenEye item touch writes the item's kind bit: kind 6 (the port's goldeneye[0]) →
/// 0x4, kind 7 (goldeneye[1]) → 0x8 [M 0x187568:231-236 — msgs 0x35/0x36, clear=0]. Sister:
/// neither touch leaks onto the other slot or the flag bit.
#[test]
fn goldeneye_touch_sets_the_item_kind_bits() {
    let mut rng = Rng::new(7);
    let mut m = make(scenario::GOLDENEYE);
    join(&mut m, 0, 0);
    join(&mut m, 1, 1);
    // one item of each kind (types 6/7 [M 0x185de8 cases 6-7])
    m.register_mp_object([40.0, 1.0, 40.0], [0.0, 0.0, 1.0], 6, [6, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([60.0, 1.0, 60.0], [0.0, 0.0, 1.0], 7, [7, 0, 0, 0, 0, 0, 0], 0);
    m.create_mode_objects(&mut rng);
    assert!(m.objects_rt.ge[0].is_some() && m.objects_rt.ge[1].is_some(), "both items registered");
    m.touch_ge_item(0, 0);
    assert_eq!(m.game.slots[0].status_bits & 0x4, 0x4, "kind-6 item → bit 0x4 [M 0x187568:231-236]");
    assert_eq!(m.game.slots[0].status_bits & 0x8, 0, "sister: the kind-7 bit stays clear");
    m.touch_ge_item(1, 1);
    assert_eq!(m.game.slots[1].status_bits & 0x8, 0x8, "kind-7 item → bit 0x8");
    assert_eq!(m.game.slots[1].status_bits & 0x4, 0, "sister: slot 1 carries no kind-6 item");
    // the death drop clears the victim's own item bit [M 0x187568:329-336]
    m.player_killed(0, Some(1), false, false, &mut rng);
    assert_eq!(m.game.slots[0].status_bits & 0x4, 0, "the drop clears bit 0x4");
    assert_eq!(m.game.slots[1].status_bits & 0x8, 0x8, "sister: slot 1 still carries");
}
