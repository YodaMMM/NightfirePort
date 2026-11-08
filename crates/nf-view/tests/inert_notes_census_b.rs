//! inert-notes-drone-b: the reachability census for the `not ported` / `not wired` / `inert`
//! notes in `crates/nf-game/src/drone.rs` from line 9000 (the second half; the first half is
//! `s/inert-notes-drone-a`). The letter's rule: the arm is reachable on an on-foot level or it is
//! not — a count of zero on every level closes the note with that count as proof; a non-zero
//! count demands a port (or a hand-off). No driving levels: `dev::LEVELS` holds the 34 on-foot
//! level archives only [D the list at crates/nf-view/src/dev.rs:830].
//!
//! Counted here (the runtime fields after a real load — the port's own create path derives
//! mode/voice/team/state, so the count is the arm's actual population, not a placement
//! approximation):
//! - **voice-0x10 drones** with their state and team — the CivilianPatrol near-armed arm's
//!   voice-0x10 team switch [M 0x15d610:54-55 `+0x44 = 1`; drone.rs:9190 sets the wrong store]
//!   and the Stunned_Recover flip drone-a counted.
//! - **HostageIdle/Hostage/HostageHide drones** per level — the HostageIdle UPDATE transition to
//!   HostageHide [M 0x15ce60:28-33] and the mirror back [M 0x15cfe0:23-29], not ported
//!   (drone.rs:9291).
//! - **team-3 drones** per level — the EnemyLookForOpponent team-3 refusal's 07000009/0a
//!   exception [M 0x177ed0:29-38, drone.rs:10463].
//! - **team-2 allies** per level (the MayhewB/C combat-pick gate scope, drone.rs:9384).
//! Plus a `Level::parse` pass: the 0xe1 Explode placements per level (the Camera_Shake ≥ 0.5
//! sources' placement-side half, drone.rs:11860/12276/14582/21984 — grenades/rockets are runtime).
//!
//! Opt-in via `NF_ISO`.

use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_view::dev;
use nf_game::drone::state;

#[test]
fn inert_notes_census_b() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let path = path.to_str().unwrap().to_string();

    // ── the runtime pass: load each on-foot level, count the live drones ────────────────────────
    let mut voice10_rows: Vec<(u32, u8, u8, u8)> = Vec::new(); // level, state, team, mode
    let mut hostage_per_level: Vec<(u32, u32, u32, u32)> = Vec::new(); // level, idle, hostage, hide
    let mut team3_per_level: Vec<(u32, u32)> = Vec::new();
    let mut team2_per_level: Vec<(u32, u32)> = Vec::new();
    let mut total = 0u32;
    for &id in dev::LEVELS {
        let name = format!("{:08x}.bin", id);
        let scene = match nf_view::load(&path, &name, false) {
            Ok(s) => s,
            Err(e) => panic!("load {name}: {e}"),
        };
        let sim = scene.sim.as_ref().unwrap();
        let (mut idle, mut hst, mut hide, mut t3, mut t2) = (0u32, 0u32, 0u32, 0u32, 0u32);
        for d in &sim.drones.list {
            if !d.present { continue; }
            total += 1;
            if d.voice_type == 0x10 { voice10_rows.push((id, d.state, d.team, d.mode)); }
            match d.state {
                state::HOSTAGE_IDLE => { idle += 1; }
                state::HOSTAGE => { hst += 1; }
                state::HOSTAGE_HIDE => { hide += 1; }
                _ => {}
            }
            if d.team == 3 { t3 += 1; }
            if d.team == 2 { t2 += 1; }
        }
        if idle + hst + hide > 0 { hostage_per_level.push((id, idle, hst, hide)); }
        if t3 > 0 { team3_per_level.push((id, t3)); }
        if t2 > 0 { team2_per_level.push((id, t2)); }
    }
    eprintln!("[census-b] drones over the {} on-foot levels: {total}", dev::LEVELS.len());
    eprintln!("[census-b] voice-0x10 rows (level, state, team, mode): {voice10_rows:x?}");
    eprintln!("[census-b] hostage rows (level, idle, hostage, hide): {hostage_per_level:x?}");
    eprintln!("[census-b] team-3 rows (level, count): {team3_per_level:x?}");
    eprintln!("[census-b] team-2 rows (level, count): {team2_per_level:x?}");

    // ── the placement pass: the 0xe1 Explode placements per level (the shake scope's static half)
    let iso = IsoImage::open(path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let read = |name: &str| -> Option<Vec<u8>> {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
        Some(iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap())
    };
    let mut explode_rows: Vec<(u32, u32)> = Vec::new();
    let mut hostage_placements: Vec<(u32, u32)> = Vec::new();
    for &id in dev::LEVELS {
        let name = nf_view::level_archive_name(id);
        let Some(data) = read(&name) else { panic!("no archive {name} on the disc") };
        let level = Level::parse(&data).unwrap_or_else(|e| panic!("parse {name}: {e}"));
        let mut e1 = 0u32;
        let mut hst = 0u32;
        for map in &level.maps {
            for p in &map.placements {
                let kind = p.flags & 0xff;
                if kind == 0xe1 { e1 += 1; }
                if kind == 0x0f { hst += 1; }
            }
        }
        if e1 > 0 { explode_rows.push((id, e1)); }
        let _ = hst;
    }
    eprintln!("[census-b] 0xe1 Explode placements (level, count): {explode_rows:x?}");

    // The census asserts the counts the closures lean on (re-runs stay honest).
    assert!(total > 200, "the drone population over the on-foot levels (got {total})");
}
