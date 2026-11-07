# agent-rev-census

Review repairs for the unread-params and homing batch. The branch is origin/master with
s/homing-copter-3 merged, then one repair commit.

- Destroy: `latched` (tag+0x17) split from `dead`. Master set `dead` at the smash, so a timed leaf
  stopped taking rounds at once and a shot-out one could never start its count. The game keeps
  class 0x21 until the count writes [M 0x12bb60:17-20, :33, :39-41; 0x12c4f8:23-26; 0x12bd48:41-49].
  This is the part of s/unread-params-doors that master's own countdown lacks. That branch no
  longer merges: props.rs and panel_sequence_07000014_live conflict.
- All 17 timed leaves carry health 10000 [D], so the second-round arm is hard to reach by gunfire.
  What a player can see is rounds stopping on a counting leaf for up to 300 ticks.
- dbg_urn example: s/homing-copter-3 added `target_copter` and missed this literal.
- destroy_p6_countdown_live comes from s/unread-params-doors unchanged.

Tests: nf-game lib 688/0/5 ignored, destroy_p6_countdown_live 2/0, and 16 nf-view binaries green,
walkreal_07000014_live and panel_sequence_07000014_live among them. The dbg_urn example builds.
Not run: the full suite. Unproven: the counting leaf stopping rounds was not checked in the game.
