# Aim snap: the 8 walks, with a corrected premise

Branch from `origin/s/aim-candidates`, merge current `origin/master`, push to `s/aim-console-phase`.

CORRECTION: 10-aim-candidates said the port stops on objects the game does not have. That is withdrawn. The PCSX2 rays and object reads behind it ran with Bond idle, before the launch order, when the rigs were down.

What is now measured on 0700001b:
- The port's rig pose matches the game. r 1.796 pieces: t 0 (24.44, -8.80, -11.39) in both; game frame 1782 (26.25, -3.02, -11.39), port t 1620 (26.28, -3.04). Class 0, f0 0x80, fa 0xc4, glist+8 set, in the cel chain, so the game's pick can stop on them.
- Idle, only one rig rises. The rest come with the console phase: channel 19, raised by the first wave's death. The walk starts `shoot_consoles` at frame 2746. Writing channel 19 alone in PCSX2 raised nothing, so the phase needs more than that write.
- So there is no evidence the game's ray skips raised rigs. Do not remove candidates to make walks pass.

Do not search for a discriminator between "real" and "fake" entities. Instead:
1. Log what the snap stops on at each failing shot of consoles 1 and 2: the entity, its role in the launch script, and where the console's own hit sphere sits behind it. Is a raised rig really between Bond and the console from that stance?
2. If so, the snap may be correct and the walk's stance wrong: a player would move to a clear line. `level_station` plans from `sphere_shot_ok`, which does not see these entities, so make it use the same candidates as the snap. Try that first on 1b.
3. Then look at the other six reds (08, 0c, 14 x3, 16) the same way, one at a time. The aim stack changes where rounds land, so fights tuned to the raw aim may need route levers. Say which is which. 14's three are a nav stick, "the corridor south to the leg-2 spot", so look at what stands there.
4. To reach the real phase in PCSX2 later, list what the first wave's death does besides channel 19, in order, with citations.

Keep the [L] 08 pins and the 0x30000000 fix. Quote the six aim binaries and walkreal 08 / 0c / 14 / 16 / 1b.

---
claimed-by: sandbox4 at 2026-11-06T10:42Z

---
## Report (done, 2026-11-06T11:47Z)

