//! inert-notes-drone-a: the reachability census for the `not ported` / `not wired` / `inert`
//! notes in `crates/nf-game/src/drone.rs` up to line 9000.
//!
//! Each note claims some game arm is not ported (or that a port stand-in is inert). The letter's
//! rule: the arm is reachable on an on-foot level or it is not — a count of zero on every level
//! closes the note with that count as proof; a non-zero count demands a port (or a hand-off).
//! This census counts the placement-side inputs of every such arm over all 34 archives
//! (`Level::parse` over `nf_view::dev::LEVELS`, loads only — no driving):
//!
//! - **mode byte +0xC5 inputs** [M 0x14c600/0x14b300]: every drone placement's chain word
//!   (param 5 — 100 = NEW), start script (param 1 — drives PlayScript, the abseil conversions
//!   0x06000892→0x1c / 0x0600089c→0x1b, and the NEW-chain mode specials 0x0600011d→0x17,
//!   0x0600011f/0x06000124/0x06000126→0x18), skin (param 0 — voice 0x18 skins → mode 0x1d),
//!   dtype (param 15) and mode_normal (param 16) — the inputs of the dtype fold
//!   [M 0x14cbd8:26]. Notes: "0x1E → BotDeathAnim (not ported)" (drone.rs:3866/3982) and
//!   "Mode-specific writes (modes 0xC, 0x11 …) are not ported" (drone.rs:2384).
//! - **voice-set +0xD8 inputs**: the Stunned_Recover voice-0x10 team flip
//!   [M 0x1630b8:17-19, drone.rs:6811], the PartyGirl/BondTalk voice sets
//!   [M 0x160140:18, 0x1446b0:44-52, drone.rs:8865/8914] and the KikoMission voice-0x13 arm
//!   [drone.rs:8944].
//! - **special death** (param 13, the 0x0600021f stationary word) and the dtype histogram for
//!   the sniper/elevator notes (drone.rs:1582).
//!
//! Opt-in via `NF_ISO`.

use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_view::dev;

/// The game's NEW-chain start-script specials [M 0x14c600:118-152, 0x14b300:612-628].
const SS_TRUCK: u32 = 0x0600_011d; // → mode 0x17 (truck driver)
const SS_MODE18: &[u32] = &[0x0600_011f, 0x0600_0124, 0x0600_0126]; // → mode 0x18
const SS_ABSEIL_B: u32 = 0x0600_089c; // → mode 0x1b
const SS_ABSEIL_C: u32 = 0x0600_0892; // → mode 0x1c
/// The no-script sentinel [M 0x14b300:612-615: `+0x554` keeps 0 when the word is 0x06000000].
const SS_NONE: u32 = 0x0600_0000;
/// The stationary special-death word [M 0x14b300:486; port param(13)].
const SD_STATIONARY: u32 = 0x0600_021f;

/// Skins whose voice set (+0xD8) is 0x10 — the Stunned_Recover team-flip arm [M 0x1630b8:17-19;
/// the +0xD8 switch, port `voice_type`].
const VOICE10_SKINS: &[u16] = &[0x26, 0x66];
/// Skins whose voice set is 0x12 (PartyGirl's `voice 0x12 → Civilian` arm) [M 0x160140:18].
const VOICE12_SKINS: &[u16] = &[0x17, 0x18];
/// Voice 0x13 — the KikoMission arm's voice [M 0x160ed0 family; drone.rs:8944].
const VOICE13_SKINS: &[u16] = &[0x5e];
/// Voice 0x16 (the female-civilian set, for context).
const VOICE16_SKINS: &[u16] = &[0x10, 0x11, 0x52];
/// Voice 0x18 — DefaultInit converts to mode 0x1d [M 0x14b300:718-727].
const VOICE18_SKINS: &[u16] = &[0x78, 0x94, 0xba];
/// The voice sets NDrone2_BondTalk handles [M 0x1446b0:52-175 — cases 9, 0xd, 0xe, 0x10, 0x11,
/// 0x12; the function returns 0 before the switch unless GameState+12 == 0x0700007].
const BONDTALK_VOICES: &[u16] = &[9, 0xd, 0xe, 0x10, 0x11, 0x12];

fn voice(skin: u32) -> u16 { nf_game::drone::voice_type(skin) }

