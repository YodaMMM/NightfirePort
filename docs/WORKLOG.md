
## level-14-rook-cutscene

Task: on 07000014 the post-cascade cut-scene at the Delta Sector door (camera ~(-34.62, 1.07, 26.19), frame ~7735) must show Rook (drone skin 0x05000066, [D] docs/levels/07000014.md: pos (27.81, 0.20, 45.78), wake ch 0x0D, death ch 0x54); the port draws a masked guard instead. Find the cut-scene script + actor list ([D] NIS streams; [M] Script_CreateEntity 0x1d8968 + actor binding), decide what the game binds, fix the port, pin with tests naming the entity + skin drawn at cut-scene start (sister fails on the guard skin).
