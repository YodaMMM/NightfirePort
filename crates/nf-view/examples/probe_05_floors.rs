//! Scratch (level-0a-office-hatch-6): the disc's floor tris under the three ee_start spawns,
//! with exact bits — the settle-vs-no-settle discriminator. The game's placement root = the
//! placement ray's hit + 1.1 [M 0x190d88:107-113]; the snapshot roots [L]:
//!   05 @0x16cf010+0x34 = 24.963300704956055, 01 @0x18f5790+0x34 = 6.4286932945251465,
//!   0a @0x1b78640+0x34 = 1.089111089706421.
//! A tri whose plane y at the spawn xz equals root−1.1 → the boot RESTED at the placement
//! (no settle); root−1.0891111 → the settle ran.
use nf_game::world::CollisionWorld;
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let spots: [(&str, [f32; 2], f32); 3] = [
        ("07000005", [104.95400, 108.82527], 24.963300704956055),
        ("07000001", [-2.78015, -12.40357], 6.4286932945251465),
        ("0700000a", [-3.93185, 10.39776], 1.089111089706421),
    ];
    for (name, xz, snap_root) in spots {
        let scene = nf_view::load(&path, &format!("{name}.bin"), false).expect(name);
        let sim = scene.sim.as_ref().unwrap();
        println!("== {name}: snapshot root {:.7} ({:#x})", snap_root, snap_root.to_bits());
        println!("   root-1.1      = {:.7} ({:#x})", snap_root - 1.1, (snap_root - 1.1).to_bits());
        println!("   root-1.0891111= {:.7} ({:#x})", snap_root - 1.0891111, (snap_root - 1.0891111).to_bits());
        let lo = [xz[0] - 1.5, snap_root - 2.5, xz[1] - 1.5];
        let hi = [xz[0] + 1.5, snap_root + 1.5, xz[1] + 1.5];
        let mut seen: std::collections::BTreeSet<u32> = Default::default();
        for t in sim.world.tris_near(lo, hi) {
            // Plane y at the spawn xz (bilinear on the tri): only for near-horizontal tris.
            let ys = [t[0][1], t[1][1], t[2][1]];
            if ys.iter().any(|y| (y - snap_root).abs() > 3.0) { continue; }
            // Barycentric solve on xz.
            let (a, b, c) = (t[0], t[1], t[2]);
            let d = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
            if d.abs() < 1e-9 { continue; }
            let l1 = ((b[2] - c[2]) * (xz[0] - c[0]) + (c[0] - b[0]) * (xz[1] - c[2])) / d;
            let l2 = ((c[2] - a[2]) * (xz[0] - c[0]) + (a[0] - c[0]) * (xz[1] - c[2])) / d;
            let l3 = 1.0 - l1 - l2;
            if l1 < -0.05 || l2 < -0.05 || l3 < -0.05 { continue; }
            let y = l1 * a[1] + l2 * b[1] + l3 * c[1];
            if seen.insert(y.to_bits()) {
                println!("   tri plane y at spawn xz = {:.7} ({:#x})  vert ys {:.7} {:.7} {:.7}", y, y.to_bits(), a[1], b[1], c[1]);
            }
        }
    }
}
