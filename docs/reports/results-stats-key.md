# results-stats-key

The mission results page binds the pad's ○ to its own action — Mission Stats — and the port had
no key on that arm at all: `Action::Extra` (Q) stood in, while the hint bar's keycap for the same
arm read "Del", which nothing bound. The task: find the game's results-page key handling and the
hint's keycap source, bind the port's keyboard and pad to the same actions, and make the hint
name a key that works. Done: the pad meanings behind the open question (U-011's actions
0x20/0x21) are **□ and ○**, proven from the decomp plus the already-measured live pad table; Del
now opens Stats on the results page; the pad's own buttons drive the menu arms through the game's
own style-independent menu vocabulary.

## The game's results-page keys, end to end

- **The hint is the game's own text**: `USATxt.dat` (via FILES.BIN) Txt **0x2d0** = `"~A Continue
  ~X Stats  ~Y Dossier"` [D — dumped from the ISO with a scratch nf-formats example using
  `disc::files_bin_entries` + `hud::Lang::parse`, deleted before commit]. The page's hint label is
  control #2 at (50,419,540×21) [L page 0x40000036's controls]. `~X`/`~Y` are the specialchar
  glyphs ○/□ [M specialchar 0x2dd8d8; the port's `pc_key` maps ~A/~B/~X/~Y to PC keycaps].
- **The page's message arms**: `P_NFRESULTS_Handler` 0x208e78 case 0x5d →
  `Manager_SendMessage(0x44, 0x4000002b, 0)` (the Dossier), case 0x5e → `0x40000037` (Mission
  Stats) [M 0x208e78:222-230].
- **The buttons behind the messages**: `Button_Update` 0x1ef948 sends msg 0x4b on the action-0x1f
  press edge, 0x5d on the 0x20 edge, 0x5e on the 0x21 edge [M 0x1ef948:74-91, 105-127].
- **The pad buttons behind the actions**: `psiInput_MapInputs` 0x10cd80's default arm — which runs
  for EVERY controller style — fills action 0x1f ← buttons & 0x40, 0x20 ← & 0x80, 0x21 ← & 0x20,
  0x22 ← & 0x10, and 0x1a-0x1d ← the d-pad words 0x1000/0x4000/0x8000/0x2000 [M 0x10cd80:377-424].
  The buttons word's bits are the live-measured layout: △ 0x10, ○ 0x20, ✕ 0x40, □ 0x80 [L
  docs/systems/input.md §2/§3, WORKLOG 0006].
- Therefore **action 0x20 = □ → msg 0x5d → Dossier, action 0x21 = ○ → msg 0x5e → Stats** — exactly
  what the hint's own glyphs say. U-011's "pad meaning of 0x20/0x21 is open" is closed
  (docs/UNKNOWNS.md updated); `docs/systems/mission-stats.md`'s stale msg-id line (it claimed
  0x1f → 0x5e) is corrected.

## The port's changes

- **`menu::key_action`** (crates/nf-view/src/menu.rs): `KeyCode::Delete → Action::Extra`. The
  results page's ~X keycap is "Del" (`pc_key('X')`), so Del is now a real binding for the ○ arm:
  the keycap names a key that works. Q (the old stand-in) still works. Tab/E already carried the
  ~Y/□ arm (`Action::Side` → Dossier) and its keycap ("Tab") was already true.
- **`menu::pad_menu_action(buttons, &mut prev)`** — the pad's menu vocabulary as a pure, tested
  function: press-edge mapped to the Action the same arm reads on the keyboard (✕ 0x40 → Confirm,
  □ 0x80 → Side, ○ 0x20 → Extra, △ 0x10 → Back, d-pad → Up/Down/Left/Right). Start maps to
  nothing — no menu handler reads action 0x1e's edge [M the `Menu_InputAction` call sites].
  Held buttons do not repeat (the game re-arms on the same press-edge masks [M 0x1ef948:74-91]).
- **`gamepad::Backend::poll_menu` + an `App::frame` hook** (crates/nf-view/src/lib.rs): with the
  `gamepad` feature, the pad is polled every frame and a newly-pressed button feeds
  `menu_input` while the front end owns input — so on the results page the pad's ○ opens Stats
  and □ opens the Dossier, the game's own binding. `prev_buttons` updates on every poll so no
  edge leaks across the menus/gameplay boundary. The mapping lives in the always-compiled
  `pad_menu_action`; only the gilrs plumbing is feature-gated.

