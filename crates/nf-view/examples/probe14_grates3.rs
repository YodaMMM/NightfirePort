// level-14-vent-grates scratch 3: the grate script 0x06000a3b — streams, commands, and the
// key tables the entity matrix samples (Script_GetInterp). Assertion-free.
use nf_formats::nis::Cmd;

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let lib = &sim.drones.scripts.lib;
    println!("scripts in lib: {}", lib.len());
    let h = 0x0600_0a3b;
    let Some(nis) = lib.get(&h) else {
        println!("script {h:08x}: NOT in lib");
        return;
    };
    println!("script {h:08x}: camera {} loop_start {} length {} streams {} keys {}",
        nis.camera, nis.loop_start, nis.length, nis.streams.len(), nis.keys.len());
    for (si, st) in nis.streams.iter().enumerate() {
        println!(" stream {si}: keys {}..{} commands {}", st.first_key, st.last_key, st.commands.len());
        for c in st.decode() {
            println!("   {c:?}");
        }
    }
    for (ki, k) in nis.keys.iter().enumerate() {
        println!("  key[{ki}] t{:.2}: pos ({:.3}, {:.3}, {:.3}, {:.3}) scale {:.3} w28 {:.3}",
            k.time, k.pos[0], k.pos[1], k.pos[2], k.pos[3], k.scale, k.w28);
    }
}
