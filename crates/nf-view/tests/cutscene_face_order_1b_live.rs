//! cutscene_face_order_1b: the 0700001b arrival scene's seated actor (skin 0x0500008F, glist
//! 0x0200087D 'polySurface1') shows its face through the helmet visor, like the PS2.
//!
//! The glist's batches [D 0700001b.bin]: the face on local texture 9 (batches 2-16), the visor
//! tint on texture 7 (batch 56: 16×16, every texel alpha 1-0x3f, ALPHA 0x44 / TEST GEQUAL 1 /
//! ZBUF write on), the env-mapped glass on texture 8 (batch 57, no depth write). The game sends
//! the batches in packet order, so the depth-writing tint lands after the face. The port grouped
//! a character's batches by texture index, which drew the tint first and its depth write hid the
//! face (an empty visor). Pinned: within each character's run, the tint group follows the face.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_visor_tint_draws_after_the_face() {
    let iso = iso();
    let scene = nf_view::load_with_carry_difficulty(&iso, "0700001b.bin", false, None, 3).expect("load 1b");
    let lv = {
        use nf_formats::{disc, iso9660::IsoImage, level::Level};
        let img = IsoImage::open(&iso).unwrap();
        let (entries, bin) = disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700001b.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        Level::parse(&data).unwrap()
    };
    let map = lv.by_hash[&0x0200_087d].0;
    let mats: Vec<(usize, u32, [f32; 3])> = scene.materials.iter().enumerate()
        .filter(|(_, m)| m.layer == nf_view::scene::Layer::Character)
        .filter_map(|(i, m)| m.key.filter(|k| k.0 == map).map(|k| (i, k.1, m.centre))).collect();
    let tints: Vec<_> = mats.iter().filter(|m| m.1 == 7).collect();
    assert!(!tints.is_empty(), "the actor's visor tint group is in the draw list");
    for t in &tints {
        let face = mats.iter().find(|m| m.1 == 9 && m.2 == t.2).expect("the same actor's face group (texture 9)");
        assert!(face.0 < t.0, "the face (material {}) draws before the depth-writing visor tint (material {})", face.0, t.0);
    }
}
