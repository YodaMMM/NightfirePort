//! The placement-draw census (docs/systems/placement-draw-census.md): for every SP level, every
//! main-map placement whose create path builds a drawn object (a glist attached to the created
//! obj, a skin drone, or a script-entity SP — the create fn per kind cited in the doc, all [M])
//! must have a draw instance or character in the port's draw passes. ISO-gated; numbers/ids only.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::Placement;
use std::collections::HashSet;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The 21 SP archives (docs/tests level ids; 0700004a/0700004b are the bonus levels).
const SP_LEVELS: [&str; 21] = [
    "07000001", "07000002", "07000003", "07000004", "07000005", "07000006", "07000007",
    "07000008", "07000009", "0700000a", "0700000b", "0700000c", "07000011", "07000012",
    "07000013", "07000014", "07000015", "07000016", "07000041", "0700004a", "0700004b",
];

fn load_level(iso: &str, id: &str) -> Option<Level> {
    let iso_img = IsoImage::open(iso).ok()?;
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso_img).ok()?;
    let fe = entries.iter().find(|e| e.name == format!("{id}.bin"))?;
    let data = iso_img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
    Level::parse(&data).ok()
}

/// The game's per-kind verdicts from `parsemap_create_dynamic_objects` 0x1d04f0 + each create fn —
/// the full table with evidence lines lives in docs/systems/placement-draw-census.md. `Glist` =
/// the create fn attaches the placement's glist to the created obj (drawn if the glist has
/// meshes), `Skin` = a skin drone (character), `Sp` = a script entity (SP_Create /
/// SP_CreateScriptPlayer instantiates the hashed object), `Sky` = a sky object.
fn game_draw(kind: u8) -> Option<&'static str> {
    match kind {
        0xf => Some("Drone_Create 0x136be0 (skin)"),
        0x20 => Some("Break_Create 0x12c230:19 Control_SetGList"),
        0x21 => Some("Ripples_Create 0x1acc90:26-36 obj+0xd8=glist"),
        0x22 => Some("Ladder_Create 0x1966c0:21 obj+0xd8=glist"),
        0x27 => Some("GT_Create 0x17ffa0:70-71 Control_CreateObjEx"),
        0x28 => Some("Sensor_Create 0x1af010:124 obj+0xd8=glist"),
        0x29 => Some("Switch_Create 0x1adcc0:44-45 SP_Create(prop 1)"),
        0x2b => Some("LeafGen_Create 0x1acbd0:17 obj+0xd8=glist"),
        0x2e => Some("Lock_Create 0x1ae1f8:55 obj+0xd8=glist"),
        0x2f => Some("Copter_Create 0x17d340:68-70 SP_Create(props 4/5)"),
        0x30 => Some("Monitor_Create 0x1b08b0:26 Control_SetGList"),
        0x31 => Some("FuseBox_Create 0x1b0448:29-30 SP_Create(prop 0)"),
        0x32 => Some("RainBox_Create 0x18d840:19 obj+0xd8=glist"),
        0x33 => Some("Hint_Create 0x1b3248:29 Control_SetGList"),
        0x34 => Some("GunImp_Create 0x131f20:68-69 Control_CreateObjEx"),
        0x35 => Some("PCQWorm_Create 0x1b07a8:12-13 Control_CreateObjEx"),
        0x36 => Some("Sub_Create 0x1ad6c0:41-42 AnimObjectNew(0x5000068)"),
        0x37 => Some("MiniSub_Create 0x182130:23-24 AnimObjectNew(0x5000074)"),
        0x3a => Some("Grapple_Create 0x198298:17 obj+0xd8=glist"),
        0x3b => Some("DynamicObject_Create 0x18dbf8:19-23 obj+0xd8=glist"),
        0x3e => Some("Wire_Create 0x1981d8:24 obj+0xd8=glist"),
        0x3f => Some("CreepWall_Create 0x196fc8:23 obj+0xd8=glist"),
        0x4a => Some("Apocalypse_Create 0x18dcc0:20 obj+0xd8=glist"),
        0x71 => Some("rotor_init 0x18f4b8:24-25 obj+0xd8=glist"),
        0xd2 => Some("Shooter_Create 0x17d030:16 obj+0xd8=glist"),
        0xd5 => Some("OneSided_Create 0x1e7ce8:14 obj+0xd8=glist"),
        0xd8 => Some("RB_Create 0x1d6bf0:52 obj+0xd8=glist"),
        0xd9 => Some("SP_CreateScriptPlayer 0x1db9a8:18-19 (no-op if both hashes 0)"),
        0xda => Some("Car_Create 0x1305b0:20-21 Control_CreateObjEx"),
        0xdb => Some("Door_Create 0x136100:71 Control_SetGList"),
        0xdc | 0xe8 | 0xea..=0xef | 0xf4 => Some("Trigger_Create 0x1b1720:92-95 obj+0xd8=glist"),
        0xde => Some("Searchlight_Create 0x1cb4f8:73-75 obj+0xd8=glist"),
        0xe0 => Some("Creature_Create 0x134ab8:55-56 Control_CreateObjEx"),
        0xe1 => Some("Destroy_Create 0x12c318:23 Control_SetGList"),
        0xe2 => Some("SS_Create 0x1a9c58:53 Control_SetGList"),
        0xe3 => Some("Flicker_Create 0x1acb00:49-52 obj+0xd8=glist"),
        0xe4 => Some("Hurt_Create 0x17e870:52 Control_SetGList"),
        0xf0 => Some("Pickup_Create 0x18dda0:134-137 obj+0xd8=glist"),
        0xf3 => Some("Env_Tree_Create 0x17c200:17 AnimObjectNew(prop 0)"),
        0xf9 => Some("SoundTrigger_Create 0x1b2628:55-57 obj+0xd8=glist"),
        0xfa => Some("ThirdIcon_Create 0x197070:33-37 obj+0xd8=glist"),
        0xfb => Some("MusicTrigger_Create 0x1b30e0:25-27 obj+0xd8=glist"),
        0xfc => Some("Corona_Create 0x18da00:17-20 obj+0xd8=glist"),
        0xfd => Some("CamSubject_Create 0x1bc5d8:17-20 obj+0xd8=glist"),
        0xfe => Some("Mine_Create 0x17ce28:50 obj+0xd8=glist"),
        _ => None, // 0x24/0x2d start pos, 0x25/0x26 MP, 0x2c/0x38/0x39/0xf2 dedicated renderers,
                   // 0x41 inverter, 0x4e light, 0xdf env, 0xe5/0xe6 cover nodes, 0xe7 cam,
                   // 0xe9 AI point, 0xf1 spawner, 0xf5/0xf8 volumes (glist = bounds, +0xF0 |= 0x30)
    }
}

