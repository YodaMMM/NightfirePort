## Your task: `real-health-00agent-0d-1`, 0700000d's 00 Agent real-damage walk completes

Branch from current origin/master.

Master: death t15731 at (197.95, 10.56, -278.88) - the 0x1e sniper rounds: #22 (0x5000090) -46.0 at d 55.2 (t2609), #36 (0x5000090) -48.0 at d 55.5 (t13857).

The test: `walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage_00agent` (ignored on master; its ignore text is the measured ledger). Levers that finished the other 00 Agent walks (read their merged tests): live armour pickups near the death, crouch on long-range sniper legs, bounded aimed shots with re-pick, finish_fights, corpse-drop scoops for ammo, an opt-in tick pause (walker_b2 pause_at) as a re-roll. Goal: the test passes un-ignored.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.
**Branch:** `s/real-health-00agent-0d-1`.

Main 11-15: the test ignore text (master d73cf126a) carries the measured ledger. Short form: the Operative deck crouch at difficulty 3 reaches the gantry ride; nearest clear-line ride fire (20-60 u) gets Bond off it on ~20 health; the 10 m slot drop then costs 30 at 00 Agent (Plr_DMod 1.2, the fall skips armour) and kills him. Probe patch: audits-side, NF_0DOP/NF_0DRIDE/NF_0DPAUSE env switches. Next: more health into the slot drop.
