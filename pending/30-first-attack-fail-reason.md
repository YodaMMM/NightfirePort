# First attack: mission fail reason on prop 0x52

`DroneFunc_FirstAttack` 0x148010:95-98 calls `DroneFunc_SetMissionFailReason(drone, 8)` when prop 0x52 is set; the port's `first_attack` does not. Find which placements carry prop 0x52, port the call, fail-first test on the real level, and check the fail text the player sees.
