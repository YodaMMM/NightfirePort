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
