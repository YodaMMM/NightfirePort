
## level-14-rook-cutscene

Task: on 07000014 the post-cascade cut-scene at the Delta Sector door (camera ~(-34.62, 1.07, 26.19), frame ~7735) must show Rook (drone skin 0x05000066, [D] docs/levels/07000014.md: pos (27.81, 0.20, 45.78), wake ch 0x0D, death ch 0x54); the port draws a masked guard instead. Find the cut-scene script + actor list ([D] NIS streams; [M] Script_CreateEntity 0x1d8968 + actor binding), decide what the game binds, fix the port, pin with tests naming the entity + skin drawn at cut-scene start (sister fails on the guard skin).

**Result (2026-10-31):** the cut-scene is 07A00014.bin's escape scene **0x06000848** (SP #733, 950 ticks, the
only 07000014 scene whose camera watches the exit door area; camera keys (27.82, 0.97, −20.38) looking south
at the exit #944 [D decode]). Its actor list is two `Script_AnimStart` streams: stream 1 = skin 0x0500008C
(the masked guard, anim 0x0600085b → body seq 0x0400072c) and stream 9 = **Rook's own skin 0x05000066**
(anim 0x0600085a: facial 0x04000745 [1..979] + body 0x0400072b [1..951]) [D]. The game binds a FRESH anim
object with the skin, not Rook's drone (`control_create_object(0, 0x2d6200, 0x2d6200, 0)` + `AnimObjectNew`
[M Script_AnimStart 0x1d8f40:69-74]). The port drew the guard only: Rook's body seq roots at
(27.2446, 1.2597, −24.0801) [D 0400072b] — beside the guard, in front of the door — but the port posed him
at 1.1× that: his skin record's scale is (1.1, 1.1, 1.1) [D 0x05000066] and `local_pose_bones_with`
multiplied the ROOT translation by it. The game unscales the buffer's first (root) translation record:
`AnimFrameCopy` 0x1c5668:536 `Vec_Mul(param_2, 0x319d90)` (the inverses set in `AnimObjectUpdate`
0x1befd8:119-124; reset to 1.0 at :282-287, so `AnimGetFirstTrans`'s seed is unscaled too [M 0x1d8f40:93-94]).
Fixed in nf-formats/src/anim.rs (the first translation record unscaled); the gameplay drone path zeroes the
root anyway. Pinned by `escape_scene_actors_07000014_live` (2 tests): both actors live/posed/palette through
the real touch #456 + SEQ_AND #771 start, Rook's root at the unscaled disc value (the 1.1× root fails it as
a sister), the guard root unchanged, stream 9 must carry skin 0x05000066. Ran nf-game --lib (588 passed) and
41 nf-view cutscene/nis/script + level-14 binaries (100 passed, 0 failed, 4 pre-existing ignored); the full
suite could not run in this sandbox (background jobs are killed between tool calls — run per-binary instead).
