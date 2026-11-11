//! rb-impact-sound: the 0xD8 placement params p3 (carry-light → rb+0x154) and p4 (impact
//! sound → rb+0x158), pinned where the game actually uses them.
//!
//! The create side [M RB_Create 0x1d6bf0:107-112]: `rb+0x154 = level_tag+0x38` (p3, a byte)
//! and `rb+0x158 = level_tag+0x3c` (p4, a word, 0 → 0xFFFF). The readers:
//! - p4 — the class-5 arms of `RB_RestingContact` 0x1d7168:111-112 and
//!   `RB_RestingContactRot` 0x1d73a0:121-123: `Sound_Play3D(100.0, −1.0, −1.0, *(rb+0x158),
//!   obj+0xC0)` once per class-5 hit in the body's obj+0xD0 chain (class 5 = the round,
//!   `Bullet_init` 0x12c5a0:87); the handler drains the chain each tick, and a round dies on
//!   its contact — one call per landed shot. There is NO speed gate in the reader: the call
//!   is the arm's first statement, before any velocity use (the impulse after it reads
//!   hit+0x8·0.5·vel). The normal sound path gates 0xFFFF = none [M sound.rs play3d].
//! - p3 — `RB_Update` 0x1d6e38:76-103 (type != 0): a cached `Light_FindMapLight` handle at
//!   rb+0x150 gating a per-tick position sync. The lookup is an 8-byte stub returning 0 on
//!   this build [M 0x1cc3d0 decompile], so the handle never arms and no light follows a
//!   moving prop — ported as the stub (`rigid_body::light_find_map_light`,
//!   `RbState::s150`; unit-tested there).
//!
//! Evidence tags: [M] the decompile function + address, [D] the disc parse, [S] unverified.
//! The walk and the shot are the real input path (no teleport after the load).

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

fn parse_level(img: &nf_formats::iso9660::IsoImage, entries: &[nf_formats::filelist::FileEntry], bin: &nf_formats::iso9660::IsoEntry, lvl: u32) -> Option<nf_formats::level::Level> {
    let name = format!("{lvl:08x}.bin");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name))?;
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
    nf_formats::level::Level::parse(&data).ok()
}

/// [D census] Every dev level's main-map 0xD8 placements: the impact id p4 is 0 (→ the
/// 0xFFFF none sentinel [M 0x1d6bf0:108-112]) on ALL of them — no dev level ships an audible
/// rigid-body impact sound — and p3 (the carry-light byte) is 1 on exactly sixteen bodies,
/// all on 07000001/02/04. Thirty-five bodies total. (The port's mapping is pinned per level
/// by the interactables/creation tests; this pins the DISC side the map rides on.)
#[test]
fn census_p4_is_the_none_sentinel_and_p3_rides_four_levels() {
    let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let (mut bodies, mut p3_set, mut p3_levels) = (0usize, 0usize, Vec::<u32>::new());
    for lvl in nf_view::dev::LEVELS {
        let Some(lv) = parse_level(&img, &entries, &bin, *lvl) else { continue };
        let Some(m) = lv.main_map() else { continue };
        for p in &lv.maps[m].placements {
            if p.flags & 0xff != 0xd8 { continue; }
            let pv = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
            let mapped = if pv(4) == 0 { 0xffff } else { pv(4) };
            assert_eq!(mapped, 0xffff, "{lvl:08x} place {} carries a REAL impact id {:#x} — the census and this pin are wrong", p.index, mapped);
            bodies += 1;
            if pv(3) & 0xff != 0 {
                p3_set += 1;
                if !p3_levels.contains(lvl) { p3_levels.push(*lvl); }
            }
        }
    }
    assert_eq!(bodies, 35, "the dev discs' main-map 0xD8 census [D]");
    assert_eq!(p3_set, 16, "the carry-light byte is set on 16 of the 35 [D: 01×4, 02×10, 04×2]");
    assert_eq!(p3_levels, vec![0x0700_0001, 0x0700_0002, 0x0700_0004], "and only on those levels [D]");
}

