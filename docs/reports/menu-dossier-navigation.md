# menu-dossier-navigation — the report

Branch `s/menu-dossier-navigation` (from `origin/master` @ dcacdf1e, merged with `origin/master`
@ 83490999). Task: the Dossier (0x4000002b) and the pages it opens navigate like the game's —
every page returns with Back to the page that opened it, the cursors walk each page's controls in
the game's order, and no path strands the player.

## What the game does (the evidence)

- **The page stack** [M Manager_SendMessage 0x21c858]: a page open (msg 0x44) whose lparam has
  bit 0 CLEAR pushes {current page, current cursor control} onto the manager's stack at +0xa8
  (:262-274 — `uStack_b0 = ((uint)param_4 ^ 1) & 1`, the push at :269-274). The back key runs
  msg 0x5f (:567-610): the page's back-slot script first (the disc binds none [D the 08000002
  tag stream — every page's only SCRIPT is 0x2000000f on the 0x4c-open slot 0]), then the pop:
  `Script_PlayDefault(page, 99)` (none bound), re-open the top's page with 0x44 lparam 5 (or 13
  under an overlay) (:592-598 — through `__Menu_SendMessage`, which re-enters the manager
  because its type byte is 7 [M __Menu_SendMessage 0x1f1c18:42]) and restore the saved cursor
  with msg 0x22 (:598). On an EMPTY stack the 0x5f sends msg 99 (0x63) to the page's handler
  (:604-610): P_NFMAP (the SP list) opens Main stack-clearing [M 0x20b608:67-69]; the Dossier
  pages and Stats arm nothing, so the page stays [M P_DOSSIER 0x216de0, P_DSRECORDS 0x217248,
  P_DSREWARDS 0x2172c8, P_DSGADGETS 0x217190, P_DSWEAPONS 0x217038, P_NFSTATS 0x209938].
- **The back key** [M MenuManager_Update 0x21c3d8:77-106]: action 0x22 → the page's
  page+0xb0+4·slot back control's msg-0x5b script → the cursor control's 0x5b script →
  `Menu_PlaySound(8)` + `Manager_SendMessage(0x5f, 0, slot)`.
- **The Dossier's openers, all lparam 0 → push**: the SP list's □ [M C_SBNFMAP 0x20b8d8:47-48 —
  msg 0x5d → Menu_ChangePageCloseIris(0x4000002b), whose delayed 0x44 carries DAT_002e1adc = 0
  [M 0x202190; 0x20a648:21 zeroes the global]]; the Results page's Tab msg 0x5d →
  `0x44 0x4000002b 0` and its msg 0x5e → `0x44 0x40000037 0` [M P_NFRESULTS 0x208e78:222-230];
  the Bonus page's 0x5d/0x5e identically [M P_NFBONUS 0x209670:60-68].
- **The Dossier wheel's rows** [M C_SBDOSSIER 0x216eb0:12-32]: ✕ asks the wheel (msg 0x40) —
  rows 0/1 (Records 0x4000003a / Rewards 0x4000003b) ride the handler's DIRECT
  `Manager_SendMessage(0x44, page, 0)` (:29-31 — immediate); rows 2/3 (Gadgets 0x4000003c /
  Weapons 0x4000003d) ride Menu_ChangePageCloseIris (:20/:26). Both shapes carry lparam 0 →
  push. Cursor on arrival: Records/Rewards select the highlighted campaign row (msg 0x1e
  GameState+0x72 [M 0x217248:11-13, 0x2172c8:11-13]); Gadgets/Weapons reset to row 0 (the init
  MSG 0x2e (0,0) [D page data] + P_DSGADGETS 0x217190:10 / P_DSWEAPONS 0x217038:35). The
  Dossier's own 0x4c resets the wheel when the opener is the list/results/bonus
  [M 0x216de0:9-16].
- **The cursor wrap** [M Scroll_Update 0x2221b20:186-210]: the scroll wraps at either end only
  while its +0x144 flag stands; Scroll_Init 0x2214f0 sets the flag at create when the control
  record's field reads != 0xff (disasm 0x2217b8-0x2217cc: `lw v1,16(s3)` / `beq v1,a0` with
  a0 = 0xff), and msg 0x69 is the only clear [M Scroll_SendMessage 0x221810:136-139]. All five
  Dossier-family scrolls' parsed records carry d1 = 0, f1 = 25, f2 = 2-3, and their bound MSGs'
  a-parameters are 0 or 4 — every candidate field != 0xff [D the parsed 08000002 pages
  0x4000002b/3a/3b/3c/3d], and no 0x69 appears in any of their streams [D the parsed MSG tags].
  So the Dossier wheel 0x1000010c AND the Records 0x10000174 / Rewards 0x1000017a spinners and
  the Gadgets 0x1000016f / Weapons 0x10000173 scrolls all wrap.
- **The sub-pages have no ✕/□ arms** [M]: C_RBDSRECORDS 0x20bf00 and C_RBDSREWARDS 0x20c248 arm
  only msg 0x49/0x54 (the rebind); the wheel handlers 0x20bcc0/0x20bab0 arm 0x49/0x51/0x54;
  ✕ on them does nothing.

## What changed (crates/nf-view/src/menu.rs)

