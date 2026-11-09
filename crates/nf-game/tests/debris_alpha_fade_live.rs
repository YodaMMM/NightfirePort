//! debris-alpha-fade: the breakable debris pieces' fade byte (obj+0x106) through the real
//! bullet-kill path on the real level data, tick by tick. The game's rules [M]:
//! - `Debris_Create` 0x1aaa00:31 stores obj+0x106 = 0xff at spawn;
//! - `Debris_Update` 0x1aac00:18-24 subtracts the record's step byte (INFO+0x32 = 1 [M
//!   0x1aaa00:35]) per tick while the result stays > 0, and kills the piece the tick the
//!   subtraction would reach ≤ 0; INFO+0x30 < 1 (the tick count) kills too [M 0x1aac00:14];
//! - the breakable pieces' life is `(EffectInfo+0x81 >> 1) + Rand_Rand(EffectInfo+0x81 >> 1)`
//!   [M 0x1aaa00:34-37] — 27..54 ticks on table entry 0x10 [D ACTION.ELF 0x3037c0] — so death is
//!   by life and the last stored alpha is 255 − life ≥ 201.
//! The draw consumes the byte as the tweak A of the piece's own object draw:
//! `View_DrawObjList` → `View_SetupRenderModes_Tweak` (0x1e69a0:77) → `psiSetTweakARGB(A =
//! obj+0x106, …)` (0x1e5888:46-49) → TAmbient (R,G,B,A)/255 @0x2650d0..dc (`psiSetTweakARGB`
//! 0x124770) → the VU1 lighting matrix's ambient (`FillMatrixChainRot` 0x122310:241).
//!
//! Opt-in: `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_game::breakable::Breaks;
use nf_game::drone::Rng;
use nf_game::shatter::{Debris, EFFECT_INFO};

/// The `bullet_hit` weapon argument: a gun's weapon-definition word [M 0x1ece94; the disc's P2K
/// carries 0x0008 — the break_debris_live harness's pin].
const GUN_WORD: u16 = 0x0008;

fn level(id: u32) -> Option<Level> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let name = format!("0700{:04}.bin", id);
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name))?;
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).ok()
}

#[test]
fn a_shot_vases_pieces_step_their_fade_byte_down_one_per_tick_and_die_by_life() {
    let Some(level) = level(1) else { eprintln!("skip: NF_ISO not set"); return };
    let mut br = Breaks::create(&level);
    // The vases: kind 0 with a collision block (the kill needs cel+8 [M 0x12b970:59-63]).
    let Some(i) = br.list.iter().enumerate()
        .find(|(_, b)| b.kind == 0 && !b.src.records.is_empty()).map(|(i, _)| i)
    else { panic!("a kind-0 vase with collision on 07000001 [D census]") };
    let (lo, hi) = (br.list[i].lo, br.list[i].hi);
    let a = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, hi[2] + 2.0];
    let b = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, lo[2] - 2.0];
    let mut rng = Rng::new(0x5eed);
    let kills = br.bullet_hit(a, b, GUN_WORD, 25.0, &mut rng);
    assert_eq!(kills.len(), 1, "the first bullet kills the vase");
    assert!(!br.shatter.debris.is_empty(), "the kill threw its stride-3 pieces [M 0x1ed548]");

    // At create: 0xff on every piece [M 0x1aaa00:31], life in the entry-0x10 window
    // (base 0x37 = 55 → 27 + Rand_Rand(27) [M 0x1aaa00:34-37, D 0x3037c0]).
    let lives: Vec<i32> = br.shatter.debris.iter().map(|d| d.life).collect();
    for (k, d) in br.shatter.debris.iter().enumerate() {
        assert_eq!(d.fade, 0xff, "piece {k}: obj+0x106 = 0xff at create [M 0x1aaa00:31]");
        assert!((27..=54).contains(&d.life), "piece {k}: life {} in the entry-0x10 window [D 0x3037c0]", d.life);
    }

    // The sim's own tick (`Shatter::update` → `Debris_Update` 0x1aac00): after tick k every
    // still-alive piece's fade is 0xff − k and its tick count ran down with it; the population
    // is exactly the create rows whose life is still ≥ k+1 (the tick-count kill [M 0x1aac00:14]).
    // `retain` is order-preserving, so the j-th live piece maps to the j-th surviving row.
    for k in 1..=60u32 {
        br.shatter.update();
        let alive: Vec<usize> = (0..lives.len()).filter(|&r| lives[r] >= k as i32 + 1).collect();
        assert_eq!(br.shatter.debris.len(), alive.len(),
            "after tick {k}: exactly the pieces with life > {k} remain [M 0x1aac00:14]");
        for (j, d) in br.shatter.debris.iter().enumerate() {
            let row = alive[j];
            assert_eq!(d.fade as u32, 0xff - k,
                "piece {j}: fade after tick {k} = 0xff − {k} [M 0x1aac00:18-21, step INFO+0x32 = 1 0x1aaa00:35]");
            assert_eq!(d.life, lives[row] - k as i32,
                "piece {j}: its tick count ran down with the fade [M 0x1aac00:23-25]");
        }
        if br.shatter.debris.is_empty() { break; }
    }
    assert!(br.shatter.debris.is_empty(), "every piece died by its tick count (≤ 54 ticks) [M 0x1aac00:14]");
    // Death by life, not by alpha: the last stored byte of the longest-lived piece is
    // 255 − life ≥ 201 — no breakable piece ever reaches the alpha kill [M 0x1aac00:18-24].
    let max_life = *lives.iter().max().unwrap();
    assert!(255 - max_life as u32 >= 201, "the longest life {} leaves alpha {} on the byte [M 0x1aaa00:34-37]", max_life, 255 - max_life as u32);
}

