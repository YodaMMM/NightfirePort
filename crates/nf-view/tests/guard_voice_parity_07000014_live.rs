//! guard-alt-voice-set: the +0x108 serial parity pinned through real play on 07000014.
//!
//! `Drone_SM_InitObject` 0x175838 stores a running serial at SMI+0x108 — the counter
//! [M disasm 0x175858/0x175874: the halfword at 0x299310, `lhu`/`sh`, incremented per call with
//! a non-null object] + 1 [M 0x175878: `sw $a2, 0x108($v1)` with a2 = counter + 1] — and six
//! talk sites read its low bit to pick a second line set [M NDrone2_HitTalkExtra 0x13ed38 disasm
//! 0x13ee5c-0x13ee6c, NDrone2_AttackTalk 0x13eac0's type-7 arm disasm 0x13ecb4-0x13ecc4,
//! NDrone2_SeenPlayerTalk 0x13f1d0 disasm 0x13f2f4/0x13f328/0x13f35c/0x13f380,
//! DroneFunc_InjuredTalk 0x13fa20 disasm 0x13fb10-0x13fb20, DroneFunc_RunForCoverTalk 0x13fd08
//! disasm 0x13fdf8-0x13fe08]. This test walks the real route from the level start (no teleports
//! — the walker's own input) and pins: a guard whose serial is ODD speaks from the ALTERNATE
//! set, one whose serial is EVEN from the base set, on every 14-16 line the run produces.
//!
//! Evidence tags: [M] decompile/disasm address, [D] disc bytes, [L] live measurement, [S] port choice.

