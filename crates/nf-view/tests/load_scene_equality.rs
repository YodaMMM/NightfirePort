//! The scene-equality proof for the level-load-time cuts (level-load-time step 3).
//!
//! Every cost cut must build the SAME scene. The digests below were captured from the
//! pre-optimization tree (probe session 2026-10-16, `tests/load_scene_digest.rs` — FNV-1a over
//! vertices/indices/materials/CharVertices/characters/attachments/colliders/cels + the sim's
//! drone/pickup/door/bond facts and the stats line, plus the counts) and pinned here. The test
//! rebuilds each level through the CURRENT load path and asserts the digest is unchanged, so a
//! future edit that shifts one vertex, index or material field fails here.
//!
//! Levels: the two heaviest archives (07000014, 0700000c), the archive-heavy mid-game pair
//! (07000005, 07000002) and the smallest (07000001) — 5 of the 27 on-foot levels, spanning the
//! pool-heavy (projectile/tracer/gas/decal/casing/debris) and merge-heavy (session mesh merge)
//! shapes. Opt-in via `NF_ISO`; release-only.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// `(hash, vertices, indices, materials, char_verts, characters, attachments, colliders, cels, drones, pickups)`
/// from `load_scene_digest.rs` on the pre-optimization tree. The full FNV hash pins the contents;
/// the counts pin the shape.
// The baseline is re-pinned only for deliberate scene-content changes. Those so far: the per-character
// light pick (the game picks lights per object [M View_DrawObjList 0x1e69a0:73-81]); the swoosh ribbon
// pools and 32 projectile slots per projectile mesh; the switch-script lasers' ON art (07000014); the
// captains' swapped skins and the 0x050000b1 placements' rewritten request skin
// (`NDrone2_DefaultInit` 0x14b300:563-565 + the :82-95 rewrite — 0x050000b1 → 0x0500008c;
// the raw skin stays unregistered, AnimLoadFile 0x1bdf40); the 0x20
// breakables in the posed slot space (`Break_Kill` 0x12b970); the rain-mesh slots on DropType-1
// SNOW_gen levels (`DrawDrops` 0x18cb90, mesh 0x02000290); the watch-hand attachments; the
// suppressor swap piece (bone 43, state 8) without the duplicate +0x7C view-entity extra; and one
// draw slot per ScriptPlayer running a script; the 0700000c walkway soldiers now drawn; and no
// characters for drone placements above the load's difficulty (`Drone_Create` 0x136be0 makes none;
// 07000014 drops 18 such instances).
// camera-beam-satchel-draw: the cameras' third posed slot (the lens beam, Sensor_SetBeam 0x1af398) — 14's beams add vertices; 02's slot layout shifts.
// batch-1024a: the security cameras' body + mount posed slots (`Sensor_Create` 0x1af010 / `Sensor_SetBeam`
// 0x1af398 — 01 ×1, 02, 14 ×14), 14's b1 drones on skin 0x0500008c (`NDrone2_DefaultInit` 0x14b300:82-95).
const BASELINE: &[(&str, &str)] = &[
    // pickup-medal-gate re-pin: the fresh-boot loads now run `Pickup_Create`'s kind-6 gate
    // [M 0x18dda0:93-95] — the 007 rows (kind 6) create only with the level's gold medal
    // recorded, and these pins were captured with bonus[0] = 0 (the fresh-boot word [L
    // ee_start_05 0x2e1b88]). Each drifted level lost exactly its kind-6 rows' meshes: −418
    // verts / −314 indices / −1 material slot per token (07000002 −3, 07000005 −2, 0700000c −4,
    // 07000001 −2 pickups; 07000014 places no kind-6 and is bit-identical).
    // level-1b-audit re-pin (all four drifted levels carry a 0x3b gated dynamic object): the
    // 0x3b draw moved from a build-time static World instance (which never collapsed when the
    // sim deleted the object — DynamicObject_Update 0x18d3b8:26 mode-0 armed-B → obj+0xFE bit 0
    // → the delete pass, control_delete_object 0x133178:16-17) to a posed Character-layer slot
    // that collapses; the vertices/indices/materials/char_verts counts follow, [S port
    // engineering] like every pin here.
    // gunbeam-0d re-pin: two tracer slots for the mounted laser's beam quad 0x20005cd + flare 0x200026e (+60 verts, +2 slots per level).
    ("07000002.bin", "846c65fab8fa89fc 4515435 9444387 5350 4378448 81 10723 205 28 25 5"),
    ("07000005.bin", "8309b1c9b48e67e8 4418933 9243759 3615 4334592 73 10657 106 19 16 4"), // the two copter SPs + their script draw slots; re-pinned 10-26: the 0x3b dynamic objects ride posed slots [M View_AddObjects 0x1e62d8:33, DynamicObject_Update 0x18d3b8]
    // lift-indicators-09 re-pin: one draw slot per `Script_SubScriptStart` op per running SP
    // [M 0x1db050] — 0700000c's multi-SP sub chains add 6 slots (+248 vertices, +576 indices,
    // +1 attachment); 07000014's vent-lid SPs are placed SPs (unchanged), the rest have no
    // multi-SP sub chains.
    ("0700000c.bin", "d1cd20df0ed8f8c4 4903050 10313148 8126 4714818 96 11910 170 22 40 11"), // re-pinned 10-26: 0x3b on posed slots
    // script-entity-copies: one draw slot per ScriptPlayer running a script — 14's nine
    // SPs on the vent-lid script 0x06000A3B (+ its other shared scripts) add 20 attachments, +1600 vertices.
    // The keypads' LOD pair in the posed slots (Lock_Unlock 0x1b0650 / case 1 `Control_NextLOD`: three
    // KeypadLocked + KeypadUnlocked) and the decryptor screen's PdaDigits on datums 0..3 (10 digits x 4
    // datums x the 0x56/0x57 view models = +80 attachments) [M Player_WeaponFiring 0x1a5460:155-194].
    // cite-14-skinless-drones re-pin: 07000014's five non-captain 0x050000b1 placements now draw
    // their characters through the Drone+0xe0 rewrite (0x050000b1 → 0x0500008c before the
    // AnimObjectNew request [M 0x14b300:82-95, :563-565]) — +5 characters, +30005 vertices,
    // +59235 indices, +105 materials, +50005 char_verts, +15 attachments over the old pin.
    // fusebox-draw-slots re-pin: 07000014's five fusebox placements (kind 0x31) now seed their
    // param-0 script 0x06000A1B, +20 script-entity slots (the cabinet and wire art) [M FuseBox_Create].
    // 07000014 places two kind-6 rows as well: −836 verts / −752 indices / −2 material slots,
    // pickups 32 → 30 (same gate).
    ("07000014.bin", "b80e0ab8213520a0 7180663 14921637 6485 7029324 98 14424 173 41 37 30"), // re-pinned 10-26: p0-gated 0x3b instanced from load (created hidden [M 0x18dbf8:27-29])
    ("07000001.bin", "aa4290c18e56d4ef 4428113 9284853 3488 4355134 76 10737 101 11 20 4"), // re-pinned 10-26: the radio-room rotors on posed slots [M rotor_update 0x18f568]
];

