//! level-4a-ride-bond-float probe 2: the pack 0x060006cf stream table — per stream: op/kind byte,
//! parent byte, key range, and for the anim streams (kind 2) the object handle; for the entity
//! streams the first/last key positions. Opt-in via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_the_pack_stream_table() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700004a.bin", false).unwrap());
    w.tick(nf_game::player::Input::default(), false);
    let sim = w.scene.sim.as_ref().unwrap();
    // Load the pack script instance directly (the SP #289 script): the port's Scripts holds the
    // parsed NIS per hash.
    let sc = sim.scripts.get(&0x0600_06cfu32).expect("the pack 0x060006cf parses");
    eprintln!("pack length {} flags {:x} script_cam {}", sc.length, sc.flags, sc.script_cam);
    for (i, s) in sc.streams().iter().enumerate() {
        eprintln!(
            "s{:<2} kind {:<2} parent {:<3} start_clock {:<7} start_frame {:<5} obj {:?}",
            i, s.kind, s.parent, s.start_clock, s.start_frame, s.obj);
    }
}
