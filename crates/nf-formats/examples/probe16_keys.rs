//! Probe [D]: print a script's keys. Usage: probe16_keys <archive.bin> <script hex> <first> <last>.
//! Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, nis::Nis};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let iso = IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&a[1])).expect("archive");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lb = LevelBin::parse(&data).unwrap();
    let h = u32::from_str_radix(&a[2], 16).unwrap();
    let e = lb.entries.iter().find(|e| e.kind == 7 && u32::from_str_radix(&e.name, 16) == Ok(h)).expect("script");
    let n = Nis::parse(h, lb.data(&data, e)).unwrap();
    let (k0, k1): (usize, usize) = (a[3].parse().unwrap(), a[4].parse().unwrap());
    for ki in k0..=k1.min(n.keys.len() - 1) {
        let k = &n.keys[ki];
        println!("key[{ki}] pos ({:.2},{:.2},{:.2}) time {:.1}", k.pos[0], k.pos[1], k.pos[2], k.time);
    }
}
