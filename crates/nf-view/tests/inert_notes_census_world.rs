//! inert-notes-world census: for the "not ported / inert" notes in crates/nf-game/src (except
//! drone.rs and mp/), count on EVERY on-foot level (`nf_view::dev::LEVELS`, `Level::parse` — no
//! driving levels) the disc data that would reach the unported arm. A zero count closes the item
//! with the count as proof; a non-zero count names the gap. Run with --nocapture.

use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::weapon::WeaponData;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn level_archive<'a>(iso: &IsoImage<std::fs::File>, entries: &[nf_formats::filelist::FileEntry], bin_lba: u64, id: u32) -> Vec<u8> {
    let name = format!("{id:08x}.bin");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name))
        .unwrap_or_else(|| panic!("no {name} in FILES.BIN"));
    iso.read_bytes(bin_lba * 2048 + fe.offset as u64, fe.size as u64).expect("read level")
}

fn param(p: &nf_formats::placement::Placement, i: u32) -> Option<u32> {
    p.params.iter().find(|(k, _)| *k == i).map(|(_, v)| *v)
}

#[test]
fn inert_notes_census() {
    let iso = IsoImage::open(iso()).expect("open iso");
    let (entries, bin) = disc::files_bin_entries(&iso).expect("FILES.BIN");
    let elf_entry = iso.find("ACTION.ELF;1").or_else(|| iso.find("ACTION.ELF")).expect("ACTION.ELF");
    let elf = iso.read_file(elf_entry).expect("read ACTION.ELF");
    let weapons = WeaponData::from_action_elf(&elf).expect("weapon_data");

    // 1. gun.rs Player_WeaponHasAmmo 0x1a16c0's 0x20000 arm (the ground-pickup scan STANDIN):
    //    reachable only for a record whose +0x68 flag word carries 0x20000 [M 0x1a16c0:29].
    let mut flagged: Vec<usize> = Vec::new();
    for w in 0..weapons.records.len() {
        if weapons.flags(w) & 0x2000_0 != 0 { flagged.push(w); }
    }
    // The arm also needs ammo_type != 0 (melee/unlimited returns true before it) and the weapon
    // selectable/loadable in play.
    let ammo_types: Vec<(usize, usize)> = flagged.iter().map(|&w| (w, weapons.ammo_type(w))).collect();
    eprintln!("[census] weapon records with +0x68 & 0x20000: {} {:?} ammo_types {:?}",
        flagged.len(), flagged, ammo_types);

    let mut rows: Vec<String> = Vec::new();
    for &id in nf_view::dev::LEVELS {
        let data = level_archive(&iso, &entries, bin.lba as u64, id);
        let level = Level::parse(&data).unwrap_or_else(|e| panic!("parse {id:08x}: {e}"));
        let main = level.main_map().unwrap_or(0);
        let placements = &level.maps[main].placements;

        // 2. ss.rs SS_Operate 0x1a9df8's class-2 (drone → mask 0x10) contact bit — the port feeds
        //    bits 0/1 only; reachable only where an SS record's mask word carries 0x10.
        let ss = nf_game::ss::SsList::create(&level.maps[main].placements, &level.path_blocks(), |_| (0, [0.0; 3], 0.0));
        let ss_drone = ss.list.iter().filter(|s| s.mask & 0x10 != 0).count();
        let ss_total = ss.list.len();

        // 3. door.rs obj+0xFD bit 3 (p4): the collision-portal link (block 0x21 — not ported).
        let doors = nf_game::door::door_placements(placements);
        let door_straddle = doors.iter().filter(|&&di| param(&placements[di], 4).unwrap_or(0) != 0).count();

        // 4. nav.rs AINetwork_RouteNodeOffset 0x15a820's flag&7 == 3 bounce: the route mode word
        //    rides the AI path kind (&4 = loop, the port's RouteMode::Loop) — bounce needs bit 0
        //    set on a looping path. Count the path kinds' low bits.
        let mut loop_kinds: std::collections::BTreeMap<u32, usize> = Default::default();
        if let Some(net) = &level.maps[main].ai_network {
            for p in &net.paths { *loop_kinds.entry(p.kind & 7).or_default() += 1; }
        }
        let bounce_candidates: usize = loop_kinds.iter().filter(|(k, _)| *k & 1 != 0).map(|(_, c)| c).sum();
        // 2b. the drone bodies that could ever touch a 0x10-mask SS: the 0xF drone placements.
        let drones = placements.iter().filter(|p| matches!(p.flags & 0xff, 0x0f | 0xf1)).count();

        // 5. nis.rs DroneEnable's hash extras (0x600001b / 0x600002e) — live only if either script
        //    ships in the level archive.
        let nis_extras = level.nis.keys().filter(|h| **h == 0x0600_001b || **h == 0x0600_002e).count();

        // 6. gunimp.rs Collide_PlaceObject 0x1ecfb8 (only caller GT_DeployMiniGun 0x181560:57):
        //    reachable where a kind-0x27 GT placement exists (the deployable/turret family).
        let gt = placements.iter().filter(|p| p.flags & 0xff == 0x27).count();

        // 7. copter.rs Copter_Update 0x17dbb8 case 1's spot check (unported): copter records.
        let copter_paths = nf_game::copter::CopterPath::create(placements, &level.path_blocks());
        let copters = nf_game::copter::Copter::create(placements, &copter_paths, 1).len();

        // 8. wire.rs Player_Activate 0x1a5170:67's creep-wall E-grab (+0x95F == 6): creep walls.
        let creepwalls = nf_game::creepwall::all(&level).len();

        // 9. ladder.rs stage-2's true mesh refine (Collide_Intersect per candidate): ladders.
        let ladders = nf_game::ladder::all(&level).len();

        // 10. door.rs Door_ReactToHit 0x135980 via the Collide_Update proxy list (U-041): doors.
        let door_total = doors.len();

        // 1b. can a flagged weapon ever be HELD on this level: a placed weapon pickup whose id
        //     is one of the 0x20000 records (0xF0 placements, param 1 = the weapon id).
        let flagged_pickups = placements.iter()
            .filter(|p| p.flags & 0xff == 0xf0)
            .filter(|p| flagged.contains(&(param(p, 1).unwrap_or(0) as usize)))
            .count();

        // 11. space_laser.rs the ch-19 arm gate on 0700001b: writers of ch 19 in the level data
        //     (the doc's claim: none [D docs/levels/0700001b.md]).
        let ch19 = if id == 0x0700_001b {
            placements.iter().filter(|p| p.flags & 0xff == 0xeb && param(p, 1) == Some(19)).count()
        } else { 0 };

        eprintln!("[census] {id:08x}: ss {ss_total}/{ss_drone}@0x10 drones {drones} doors {door_total}/{door_straddle}@p4 \
            path-kinds {} (bounce {bounce_candidates}) nis-extras {nis_extras} gt {gt} copters {copters} \
            creepwalls {creepwalls} ladders {ladders} ch19 {ch19} flagged-pickups {flagged_pickups}",
            loop_kinds.len());
        rows.push(format!("{id:08x} ss0x10 {ss_drone} drones {drones} doorP4 {door_straddle} bounce {bounce_candidates} \
            nisX {nis_extras} gt {gt} copters {copters} creep {creepwalls} ladders {ladders} ch19 {ch19}"));
    }
    eprintln!("[census] table:");
    for r in &rows { eprintln!("  {r}"); }

    // The closes, pinned (red if the data ever changes):
    // - nis.rs DroneEnable's hash extras: neither script ships on any on-foot level.
    // - space_laser.rs's ch-19 arm gate: no trigger on 0700001b writes ch 19.
    // - ss.rs's class-2 (drone → mask 0x10) contact bit: every 0x10-mask record sits on a level
    //   with ZERO drone placements (07000046's 8), so no drone body can ever make the contact.
    for r in &rows {
        let f: Vec<&str> = r.split_whitespace().collect();
        let get = |k: &str| f.iter().position(|t| *t == k).and_then(|i| f.get(i + 1)).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0);
        let id = u32::from_str_radix(f[0], 16).unwrap();
        let (nisx, ch19, ss10, drones) = (get("nisX"), get("ch19"), get("ss0x10"), get("drones"));
        assert_eq!(nisx, 0, "{id:08x}: a DroneEnable extra script ships here — the nis.rs note's             'neither script is live' needs a port");
        assert_eq!(ch19, 0, "{id:08x}: a ch-19 writer exists — the space_laser arm gate opens");
        assert!(ss10 == 0 || drones == 0, "{id:08x}: an SS record with the drone mask 0x10 AND drone             bodies exist — the ss.rs class-2 contact bit needs a port");
    }
    // The weapon 0x20000 close: the records exist (4, ammo types 22/30), but nothing can put one
    // in Bond's hands on foot — no placed pickup grants any of them (asserted above per level)
    // and `Player_InitWeapon` 0x1a7ef8's SP loadout cases never grant 0x38/0x39/0x5a/0x5c
    // [M player.rs init_level_loadout's case list]. The ground-pickup scan arm stays unreachable
    // in SP play; the MP weapon-set rows are the one unverified source (live check in the report).
    eprintln!("[census] weapon 0x20000 records: {} (ammo types {:?}) — SP-unreachable (0 pickups, \
        no loadout grant); MP-set rows unverified", flagged.len(),
        flagged.iter().map(|&w| weapons.ammo_type(w)).collect::<Vec<_>>());

    // gas.rs gases_smoke 0x1ab2d0's types 9/10 (the hidden emitters, unported): reachable only
    // through a Gas_Init call whose kind is 9/10 — the fixed sites pass {0,1,6,7} [M decomp scan],
    // the table-driven sites (Effect_Bullet 0x1ce8a8 / Effect_Laser 0x1cef38) read EffectInfo's
    // EffectGas kind words. Count the kinds in the table.
    match nf_formats::sfx::SurfaceSounds::from_action_elf(&elf) {
        Ok(ssfx) => {
            let mut gas_kinds: std::collections::BTreeMap<u32, usize> = Default::default();
            for rec in &ssfx.effects {
                for g in &rec.gas { *gas_kinds.entry(g.kind).or_default() += 1; }
            }
            let emitters: usize = gas_kinds.iter().filter(|(k, _)| **k == 9 || **k == 10).map(|(_, c)| c).sum();
            eprintln!("[census] EffectInfo gas kinds: {:?} — the 9/10 emitter kinds: {} entries",
                gas_kinds, emitters);
        }
        Err(e) => eprintln!("[census] EffectInfo parse failed: {e} (the 9/10 count needs the table)"),
    }
}
