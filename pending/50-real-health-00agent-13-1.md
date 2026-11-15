## Your task: `real-health-00agent-13-1`, 07000013's 00 Agent real-damage walk completes

Branch from current origin/master.

Master: the warm chain's E-open of door plc 545 at (-10.6, 1.1, -19.3) does not open it at difficulty 3 (tower2.rs:484 assert, state 0x2 after the press). Find what holds the door at 00 Agent first.

The test: `walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_00agent` (ignored on master; its ignore text is the measured ledger). Levers that finished the other 00 Agent walks (read their merged tests): live armour pickups near the death, crouch on long-range sniper legs, bounded aimed shots with re-pick, finish_fights, corpse-drop scoops for ammo, an opt-in tick pause (walker_b2 pause_at) as a re-roll. Goal: the test passes un-ignored.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.
**Branch:** `s/real-health-00agent-13-1`.
