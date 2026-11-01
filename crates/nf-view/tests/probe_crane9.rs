//! probe9: does 0700000d carry the wire body-anim tracks? Print the WireAnims table
//! and the script-library presence for the seven wire hashes. Print-only.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "probe9: the wire anim tracks on 0d; run with --ignored --nocapture"]
fn probe9_wire_anim_tracks() {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let sim = cold.sim.as_ref().unwrap();
    for h in [0x0600_05e5u32, 0x0600_05ec, 0x0600_05ed, 0x0600_05ee, 0x0600_05ea, 0x0600_00b4, 0x0600_00b5, 0x0600_0095, 0x0600_0096, 0x0600_0094] {
        match sim.wire_anims.tracks.get(&h) {
            Some(t) => {
                let span = if t.x.len() >= 2 { (t.x[t.x.len() - 1] - t.x[0], t.z[t.z.len() - 1] - t.z[0]) } else { (0.0, 0.0) };
                eprintln!("[probe9] track {:08x}: {} frames, root span ({:+.4},{:+.4})", h, t.frames, span.0, span.1);
            }
            None => eprintln!("[probe9] track {:08x}: MISSING", h),
        }
    }
}
