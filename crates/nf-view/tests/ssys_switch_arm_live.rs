//! 07000046: the switch whose param 6 arms SSys slot 1 (`Switch_Create` 0x1adcc0:39-43 stores
//! the switch's channel into `SSysItems[1·8+4]`), and `SSys_Monitor` 0x1ad998 (ticked from
//! `Game_Run` 0x1c9790 while `ScriptCam == 0`) runs its 240 s countdown while that channel is
//! on — the arm msg 6 at 240.0 [L ee_snapshot thresholds MIN3/2/1/01 = 180/120/60/6 @0x30cd84],
//! the threshold msgs 5/4/3/2 at 180/120/60/6, and the state-2 expiry at ≤ 0 which writes the
//! switch's channel back to **0** and disarms [M 0x1ad998 case 2]. Before this port the SSys
//! machine never ran (crate::ssys was uncalled — unread-params-switches), so the channel stayed
//! on forever; the pin below is red on that tree. 240 s at the fixed 60 Hz = 14400 ticks
//! [M 0x1ad998:43 − REC_FRAME_RATE per tick].
//!
//! The channel is set through the level's own switch object (`channels.set`, the same write
//! `Switch_Update` performs on activation [M 0x1adea8]) — the machine under test is the
//! monitor, not the switch mechanics (those are pinned in crate::useobj's tests).

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_armed_ssys_slot_counts_down_and_switches_the_channel_off_like_the_game() {
    let mut scene = nf_view::load(&iso(), "07000046.bin", false).expect("load 07000046");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    let (slot, ch) = {
        let sim = scene.sim.as_ref().unwrap();
        let sw = sim.drones.useobjs.switches.iter().find(|s| s.ssys_slot != 0)
            .expect("07000046 must carry a switch with a non-zero param 6 [D census]");
        (sw.ssys_slot, sw.ch)
    };
    assert_eq!(slot, 1, "07000046's SSys arm is slot 1 (laser 1) [D census]");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(ch as u32, 1, sim.drones.frame);
    }
    // The arm tick's own sounds (`Sounds::started` is a per-tick buffer — read it now): the
    // monitor arms on this first tick with the channel up, msg 6 = Laser1 id 6 = 0x2a2
    // [L ee_start_05 table 0x2bef60; M SSys_Msg 0x1ad878].
    scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    let arm_sounds: Vec<u32> = scene.sim.as_ref().unwrap().drones.audio.started.iter().map(|(id, _)| *id).collect();
    assert!(arm_sounds.contains(&0x2a2),
        "the arm msg's sound (Laser1 id 6 = 0x2a2) must play on the arm tick [L 0x2bef60]; got {arm_sounds:?}");
    let mut armed_seen = true;
    let mut still_on_at_2000 = None;
    let mut off_at = None;
    for t in 0..15_200u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let on = sim.drones.channels.on(ch as u32);
        if sim.drones.ssys.items[slot as usize].state == 6 { armed_seen = true; }
        if t == 2000 { still_on_at_2000 = Some(on); }
        if !on && armed_seen { off_at = Some(t); break; }
    }
    assert_eq!(still_on_at_2000, Some(true),
        "the channel must still be on at t=2000 — the countdown is the game's 240 s arm [M 0x1ad998]");
    assert!(armed_seen, "the slot must have armed (state 6 at 240.0) [M 0x1ad998]");
    let off_at = off_at.expect("the SSys expiry must switch the channel OFF [M 0x1ad998 case 2]");
    // 14400 ticks of countdown after the arm tick (the arm tick itself spends the first timer
    // write), plus the load's enable window — the assert pins the game's scale, not an instant.
    assert!((14_300..14_900).contains(&off_at),
        "the expiry must land at the 240 s scale (14400 ticks [M 0x1ad998:43]), landed at t={off_at}");
}