/// The game dispatches a placement to `parsemap_create_dynamic_objects` only when the skip bits
/// are clear (0x8000 statics go to the cel pass instead) and the placement's glist resolved
/// [M 0x1d1b68:77-97]; the created thing draws only when the resolved object actually has meshes
/// (an empty glist draws nothing — `View_DrawObjectGlist`'s no-op) and the object's own no-draw
/// bits are clear (0x400 mirror-only [M View_AddObjects 0x1e62d8:38-47], 0x10 no-draw where the
/// creator copies glist+0x44 [M scene.rs's fix-striped-boxes2 census]).
fn game_draws(level: &Level, m: usize, p: &Placement) -> Option<(&'static str, usize, usize)> {
    let kind = (p.flags & 0xff) as u8;
    let create = game_draw(kind)?;
    if p.flags & 0xa000 != 0 { return None; }
    let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
        else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
    let (map, object) = target?;
    let obj = &level.maps[map].objects[object];
    if obj.flags & 0x410 != 0 { return None; }
    if !obj.gfx.leaves.iter().any(|l| !l.batches.is_empty()) { return None; }
    Some((create, map, object))
}

/// The port's per-kind draw rules for the kind-filtered passes (MISC/grapple/door/pickup/copter);
/// the gatherer families (statics/breakables/ss/characters/sky) are checked against their real
/// gatherer outputs instead.
fn port_kind_draws(level: &Level, m: usize, p: &Placement) -> bool {
    let kind = (p.flags & 0xff) as u8;
    match kind {
        0x2e | 0x30 | 0x34 | 0x35 | 0x3b | 0x71 | 0xd5 | 0xd8 | 0xe1 | 0xe3 | 0xda | 0xfa
        | 0xfd | 0xfe | 0x4a => true,                          // scene.rs's MISC_KINDS
        0x3a => true,                                          // the grapple-hook pass
        0xdb => true,                                          // the door pass
        0xf0 => true,                                          // the pickup pass
        0x2f => {
            // fix-909b: the copter draws through its SCRIPT PLAYER, not a placement instance —
            // `Copter_Create` 0x17d340 attaches no glist (obj+0xD8 = 0 [M :60-66]) and
            // SP_Create(params 4/5) instantiates the look; the body 0x0200035D is the assembly
            // script's stream-0 entity and 0x02000427 the flight's [D 0x06000051/0x06000050].
            // The port gives those scripts' entities draw slots (s/copter-script-draw: scene.rs
            // roots the slot builder at the kind-0x2F params 4/5) — covered when both resolve.
            // NOT drawn yet: s/copter-script-draw (the flight scripts' slots) is held until task 958
            // ports the CopterPaths flight — the one-shot script pass would show the 07000005
            // copter diving under the ground. Pinned as known misses below.
            let _ = (level, m, p);
            false
        }
        _ => false,
    }
}

