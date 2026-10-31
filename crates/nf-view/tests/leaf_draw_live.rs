//! The falling leaves draw. `LeafGen_Update` 0x1acc48 spawns a `Leaf_Create` 0x1abc88 object
//! (class 0x40, gfx "Leaf" 0x020002ca) and `Leaf_Update` 0x1abee0 moves it; the port simulated
//! them (`WorldFx::leaves`) but no draw slot read them. The game draws each leaf as an ordinary
//! glist object: the forced-object arm (`build_LinkToRoom` 0x1c9680 on +0xf0 bit 0x20000000 →
//! `View_AddForcedObjects` 0x1e5fb8), the mesh's flag bit 0x1 → sort list 1 →
//! `View_DrawObjectGlist` 0x1e6780 at `Control_GetObjMatrix` 0x134210's
//! `View_RotTransMatrix(obj+0x50, obj+0x30)`, scale 1, tinted by the generator cel's ambient
//! (`control_link_object_to_cel` 0x1346b8 + `View_SetupRenderModes_Tweak` 0x1e5888's 0x800 arm).
//!
//! These tests pin the draw records: one Effect pool per generator (07000004 places two [D]),
//! the Leaf mesh's own texture and GS state, every live leaf posed at its own matrix, nothing
//! posed for a leaf still on its spawn tick, and the generator-cel tint in the colour slot.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_view::scene::{Layer, Scene, LEAF_MESH, LEAF_SLOTS};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(level: &str) -> Scene {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    nf_view::load(&iso().unwrap(), level, false).unwrap_or_else(|e| panic!("load {level}: {e}"))
}

