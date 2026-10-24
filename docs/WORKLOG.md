# Worklog

## dev-merge-weapon-sounds

- (13:36 UTC, 10-24) Branch created from origin/master (26543436). Task: port the missing pieces of the dev asset merge so the two ignored tests in crates/nf-view/tests/fix_weapon_sounds_live.rs pass — a_foreign_weapon_fires_with_sounds_on_the_indoor_level_07000001 (foreign AR draws, held fire spawns no round) and the_laser_loop_starts_on_press_and_stops_on_release (Laser 0x4e has no first-person model in the dev merge). Real input only (step_bond; no teleports past the blocker, no pokes). Un-ignore each when it passes, with a sister that can fail. Sandbox reused (repo, work/, ISO 1061754880 bytes intact; Bash had failed at the end of the previous session and recovered).
