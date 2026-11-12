# WORKLOG

## guard-castle-chat-states (branch s/guard-castle-chat-states)

Task: the castle's chatting and interrogating guards act as in the game. Port
CastleChatGuard1 [M 0x1612e0] and InterogateWalk [M 0x162338] (named unported in
docs/reports/guard-noise-attack-states.md on origin/s/guard-noise-attack-states), with their
message handling, in the game's order, keeping the Rand count. Red test first through real
input, pin via real play on a level that uses them, then every walkreal_*_live for the levels
that use them, one level per tool call, repairs gated to level. Run ai binaries + nf-game.

- Checkpoint 0 (11:59 UTC): sandbox was reset — full redo of setup: clone from origin/master
  (784ba2d99), work-slim + ISO re-downloaded (ISO size 369823203 exact). Branch cut, WORKLOG
  started. Next: read both handlers in full with addresses, find the levels' placements [D],
  red test, port.
- Checkpoint 1 (13:06 UTC): the port landed — the DoModeSettingsNEW start-script specials [M
  0x14c600:130-158] (the entry path the unreached.md audit missed: its census covered placement
  params 15/16, not the param-1 specials), the PlayScript mode-0x18 redirect [M 0x15ba68:26-30],
  CastleChatGuard1 0x36 [M 0x1612e0] and InterogateWalk 0x3d [M 0x162338] with their msg handling;
  zero Rand draws in both handlers [M]. Red first: 0 passed / 2 failed on the old code; green 4/4.
  [D] census: 07000006 is the only level carrying the specials (0x06000124 + 0x06000126, the dock
  chat pair). nf-game lib whole 786/0/4i. Open: the pair does not spawn at level start (event-
  gated — the live probe is #[ignore]d with the dump evidence); the full suite died on ENOSPC
  (6.2 GB target on a 9.9 GB disk).
- Checkpoint 2 (13:48 UTC): report written, branch pushed.

## guard-castle-chat-gate
Started 2026-11-12 02:15 UTC. Branch s/guard-castle-chat-gate cut from origin/s/guard-castle-chat-states c1055e9cf, merged origin/master 08a276ce0 (fb2fcabb6).
Task: finish guard-castle-chat-states "What is left" 1 and 2 — find the gate that spawns the
0x06000124/0x06000126 castle chat pair on 07000006 (census the level's creators/events/scripts
keyed on those ids or their map, addresses [M]/[D]), census who raises channel 0x11 (the chat's
listen channel [M 0x14c600:147-158]), fix the port's spawn/channel in the game's way if they
differ (red test first), then un-ignore walkreal_07000006_live::the_castle_chat_pair_stands_
chatting_and_breaks_off_when_alerted (legs, state-0x36 asserts, burst break-off written).
Real input only, no top-ups, never lower damage or health. Run walkreal_07000006_live whole and
interactables_07000006_live.

## guard-castle-chat-gate-2 (2026-11-12T03:52Z, branch s/guard-castle-chat-gate-2)

- Claimed. Base: origin/s/guard-castle-chat-gate (bf162d825). The duplicate fns the driver
  reported were an appension INSIDE that branch's walkreal_07000006_live.rs (its base already
  equals master 08a276ce0) — removed the appended copies, keeping master's versions of
  real_damage_agent / real_damage_00agent / the_wellhead; the branch's ignored chat test
  replaced with the castle-chat-real-path green version (the 15 u watch spot: the pair holds
  0x36 through the NIS and the legs on THIS tree too — the sight-bit fix included — and the
  test passes: spawn 0x36, ch 0x11 up, the scripts playing, the burst breaks the chat off with
  the un-shot guard alive).

- The ch 0x11 raiser census CLOSED with the decompile: the gate branch's "512 fans out, not a
  raiser" cited the wrong case — Trigger_Update 0x1b1a78's case 5 (MultiplexOut) only runs for
  type-5 triggers; #512's type is 0xc, and case 0xc (lines 167-180) ORs the +0x26 input list
  into the out channel [M]. So #512 IS the raiser: plate #528 (out 0x3b) or #513 (out 0x3a)
  fires, the list-OR raises 0x11 [D census + the sim: 0x3a never on, 0x11 on after #528]. The
  NIS/event-layer sweep is CLEAN: 07000006's 16 op-0x12 event sites carry no 0x11 write (the
  0x0d sets are 0x40/0x3f/0x3d/0x3e in the reveal NIS, 0x83 + 0x02 in the dressing cut-scene)
  [D nis-census].
- vision_bit_census green (CASTLE_CHAT_GUARD1 + INTEROGATE_WALK moved to REQUIRED — both port
  ENTERs set vision_on [M 0x1612e0:21 + 0x162338:20]); fix_npcs_live green after correcting
  its gate-guard test to the decompile (487 = the chat pair's north guard in 0x36, the old
  IDLE read predates the port); walkreal_07000006_live whole 7/0/2i (the balcony pre-clear
  lever ported; the gate ledger's t3964 death re-measured identical before it). Then EVERY
  walkreal_*_live binary on the final tree: all green (see the report's list).

## guard-castle-chat-gate-3 (2026-11-12T05:55Z, branch s/guard-castle-chat-gate-3)

- Claimed. Base: origin/s/guard-castle-chat-gate-2 (80f622052) merged with origin/master
  (clean). The chat pair itself checks out against the game [L PCSX2 fresh 07000006]. This
  task: the 4 suite reds that pass on master but fail on master + this branch —
  guard_parity_live::a_tower_guard_outside_his_civilian_states_can_attack_on_sight,
  guard_ai_other4_live::investigate_stands_down_to_idle_when_nothing_is_found,
  rook06_realpath_live::the_switch16_guard_fights_through_the_door47_doorway,
  walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage.
  Each red gets a cause with the address; a test wrong against the decompile gets corrected
  with the address; nothing weakened. Then the full suite.