/// The alpha-kill arm of `Debris_Update` 0x1aac00:18-24, pinned at the unit level — the real
/// records never reach it (life ≤ 54 < 255), but the rule is the game's and the draw's end value
/// depends on it: a piece dies the tick its subtraction would store ≤ 0, and the byte keeps its
/// last stored value (≥ 1) until then [M 0x1aac00:18-24, 56-63].
#[test]
fn the_fade_byte_kills_the_piece_when_its_step_would_reach_zero() {
    let piece = |life: i32, fade: u8| Debris { model: 0x0200_012d, pos: [0.0; 3], rot: [0.0; 3], vel: [0.0; 3], spin: [0.0; 3], life, fade, e8: 0.5, draw: Default::default() };
    // fade 3, life 100: tick 1 stores 2 (alive), tick 2 stores 1 (alive — 1 > 0), tick 3's
    // subtraction 1 − 1 = 0 is NOT > 0 → dead. The byte's last stored value is 1.
    let mut list = vec![piece(100, 3)];
    nf_game::shatter::Shatter::update_debris_list(&mut list);
    assert_eq!(list[0].fade, 2, "tick 1: 3 − 1 = 2 stored [M 0x1aac00:18-21]");
    nf_game::shatter::Shatter::update_debris_list(&mut list);
    assert_eq!(list[0].fade, 1, "tick 2: 2 − 1 = 1 stored (1 > 0 stays alive) [M 0x1aac00:18-21]");
    nf_game::shatter::Shatter::update_debris_list(&mut list);
    assert!(list.is_empty(), "tick 3: 1 − 1 = 0 is not > 0 → the piece dies [M 0x1aac00:18-24, 56-63]");

    // The tick-count kill fires first when it is the smaller clock: life 1, fresh fade.
    let mut list = vec![piece(1, 0xff)];
    nf_game::shatter::Shatter::update_debris_list(&mut list);
    assert!(list.is_empty(), "life 1 → the tick-count kill after one tick [M 0x1aac00:14]");
    // A dead entry (life < 1 at entry) never stores a fade step.
    let mut list = vec![piece(0, 0xff)];
    nf_game::shatter::Shatter::update_debris_list(&mut list);
    assert!(list.is_empty(), "life 0 at entry dies before any fade step [M 0x1aac00:12-14]");
    // The EFFECT_INFO window sanity the real pins lean on: entry 0x10's base is 0x37 [D].
    assert_eq!(EFFECT_INFO[0x10].life_base, 0x37, "entry 0x10's lifetime base [D ACTION.ELF 0x3037c0]");
}
