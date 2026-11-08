# agent-07-straddle-doors: 07000007 doors #244-#248 do answer E

**No door bug. The census result was a harness artefact.**

## What the census saw

On `s/agent-use-pick`, `use_pick_census_live::census_07000007` listed doors #244, #245, #246,
#247 and #248 under "doors E never moves". The census presses its targets in sequence on one
Sim. It restores the target and the channels before each press, but not the mission state.

## Why

During door #243's second approach, Bond is placed at (100.5, 44.1, -56.0). The mission then
fails with FailLabel 0x4000033 ("Cover Lost", `Mission.internal` 2, then 7). The census never
undoes that. From then on, `step_bond`'s mission-hold gate (`mission.internal == 7`,
`GS_PausePlayer(1, 0)` [M 0x182c60:123-130]) zeroes the use key every tick. Every later door gets
no press. The doors come after #243 in placement order, so they are exactly #244-#248.

## Proof

- Census on `s/agent-use-pick` with a mission-state print: every press from #244 on runs with
  internal 2 or 7 and fail 0x4000033.
- The same census with the mission state (internal, state, fail_label) restored before each
  press: `07000007: 7 targets, 14 presses used, 0 refusals, 0 failures, doors E never moves: []`.
- On master with the real E path (`step_bond` with the use key), each door #242-#245 moves from
  both broad faces (state 0x2 to 0x8, pos01 stepping 0.0172/tick) while the mission is live. A
  direct `Doors::player_use` and a direct `Doors::activate` from Bond's head also move every door
  #242-#248.

The doors are kind 0 swing doors: group 0, no open or lock channel, par-4 set, flags 0x3,
3 keys, approach 0, locked flag 0 [D]. Nothing in `Door_Activate` 0x135be0's port refuses them.

## Suggested census fix (for `s/agent-use-pick`, not applied here)

In `census()`, save `(mission.internal, mission.state, mission.fail_label)` next to `ch0`.
Restore them wherever the channels are restored, or assert that `mission.internal` has not
changed so that a blown cover fails loudly instead of showing up as dead doors.

## Unproven

- Whether standing at (100.5, 44.1, -56.0), west of door #243, really costs the cover in the
  game. The port's Cover Lost there was not traced.
