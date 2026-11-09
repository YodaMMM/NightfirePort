//! pain-generic-gate: the game's health gate on every damage path — `Player_HandlePain`
//! 0x1902c8 returns on health <= 0 BEFORE writing anything, and the port's `pain_generic`
//! wrote `dir_bits` (BL+0x967) before that gate, changing state on a dead or zero-health
//! player.
//!
//! Evidence base:
//! - `Player_HandlePain` 0x1902c8's gate order [M decompile :14-26 + disasm 0x1902f8-0x190378]:
//!   `switch_channels[0x5f]` set → return; health `BL+0x894` <= 0 → return (0x190318-0x19032c,
//!   `c.ole.s`/`bc1t` to 0x190604); damage <= 0 → return; (MP) gameflow state != 2 → return.
//!   The FIRST store of the body is `BL+0x8bc = 1.0` at 0x190378 — a gated pain writes
//!   NOTHING: no `+0x967`, no `+0x968` glow, no armour/health, no 1-in-4 grunt draw
//!   (`Rand_Rand(4)` sits at :110-119, inside the body).
//! - The generic `+0x967` write (:121-125: `0xC`, or 2 for pain type 6) sits AFTER the
//!   gates — the old `pain_generic` set it before calling the gated `apply`.
//! - The bullet arm is DIFFERENT by design [M `Player_DealWithObjHit` 0x194458:195-226]:
//!   `Player_HandlePain` runs first (gated), then `+0x967` is written from the attacker's
//!   position OUTSIDE HandlePain, unconditionally — so a bullet on a dead body still writes
//!   the direction bits in the game. The port's `pain_from` mirrors that (the pre-write is
//!   the same net store); it is NOT part of this fix.

use nf_game::bullet::PlayerHealth;
use nf_game::drone::Rng;

/// A pain on a zero-health player writes NOTHING [M 0x1902c8:18-20 the gate precedes every
/// store]. The old port set `dir_bits` before the gate — the generic pain types (7 = drown,
/// 0 = blast/hurt-volume/live-wire) both land here with the body's 0xC.
#[test]
fn pain_generic_writes_nothing_on_a_zero_health_player() {
    let mut h = PlayerHealth::default();
    h.health = 0.0;
    h.armour = 25.0;
    h.flash = 40.0;
    h.dir_bits = 0x4; // a live directional store from the killing bullet [M 0x194458:203-226]
    let mut rng = Rng::new(11);
    let draws_before = rng.draws;
    // The drown arm (type 7) on the corpse: the game returns at the health gate.
    h.pain_generic(&mut rng, 2.0, 1, 0x0700_0002, 7);
    assert_eq!(h.dir_bits, 0x4, "no +0x967 write on a dead player [M 0x1902c8:18-20 vs :121-125]");
    assert_eq!(h.health, 0.0);
    assert_eq!(h.armour, 25.0, "no armour store [M the gate precedes 0x1904ec's MIN]");
    assert_eq!(h.flash, 40.0, "no +0x968 glow store [M :126-130 sits after the gate]");
    assert_eq!(rng.draws, draws_before, "no 1-in-4 grunt draw on a gated pain [M :110-119 is body code]");
}

/// Same pin at the exact death boundary: health below 1 clamps to 0 (the kill), and the NEXT
/// pain — what a continuing hazard delivers — must write nothing.
#[test]
fn the_pain_after_the_killing_pain_writes_nothing() {
    let mut h = PlayerHealth::default();
    let mut rng = Rng::new(3);
    // The kill: a 200-damage type 0 blast on a 100-health, 0-armour player — 05's difficulty-1
    // factor 0.6 [L ReadTuningVars 0x1d3c44] lands 120.0 ≥ 100 → the floor clamps to 0.0.
    h.health = 100.0;
    h.pain_generic(&mut rng, 200.0, 1, 0x0700_0005, 0);
    assert_eq!(h.health, 0.0, "the blast kills (damage 100 ≥ health 100, armour 0)");
    let bits_at_death = h.dir_bits;
    let flash_at_death = h.flash;
    let draws_at_death = rng.draws;
    // The next drown tick (30-tick cadence [M Player_MonitorAir 0x191a70:135-143]) arrives dead.
    h.pain_generic(&mut rng, 2.0, 1, 0x0700_0002, 7);
    assert_eq!(h.dir_bits, bits_at_death, "the corpse's +0x967 stays at the kill's store");
    assert_eq!(h.flash, flash_at_death);
    assert_eq!(h.health, 0.0);
    assert_eq!(rng.draws, draws_at_death, "the gated pain consumes no stream draws");
}

/// A pain with damage <= 0 writes nothing either (the second gate, :21-23 — the game's order
/// is health first, then damage).
#[test]
fn a_zero_damage_pain_writes_nothing_on_a_live_player() {
    let mut h = PlayerHealth::default();
    h.health = 57.0;
    h.armour = 10.0;
    h.flash = 3.0;
    h.dir_bits = 0x1;
    let mut rng = Rng::new(5);
    let draws_before = rng.draws;
    h.pain_generic(&mut rng, 0.0, 1, 0x0700_0005, 0);
    h.pain_generic(&mut rng, -1.0, 1, 0x0700_0005, 0);
    assert_eq!(h.dir_bits, 0x1);
    assert_eq!(h.health, 57.0);
    assert_eq!(h.armour, 10.0);
    assert_eq!(h.flash, 3.0);
    assert_eq!(rng.draws, draws_before);
}

/// The live pain still writes everything, in the game's net effect: the type-0 armour absorb,
/// the health floor, the 0xC bits (pain type 6 → 2), the glow, the draw. Guards the fix
/// against over-gating.
#[test]
fn a_live_pain_still_lands_every_store() {
    let mut h = PlayerHealth::default();
    h.health = 57.0;
    h.armour = 10.0;
    h.dir_bits = 0x1;
    let mut rng = Rng::new(7);
    let draws_before = rng.draws;
    // Difficulty 1 on 07000005: the level factor 0.6 [M ReadTuningVars 0x1d3c44 group]; a 10.0
    // hit lands as 6.0: armour 10 absorbs 4 (the min of 6 and 10 is 6 — wait: absorbed =
    // min(6.0, 10.0) = 6.0? no: d·frac = 6.0, armour 10 → absorbed 6.0, armour 4.0, health 57).
    h.pain_generic(&mut rng, 10.0, 1, 0x0700_0005, 0);
    assert_eq!(h.armour, 4.0, "type 0: armour absorbs min(d·1.0, armour) = 6.0 [M 0x1904ec MIN]");
    assert_eq!(h.health, 57.0, "6.0 absorbed covers the whole 6.0 — health untouched");
    assert_eq!(h.dir_bits, 0xC, "the generic body's +0x967 = 0xC [M :121-125]");
    assert!(h.flash > 3.0, "the glow grew by min(d·21.333, 128)");
    assert_eq!(rng.draws, draws_before + 1, "the 1-in-4 roll [M :111]");
}
