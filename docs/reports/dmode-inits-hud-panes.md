# dmode-inits-hud-panes — the last never-mentioned on-foot functions: DMODE inits, HUD pane creators, Text_SendMessage

2026-10-26. Branch `s/dmode-inits-hud-panes` (from origin/master 7e35bd77). Everything below is
either [M] (decompile/disassembly, ACTION.ELF), [D] (disc bytes), or marked otherwise.

## What was done

### 1. The 33 `NDrone2_init_DMODE_*` family (0x14d528..0x14e568) — per-init summaries + disc-table pin

- Read all 33 bodies and appended a grouped per-init appendix to
  `docs/systems/ai-dmode-inits.md` (sizes, exact `behaviour_util_setProperty` id lists, the direct
  stores: CastleChatGuard1/2 write the chat-phase byte `drone+0x13d` [M 0x14e370/0x14e3a0; readers
  `NDrone2_DSTATE_CastleChatGuard1` 0x1612e0 + `Player_Update` 0x193600], PartyGirlLooker writes
  the head-tracking weight `drone+0x35c = 2` [M 0x14e490; sole reader `DoHeadTrackingBehaviour`
  0x147578], Zoe is the family's only 0-write [prop 0x21 = 0], DeleteMe/Ninja/Bot have empty 8-byte
  bodies). Groupings: 6 pure `Defaults` tail-calls, 6 `Defaults`+extras, 15 standalone prop lists,
  2 `Defaults`+store, 3 empty.
- New test `crates/nf-formats/tests/dmode_row_table.rs` [D]: reads the disc `DroneModeSettings`
  0x29ad40 (35 rows × 12 B) out of the ISO's ACTION.ELF and pins every row's mode byte, team byte,
  resting alertness word and init-fn pointer. Finding while writing it: **`Defaults` 0x14d528 is
  not a row fn** — the 32 non-NULL row pointers are exactly the other family members; Defaults is
  reached only through row 0's `Normal` tail-call [D table + M 0x14d958].
- Reachability (from the merged `ai-dmode-inits` work, unchanged): every shipped drone placement
  takes parameter 5 = 100 → the NEW path (`DoModeSettingsNEW` 0x14c600), which never calls any row
  fn; only row 0 runs (coder event + MP bots) → `Normal` → `Defaults` [D census + M]. The port's
  whole DMODE surface stays `nf-game/src/drone.rs` `init_properties` (row-0 block) + the NEW-path
  rebuild; I added the family citation there pointing at the appendix and the row test. None of
  rows 1-34 needs a port path.

### 2. The four HUD pane creators — audited, equal, pinned

- `HUD_CreateAmmoPane` 0x1a0ed8 = `HUD_CalcWidthHeight` + `HUD_CreateDefault` only [M]. The port's
  template-driven ammo pane (the worked example in `docs/fields/hud-panes.md`) is that result; the
  create pointer is already pinned [D `hud_pane_census.rs`].
- `HUD_CreateHealthPane` 0x1a0f40: Calc+Default, then a 4-iteration loop over sprites 0x14..0x17
  writing x=0, y=0, w=0x200 (512), h=0x1c0 (448), then `ThirdIconTimer = 0; CrouchIconTimer = 0`
  [M 0x1a0f40:15-28]. Sprites 20-23 are the damage-glow quads and their `HealthSprInfo` templates
  ALREADY carry exactly those values [D decode; new test `hud_pane_creators.rs`] — so the loop is
  idempotent and the port's full-screen glow quads (`Anchor::Stretch`, SW×SH, nf-view/src/hud.rs)
  reproduce create + template without running the fn. The two timers are boot-zero BSS [L
  ee_start snapshots] that create only re-zeroes — no port equivalent needed. Documented in
  `docs/fields/hud-panes.md` ("The four create fns" section).
- `HUD_CreateMissionStatusPane` 0x1a1028 / ObjectiveStatus 0x1a1070 / InfoStatus 0x1a10b8 /
  PickupStatus 0x1a1140: `HUD_CreateDefault` + three secondary-slot writes — [1]=0, [2]= the
  TXTMSG_TYPE (3/2/1/6), [3]= 0x20 mission / 0x10 the other three [M :8-10 each]. This upgrades
  the doc's [S] on slot [2] to writer-proven [M] (hud-panes.md edited in place). The info/pickup
  pair's MP branch (`MPSettings+0x180 != 0` → `HUD_FixMP` + sprite y += 0x48) is MP-only [M
  0x1a10b8:11-14, 0x1a1140:11-14]; the port's SP path never sees it.

