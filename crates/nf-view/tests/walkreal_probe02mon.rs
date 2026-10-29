//! walkreal probe: 07000002's monitors (0x30) + sensors (0x28) — the param-8 `monitor_ch`
//! (`SENSOR data+0x5C` = placement +0x4C [M 0x1af010:78-80]) that `Monitor_Update`
//! 0x1afcc8:170-174 raises while the monitor's camera view is live. Diagnostic only
//! — run with --nocapture.

#[path = "common/walker.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_07000002_monitors_sensors() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== 07000002 monitors ===");
    for m in &sim.drones.monitors.list {
        eprintln!(
            "mon placement={} at=({:.2},{:.2},{:.2}) group={} health={} state={}",
            m.placement, m.pos[0], m.pos[1], m.pos[2], m.group, m.health, m.state
        );
    }
    eprintln!("=== 07000002 sensors ===");
    for s in &sim.drones.sensors.list {
        eprintln!(
            "sensor placement={} at=({:.2},{:.2},{:.2}) out={:#04x} disable={:#04x} mon_id={} mon_ch={:#04x} range={}",
            s.placement, s.pos[0], s.pos[1], s.pos[2], s.out, s.disable, s.monitor_id, s.monitor_ch, s.range
        );
    }
    eprintln!("=== 07000002 simple scripts ===");
    for s in &sim.drones.sss.list {
        eprintln!(
            "ss placement={} at=({:.2},{:.2},{:.2}) ch={:#04x} mask={:#06x} state={} sound={:#06x} keys={}",
            s.placement, s.pos()[0], s.pos()[1], s.pos()[2], s.channel, s.mask, s.state, s.sound, s.keys.len()
        );
    }
}
