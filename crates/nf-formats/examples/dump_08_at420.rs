//! level-08-pane-shootout probe: the AT-420 (0x2c) weapon record's flight-relevant words —
//! the spread cone (+0x24 base, +0x98 bloom, +0x68 flags), the speed (+0x20), the guided
//! flags (+0x6c). Prints only. [D]
use nf_formats::iso9660::IsoImage;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let elf = img.read_file(img.find("/ACTION.ELF").expect("ACTION.ELF")).expect("read");
    let wd = nf_formats::weapon::WeaponData::from_action_elf(&elf).expect("weapon data");
    for k in [0x02usize, 0x03, 0x2c, 0x2d] {
        let w = &wd.records[k];
        let f32_at = |o: usize| f32::from_le_bytes(w[o..o + 4].try_into().unwrap());
        let u32_at = |o: usize| u32::from_le_bytes(w[o..o + 4].try_into().unwrap());
        println!("rec {k:#04x}: +0x18 pellets {} +0x20 speed {:.4} +0x24 spread {:.1} +0x68 {:#010x} +0x90 ammo {:#04x} +0x98 bloom {:.4} +0x6c {:#010x}",
            u32_at(0x18), f32_at(0x20), f32_at(0x24), u32_at(0x68), w[0x90], f32_at(0x98), u32_at(0x6c));
        let spread_rad = f32_at(0x24) * 0.0014;
        println!("          spread cone ≈ ±{spread_rad:.3} rad (units × 0.0014 [M Bullet_init 0x12c5a0:139-170])");
    }
}