fn tick(scene: &mut Scene, n: u32) {
    for _ in 0..n { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
}

fn is_far(m: &[[f32; 4]; 4]) -> bool { m[3][1] <= -9000.0 }

/// The leaf object in the level archive (by_hash is first-wins, the scene's maps keep its index).
fn leaf_object(level_name: &str) -> (usize, Option<u32>, nf_formats::level::Level) {
    let iso_img = nf_formats::iso9660::IsoImage::open(iso().unwrap()).expect("iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso_img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(level_name)).expect("level entry");
    let archive = iso_img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("archive");
    let level = nf_formats::level::Level::parse(&archive).expect("level");
    let &(lm, lo) = level.by_hash.get(&LEAF_MESH).expect("the Leaf mesh resolves");
    let tex = level.maps[lm].objects[lo].gfx.leaves.iter().flat_map(|l| &l.batches).find_map(|b| b.texture);
    (lm, tex, level)
}

#[test]
fn the_leaves_draw_through_a_pool_per_generator() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    if cfg!(debug_assertions) { eprintln!("release-only"); return; }
    let mut scene = load("07000004.bin");
    let (lm, tex, level) = leaf_object("07000004.bin");
    let obj = &level.maps[lm].objects[lo_of(&level)];
    assert_eq!(obj.name.trim(), "Leaf");
    assert_eq!(obj.flags, 0xaa1, "the Leaf glist flags [D]: bit 0x1 alpha list, 0x800 tint dim, no 0x8/0x4 billboard/jitter");
    let gens = scene.sim.as_ref().unwrap().drones.fx.leafgens.len();
    assert_eq!(gens, 2, "07000004 places two 0x2b generators [D main map #41/#42]");

    // 1. The pool: LEAF_SLOTS attachments per generator, none past the last generator.
    for g in 0..gens {
        for k in 0..LEAF_SLOTS {
            assert!(scene.effect_attachment(9, g * LEAF_SLOTS + k, LEAF_MESH).is_some(), "generator {g} slot {k} has a draw slot");
        }
    }
    assert!(scene.effect_attachment(9, gens * LEAF_SLOTS, LEAF_MESH).is_none(), "no slot past the last generator");

    // 2. The draw records: the Leaf texture, the Effect pass, one strip per slot.
    let key = tex.map(|t| (lm, t));
    let ai0 = scene.effect_attachment(9, 0, LEAF_MESH).unwrap();
    let verts = scene.char_verts.iter().filter(|cv| cv.attachment == Some(ai0)).count();
    // The material that carries the leaf slots: its indices reach the slot-0 verts.
    let owns = |m: &nf_view::scene::Material| scene.indices[m.first_index as usize..(m.first_index + m.index_count) as usize].iter()
        .any(|&v| (v as usize).checked_sub(scene.char_vertex_start).and_then(|c| scene.char_verts.get(c)).is_some_and(|cv| cv.attachment == Some(ai0)));
    let mats: Vec<_> = scene.materials.iter().filter(|m| owns(m)).collect();
    assert_eq!(mats.len(), 1, "the leaf slots ride one material");
    let m = mats[0];
    // The Leaf mesh's GS registers [D]: ALPHA_1 0x44 (Cs−Cd)·As+Cd, ZBUF_1 0x1_3100_0000 (ZMSK —
    // no depth write), TEST_1 0x507fd (alpha test on, AREF 0x7f, depth test GEQUAL).
    let gs: Vec<_> = obj.gfx.leaves.iter().flat_map(|l| l.gs.iter().map(|&(r, v, _)| (r, v))).collect();
    assert_eq!(gs, vec![(0x42, 0x44), (0x4e, 0x1_3100_0000), (0x47, 0x507fd)], "the Leaf mesh's GS registers [D]");
    assert_eq!(m.layer, Layer::Effect);
    assert_eq!(m.key, key, "the leaf slots bind the Leaf mesh's own texture {key:?}");
    let per_slot = m.index_count as usize / (gens * LEAF_SLOTS);
    assert_eq!(m.index_count as usize, per_slot * gens * LEAF_SLOTS, "the pool's indices are whole slots");
    eprintln!("LEAF material: index_count {} per_slot {per_slot} draw {:?}", m.index_count, m.draw);
    assert_eq!(per_slot, 12, "the Leaf strip (8 verts) is 4 triangles per slot [D]");
    assert_eq!(m.draw.pass, 5, "the leaves draw in the effects pass");
    assert_eq!((m.draw.blend, m.draw.zwrite, m.draw.atest), (LEAF_BLEND, LEAF_ZWRITE, LEAF_ATEST), "the Leaf mesh's own GS state [D]");
    assert_eq!(verts, 8, "a leaf slot carries the mesh's 8 verts");

    // 3. The pose: after a long run every live leaf rides a slot at its own matrix; free slots
    //    and spawn-tick leaves stay far. The peak never overflows the slots.
    let (layout, _attrs) = scene.skin_layout();
    let mut peak = 0usize;
    let mut saw_fresh = false;
    let mut checked = 0usize;
    for t in 0..1500 {
        tick(&mut scene, 1);
        let sim = scene.sim.as_ref().unwrap();
        for g in 0..gens {
            peak = peak.max(sim.drones.fx.leaves.iter().filter(|l| l.gen as usize == g).count());
        }
        if t % 50 != 49 && !(sim.drones.fx.leaves.iter().any(|l| l.frames_left == 256) && !saw_fresh) { continue; }
        let (pal, cols) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
        let sim = scene.sim.as_ref().unwrap();
        for g in 0..gens {
            let live: Vec<_> = sim.drones.fx.leaves.iter().filter(|l| l.gen as usize == g && l.frames_left < 256).collect();
            for k in 0..LEAF_SLOTS {
                let ai = scene.effect_attachment(9, g * LEAF_SLOTS + k, LEAF_MESH).unwrap();
                let pm = pal[(layout.attach_base + ai) as usize];
                match live.get(k) {
                    Some(l) => {
                        let want = nf_formats::placement::object_matrix(l.rot, l.pos);
                        assert_eq!(pm, want, "slot {k} of generator {g} poses its leaf at View_RotTransMatrix(rot, pos) [M 0x134210]");
                        checked += 1;
                    }
                    None => assert!(is_far(&pm), "free slot {k} of generator {g} stays far"),
                }
                // The tint: the generator cel's ambient, 0xff·amb >> 8 [M 0x1e5888, 0x1346b8].
                let amb = sim.ambient_at(sim.drones.fx.leafgens[g].pos).expect("the generator sits in a cel");
                let c = cols[(layout.col_att + ai) as usize];
                for ch in 0..3 {
                    let want = ((0xff * amb[ch] as u32) >> 8) as f32 / 255.0;
                    assert!((c[ch] - want).abs() < 1e-6, "leaf colour channel {ch} = {} want {want}", c[ch]);
                }
                assert_eq!(c[3], 1.0, "alpha obj+0x106 stays 0xff (Leaf_Update spends it at cost 0)");
            }
            // A spawn-tick leaf is not drawn: its world sphere is unbuilt until the next movement pass.
            for l in sim.drones.fx.leaves.iter().filter(|l| l.gen as usize == g && l.frames_left == 256) {
                saw_fresh = true;
                let at = |m: &[[f32; 4]; 4]| (m[3][0] - l.pos[0]).abs() < 1e-6 && (m[3][1] - l.pos[1]).abs() < 1e-6 && (m[3][2] - l.pos[2]).abs() < 1e-6;
                let hit = (0..LEAF_SLOTS).any(|k| at(&pal[(layout.attach_base + scene.effect_attachment(9, g * LEAF_SLOTS + k, LEAF_MESH).unwrap()) as usize]));
                assert!(!hit, "a leaf on its spawn tick has no posed slot");
            }
        }
    }
    eprintln!("LEAF peak per generator {peak}, posed checks {checked}, fresh seen {saw_fresh}");
    assert!(peak <= LEAF_SLOTS, "the per-generator peak {peak} fits the {LEAF_SLOTS} slots");
    assert!(checked > 50, "the run posed live leaves ({checked} checks)");
    assert!(saw_fresh, "the run met a spawn-tick leaf");
}

/// Sister: a level with no 0x2b generator gets no leaf pool (07000001 places none [D]), so the
/// pool check above can fail.
#[test]
fn a_level_without_generators_has_no_leaf_pool() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    if cfg!(debug_assertions) { eprintln!("release-only"); return; }
    let scene = load("07000001.bin");
    assert!(scene.sim.as_ref().unwrap().drones.fx.leafgens.is_empty());
    assert!(scene.effect_attachment(9, 0, LEAF_MESH).is_none(), "no leaf slot without a generator");
    assert!(scene.attachments.iter().all(|a| !matches!(a.effect, Some((9, _, _)))));
}

fn lo_of(level: &nf_formats::level::Level) -> usize { level.by_hash[&LEAF_MESH].1 }

/// The port's draw state for those registers (blend 0 = the normal alpha blend, zwrite off, the
/// alpha-test mode the TEST_1 word decodes to).
const LEAF_BLEND: u8 = 0;
const LEAF_ZWRITE: bool = false;
const LEAF_ATEST: u8 = 2;