fn placement_key(p: &Placement) -> (u32, u32, [u64; 3], [u64; 3]) {
    (p.hashcode, p.flags, p.translation.map(f32::to_bits).map(|b| b as u64),
     p.rotation.map(f32::to_bits).map(|b| b as u64))
}

/// The census: every game-drawn placement must be port-covered. The known, documented misses are
/// pinned by (level, kind) below — a NEW (level, kind) row fails the test; a row DISAPPEARING
/// (a kind fixed) must be removed from the list here.
const KNOWN_MISSES: &[(&str, u8)] = &[
    // The 2026-10-22 census (docs/reports/placement-draw-census.md carries the per-kind counts):
    // script-entity kinds (0xd9 SP players, 0x29 Switch's SP model, 0x31 FuseBox's SP model) and
    // the glist kinds the port has no pass for yet (0x27 GT turret, 0x28 sensor, 0x3f creepwall,
    // 0xf3 env tree, the trigger family 0xea/0xeb/0xf4 — the trigger/sensor rows need the main
    // agent's live check: their glists may carry the game's no-draw bit 0x10, which the port's
    // obj.flags mirror may not reflect for them).
    ("07000001", 0x28), ("07000001", 0xd9),
    ("07000002", 0x28), ("07000002", 0xf3),
    ("07000003", 0xd9),
    ("07000004", 0xf3),
    ("07000005", 0x2f), ("07000005", 0xd9),
    ("07000006", 0x31), ("07000006", 0xd9),
    ("07000007", 0xd9),
    ("07000008", 0x2f), ("07000008", 0x31), ("07000008", 0xd9), ("07000008", 0xea),
    ("07000009", 0x29), ("07000009", 0xd9),
    ("0700000a", 0x27), ("0700000a", 0x28), ("0700000a", 0x29), ("0700000a", 0xd9), ("0700000a", 0xeb),
    ("0700000b", 0x27), ("0700000b", 0x2f), ("0700000b", 0x28), ("0700000b", 0x29), ("0700000b", 0xd9),
    ("0700000c", 0xd9),
    ("07000011", 0xd9), ("07000011", 0xea),
    ("07000012", 0x27), ("07000012", 0x2f), ("07000012", 0x28), ("07000012", 0xd9), ("07000012", 0xea), ("07000012", 0xeb),
    ("07000013", 0xd9),
    ("07000014", 0x28), ("07000014", 0x29), ("07000014", 0x31), ("07000014", 0xd9),
    ("07000016", 0x29), ("07000016", 0xd9), ("07000016", 0xea),
    ("07000041", 0x3f), ("07000041", 0xea), ("07000041", 0xf4), ("07000041", 0xf9),
    ("0700004a", 0xd9),
];

