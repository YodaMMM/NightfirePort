//! The laser-impact gas surface gate (`Effect_Laser` 0x1cef38:59,80-83) — the `EffectInfo`
//! +0x10 entity-gfx mesh column the port now parses (`SurfaceSounds::gas_mesh`). ISO-gated:
//! the table lives in the disc's ACTION.ELF .rodata (symbol `EffectInfo`, 0x3037c0, 23 × 0x88).
//! Every assertion is a [D] byte read of that table (dumped word-by-word at the offset);
//! the sister assertions fail on a table that lost its zero-mesh gate or its group meshes.

use nf_formats::iso9660::IsoImage;
use nf_formats::sfx::SurfaceSounds;

fn surfaces() -> SurfaceSounds {
    let path = std::env::var_os("NF_ISO").expect("NF_ISO must point at the disc image");
    let iso = IsoImage::open(&path).unwrap();
    let elf = iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap();
    SurfaceSounds::from_action_elf(&elf).unwrap()
}

/// [D EffectInfo@0x3037c0, +0x10 per 0x88 record]: the per-material laser-impact gas mesh.
/// Zero = `Effect_Laser`'s `if (mesh != 0)` gate spawns NO puff on that surface:
/// NONE(0), CARPET(1), GRASS(7), EARTH(8), SAND(9), GRAVEL(11), SNOW(12), DEEPWATER(13),
/// SHALWATER(14), FOLLAGE(19), FABRIC(20), LAST(22). Every other row carries the one
/// hard-material puff mesh 0x02000215 (PAPER/WOOD×2/METAL×2/STONE/ICE/GLASS/CERAMIC×2/GENERIC).
#[test]
fn effectinfo_gas_mesh_column_matches_the_disc_table() {
    let s = surfaces();
    // The zero-mesh gate rows (no laser gas).
    for idx in [0u8, 1, 7, 8, 9, 11, 12, 13, 14, 19, 20, 22] {
        assert_eq!(s.gas_mesh(idx), 0, "surface {idx}: the disc's mesh word is 0 — no puff");
    }
    // The hard-material puff mesh.
    for idx in [2u8, 3, 4, 5, 6, 10, 15, 16, 17, 18, 21] {
        assert_eq!(s.gas_mesh(idx), 0x0200_0215, "surface {idx}: the hard-material puff mesh");
    }
    // Sister: the high bits of the HITDATA+0x50 byte (pierce/flags) must not change the row,
    // and an out-of-table index reads 0 (the gate's silent skip).
    assert_eq!(s.gas_mesh(10 | 0x40), 0x0200_0215, "0x40-flagged stone reads the same row");
    assert_eq!(s.gas_mesh(23), 0, "out of table: no puff");
    assert_eq!(s.gas_mesh(255), 0, "out of table (255 & 0x3f = 63): no puff");
}