#[test]
fn the_cut_costs_build_the_same_scene() {
    let Some(iso) = iso() else { return };
    if cfg!(debug_assertions) { eprintln!("scene equality: release-only, skipped on a debug build"); return; }
    let mut drifted: Vec<String> = Vec::new();
    for (level, expected) in BASELINE {
        let scene = nf_view::load(&iso, level, false).unwrap_or_else(|e| panic!("{level}: {e}"));
        let sim = scene.sim.as_ref().expect("a level scene carries a sim");
        let got = format!("{} {} {} {} {} {} {} {} {} {} {}",
            // The digest hash itself (tests/load_scene_digest.rs computes it; recompute here over
            // the same fields so this test is self-contained).
            digest_hash(&scene, sim),
            scene.vertices.len(), scene.indices.len(), scene.materials.len(), scene.char_verts.len(),
            scene.characters.len(), scene.attachments.len(), scene.colliders.len(), scene.cels.len(),
            sim.drones.list.len(), sim.drones.pickups.len());
        // [S port engineering] scene-equality of the load-time cuts (digests re-captured on master bee8c129), no game claim
        eprintln!("{level}: {got}");
        if &got != expected { drifted.push(format!("{level}: got {got}, pinned {expected}")); }
        assert!(scene.vertices.len() > 0, "{level}: the pin must not be vacuous");
    }
    assert!(drifted.is_empty(), "the built scenes changed (level-load-time cuts must be scene-preserving):\n{}", drifted.join("\n"));
}

