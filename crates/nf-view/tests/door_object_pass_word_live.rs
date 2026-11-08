//! `Bullet_CollisionHandler` 0x12ce80's object-hit pass reads obj+0xF0 & 0x60 [M andi at 0x12cf18;
//! `movn s4,s7,v0` at 0x12cf20 sits in the branch delay slot, so it always runs], and for a door
//! obj+0xF0 is the glist record's +0x44 word [M 0x134960:11]. Every door on every level must take
//! its pass from that word, not from the placement's flags.

#[test]
fn every_door_passes_rounds_on_the_glist_word() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let (mut doors, mut passing) = (0, 0);
    for lv in nf_view::dev::LEVELS {
        let name = format!("{lv:08x}.bin");
        let Ok(s) = nf_view::load(&iso, &name, false) else { continue };
        let Some(sim) = s.sim.as_ref() else { continue };
        for d in &sim.drones.doors.list {
            doors += 1;
            let want = d.f0 & 0x60 != 0;
            passing += usize::from(want);
            assert_eq!(d.bullet_pass, want, "{name} door placement {} f0 {:#x}", d.placement, d.f0);
        }
    }
    eprintln!("[doors] {doors} doors, {passing} carry the pass word");
    assert!(doors > 0);
}
