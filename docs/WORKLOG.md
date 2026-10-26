# WORKLOG

## fusebox-inverter-2 (2026-10-26, sandbox expiring ~08:16 UTC)
Branch `s/fusebox-inverter-2` = origin/s/fusebox-inverter merged with origin/master (0b25f0d5).
Task: `InverterTrigger_Update` (0x1b30a8) triggers built from type-0xF9 placements (p0 input,
p1 output) go red with master on four live tests — fix_guard_misc_live::probe_sniper_direct_bullet
and ::the_0c_sniper_takes_damage_when_shot (0700000c), interactables_07000041_live::
the_searchlight_family_sweeps_as_one_light and ::the_wrapper_ends_into_the_space_station_after_
the_chain (07000041). Each is green on master. Plan: list the 0xF9 placements on 0c and 41 and
what the new inverters do to the channels those tests depend on; port the game's exact create
gate for 0xF9 [M parsemap_create_dynamic_objects 0x1d04f0's 0xF9 arm + InverterTrigger_Create]
and the update's exact rule [M 0x1b30a8]; where the game really inverts those channels, fix the
tests to the game's behaviour with evidence. Goal: all four green + the 14 inverter test; full
suite on the merged tree.
