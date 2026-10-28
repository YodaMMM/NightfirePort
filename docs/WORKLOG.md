# Worklog

## level-06-searchlight-state (branch s/level-06-searchlight-state, from origin/master 3c312512)

Sandbox opened 18:13 UTC, expires ~20:12 UTC. Scope: on 07000006 (The Exchange) after the
fusebox laser cut (green wire), most searchlights go dark but a couple keep a bright
lens/housing with no beam. Find which placements stay bright and what distinguishes them [D
placements, off channels, the cut's channel, linked parts]; pin the game's off path on the
head object [M Searchlight_Update's off-channel kill]; port the sim/data side so every
light ends in the same off state; expose per-light on/off; list the draw deltas for the
main agent. Never hide the searchlight (the housing stays drawn).

Plan: (1) read the 07000006 object placements for searchlight heads + their linked parts
[disc]; (2) read the fusebox laser script path for the channel the cut writes [M/D]; (3) read
Searchlight_Update's off-kill in the decompile [M]; (4) port the sim side, test through the
real fusebox laser path; (5) full suite.

## level-06-searchlight-state progress (19:00 UTC)

The 07000006 bright-lens diagnosis landed. The fusebox cut reaches ALL four courtyard lights
([D] placements #2/#4/#6/#8: off-channel 57 = the cut's group-0 word; [M] the kill gate reads
switch_channels[data+0x35] every tick, 0x1cb900:92) — wiring is not the difference. The real
gaps were port-side: (1) the off-kill hid the MOUNT and dropped its link (the game's walk covers
only psVar2+0x14/0x16/0x18 = spot/beam/corona [M 0x1cb900:113-135; mount untouched]), and (2)
the game's kill runs Control_NextLOD on the head [M :138 → 0x1349b0: obj+0xD8 ← *(glist+0x40)],
whose chain target for every 07000006 head is map object 131 OBJECT_SearchLightBroken [D block
0x27 via parsemap_block_LOD 0x1d1f10] — the dark-lens variant. The port kept drawing the lit
lens model forever (the reported bright lens/housing) and lost the mount. Fixed: the kill keeps
the mount (drawn + linked), zeroes on/off/alarm bytes [M :134-137], and pins the swap as
Part::lod_next (also on the bullet break [M :454]); renderer work (draw the Broken mesh for a
lod_next head) listed for the main agent. Tests: unit `the_off_channel_kill_matches_the_game_
field_for_field` (nf-game) + real-path `searchlight_off_07000006_live` (laser cut through the
guardroom fusebox → every head in the same game off state) + the [D] chain-pin test. Full suite
to run before the final push.