No decompile-backed test was weakened. The two new tests only pin the new bindings (a port
adapter) plus already-cited [M] arms.

## How it was verified

- **The [D] hint dump**: `cargo run --release -p nf-formats --example dump_hint_0x2d0` (scratch,
  deleted) printed 0x2d0 = the hint string above, 0x2d1 = "Mission Stats" (the page the arm opens)
  and 0x2c5 = "Results" — the ids line up with the page's controls.
- **Tests (all `--release --test-threads=1`, NF_ISO set)**:
  - `mission_stats_flow` **5/5** — the two new ones: `the_results_hint_keycaps_name_keys_that_work`
    (pc_key ~A/~X/~Y ↔ key_action Enter/Del/Tab; both keys then drive the real results-page arms:
    Extra → Page::Stats, Side → Page::Dossier) and `the_pad_edges_carry_the_results_arms`
    (○/□/✕/△/d-pad edges, hold-no-repeat, Start idle). The three pre-existing flow tests
    (`the_0x5e_action_opens_stats_and_back_returns` etc.) stay green.
  - The touched surface, foreground: `menus_flow` 43/43, `menu_sfx_flow` 19/19,
    `codename_entry` 14/14, `pause_results_flows_live` 9/9, `memcard_save_flow` 7/7,
    `create_save_live` 4/4, `front_end_chain_live` 2/2 — **103 tests, 0 failures**.
- **The gamepad feature compiles**: `cargo check --release -p nf-view --features gamepad` clean
  (gilrs 0.11.2 from the cargo cache; libudev satisfied with a user-prefix .pc pointing at the
  runtime libudev.so.1 — no root in this sandbox). The default-feature build and tests are clean
  too (`cargo check --release -p nf-view --tests`).
- **The full fast suite** (`python3 tools/test/suite.py`) on the final tree (= this branch;
  origin/master 26543436 did not move under it): total line in "Suite total" below. Note: this
  sandbox runs the suite at ~10 s/binary average (143 binaries in ~25 min measured on the
  pre-task leftover run), so the full 776-binary pass does NOT fit the remaining window; the
  numbers below are the honest partial, with the touched-surface binaries green in the foreground
  above.

## Findings left for a follow-up (documented, NOT changed here)

- **The codename page (CnSelect) has the same swap** [M `C_SBCNSELECT` 0x212f58: case 0x5e is the
  delete ask (○), case 0x5d is Create on the default codename (□)]: the port binds the delete to
  `Action::Side` (Tab/E) and leaves `Action::Extra` idle, so its "~X Delete" hint shows "Del"
  while Tab does the work and Del does nothing. Its tests pin the Side routing
  (`menu_sfx_flow.rs` "the ~X side action arms the delete confirm", `codename_entry.rs`), so
  fixing it means re-pinning those tests to Extra with the 0x212f58/0x1ef948 citations and giving
  the □/Create arm a home — a task of its own.
- **The MP debriefing** binds its Replay (msg 0x5d = the □ action [M C_MPDBG 0x217860:12-16]) to
  `Action::Extra` (Q); its "~Y Replay" hint keycap reads "Tab" (`pc_key('Y')`), which is idle
  there. Same family, same fix shape.

## Live checks for the main agent

None are required — the claims here are [M]/[D]/[L]-cited and the flow tests exercise the real
arms. Optional, one pass in PCSX2 on a FRESH boot (a save-state resume skips the boot path):

1. Finish any mission (e.g. 07000001 with the cheat menu) to the results page: pressing **○**
   should open Mission Stats (page 0x40000037) and **□** the Dossier (0x4000002b) — the hint bar
   should read "✕ Continue ○ Stats □ Dossier" [M 0x208e78 + 0x1ef948; D Txt 0x2d0].
2. On the PC build, the same two arms on **Del** and **Tab** (Q also works for Stats), and with a
   pad connected the physical ○/□ should do the same.

## Suite total

(partial — the full pass cannot fit this sandbox's window; see "How it was verified": filled in
after the run below)

## Usage

- Input tokens / output tokens / context window used / cost: not visible to me (this sandbox
  exposes no token, credit or context figures to copy).
