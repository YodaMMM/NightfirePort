### menu-dossier-navigation — start (2026-10-27 ~08:52 UTC)

Branch `s/menu-dossier-navigation` created from `origin/master` @ dcacdf1e.
Task: in the front end, the Dossier (page 0x4000002b and the pages it opens,
including the one reached from the mission Results page's Tab) is hard to move
around in, and from it the player can land on the mission-select page with no
way back. Port the front end's page stack and per-page Back/Cancel/cursor arms
exactly as the game runs them [M MenuManager_Create 0x21b908's page tags, the
MenuManager push/pop and back-key handling, each Dossier page's handler and its
control focus order]: every page reachable from the Dossier returns with Back
to the page that opened it, the cursor moves through each page's controls in
the game's order, and no path strands the player. Drive it through the real
menu path in a test (main menu -> Dossier -> each sub-page and back; Results ->
Tab -> Dossier -> back) with the page ids and focus order cited [M].

Plan:
- Read MenuManager_Create 0x21b908's decompile for page tags, push/pop and
  back-key handling; each Dossier page's handler and control focus order.
- Reproduce the stranded-player path through the real menu path in a test.
- Port the game's page stack / Back arms; cite addresses for every pin.
