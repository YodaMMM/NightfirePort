// Scratch (untracked): where are 07000014's DESTROY_BoxSatellite leaves, and does the port draw them.
#[test]
#[ignore]
fn probe14_sat() {
    let iso = std::env::var("NF_ISO").unwrap();
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    for d in sim.drones.props.destroy.iter() {
        eprintln!("destroy #{} pos ({:.1},{:.1},{:.1}) lo {:?} hi {:?} kind {}", d.placement, d.pos[0], d.pos[1], d.pos[2], d.lo, d.hi, d.kind);
    }
    let centres: Vec<usize> = scene.destroy_centres.iter().map(|(_, p)| *p).collect();
    eprintln!("destroy_centres placements: {centres:?}");
}