### 3. `Text_SendMessage` 0x223600 — proven unreachable in the shipped game

- Body read [M 0x223600]: arms 0x14 (`Script_ExitLoop`), 0x16 (`Script_Play`), 0x21
  (`Script_Select` into one of 7 slots at +0x7c), 0x2b (byte +0x7a = a), 0x39 (identity), 0x3b,
  0x3d, 0x47 (free-all destroy chain), 0x53 (flag clears + `+0x76 = (short)f32[+0x9a8] + 5` +
  notify parent), default → `Label_SendMessage(control+0x94)` (the embedded label).
- Dispatch: `__Menu_SendMessage` 0x1f1c18 switches on `control+0x7b` (the type byte), case 0xc →
  `Text_SendMessage` [M; the C shows it, and capstone finds `jal 0x223600` at 0x1f1d8c]. The other
  two call sites are `TextBox_Update` 0x2237c8 (0x223ddc, 0x223eec), both self-sends of msg 0x53
  [M].
- Constructor census: the only `Text_Init` 0x222b80 call in the whole ELF is
  `Manager_SendMessage` 0x21c858's case 0xc (jal at 0x21cad4) [M + disassembly scan]. So a
  textbox exists only if a menu def carries a type-0xc control.
- New test `crates/nf-formats/tests/textbox_msg_census.rs` [D]: walks every 07*.bin archive on the
  disc, parses every kind-8 def (entry hash + the de-folded 0x8000000N PAGE-hash family, keeping
  the most-pages parse — the calltree_census pattern), 34 defs / 283 pages / ~5000 controls: **zero
  type-0xc controls anywhere**. Therefore `Text_Init`, `Text_SendMessage` and `TextBox_Update` are
  unreachable in the shipped game, on-foot or otherwise. The port needs nothing for them; its
  generic `control_msg` (nf-formats/src/menudef.rs: 0x1a/0x30/0x39/0x14) covers every widget type
  the shipped data builds. (Sister census: `hud_pane_census.rs::menu_def_textbox_census`, the
  front-end def alone, prints the same empty set.)
- Format note learned on the way: `MenuDef::parse`'s page filter keys on the PAGE tag's own file
  hash (0x8000000N), not the entry-name hash (0x0800000N) — parsing with the entry hash yields
  0 pages for every def.

## Tests added (all pass)

- `crates/nf-formats/tests/dmode_row_table.rs` — 35 disc rows pinned (mode/team/alert/fn) + the
  32-fn symbol-set equality. First run failed on my own FNS list (Defaults wrongly included);
  fixed — that correction is itself in the appendix.
- `crates/nf-formats/tests/hud_pane_creators.rs` — glow templates carry the create-loop's
  (0, 0, 512, 448) values; ammo/health create pointers in the disc defs.
- `crates/nf-formats/tests/textbox_msg_census.rs` — the zero-0xc census + walk sanity.

## What I ran

- The three new test files, `cargo test --offline --release -p nf-formats --test <name>` each:
  4 tests, all green.
- The touched crate's full suite in the foreground: `cargo test --offline --release -j2
  -p nf-formats -- --test-threads=1` (see the WORKLOG entry for the total line).
- The full four-crate fast runner (`tools/test/suite.py`) could not run here: this sandbox kills
  every background job between tool calls (three nohup attempts, including a dedicated env
  launcher, all died silently with empty logs — the same reason the driver says to run downloads
  in the foreground). Foreground chunks of nf-game/nf-view were time-boxed out by the session
  deadline; nothing I changed touches nf-game behaviour beyond a comment, and nf-formats (the
  crate all three tests live in) ran whole. Honest gap, not a pass claim.
- No existing test was weakened; no decompile-backed assertion was changed.

## What's left (for the next message, if anything)

- Nothing blocking. Optional follow-ups: model the `TextBox_Update` 0x53 reveal-tick if a future
  task ever makes type-0xc controls constructible (they are not, per the census); the MP branch of
  the info/pickup status creates if the port's MP HUD ever consumes those BSS tables.
- Re-run the full suite on a machine where background jobs survive (the next worker can start
  `tools/test/suite.py` normally).
- Live checks for the main agent: none required — every claim above is [M]/[D].

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session, so there are no numbers to copy. (No estimate given, per instructions.)
