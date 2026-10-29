//! level-14-panel-sequence probe: dump the loaded 07000014's destroyable list — which 0xe1
//! placements became Props::destroy tags, their kill/death channels and bounds — plus where the
//! fuseboxes' panel boxes sit. Prints only.
use nf_game::player::Input;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_mut().unwrap();
    println!("level {:08x} destroyables {}", sim.level, sim.drones.props.destroy.len());
    for d in &sim.drones.props.destroy {
        println!(
            "  placement {} kill {:#06x} death {:#06x} kind {} health {} blast {:.2} lo {:?} hi {:?}",
            d.placement, d.kill_channel, d.channel, d.kind, d.health, d.blast_damage, d.lo, d.hi
        );
    }
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    println!("doors with ch_lock != 0:");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if d.ch_lock != 0 {
            println!("  door {i} placement {} group {:#06x} ch_lock {:#06x}", d.placement, d.group, d.ch_lock);
        }
    }
}