1. **Results → Tab/Stats now push the opener** (the strand fix): the Complete results' Side
   (msg 0x5d) and Extra (0x5e) arms push {Results, cursor} before opening the Dossier/Stats.
   The old push-less Tab hop left the Dossier over an empty stack whose 0x5f does nothing, and
   the port fell through to the mission list — the task's stranded player.
2. **Bonus → Tab/Stats push the Bonus** the same way [M 0x209670:60-68].
3. **The Dossier wheel's Confirm pushes the Dossier for all four rows** — rows 0/1 immediate
   `enter_page` (the game's direct 0x44), rows 2/3 through `close_iris_page` (the 10-tick
   delayed 0x44). Rewards now arrives on the highlighted campaign row (msg 0x1e
   GameState+0x72 [M 0x2172c8:13]); the old code reset only Records'.
4. **Every Back on the Dossier family pops the stack** and restores the saved page + cursor
   (+ the flow.page mirror); the empty-stack arms are no-ops with the [M] citations (the old
   code hopped to MissionSelect from the Dossier and hard-coded page/cursor 0 on the sub-pages).
5. **✕/□ on Records/Rewards/Gadgets/Weapons do nothing** [M the handlers above] — the old
   ✕-returns-to-Dossier arm had no decompile basis.
6. **Records/Rewards/Gadgets/Weapons cursors wrap** at both ends [M 0x221b20 + the init disasm;
   D the records] — the old Rewards/Gadgets/Weapons clamp was the inverted 0x69 reading.
7. The Dossier's □ (Side) is a no-op — P_DOSSIER/C_SBDOSSIER have no 0x5d arm [M].

## Tests (all through the real input path)

- NEW `menu_sp_e2e.rs`: `the_results_tab_opens_the_dossier_and_back_returns_to_the_results` —
  boot → main → codename → difficulty → the mission list → The Exchange → the runner → the
  results page (the load tail's reopen) → Tab → Dossier → Back → Results with the saved cursor;
  then Extra → Stats → Back → Results; then the empty-stack Back stays on Results.
  `the_dossier_family_walks_back_through_the_page_stack` — missions □ → Dossier → each of the
  four rows opens its page (with the CloseIris ticks) → Back → the wheel on the row that opened
  the page → Back → the list with ITS saved cursor.
- Fixed tests (the tests were wrong against the decompile):
  - `menus_mouse_parity.rs` the_records_spinner…: a click on a Records row (✕) does nothing
    [M 0x20bf00 — no 0x4b arm]; the old pin expected a hop to the Dossier.
  - `menus_mouse_parity.rs` the_dossier_gadget_and_weapon_wheels…: Back from Gadgets returns
    the wheel to the row that opened the page (2), not row 0 [M 0x21c858:598-600].
  - `dossier_pages.rs`: row 1 (Rewards) opens immediately, not through CloseIris
    [M 0x216eb0:29-31]; the three lists wrap [M+D]; the empty-stack Back on Gadgets stays
    [M 0x21c858:604-610]; the pushed pop restores the opening row.
- What I ran (all green): `cargo test --offline --release -j2 -p nf-view --test menu_sp_e2e /
  --test menus_flow / --test dossier_pages / --test mission_stats_flow / --test
  menus_mouse_parity / --test menu_graph_walk` (`-- --test-threads=1`) — 11 + 43 + 7 + 17 + 2
  passed; plus the four-crate suite below.

## The fast suite

`NF_ISO=… python3 tools/test/suite.py` on the final tree (merged with origin/master @ 83490999):
see the tail line below (filled at the finish).

## What's left / notes

- **The Results→Dossier iris**: P_DOSSIER's 0x4c runs `Menu_StartIris(0, …, 0x1000010b)` + the
  delayed 0x2e when the opener is 0x4000001c/36/38 [M 0x216de0:9-16]. The port resets the wheel
  but does not run that iris animation (draw work — visual, needs the main agent's render).
- The pad meanings of the results' actions 0x20 (□) / 0x21 (○) are pinned only by the hint
  glyphs [D USATxt.dat 0x2d0] — U-011 remains [S].
- P_NFMAP's stack-dependent hint (0x208 with a non-empty stack vs 0x283 [M 0x20b608:49-57]) is
  not modeled (hint text only).

## Live checks for the main agent

1. **Wrap-flag init field**: disassemble Scroll_Init 0x2214f0 around 0x2217b8 — confirm the
   `lw +0x10` of the runtime control record maps to the menu-def record's d1 (offset 16) and
   watch `*(u8*)(scroll+0x144)` after opening 0x4000002b: expect 1 for 0x1000010c (the wheel),
   and after opening 0x4000003a/3b/3c/3d expect 1 for 0x10000174/0x1000017a/0x1000016f/
   0x10000173 (all five wrap). A ○/△ press past each end should wrap, not clamp.
2. **Fresh boot, Results → □ → Dossier → ○**: the Dossier must return to the RESULTS page (not
   the mission list), with the results cursor where it was. Screenshot after the ○.
3. **Fresh boot, SP list □ → Dossier → Gadgets (row 2) → ○**: the wheel must land back on
   GADGETS (row 2), not Records. Screenshot after the ○.
4. **The Records/Rewards spinner ○ at the ends**: Up at row 0 wraps to row 11; Down at row 11
   wraps to row 0.

## Usage

not visible to me (no token, cost or context-window figures are exposed in this environment).
