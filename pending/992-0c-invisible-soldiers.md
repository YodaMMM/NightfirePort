## Your task: `0c-invisible-soldiers`, soldiers on 0700000c that are not drawn

User playtest 2026-10-22 (check ladder0cm): "that worked but there are also invisible soldiers now". (0c = Chain Reaction's first level; the ladder check starts at `--bond 11.64,1.04,-184.13,-2.578,0`.)

1. Census every drone on 0700000c from the start up to the yard (placements, script spawns, reinforcements): skin id, whether its skin/model resolves in the port's scene, whether the draw list holds it while present (missing-drone-skins 110c/110c2 and the G-021 session skins are prior art).
2. Find the ones that are present and active (shooting, walking, colliding) but have no drawn model, and why: an unresolved skin (which archive carries it [D]), a hidden flag left set, a cull rule?
3. Fix the sim/load side with evidence, plus a real-path test that every present 0c drone has a resolved model. The main agent checks the render.
