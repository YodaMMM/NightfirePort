//! `unported-misc-3`: `Doors_Calc` 0x135fe0's per-frame `DoorGroupsStates` memset (0x135fe0:13)
//! rides `Doors::begin_pass`, called from `Drones::tick` before the object walk (the game runs
//! `Doors_Calc` at the top of `control_movement_object_handler` 0x1336f8, before the walk's door
//! updates OR in — `Door_IsClosed` 0x1367b0 must never see last frame's ORs).

use nf_game::door::Doors;

#[test]
fn begin_pass_wipes_group_states_the_game_memsets_each_frame() {
    let mut doors = Doors::default();
    // The OR `Door_Update` 0x1363c8:170-171 writes (`group_states[g] |= d.state`).
    doors.group_states[3] = 0x12;
    doors.group_states[100] = 0xffff;
    doors.begin_pass();
    assert_eq!(doors.group_states, [0u16; 101]);
    // Sister: without the wipe the stale OR survives (the bug this fixes — a grouped door that
    // has ever been open would never read CLOSED again through `Door::is_closed`'s `== 2`).
    doors.group_states[3] = 0x12;
    assert_eq!(doors.group_states[3], 0x12);
    doors.begin_pass();
    assert_eq!(doors.group_states[3], 0);
}