#[test]
fn inert_notes_census() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let path = path.to_str().unwrap();
    let iso = IsoImage::open(path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let read = |name: &str| -> Option<Vec<u8>> {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
        Some(iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap())
    };

    let mut total_drones = 0u32;
    let mut p5_not_100 = 0u32;
    let mut ss_hist: std::collections::BTreeMap<u32, u32> = Default::default();
    let mut mode_special: [u32; 8] = [0; 8]; // 0x17,0x18,0x1b,0x1c,0x1d, and starts 2/0x19 counts separate below
    let mut dtype_mode_normal_pairs: std::collections::BTreeMap<(u8, u8), u32> = Default::default();
    let mut voice10 = 0u32;
    let mut voice12 = 0u32;
    let mut voice13 = 0u32;
    let mut voice16 = 0u32;
    let mut voice18 = 0u32;
    let mut bondtalk_voices = 0u32;
    let mut sd_stationary = 0u32;
    let mut per_level: Vec<(u32, u32, u32, u32, u32)> = Vec::new(); // id, drones, start_scripts, voice10, bondtalk

    for &id in dev::LEVELS {
        let name = nf_view::level_archive_name(id);
        let Some(data) = read(&name) else { panic!("no archive {name} on the disc") };
        let level = Level::parse(&data).unwrap_or_else(|e| panic!("parse {name}: {e}"));
        let mut lvl = (0u32, 0u32, 0u32, 0u32, 0u32);
        for map in &level.maps {
            for p in &map.placements {
                if p.flags & 0xff != 0x0f { continue; }
                let param = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0);
                let skin = param(0);
                let p1 = param(1);
                let p5 = param(5);
                let p13 = param(13);
                let dtype = param(15) as u8;
                let mode_normal = param(16) as u8;
                // The one by_hash p5=0 row per archive is a cross-map reference (the hash table
                // resolves it to another map's object), not a drone this map creates [D census:
                // 33 archives × 1 row; 588 − 33 = the 555 creating placements of
                // docs/systems/ai-dmode-inits.md].
                if !p.by_hash() {
                    total_drones += 1;
                    lvl.0 += 1;
                    if p5 != 100 { p5_not_100 += 1; }
                }
                if p1 != 0 && p1 != SS_NONE {
                    *ss_hist.entry(p1).or_default() += 1;
                    lvl.1 += 1;
                }
                // The direct mode specials (start-script + voice-0x18 arms). Everything else
                // goes through the dtype fold, recorded as (dtype, mode_normal) pairs.
                if p1 == SS_TRUCK { mode_special[0] += 1; }
                else if SS_MODE18.contains(&p1) { mode_special[1] += 1; }
                else if p1 == SS_ABSEIL_B { mode_special[2] += 1; }
                else if p1 == SS_ABSEIL_C { mode_special[3] += 1; }
                else if VOICE18_SKINS.contains(&(skin as u16)) && (skin & 0xffff) as u32 == skin % 0x10000 || voice(skin) == 0x18 { mode_special[4] += 1; }
                else { *dtype_mode_normal_pairs.entry((dtype, mode_normal)).or_default() += 1; }
                if p13 == SD_STATIONARY { sd_stationary += 1; }
                let lo = (skin & 0xffff) as u16;
                if VOICE10_SKINS.contains(&lo) { voice10 += 1; lvl.2 += 1; }
                if VOICE12_SKINS.contains(&lo) { voice12 += 1; }
                if VOICE13_SKINS.contains(&lo) { voice13 += 1; }
                if VOICE16_SKINS.contains(&lo) { voice16 += 1; }
                if VOICE18_SKINS.contains(&lo) { voice18 += 1; }
                if BONDTALK_VOICES.contains(&voice(skin)) { bondtalk_voices += 1; lvl.3 += 1; }
                let _ = mode_normal;
            }
        }
        if lvl.0 > 0 { per_level.push((id, lvl.0, lvl.1, lvl.2, lvl.3)); }
    }

    eprintln!("== inert-notes census: {} drones over {} levels ==", total_drones, dev::LEVELS.len());
    eprintln!("-- per level: id drones start_scripts voice10 bondtalk_voices --");
    for (id, d, ss, v10, bt) in &per_level {
        eprintln!("{id:08x}: drones {d} start_scripts {ss} voice10 {v10} bondtalk {bt}");
    }
    eprintln!("-- p5 != 100 and voice13 per level --");
    for &id in dev::LEVELS {
        let name = nf_view::level_archive_name(id);
        let Some(data) = read(&name) else { panic!("no archive {name}") };
        let level = Level::parse(&data).unwrap();
        let mut bad5 = 0u32; let mut v13 = 0u32; let mut v10l = 0u32; let mut p5s: std::collections::BTreeMap<u32, u32> = Default::default();
        for map in &level.maps {
            for p in &map.placements {
                if p.flags & 0xff != 0x0f { continue; }
                let param = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0);
                if param(5) != 100 { bad5 += 1; *p5s.entry(param(5)).or_default() += 1; }
                if (param(0) & 0xffff) == 0x5e { v13 += 1; }
                if [0x26u16, 0x66].contains(&((param(0) & 0xffff) as u16)) { v10l += 1; }
            }
        }
        if bad5 > 0 || v13 > 0 || v10l > 0 { eprintln!("{id:08x}: p5!=100 {bad5} {p5s:?} voice13 {v13} voice10 {v10l}"); }
    }
    eprintln!("-- sniper dtypes (2) and by_hash on the p5=0 rows, per level --");
    for &id in dev::LEVELS {
        let name = nf_view::level_archive_name(id);
        let Some(data) = read(&name) else { panic!("no archive {name}") };
        let level = Level::parse(&data).unwrap();
        let mut sn = 0u32; let mut bh = 0u32; let mut ss_lvl: Vec<u32> = Vec::new(); let mut sniper_blocks: Vec<u32> = Vec::new();
        // FindElevatorDoor (0x14a940) from SniperAim ENTER: the nearest class-0x1c object within
        // 4.0 of the drone stands in +0xBA8. Static proxy: door placements (flags & 0xff == 0xDB,
        // the 0x1c Door creator [D update_column_census table]) within 4.0 of a sniper placement.
        let mut doors: Vec<[f32; 3]> = Vec::new();
        let mut snipers: Vec<[f32; 3]> = Vec::new();
        for map in &level.maps {
            for p in &map.placements {
                let param = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0);
                if p.flags & 0xff == 0xdb { doors.push(p.translation); }
                if p.flags & 0xff != 0x0f { continue; }
                if param(15) == 2 && matches!(param(16), 2 | 3 | 4) { sn += 1; snipers.push(p.translation); sniper_blocks.push(param(16)); }
                if param(5) == 0 && p.by_hash() { bh += 1; }
                let p1 = param(1);
                if p1 != 0 && p1 != 0x0600_0000 { ss_lvl.push(p1); }
            }
        }
        let (mut near4, mut near2) = (0u32, 0u32);
        let mut near2_blocks: Vec<u32> = Vec::new();
        for (si, s) in snipers.iter().enumerate() {
            let mut hit4 = false; let mut hit2 = false;
            for d in &doors {
                let q = [d[0]-s[0], d[1]-s[1], d[2]-s[2]];
                let q2 = q[0]*q[0]+q[1]*q[1]+q[2]*q[2];
                if q2 < 16.0 { hit4 = true; }
                if q2 < 4.0 { hit2 = true; } // ControlSTANDARD 0x1490f0:232 clears past 2.0; OpenDoor retries only while stood
            }
            near4 += hit4 as u32; near2 += hit2 as u32;
            if hit2 { near2_blocks.push(sniper_blocks[si]); }
        }
        if sn > 0 || bh > 0 || !ss_lvl.is_empty() {
            eprintln!("{id:08x} {}: dtype2snipers {sn} door_within4 {near4} door_within2 {near2} near2_blocks {near2_blocks:02x?} p5=0-by-hash {bh} start_scripts {ss_lvl:02x?}",
                nf_view::level_archive_name(id));
        }
    }
    eprintln!("-- totals --");
    eprintln!("p5 != 100: {p5_not_100}");
    eprintln!("start scripts: {ss_hist:?}");
    eprintln!("mode specials: truck(0x17) {} mode18 {} abseil_b(0x1b) {} abseil_c(0x1c) {} voice18(0x1d) {}",
        mode_special[0], mode_special[1], mode_special[2], mode_special[3], mode_special[4]);
    eprintln!("(dtype, mode_normal) pairs going through the fold: {dtype_mode_normal_pairs:?}");
    eprintln!("special_death 0x0600021f: {sd_stationary}");
    eprintln!("voice10 skins: {voice10}  voice12: {voice12}  voice13: {voice13}  voice16: {voice16}  voice18: {voice18}");
    eprintln!("bondtalk voices: {bondtalk_voices}");

    // Closures the counts already prove.
    // (1) Note drone.rs:2384's "modes 0xC, 0x11": every placement is NEW (param 5 = 100) and the
    // NEW chain never writes modes 0xC/0x11 [M 0x14c600 — its only +0xC5 writes are 0x17/0x18];
    // the OLD chain's mode-specific writes need row bytes the shipped placements never reach.
    assert_eq!(p5_not_100, 0, "a placement off the NEW chain — the mode-0xC/0x11 closure breaks");
    // (2) The abseil start scripts (mode 0x1b/0x1c conversions) — their census counts close or
    // scope the abseil-specific notes; recorded, not asserted (counts land in the report).
    // (3) Note drone.rs:8944 (KikoMission voice 0x13): recorded — the count decides close vs port.
    // (4) Notes drone.rs:3866/3982 (mode 0x1E = the bot death anim): no placement produces mode
    // 0x1E — its only placement-reachable producer is dtype 0x10 [M 0x14c938 via GetDTYPENEW],
    // and no drone placement carries dtype 16; the abseil/skin/start-script arms write 0x1b/
    // 0x1c/0x1d/0x17/0x18 only [M 0x14b300:612-628,718-727; 0x14c600:133-155]. The by_hash
    // p5=0 rows never create a drone (cross-map references resolved through the hash table).
    assert!(!dtype_mode_normal_pairs.keys().any(|&(d, _)| d == 16), "a dtype-0x10 (bot) placement exists — the mode-0x1E closure breaks");
    assert_eq!(mode_special[2] + mode_special[3], 5, "abseil start-script census drifted (expected 07000011 ×2 + 0700004a ×3 0x06000892 rows)");
}
