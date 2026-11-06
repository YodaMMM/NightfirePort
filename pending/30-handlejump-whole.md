# Player_HandleJump: every arm against the port

Read all of `Player_HandleJump` 0x1916b0 against player.rs. The launch, head hit and landing arms were just corrected on master. Check the rest:

- state 0: the 07000007 denied-jump arm (sound 0x1db while switch channel 0x48 is clear) [M :106-113]
- the crouch press and substate change, the 0x940 gate, flag 0x20 blocking a launch
- the 0x95f == 3 wire search and `Player_CollWire`
- the `Collide_LineOfSight` headroom check before launch
- state 2 landing needs no delay in the game, the port waits on land_delay for every state
- `Player_InWater` 0x195e50:23 clearing the state

One line per arm: MATCH or WRONG with the citation. Fix each WRONG arm with a test that drives it.