/// The same FNV-1a field walk `tests/load_scene_digest.rs` runs (kept in sync; see that file).
fn digest_hash(s: &nf_view::scene::Scene, sim: &nf_view::scene::Sim) -> String {
    use std::hash::Hasher;
    let mut h: u64 = 0xcbf29ce484222325;
    let mut f = |h: &mut u64, b: &[u8]| { for &x in b { *h ^= x as u64; *h = h.wrapping_mul(0x100000001b3); } };
    for v in &s.vertices {
        for p in v.pos { f(&mut h, &p.to_bits().to_le_bytes()); }
        for p in v.uv { f(&mut h, &p.to_bits().to_le_bytes()); }
        f(&mut h, &v.rgba); f(&mut h, &v.nrm.map(|x| x as u8));
    }
    for i in &s.indices { f(&mut h, &i.to_le_bytes()); }
    for m in &s.materials {
        f(&mut h, &[m.layer as u8]);
        for p in m.centre { f(&mut h, &p.to_bits().to_le_bytes()); }
        f(&mut h, &m.radius.to_bits().to_le_bytes());
        f(&mut h, &m.cel.map(|c| c.to_le_bytes()).unwrap_or([0xff, 0xff]));
        f(&mut h, &m.chain.to_le_bytes()); f(&mut h, &m.node.to_le_bytes());
        if let Some((map, tex)) = m.key { f(&mut h, &(map as u64 | ((tex as u64) << 32)).to_le_bytes()); } else { f(&mut h, &[0xff]); }
        f(&mut h, &m.lflags.to_le_bytes());
        f(&mut h, &m.first_index.to_le_bytes()); f(&mut h, &m.index_count.to_le_bytes());
    }
    for cv in &s.char_verts {
        f(&mut h, &cv.character.to_le_bytes());
        for p in cv.pos { f(&mut h, &p.to_bits().to_le_bytes()); }
        f(&mut h, &cv.weight.to_bits().to_le_bytes());
        f(&mut h, &cv.slots[0].to_le_bytes()); f(&mut h, &cv.slots[1].to_le_bytes());
        f(&mut h, &cv.attachment.map(|a| a.to_le_bytes()).unwrap_or([0xff; 4]));
        f(&mut h, &cv.searchlight.map(|a| a.to_le_bytes()).unwrap_or([0xff; 2]));
        f(&mut h, &cv.door.map(|a| a.to_le_bytes()).unwrap_or([0xff; 2]));
        f(&mut h, &cv.pickup.map(|a| a.to_le_bytes()).unwrap_or([0xff; 2]));
        f(&mut h, &cv.morph.to_le_bytes());
    }
    for c in &s.characters {
        f(&mut h, &c.skin.hashcode.to_le_bytes());
        for row in &c.matrix { for p in *row { f(&mut h, &p.to_bits().to_le_bytes()); } }
        if let Some(fl) = c.floor { f(&mut h, &fl.to_bits().to_le_bytes()); } else { f(&mut h, &[0xff]); }
    }
    for a in &s.attachments {
        f(&mut h, &a.character.to_le_bytes()); f(&mut h, &a.bone.to_le_bytes());
        for row in &a.local { for p in *row { f(&mut h, &p.to_bits().to_le_bytes()); } }
    }
    for (name, lo, hi) in &s.colliders {
        f(&mut h, name.trim().as_bytes());
        for p in *lo { f(&mut h, &p.to_bits().to_le_bytes()); }
        for p in *hi { f(&mut h, &p.to_bits().to_le_bytes()); }
    }
    f(&mut h, &sim.level.to_le_bytes());
    for p in sim.bond.pos { f(&mut h, &p.to_bits().to_le_bytes()); }
    f(&mut h, &sim.drones.list.len().to_le_bytes());
    f(&mut h, &sim.drones.pickups.len().to_le_bytes());
    f(&mut h, &sim.drones.doors.list.len().to_le_bytes());
    f(&mut h, s.stats.as_bytes());
    format!("{h:016x}")
}