/// The real-fire pin on 07000004 (a census level: both lamps carry p3 = 1, p4 → 0xFFFF [D]).
/// From the level load, Bond walks under lamp #40 and fires the real trigger path at it
/// (`Walker::shoot_point` — the turn/look inputs and the trigger through `Sim::step_bond`).
/// The game's rule [M 0x1d73a0:121-123]: the landed round's class-5 contact makes the
/// handler raise ONE `Sound_Play3D` with the body's own id — here 0xFFFF (none [D census]),
/// which the sound path drops [M sound.rs] — at the body's translation. So: exactly one
/// recorded call for #40 (one bullet, one contact, one call — not one per tick), and the
/// undisturbed sister #39 records NOTHING, before or after. The carry-light rides along: the
/// shot lamp's +0x150 handle stays 0 behind the stub [M 0x1cc3d0], so no light follows the
/// swinging body (the render check for that is in the report's live checks).
#[test]
fn a_shot_lamp_records_its_impact_call_once_and_the_sister_stays_silent() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000004.bin", false).expect("load 07000004"));
    let mut ready = false;
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { ready = true; break; }
    }
    assert!(ready, "the intro + story scenes must settle");
    // The settle window: no 0xD8 impact calls of any kind before a shot (nothing else on this
    // level fills a body's obj+0xD0 with a class-5 hit [M the class-5 arms need that contact]).
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.rb_sound_events.is_empty(), "no impact calls before any shot, got {:?}", sim.drones.rb_sound_events);
    let lamps: Vec<(u16, [f32; 3])> = sim.drones.rbodies.list.iter()
        .filter(|o| o.s154 == 1)
        .map(|o| (o.placement, o.st.trans))
        .collect();
    assert_eq!(lamps.len(), 2, "both p3=1 lamps exist [D census: place 26 ×2]");
    let (shot_id, at) = *lamps.iter().find(|(id, _)| *id == 40).expect("lamp #40");
    let _ = shot_id;
    drop(sim);

    // Under lamp #40 (the interactables test's spot): the walk stands ~3 units out, the lamp
    // hangs at y 7.86 [D #40 t (8.01, 7.86, 23.03)].
    w.walk_nav_to([8.0, 4.4, 26.5], "under the whitelamp", 1.2);
    let lamp_trans = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.rbodies.list.iter().find(|o| o.placement == 40).map(|o| o.st.trans).expect("lamp #40")
    };
    assert_eq!(lamp_trans, at, "the lamp never translates [M EulerEvolve's type-0-only position arm]");
    // Fire the real path; the detector trips on the first recorded impact call for #40.
    let fired = w.shoot_point(at, "the whitelamp", |sim| {
        sim.drones.rb_sound_events.iter().any(|(p, _, _)| *p == 40)
    });
    assert!(fired, "the shot must land: the class-5 contact must raise the impact call [M 0x1d73a0:121-123]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let mine: Vec<_> = sim.drones.rb_sound_events.iter().filter(|(p, _, _)| *p == 40).collect();
        assert_eq!(mine.len(), 1, "ONE call for the ONE landed round — the chain drains per handler tick and the round dies on contact [M Bullet_CollisionHandler 0x12ce80; the port's b.dead], got {}", mine.len());
        let (_, id, pos) = *mine[0];
        assert_eq!(id, 0xffff, "the id is the body's p4 (none on every dev level [D census]; M 0x1d6bf0:108-112)");
        assert_eq!(pos, lamp_trans, "at the body's translation (obj+0xC0; the Rot body never moves) [M 0x1d73a0:122]");
        // The undisturbed sister: no call of hers, ever.
        let other: Vec<u16> = sim.drones.rbodies.list.iter().map(|o| o.placement).filter(|p| *p != 40).collect();
        for p in &other {
            assert!(!sim.drones.rb_sound_events.iter().any(|(q, _, _)| q == p), "lamp {p} was never shot and must stay silent");
        }
        // The carry-light rode along inert: the stub kept the handle at 0 [M 0x1cc3d0].
        assert_eq!(sim.drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().st.s150, 0,
            "the shot lamp's carry-light handle stays 0 — no light follows [M 0x1cc3d0]");
    }
    // And the rule does not tick: 300 quiet ticks later still exactly one call for #40 (the
    // hit chain drained; nothing re-arms it), #39 still silent.
    for _ in 0..300 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.rb_sound_events.iter().filter(|(p, _, _)| *p == 40).count(), 1, "no re-trigger after the chain drained");
    assert!(!sim.drones.rb_sound_events.iter().any(|(p, _, _)| *p != 40), "the level stays silent otherwise");
}
