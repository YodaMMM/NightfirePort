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

Main 11-15 (2): 48+48 random combinations of the Operative levers (yard armour, roof act, deck crouch, ride fire) + nearest clear-line ride fire (20-60 u) + a tick pause: best death t20142 just after the ride (the slot drop). A bounded clear before boarding from the pulley arm (radius 30-60) kills #0 but costs the gantry park (the claw then misses the pulley #370); from the E-plate spot it kills #1 (#0 survives) and later legs wedge (the stick turn to yaw -1.5708 assert). The yard is a 0x1e gallery at 00 Agent (#32 west roof (151.4, 22.7, -238.3), #20 (196.9, 14.0, -264.1), -46 a round at 50-68 u).

Main 11-15 (3): a shorter drop exists - the down-ray map under the ride line (z -296) reads a container top at y 7.1 for x 145-151 (and 156-158) against the yard 4.0 [L the probe at the apex]; the slot sits at ~ox+0.9 (ox = the gantry entity x, apex 137.59). Breaking the ride at ox 146-151 and hopping does NOT land Bond on it - every early hop lands back on the girder (the gantry still moving), the re-hops carry him to the apex slot and the yard drop kills him as before (t19881-19950). Next: hop with the drift matched to the moving slot, or stop the gantry (its park only at the ends).

Main 11-15 (4): THE CONTAINER DROP WORKS - break the ride when the gantry entity x < 153-154 and hop with the west drift held 35-37 ticks (instead of 20): Bond lands ON the container top (x 145-149, y 8.2 = floor 7.1 + 1.1) with ~18.6 health - the short fall costs almost nothing (the slot drop at the apex cost 30). The landing check must accept y < 8.6 there, and the apex/fall-height asserts gate off. On that roll the yard garrison then kills him on the container: #1 (0x500007e, 0x12) from (150.4, 5.0, -290.2) at 6.8 u, -1.8 a round; the walk's sweep had the 0x1e rifle with clip 0-1 (reload leaves 1+15); the P2K / 0x12 switch before the hop measured no pass (deaths t19709-20366). Next: answer #1 from the container with a full-clip automatic (pick it up earlier) or land on the second container top (x 156-158, y 7.1) farther from #1. Probe patch: audits/tools-wip/walk-0d-probes-1115d.patch (NF_0DEARLY/NF_0DHOLD/NF_0DGUN/NF_0DRELOAD + the earlier switches).

Main 11-15 (5): 48 random combinations over the container drop (break at gantry x 151-155, hold 33-38) x the Operative levers x ride fire 20-60 u x a pause x a P2K/0x12 switch x a reload: no pass, best death t20219 (in the yard after the landing). The yard after the drop on ~18 health is the wall now: the drop-then-sweep has to beat #1 (6.8 u) and the 0x12 trickle with that budget.

Main 11-15 (6): correction - the container landing is NOT always cheap: the hop is a JUMP, so the fall timer counts from the apex, and re-hops add height; the P2K run 154:37 landed on the container and its summary shows other 33.6 (the falls) - only the clean first-hop landings (e.g. 154:35, ~18.6 health on landing) are cheap. A lever that lowers the hop (no jump: walk off a girder edge where one opens, or a crouch-jump) would make the container drop reliable.