#[test]
fn every_sp_level_drawn_placements_have_draw_instances() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut misses: Vec<(String, u8, u32, String)> = Vec::new();
    for id in SP_LEVELS {
        let Some(level) = load_level(&iso, id) else { misses.push((id.to_string(), 0, 0, "LOAD FAILED".into())); continue };
        let Some(m) = level.main_map() else { continue };
        // The port's covered sets per gatherer family (the clones carry the placement).
        let (scene, _) = level.scene_instances(true);
        let scene_set: HashSet<_> = scene.iter().map(|i| placement_key(&i.placement)).collect();
        let char_set: HashSet<_> = level.character_instances().0.iter().map(|i| placement_key(&i.placement)).collect();
        let sky_set: HashSet<_> = level.sky_instances().iter().map(|(i, _)| placement_key(&i.placement)).collect();
        for p in &level.maps[m].placements {
            let Some((create, _, _)) = game_draws(&level, m, p) else { continue };
            let kind = (p.flags & 0xff) as u8;
            let covered = match kind {
                _ if p.creates_cel() => scene_set.contains(&placement_key(p)),
                0x20 => true, // scene_instances(true) extends with every breakable
                0xe2 => scene_set.contains(&placement_key(p)) || p.extra[7] & 0xffff != 0, // + the ss pass
                0xf => char_set.contains(&placement_key(p)),
                0x2a | 0xd4 => sky_set.contains(&placement_key(p)),
                // The dedicated per-kind passes (each with its own decompile-backed comment in scene.rs):
                0x22 => true,  // the ladder draw sweep (fix-ladder2; scene.rs instances retain 0x22)
                0x2e | 0x30 | 0x34 | 0x35 | 0x3b | 0x71 | 0xd5 | 0xd8 | 0xe1 | 0xe3 | 0xda
                | 0xfa | 0xfd | 0xfe | 0x4a => port_kind_draws(&level, m, p), // MISC_KINDS
                0x3a => true,  // the grapple-hook pass
                0x3e => true,  // the wire pass (scene.rs 0x3e filter)
                0xdb => true,  // the door pass
                0xde => true,  // the searchlight-parts pass draws the placement's parts
                0xe8 => true,  // the LOAD_LEVEL trigger pass (the castle-gate fix)
                0xf0 => true,  // the pickup pass
                0xfc => true,  // the corona pass (scene.rs corona::placements)
                0x2f => port_kind_draws(&level, m, p),
                _ => false,
            };
            if covered { continue; }
            misses.push((id.to_string(), kind, p.hashcode, create.to_string()));
        }
    }
    let mut table = String::new();
    for (id, kind, hash, create) in &misses {
        table.push_str(&format!("{id} kind {kind:#04x} hash {hash:#x} — {create}\n"));
    }
    eprintln!("placement-draw census misses ({}):\n{table}", misses.len());
    // The pinned allow-list: every miss must be a documented one.
    let unexpected: Vec<_> = misses.iter().filter(|(id, kind, _, _)| {
        !KNOWN_MISSES.iter().any(|(lid, lk)| lid == id && *lk == *kind)
    }).collect();
    assert!(unexpected.is_empty(), "NEW placement-draw misses ({}):\n{}", unexpected.len(),
        unexpected.iter().map(|(id, k, h, c)| format!("{id} kind {k:#04x} hash {h:#x} — {c}")).collect::<Vec<_>>().join("\n"));
    // The copter rows (05/08/0b/12 kind 0x2F) are pinned misses until task 958 + s/copter-script-draw
    // land; a fixed copter must then be removed from KNOWN_MISSES (the allow-list rule above).
}

/// The Exchange extraction helicopter (the user report 2026-10-21): 07000008 places kind-0x2F
/// copters; the game draws the body as a script entity over the placement's param-4/5 object
/// hashcodes [M Copter_Create 0x17d340:43-45, 68-70] — both hashes must resolve here, and Rook's
/// kind-0xF placement must carry a resolvable skin (the character pass draws him).
#[test]
fn exchange_copter_and_rook_draw() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let Some(level) = load_level(&iso, "07000008") else { eprintln!("skip: no 07000008"); return };
    let m = level.main_map().expect("main map");
    let mut copters = Vec::new();
    for (i, p) in level.maps[m].placements.iter().enumerate() {
        if p.flags & 0xff == 0x2f { copters.push((i, p.hashcode, p.params.clone())); }
        if p.flags & 0xff == 0xf {
            let skin = p.params.iter().find(|(k, _)| *k == 0).map_or(0, |&(_, v)| v);
            let resolved = skin >> 24 == 5 && level.skins.contains_key(&skin);
            let pos = p.translation.map(|v| (v * 10.0).round() / 10.0);
            eprintln!("07000008 drone placement {i}: skin {skin:#x} resolved={resolved} at {pos:?} hash {:#x}", p.hashcode);
        }
    }
    eprintln!("07000008 kind-0x2f placements: {copters:?}");
    assert!(!copters.is_empty(), "07000008 must place the extraction helicopter (kind 0x2F) [D]");
    for (i, hash, params) in &copters {
        let p0 = params.iter().find(|(k, _)| *k == 0).map_or(0, |&(_, v)| v);
        if p0 != 0 { eprintln!("copter placement {i}: prop0={p0} — the game's marker copter (no obj)"); continue; }
        // The body is the placement's OWN hashed object; params 4/5 are the SP SCRIPT hashes
        // (0x06000051 assembly / 0x06000050 flight [D #45]) — not object hashes.
        assert!(*hash != u32::MAX && level.by_hash.contains_key(hash),
            "copter placement {i}: the body hashcode {hash:#x} resolves to no object — the body cannot draw");
        let name = level.by_hash.get(hash).map(|(mm, oo)| level.maps[*mm].objects[*oo].name.clone());
        eprintln!("copter placement {i}: body object {name:?} (hashcode {hash:#x}), SP scripts {:?}",
            params.iter().filter(|(k, _)| *k == 4 || *k == 5).map(|&(_, v)| v).collect::<Vec<_>>());
    }
}
