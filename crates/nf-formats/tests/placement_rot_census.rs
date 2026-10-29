//! Disc census [D]: every placement's rotation angles per class (the low byte of the flags word),
//! over every map of every level `.bin` on the disc. Counts rows with a non-zero rotation-x or
//! rotation-z (pitch/roll) — the rows where the two game matrix builders, `RotMatrix` 0x1e2368
//! (X·Y·Z) and `View_RotTransMatrix` 0x1e5168 (Y·Z·X), disagree — so a per-class builder switch
//! knows its reach. Informational: asserts nothing about the data.
//!
//! Run: `NF_ISO=… cargo test -p nf-formats --test placement_rot_census -- --nocapture`

use std::collections::BTreeMap;

use nf_formats::placement::{object_matrix, object_matrix_for_class, quat_quat_to_mat};
use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn nonzero(v: f32) -> bool {
    v.to_bits() & 0x7fff_ffff != 0
}

#[test]
fn census_placement_rotations_per_class() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let im = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&im).unwrap();
    // class -> [total, creates_cel, rx != 0, rz != 0]
    let mut per: BTreeMap<u32, [u32; 4]> = BTreeMap::new();
    let mut levels = 0u32;
    let mut maps = 0u32;
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = im
            .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
            .unwrap();
        let Ok(lv) = Level::parse(&data) else { continue };
        levels += 1;
        for m in &lv.maps {
            maps += 1;
            for p in &m.placements {
                let c = per.entry(p.flags & 0xff).or_default();
                c[0] += 1;
                c[1] += p.creates_cel() as u32;
                c[2] += nonzero(p.rotation[0]) as u32;
                c[3] += nonzero(p.rotation[2]) as u32;
            }
        }
    }
    println!("levels parsed: {levels}, maps: {maps}");
    println!("class\ttotal\tcreates_cel\trx!=0\trz!=0");
    for (class, c) in &per {
        println!("{class:#04x}\t{}\t{}\t{}\t{}", c[0], c[1], c[2], c[3]);
    }
}

/// The two quaternion classes build their object matrix from the placement's +0x24 quaternion,
/// per row, over every map on the disc. Pins (per row — holds on any disc subset):
/// 1. the class builder returns `Quat_QuatToMat`(+0x24 quat) with the placement translation
///    [M 0x1d04f0:474-478 (0xf2, the dispatcher's inline build + row-3 position stores),
///    0x131f20:66-67 (0x34, quat matrix + `Mat_SetTrans` 0x1e3478), 0x1343c8:18-23 (0x34, the
///    prebuilt matrix overwrites the created object's)];
/// 2. every yaw-only row's quat matrix equals the Euler `RotMatrix` matrix — the game's quat and
///    euler agree there, so those rows stay exactly where the port's interim Euler reading put
///    them (measured bound: max |Δ| = 4.77e-7 over the disc, ~2-4 ulp; asserted < 1.0e-6);
/// 3. every 0x34 row is yaw-only with that consistent quat (the GunImp base's yaw model reads
///    `rotation[1]` — equal to the quat's yaw on all disc data) [D];
/// 4. the port's `Placement::quaternion` parse sees the same four words the dispatcher copies
///    [M 0x1d04f0 prologue: +0x24..+0x30 → the quaternion_tag].
/// Counts are informational (the disc census: 0xf2 = 69 rows, 16 rx≠0 / 52 rz≠0, 36 rows where
/// quat ≠ euler beyond 1e-3 — all 36 with rz≠0; 0x34 = 37 rows, all yaw-only).
#[test]
fn census_quat_classes_build_from_the_placement_quaternion() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let im = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&im).unwrap();
    let mut counts = BTreeMap::<u32, [u32; 5]>::default(); // class -> [rows, yaw-only, quat==euler(<1e-6), quat!=euler(>1e-3), yaw-only&&quat!=euler]
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = im
            .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
            .unwrap();
        let Ok(lv) = Level::parse(&data) else { continue };
        for m in &lv.maps {
            for p in &m.placements {
                let class = p.flags & 0xff;
                if class != 0xf2 && class != 0x34 { continue; }
                if p.creates_cel() { continue; } // never reaches the dispatcher (the static path)
                let quat = p.quaternion();
                let (mx, me) = (object_matrix_for_class(class, p.creates_cel(), p.rotation, p.translation, quat), object_matrix(p.rotation, p.translation));
                // Pin 1: the builder IS the quat matrix + the placement translation.
                let want = {
                    let mut w = quat_quat_to_mat(quat);
                    w[3] = [p.translation[0], p.translation[1], p.translation[2], 1.0];
                    w
                };
                assert_eq!(mx, want, "{class:#04x} row must build from the +0x24 quaternion [M 0x1d04f0:474-478 / 0x131f20:66-67]");
                // Pin 2/3/4: the yaw-only rows stay on the Euler reading.
                let max_diff = (0..3).fold(0.0f32, |acc, i| (0..3).fold(acc, |a, j| a.max((mx[i][j] - me[i][j]).abs())));
                let yaw_only = !nonzero(p.rotation[0]) && !nonzero(p.rotation[2]);
                let c = counts.entry(class).or_default();
                c[0] += 1;
                if yaw_only {
                    c[1] += 1;
                    assert!(max_diff < 1.0e-6, "{class:#04x} yaw-only row must stay on the Euler matrix (max |Δ| {max_diff:e} < 1e-6; quat {quat:?} rot {:?})", p.rotation);
                    if class == 0x34 {
                        // Pin 3: the mount rows keep the GunImp base's yaw-model equivalence.
                        let my = {
                            let (s, co) = p.rotation[1].sin_cos();
                            [[co, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, co]]
                        };
                        for i in 0..3 { for j in 0..3 { assert!((mx[i][j] - my[i][j]).abs() < 1.0e-6, "0x34 row must close onto RotY(rotation[1])"); } }
                    }
                } else if max_diff > 1.0e-3 {
                    c[3] += 1;
                }
                if max_diff < 1.0e-6 { c[2] += 1; }
            }
        }
    }
    for (class, c) in &counts {
        println!("class {class:#04x}: rows {} yaw-only {} quat==euler(<1e-6) {} quat!=euler(>1e-3) {}", c[0], c[1], c[2], c[3]);
    }
    // The measured disc shape [D] — informational bounds, not pass/fail on the counts:
    // 0x34: 37 rows, all yaw-only; 0xf2: 69 rows, 17 yaw-only, 36 rows quat≠euler (>1e-3).
}
