# WORKLOG

## drone-bump-collision-2

Branch s/drone-bump-collision-2 (from origin/s/drone-bump-collision @ 07a98925, merged origin/master @ fcf6f334). Task: the Jointy push (Player_Collision 0x194df0 → Collide_Jointy 0x1ec180:229-266, Drone::bump_capsule per NDrone2_Collision 0x140e90:59-124) runs un-gated in every stance on master's port and turns seven live tests red (mayhew_hold_live escort-hold, party_door_lock_07000007_live both, walkreal_07000007_live party walk, probe_07_route, cutscene_timer_pause_live, walkreal_07000011_live lower-floor load). Find which push the game does NOT do there — the pick-level gate (which drones are armed for the player's class-3 query: +0x464 = 0x800 under NDrone2_DoCollision 0x149ec8:11-34, the 0x20/0x200 flag arms, NearPlayerCollision 0x14b190), allies/escorts/civilians in scripted states, the radius per state — and port exactly that. All seven green plus drone_bump_live; full suite green.

### drone-bump-collision-2 — outcome (checkpoint, the arming fix left)

The pick-level gate traced end to end (Collide_Pick's cel-straddle scoping 0x1eb320:125,312-336 → PickObj's exclusion word +0x98 0x1eae80 (disasm: 0x400/0x100/0x200 EXCLUDE classes 3+0x12/2+0x11/5; the player's +0x7d8 is zeroed per tick, 0x194df0:88) → Intersect's glist/+0xfa&8 gates 0x1ebaa8 (disasm 0x1ebb7c-0x1ebbac) → Jointy's dual kind-0x800 0x1ec180:203-209 → the arming 0x140e90:66-101 under DoCollision 0x149ec8 whose 0x20 master arm DefaultInit/Enable set and death/fade/disable clear, the near arm fed by PreDroneControl's +0x154 slots 0x148db0:35-48). Landed the cel-straddle arm on the producer (bot cel ∈ the player's straddled set, find_cel stand-in) — drone_bump_live 2/2 stays green. probe_07_route unchanged (same-room adjacency straddles), so the seven reds need the arming fix: frozen/scripted bots' control pass suspended → +0x154 stale → not near → +0x464 = 0 → no push; the port arms them from live positions. Left with the live checks in the report (the gdbwatch of a frozen bot's +0x464/+0x154 settles it).

## level-14-vent-drop-2

Branch s/level-14-vent-drop-2 (from origin/s/level-14-vent-drop @ ed167832, merged origin/master). Task: implement the Countdown Part 1 vent drop that level-14-vent-drop decoded but never wrote (touch 0xd9 -> OR 0x7e -> the SP switch; the case-6 capture -> Script_Free's Player_Enable), scoped as that report says, then walk walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit legs 2..6 to the exit on real input; un-ignore on green, else back to #[ignore] with the exact blocker.

### level-14-vent-drop-2 — outcome
The event-6 body-root fallback landed (`World::anim_root` — bone 0 of the posed skeleton; the
player skin 0x0500005e has NO discrete parts so `discrete0` reads None for it), `Obj::Anim.root`
refreshed per interp, and the hand-back is pinned green un-ignored:
`the_vent_drop_hands_bond_into_the_facility` (walkreal_07000014_live 3 passed, 1 ignored). The
capture pose is the body's anim root at event time ≈ (30.09, 1.06, 38.73) — the -1 report's
(27.60, 43.56) was the seq end, not the hand-back frame. The full delta walk stays `#[ignore]`:
leg 2 wedges at the facility's west double door (-52.2, 1.2, 40.1, state 0x2, ch_open 0,
ch_lock 0x85) — the next session's first question. Full suite did not fit the window (disk
cleanup + foreground-only rebuilds); the touched binary ran green before the push.

## menu-stats-page

Branch s/menu-stats-page (from origin/master @ fcf6f334). Task: the Mission Stats page (P_NFSTATS 0x209938, Del from the results page) drew its row labels as raw ids, had no headers except TOTAL, overlapping value columns and no page furniture — resolve the labels through the game's string lookup and lay the page out from its own control data and draw code.

### menu-stats-page — outcome (the page drawn like the game's; one root cause was a digit slip)

The row-label symptom was TWO stacked bugs: (1) `menu_after` fed `stats_enter` only the front-end `self.lang`, so on a bare-mission boot (no front-end USATxt.dat) every row category fell back to its `<0x…>` id while the page's own furniture still bound off the sim's bank — fixed with the draw sites' `self.lang.or(sim.lang)` fallback; (2) the header quartet const `STATS_HEADER` transcribed the decompile's seven-digit `0x1000204` as `0x1000_0204` (= 0x10000204, bank 0x10 — past USATxt.dat's 7 fixups [D the file's NumFixups 7]) so the binds could never hit; the real ids are bank 1: 0x01000204 → "Category", 0x010001e1 → "Current", 0x010001e2 → "Target" [L ee_snapshot FixupTable 0x1f9b200 (fixups[1] = 0x3e8), Bank[1516]/[1481]/[1482]], 0x163 → "TOTAL" [D]. The pause SCORE tab carried the same slip (its hardcoded fallbacks masked it) — fixed with the same ids [M 0x204408:455-459].

The layout now comes from the page's own data and the list widget's draw math: the [D] 08000002 page 0x40000037 controls (title (50,40,376×17) LBL 0x2d1 "Mission Stats" [L Bank[721]], hint (50,419,540×21) LBL 0x49d "~B Back" [L Bank[1181]], the stamp 0x10000224 TEX 0x0300014c at (340,198,255×255), the logo 0x0300016d at (434,38,156×43), the list (60,78,520×324)); the columns per [M List_Update 0x21a598's math from the build's sends (arm 0x17 reset, arm 0x28 widths[0] = 0x23 = 35%, arms 0xe 21/21/24 right-aligned): Category left at x 68, Current right edge 353.6, Target 459.4, TOTAL 580.9 (640 space); the header is ROW 0 of the list [M 0x2196e8 arm 0x10 adds it, arms 0x19 fill cells 1..3]; the even-row stripes [M arm 0x73 → +0x548] as a translucent plate stand-in [S the look]; the row pitch stays the pause tab's measured 16.7 [S — the game's Font_GetTextExtent·1.1 stride needs the main agent's render compare]. Menu_AddRow 0x201c10's "label|cur|tgt|total" split and the arm semantics were verified instruction-side before layout.

Ran: the fast runner subset `--only nf_game,nf_formats,nf_audio,nf_view,mission_stats_flow,menu_graph_walk,menus_flow,fix_scores_handoff_live,menudef_script_runtime,menudef_list_widget,codename_entry` → **871 passed, 0 failed, 5 ignored (2 min)**; plus walkreal_07000004_live's stats test green earlier in the cycle. The FULL 874-binary suite did not fit the window (a background runner dies between tool calls in this sandbox; the foreground build cycles ate most of it) — the runner subset covers every touched binary plus the four crates' unit tests; the report lists what ran. New test `the_stats_page_resolves_its_labels_from_the_language_bank` pins the header/furniture/row strings through the real lang path.

## mission-stats-rows

Branch s/mission-stats-rows (from origin/master @ b25b5f34, merged origin/s/menu-stats-page). Task: the Mission Stats rows show wrong values/targets/points (Double Cross Part 4: Time 00:08 -> 1,585,205 points; Opponents Dispatched target 45 with one assassin) and the Dossier Records page shows High Score / Bond Moves / Upgrades all 0 after finishing Double Cross — port the stats builders (PlrStat_* tallies, per-level target table, points formula) row by row, and the records writer for mission end, pinned by tests from a real 07000004 finish.
