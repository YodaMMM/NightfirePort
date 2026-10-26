# WORKLOG

## dmode-inits-hud-panes

Started 2026-10-26 ~12:20 UTC. Branch `s/dmode-inits-hud-panes` from origin/master (7e35bd77).

Task: port or prove-equal the 28 never-mentioned `NDrone2_init_DMODE_*` guard-mode init
functions (0x14d978..), the four HUD pane creators (`HUD_CreateAmmoPane` 0x1a0ed8,
`HUD_CreateHealthPane` 0x1a0f40, `HUD_CreateInfoStatusPane` 0x1a10b8,
`HUD_CreatePickupStatusPane` 0x1a1140), and `Text_SendMessage` 0x223600. For each: find
the on-foot caller, compare with the port, port diffs with [M] addresses, add tests per
ported behaviour.

Work log:
- Setup: cloned repo, created branch, pushed initial checkpoint.
- Found prior art: docs/systems/ai-dmode-inits.md + drone.rs init_properties already prove the
  DMODE dispatch census (all 555 placements NEW, only row 0 dispatched). Remaining: per-init [M]
  summaries for the 26 still-uncited family members, a disc-table row test, HUD pane create
  audit (HealthPane 4-sprite loop + timers, InfoStatus/PickupStatus secondary slots), and
  Text_SendMessage 0x223600 (not cited anywhere in crates).
- Added: docs/systems/ai-dmode-inits.md appendix (all 33 init bodies, grouped), drone.rs family
  citation, nf-formats/tests/dmode_row_table.rs (35 disc rows pinned — Defaults is not a row fn,
  fixed after first run), nf-formats/tests/hud_pane_creators.rs (glow templates == the
  HealthPane create-loop writes; ammo/health create pointers), hud-panes.md [S]->[M] upgrades
  for the four status creates + a create-fn audit section. Both new tests pass.
- Next: Text_SendMessage 0x223600 caller census ([M] __Menu_SendMessage 0x1f1c18 type-0xc arm +
  TextBox_Update 0x2237c8 self-0x53) vs the port's generic control_msg — census test pending.
- Text_SendMessage 0x223600 [M]: dispatcher read (arms 0x14/0x16/0x21/0x2b/0x39/0x3b/0x3d/0x47/0x53,
  default -> Label_SendMessage(+0x94)); dispatch proven at __Menu_SendMessage 0x1f1c18 (type byte
  +0x7b, case 0xc); sole constructor Manager_SendMessage 0x21c858 case 0xc -> Text_Init 0x222b80
  (sole jal 0x21cad4). Caller census [D]: new nf-formats/tests/textbox_msg_census.rs walks all 34
  shipped menu defs (283 pages) via the calltree_census multi-hash parse — ZERO type-0xc controls
  anywhere -> Text_SendMessage and TextBox_Update 0x2237c8 are unreachable in the shipped game;
  the port needs nothing for them (its generic control_msg covers every shipped widget type).
  Note: PAGE tags carry 0x8000000N file hashes, not the entry-name hash (learned via 0-page parses).
- Full suite restarted in background (env launcher in /tmp; earlier background runs died silently).
- Suite: this sandbox kills every background job between tool calls (3 nohup attempts incl. an env
  launcher died silently, empty logs), so the fast runner could not run. Ran the touched crate's
  whole suite in the foreground instead: `cargo test --offline --release -j2 -p nf-formats --
  --test-threads=1` => TOTAL 105 passed, 3 failed — the 3 are the known slim-ISO `disc_golden`
  failures (bigf_header_fields_behave_as_documented, bigf_tables_match_the_golden_manifest,
  directory_tree_matches_the_golden_manifest; all in tests/disc_golden.rs). All 4 new/changed
  test files green. nf-game change is comment-only (drone.rs citation); not re-run, stated honestly.
- Report written: docs/reports/dmode-inits-hud-panes.md.
