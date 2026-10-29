//! The seal's exact faces: every collision triangle with a vertex or centroid in the box
//! x −35.5..−24, y 33..40, z −19..−22 — dumped with normals, so step-vs-slope-vs-wall is
//! decided from the disc's own data [D].

use nf_formats::level::Level;

fn main() {
    let iso = nf_formats::iso9660::IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse 07000005");
    let (instances, _missing) = level.instances(true);
    let tx = |m: &[[f32; 4]; 4], p: [f32; 3]| -> [f32; 3] { nf_formats::placement::transform(m, p) };

    let in_box = |p: [f32; 3]| p[0] >= -36.0 && p[0] <= -24.0 && p[1] >= 33.0 && p[1] <= 40.5 && p[2] >= -22.5 && p[2] <= -19.0;

    for inst in instances.iter() {
        let om = &level.maps[inst.map].objects[inst.object];
        let Some(c) = &om.collision else { continue };
        if om.flags & 0x20 != 0 { continue; }
        for (t, _mat) in c.triangles_with_material() {
            let v = [tx(&inst.matrix, t[0]), tx(&inst.matrix, t[1]), tx(&inst.matrix, t[2])];
            let cen = [(v[0][0]+v[1][0]+v[2][0])/3.0, (v[0][1]+v[1][1]+v[2][1])/3.0, (v[0][2]+v[1][2]+v[2][2])/3.0];
            if !(in_box(v[0]) || in_box(v[1]) || in_box(v[2]) || in_box(cen)) { continue; }
            // normal
            let (e1, e2) = ([v[1][0]-v[0][0], v[1][1]-v[0][1], v[1][2]-v[0][2]], [v[2][0]-v[0][0], v[2][1]-v[0][1], v[2][2]-v[0][2]]);
            let n = [e1[1]*e2[2]-e1[2]*e2[1], e1[2]*e2[0]-e1[0]*e2[2], e1[0]*e2[1]-e1[1]*e2[0]];
            let len = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
            if len < 1e-9 { continue; }
            let n = [n[0]/len, n[1]/len, n[2]/len];
            println!("tri obj \"{}\" v1({:.2},{:.2},{:.2}) v2({:.2},{:.2},{:.2}) v3({:.2},{:.2},{:.2}) n({:.3},{:.3},{:.3})",
                om.name.trim(), v[0][0], v[0][1], v[0][2], v[1][0], v[1][1], v[1][2], v[2][0], v[2][1], v[2][2], n[0], n[1], n[2]);
        }
    }
}