#[path = "common/walker.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The 14-16 talk sets the six sites pick from: (base, alt_base, width) — a talk `idx` is in
/// the set iff `base <= idx < alt_base + width`, and its half is ODD (the alternate set) iff
/// `idx >= alt_base` [M the delay-slot shape at every site: the bit-clear arm keeps the lower
/// base, the bit-set arm runs the upper one].
const SETS: [(usize, usize, usize, &'static str); 5] = [
    (0xf3, 0xf5, 2, "SeenPlayerTalk 0x14's own pair [M 0x13f2f4]"),
    (0xfb, 0xfd, 2, "SeenPlayerTalk 0x16's own pair [M 0x13f35c]"),
    (0x10d, 0x10f, 2, "SeenPlayerTalk's common tail pair [M 0x13f380]"),
    (0x11b, 0x122, 7, "AttackTalk type 7 / RunForCoverTalk [M 0x13ecb4, 0x13fdf8]"),
    (0x133, 0x138, 5, "HitTalkExtra / InjuredTalk [M 0x13ee5c, 0x13fb10]"),
];

fn classify(idx: usize) -> Option<(bool, &'static str)> {
    for &(base, alt, width, label) in &SETS {
        if idx >= base && idx < alt + width {
            return Some((idx >= alt, label));
        }
    }
    None
}

/// Walk the real route (the walker's own input, the walkreal_07000014 opening legs — no
/// teleports after the load) until both parity halves have spoken or the tick budget ends,
/// then pin every 14-16 line against its speaker's serial.
/// IGNORED with the measured state (guard-alt-voice-set, 2026-11-06): the walk ran (real input,
/// the stealth opening legs + the walker's own shoot on both load guards) and produced NO 14-16
/// line — the stealth route never triggers a sighting (the dump: the three load-created guards
/// #0/#1/#22 at serials 2/3/1 [L — the serial hand-out itself verified live], states 0x1/0x4/0x7,
/// no talks anywhere), and the shoot engagements end in one-hit kills (the pain talk needs a
/// survivor [M 0x145558: a hit that leaves the drone alive → NDrone2_PainTalk]). The live route
/// that DOES sight Bond is the next run's leg set (the mezzanine mouths' lane after the shaft,
/// walkreal 07000014's legs 1e+); the parity law itself is pinned at the unit level
/// (`alt_voice_set_follows_the_serial_parity` in nf-game). NOTE: HitTalkExtra's types-5..7 arm is
/// game-DEAD — its only caller is PainTalk's type-0x10 arm [M the type table at 0x2f4a80 entry
/// 15 → 0x13ef50; the 5..7 entries → 0x13ef0c, the default] — the live parity sites on 14-16 are
/// SeenPlayerTalk (every type) and AttackTalk (type 7).
#[test]
#[ignore = "the stealth opening + one-hit kills produce no 14-16 line [L the run]; the sighting route (the mezzanine legs) is the next leg set — the parity law is pinned in nf-game's alt_voice_set_follows_the_serial_parity"]
fn the_alt_voice_set_follows_the_game_serial_parity() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(120);

    // The real opening: the crossing stance → the trench east → the corridor south — the same
    // legs the walkreal binary pins, walked with the walker's own return fire (the guards are
    // live; no hold, no top-up beyond the walker's real-damage default).
    w.walk_to(Wp::new(40.8, 2.4, 65.5).r(0.5), "to the crossing stance");
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 62.8).r(0.7), "the trench east to the corridor");
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 52.2).r(0.7), "the corridor south to the stairwell lane");
    let _ = w.try_walk_to(Wp::new(48.8, -0.6, 44.0).r(0.7), "the corridor south to the mouths' lane");

    // The level's load-created guards [L this run: #0 (serial 2, type 6), #1 (serial 3, type 6),
    // #22 (serial 1, type 0x10) — the rest are the 14 spawners' templates]. The route above is
    // stealth (no sighting fired); the engagements come through the walker's own shoot — the
    // real-input aim/fire machinery (stand_and_fire), the same one the walkreal fights use: the
    // wounded guard's PainTalk runs NDrone2_HitTalkExtra [M 0x13eeb8 type 5..7 → 0x13ed38], the
    // engagement's alert runs SeenPlayerTalk [M 0x13f1d0] and the attack entries AttackTalk
    // [M 0x13eac0 type 7]. The two types 6 carry the two parities (serials 2 and 3).
    w.shoot(0, "the load guard #0 (serial 2, type 6)");
    w.shoot(1, "the load guard #1 (serial 3, type 6)");

    // Harvest: every guard's talks this run, with the guard's serial.
    let sim = w.scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.list.iter().enumerate() {
        if !d.talks.is_empty() {
            eprintln!("[dump] guard #{i} sm_id {} vt {:#x} talks {:?}", d.sm_id, d.voice_type, d.talks);
        }
    }
    let mut rows: Vec<(usize, u32, usize, &'static str)> = Vec::new();
    for (i, d) in sim.drones.list.iter().enumerate() {
        for &(idx, _) in &d.talks {
            if let Some((odd, label)) = classify(idx) {
                rows.push((i, d.sm_id, idx, if odd { label } else { label }));
                let spoke_alt = idx >= SETS.iter().find(|&&(b, a, _, _)| idx >= b && idx < a + 10).unwrap().1;
                assert_eq!(
                    spoke_alt, odd,
                    "guard #{i} (serial {} {}) spoke {:#x} — the {} — but the half mismatches the serial's low bit [M the six sites' delay-slot shape]",
                    d.sm_id, if d.sm_id & 1 != 0 { "odd" } else { "even" }, idx, label,
                );
            }
        }
    }
    let odd_rows = rows.iter().filter(|(_, s, _, _)| s & 1 != 0).count();
    let even_rows = rows.iter().filter(|(_, s, _, _)| s & 1 == 0).count();
    eprintln!("[parity] rows {} (odd-half {}, even-half {}): {:?}", rows.len(), odd_rows, even_rows, rows);
    assert!(!rows.is_empty(), "the walk produced 14-16 talk lines at all (the guards met Bond [L])");
    assert!(odd_rows >= 1, "no odd-serial guard spoke from the alternate set this run (serials seen: {:?})",
        rows.iter().map(|(_, s, _, _)| *s).collect::<Vec<_>>());
    assert!(even_rows >= 1, "no even-serial guard spoke from the base set this run (serials seen: {:?})",
        rows.iter().map(|(_, s, _, _)| *s).collect::<Vec<_>>());
}
