## Your task: `real-health-00agent-14-landing-6`, 07000014's 00 Agent walk reaches the Delta exit

Branch from current origin/master.

Master d9562a01f: the node-21 visibility scan (landing-5) moves the death t41509 -> t41619 at (-16.80, 1.13, -24.18). Read done/50-real-health-00agent-14-landing-5.md (its "Main 11-15" notes 1-4 are measured levers). Measured on the old tree and worth porting onto this one: (a) the fights' finish_fights opt-in from leg 5 (best single option, t41509 -> t41810); (b) AMMO: on rolls that keep health, the leg-5 fights empty the 0x1a (out of ammo at +807 on #37 at node 8) - picking up the dead guards' drops within 8 u at a stop near (-24.2, 0.9, 10.7) refilled clip 25 spare 35 at 15.8 health; (c) with (a)+(b) the walk reaches past the Rook door (27.0, -20.2) and dies to the 0x500008c gauntlet there (#50 behind the slide door (22.0, -25.1), #52 (28.6, -20.2), #46 (29.3, -18.4), #47 at the second door (27.8, -19.6)) on ~5-8 health. Measured dead: armour/ammo detours to the escape room (they route west through (-42.2, 29.3) and die), crouching leg 5, a door hold, pre-door scans (nobody visible before the door opens). Goal: the 00 Agent test passes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.
**Branch:** `s/real-health-00agent-14-landing-6`.

Main 11-15 (on d9562a01f): best measured so far - finish_fights from leg 5 + an idle 120 ticks at t37000 (walker_b2 tick_inner) + a drop scoop after leg-5 node 7 (every live pickup within 8 u, dy < 1.5, then walk back to node 7): health 29.1 after the scoop (0x1a clip 22 spare 31), the node-21 scan and the delta approach clean (#39/#40/#41/#55/#51 killed for 3.5), past the Rook door on 18.6, death t43636 at (30.09, 1.04, -18.26) to #47 (0x500008c, (28.1, -20.3)) while the nav shot Rook at 23.2 u. Same outcome from idles at t35800/t36400. Rook-stretch opt-ins (hurting_first/precise/eye/world/unseen/lane off/stuck_answers/no round dodge/aimed) do not pass (26.0-30.1). Scoop at node 6 or 9 is worse. Next: answer #46/#47/#53 (the room past the second door (27.8, -19.6)) before the Rook shot - they are within 1-5 u when the walk opens that door.
Main 11-15 (2): a stop at the Rook door (26.2, -21.1) on that roll (18.6 health) + a kiko_hold-style clear: 60 ticks ends at 11.6 and dies t43286-43655 at (30.7, -24.4); 120-600 ticks die in the hold (t43220 at (27.88, -21.80)). The room is a crossfire - the answer has to come from outside the door line (or with more health in).
