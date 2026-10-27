# Report: level-4a-ride-hand-props

Task: In Phoenix Fire Part 3's (0700004a) lift-ride landing cut-scene (pack 0x060006cf, SP #289) the
guards' hand props (their weapons) are missing. Find how the game attaches props to script-animated
actors ([M] Script_CreateEntity 0x1d8968, the anim object's attachment/bone-socket path, the pack's
prop entities and their parent links, [D] the pack's streams and objects) and port it so each prop
draws in its actor's hand for the whole scene. Test: in the scene, each prop exists and its pose
follows its parent bone [M/D]; the 07000014 scenes stay unchanged.

## Plan (WORKLOG start entry)

1. Branch `s/level-4a-ride-hand-props` from origin/master (08f06953) — done.
2. Background suite started (fast runner, tools/test/suite.py).
3. Locate the port's cut-scene/script-entity path and how 0700004a's SP #289 pack 0x060006cf is
   loaded and rendered today; find where the props go missing.
4. Read the decompile: Script_CreateEntity 0x1d8968, the anim object attachment/bone-socket path,
   the pack's prop entities and parent links; check disassembly where the C looks incomplete.
5. [D] the pack's streams/objects: prop entities, parents, sockets.
6. Port the attachment so each prop draws in its actor's hand for the whole scene.
7. Test through the real scene path; assert each prop exists and follows its parent bone [M/D];
   07000014 scenes unchanged. Full suite, checkpoint pushes every 20-30 min, final report.

## What's done

- (in progress)

## How to verify

- (filled at the end)

## What's left

- (filled at the end)
