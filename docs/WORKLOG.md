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

### menu-dossier-navigation — the stack, the Back arms, the wrap (2026-10-27 ~10:05 UTC)

Read the decompiles [M]: the manager's page stack is Stack_Push({page, cursor}) on msg 0x44
lparam bit-0-clear (0x21c858:262-274), the pop on 0x5f re-opens the top's page with its saved
cursor (0x21c858:582-602, the cursor rides the 0x22), and an empty-stack 0x5f sends msg 99 to
the page's handler (0x21c858:604-610) — which P_DOSSIER 0x216de0 / the P_DS* pages / P_NFSTATS
do not arm (the page stays), while P_NFMAP 0x20b608:67-69 opens Main. The openers: the SP list's
□ (C_SBNFMAP 0x20b8d8:47-48, CloseIris), Results' Tab/0x5d (P_NFRESULTS 0x208e78:222-230) and
0x5e (:225-230), Bonus' 0x5d/0x5e (P_NFBONUS 0x209670:60-68) — ALL lparam 0 → push; the Dossier
wheel's rows (C_SBDOSSIER 0x216eb0:12-32) — Records/Rewards direct 0x44 lparam 0 (immediate),
Gadgets/Weapons CloseIris — all push.

Port fixes (crates/nf-view/src/menu.rs): Results Tab/Extra and Bonus Tab/Extra now push their
opener before opening the Dossier/Stats (the old push-less Tab hop stranded the player on the
mission list — the strand the task describes); the Dossier wheel's Confirm pushes the Dossier
for all four rows (rows 0/1 immediate, rows 2/3 CloseIris); every Back on the Dossier family
pops the stack (and restores flow.page); the empty-stack else-arms became no-ops with [M]
citations; ✕/□ on Records/Rewards/Gadgets/Weapons do nothing (their handlers arm only 0x49/
0x54/0x4c/0x50 [M 0x20bf00, 0x20c248, 0x20bcc0, 0x20bab0]); Records/Rewards/Gadgets/Weapons
cursors WRAP — Scroll_Update 0x2221b20 wraps while +0x144 stands, Scroll_Init sets it from the
control record (disasm 0x2217b8-0x2217cc: lw +0x10 != 0xff; the five scrolls' records carry
d1=0/f1=25/f2=2-3 [D the parsed 08000002 pages]) and msg 0x69 — the only clear
[0x221810:136-139] — appears in none of their streams [D the parsed MSG tags].

Tests: two new real-path tests (menu_sp_e2e.rs): the results-Tab strand (Results → Tab →
Dossier → Back → Results with the saved cursor; Stats back; the empty-stack no-op) and the
Dossier-family stack walk (missions □ → Dossier → each sub-page → Back → the wheel's saved row
→ Back → the list's saved cursor). Fixed with the addresses: the mouse-parity Records
row-click pin (✕ has no arm [M 0x20bf00]), the Gadgets Back row-0 pin (the pop restores the
opening row [M 0x21c858:598-600]), the all-rows-CloseIris comment (row 1 is the direct 0x44
[M 0x216eb0:29-31]), and the Rewards/Gadgets/Weapons clamp [S]s (wrap, [M]+[D]).
