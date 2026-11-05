# WORKLOG

## level-1b-platform-touch
Started 2026-11-05 08:12 UTC. Branch s/level-1b-platform-touch (cut from origin/master 27c191518).
Task: on 0700001b the Drake-fight platforms break at the slightest touch. Find the game's break
rule for them (damage source, threshold, what counts as a hit) in the decompile, compare it to
the port, and fix the difference. Prove with a test that touches a platform by walking on it
(no break) and one that does what breaks it in the game. Cite [M] addresses.

Findings (probes, crates/nf-view/tests/probe1b_platforms*.rs, opt-in NF_ISO + --ignored):
- The "platforms" are the 16 solar-panel placements #41-56 (flags 0xd9, param0 script
  0x06000832, param7 0x14) around the Drake arena (z=±4.62, x 10.14..56.12, y -1.72..-9.83).
- Script 0x06000832 spawns OBJECT_SolarPanelWhole 02000a42 (intact panel, type 0x31), 10
  fragment streams (OBJECT_SolarPanel1..10, each stream one fragment at Wait 5,15,...,595) and
  Event kind 3 param 0 — i.e. the break sequence is: whole panel out, fragments in over time.
