# WORKLOG

## level-14-delta-exit-walk (2026-10-28, branch s/level-14-delta-exit-walk)

Task: 07000014 (Countdown Part 1) — un-ignore walkreal_07000014_live
`walk_the_infiltration_to_the_delta_exit`. Leg 2's `follow_nav_route` crosses
the closed west door plane (Door_Slide_11A/B, ch_lock 0x85) before the
sabotage cascade can unlock it. Give legs 2-3 explicit west-side waypoints,
cut both west fuseboxes on the real laser path, walk to the E-use switch
oDroneTrigger1b (-60.30, 0.61, 38.37; touch out 0xA0, mask 4 [M
Trigger_Activate 0x1b2318]) and press E (the green
`the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door` has the
exact input), then legs 4-6 east through the opened door to the Delta exit
#944 on real input; un-ignore (or re-ignore on the next exact blocker with
its evidence). Note anything the player needs that the port does not draw
(the switch wall art, the M.Hatch indicator) in the report. Full suite,
quote the total line.

Plan:
- Fast runner started in the background in the first minutes.
- Reroute legs 2-3 west of the door (the generator room is west; the door is
  crossed only AFTER 0x85 opens it).
- Iterate the walk on real input; then the full suite on the final tree;
  push; report; finish steps.

Final (2026-10-28): `walk_the_infiltration_to_the_delta_exit` un-ignored and GREEN (70993
ticks, 0 stuck, 0 falls). Three discoveries: (1) the 0x85 doorway seals completely — the
"gap past the leaf" premise was refuted by the crossing probe (the leaves' plane x≈−52.35
blocks both ways; the frame jambs seal z 38.6..38.9); (2) the Kiko scene's scripted body
carries Bond WEST past the closed door (release (−55.49, 1.10, 42.02)) — the test stops
steering at the touch and lets the scene deliver the west side; (3) the Laser left in hand
from the cuts cannot kill Rook (198 bloodless point-blank taps) — the kill switches to the
hand weapon 0x2 first. cut_fusebox now aims at the VISIBLE cable slab with the miss-feedback
(the translation sits 1.5 m low) and picks the group-0 wire. Report:
docs/reports/level-14-delta-exit-walk.md (the switch wall art + the M.Hatch indicator noted
as undrawn, live checks listed).

## level-14-switch-wall (2026-10-28, branch s/level-14-switch-wall)

Task: 07000014 — the E-use switch oDroneTrigger1b #725 (-60.30, 0.61, 38.37; touch
out 0xA0 mask 4) draws nothing in the port, so a player cannot tell where to
press E. Find what the game shows there ([D] placements/objects/glists within a
few units, the 0x3b channel-show pairs, ScriptPlayers whose scripts draw there;
[M] the touch's own draw if any) and the M.Hatch indicator ScriptPlayers
(script 0x06000708, triggers 0x85/0x54) — objects, frames, channels. Port any
missing data/sim link; list exactly what must DRAW in the report; test the
data/sim side; full suite + total line.

Plan:
- Fast runner started in the background in the first minutes (setsid; the
  first nohup died between tool calls).
- Census + a near-switch placement probe against the disc; SP/console draw-path
  audit in nf-view; fix; full suite; report; finish steps.

Final (2026-10-28): the switch-spot premise REFUTED — the game has no drawn prop at the touch
itself ([M Trigger_Create 0x1b1720:95 obj+0xF0 = glist 0x50 bit 0x10; D leaves 0]); the visible
switch is the 0x3b CONSOLE_Electricity trio 2.1 west (Active from load → deleted at 0x8c →
Inactive → (0x5b) → Static) and the INFO-bar hints ("Power system safeguards operational" /
"Danger: safeguards offline. System critical" [D USATxt.dat 0x100020c/0d]) — all already
working in the port, now pinned. The M.Hatch SPs #947/#950/#952 (m54 indices; script 0x06000708,
triggers 0x85/0x54 = the out-channel bit [M SP_Hit 0x1dc470:17-19]) verified end to end: the
preloads spawn 02000724 at the SP poses, the replays swap to 02000725 in place at t31, the
keep flag (event 0xc at t54) preserves them, the SPs self-delete; 02000a6b "GLOW_Light_1"
resolves only via 0700000a.bin and its gfx block is EMPTY on the disc — no pixels either side.
No code change needed; the new tests carry the proof. Full suite: the sandbox kills detached
runs — 118/878 binaries done in the window (464 passed / 0 failed / 30 ignored / 0 bad),
resumable via scripts/suite_chunk.py; the 07000014 binaries are in the unrun set. Report:
docs/reports/level-14-switch-wall.md.
