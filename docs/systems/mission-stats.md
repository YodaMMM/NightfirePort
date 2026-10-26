# The Mission Stats page (`P_NFSTATS_Handler` 0x209938)

The page the results screen opens: a table **Category / Current / Target / TOTAL** over the same
`ScoringTable` entry the results page commits. The port's model is `crates/nf-game/src/stats.rs`
(`stats_rows()`); the flow lives in `crates/nf-view/src/menu.rs` (`Page::Stats`). Evidence tags:
[M] code + function + address, [D] disc, [L] EE snapshot, [S] suggested.

## The page id and the flow

| step | evidence |
|---|---|
| Page id **0x40000037** → `P_NFSTATS_Handler` [M `Handler_HandleMessage` 0x2029e0:448]. | 0x40000036 = results, 0x40000038 = bonus [M 0x2029e0:458-467]. |
| The results page opens stats on the side action: case 0x5e → `Manager_SendMessage(0x44, 0x40000037, 0)` [M `P_NFRESULTS_Handler` 0x208e78:225-226, 229-230]. The bonus page does the same [M `P_NFBONUS_Handler` 0x209670:62-66]. | msg 0x5e is the **○** action, msg 0x5d the **□** action: `Button_Update` 0x1ef948 sends msg 0x4b on the action-0x1f edge (✕ confirm), 0x5d on the 0x20 edge (□), 0x5e on the 0x21 edge (○) [M 0x1ef948:74-91, 105-127; the pad bits: action 0x20 ← buttons & 0x80 = □, 0x21 ← & 0x20 = ○ [M `psiInput_MapInputs` 0x10cd80:411-412 default arm — style-independent; [L docs/systems/input.md §2/§3, WORKLOG 0006]]. The page's own hint is the game's text "~A Continue  ~X Stats  ~Y Dossier" [D USATxt.dat Txt 0x2d0] — ~X = ○, ~Y = □ (specialchar 0x2dd8d8), which agrees. |
| Results case 0x5d (the □ action — not △; △ is action 0x22, the manager's pop [M 0x1ef948:80-83; 0x21c3d8:77-105]) → dossier 0x4000002b, arg 0 [M 0x208e78:222-224]. | |
| `P_NFSTATS_Handler` handles ONLY the build message 0x4c; it has no key arms [M 0x209938:31-124]. △ off the page is the menu-def script layer (`Page_SendMessage` 0x21ec68 case 0x5d → `Script_PlayDefault`), whose target lives in the front-end archive (not parsed). The port routes △ back to the results page [S] — live check in the report. | |

## The data: `PlrStat_GetScore(0)` then the rows

The page enter runs `PlrStat_GetScore(0)` — param 0 = **commit** [M 0x209938:36-38; the commit
block 0x1984b0:301-315] — and returns the `ScoringTable` entry for `Mission_BaseMapHCode()`
[0x1984b0:35-49]. If there is no entry (`score_table == 0`) the page draws nothing [M :38-40].
The port: `Progress::get_score(base_map, ms, difficulty, true)` then reads the entry — wrapped by
`nf_game::stats::stats_rows()`.

The entry (60 B, `docs/fields/player-stats.md`): +0x18 rows (9 × 20 B), +0x1c elapsed (1/100 s),
+0x20 the par seconds, +0x24 difficulty multiplier, +0x28 difficulty bonus, +0x2c subtotal,
+0x30 best, +0x34 total, +0x38 byte = the stealth-rows flag (1 unless the hash is 0x09xxxxxx)
[M 0x1984b0:49-56].

### The rows, in the handler's order [M 0x209938:60-116]

Each `Menu_AddRow` 0x201c10 formats `label|current|target|SeparateNumber(score)`; flag 6 indents
two spaces (the three Opponents sub-rows); flag 7 formats current/target through
`Timer_Seconds2String` [M 0x201c10:99-121]. `Menu_AddRowPercentage` 0x201df0 appends `%` and
shows the target as `----` when it is −1 [M 0x201df0:78-88].

| label | row | current | target | TOTAL |
|---|---|---|---|---|
| 0x165 Bond Moves | 0 | rows[0].value | rows[0].count | sep(rows[0].raw) |
| 0x15b Dispatched (indent) | 1 | rows[1].value | rows[1].count | sep(rows[1].raw) |
| 0x15c Subdued (indent) | 2 | — shown only when the +0x38 flag ≠ 0 [M 0x209938:68] | | |
| 0x16e Surrendered (indent) | 3 | idem | | |
| 0x15d Stealth Rating | 4 | pct(rows[4].ratio) | `----` (−1 passed) | sep(rows[4].raw) |
| 0x15e Accuracy Rating | 5 | pct(rows[5].ratio) | pct(rows[5].count) | sep(rows[5].raw) |
| 0x15f Health Remaining | 6 | pct(rows[6].ratio) | `100%` (literal) | sep(rows[6].raw) |
| 0x160 Time (flag 7) | 7 | timer(rows[7].value) | timer(rows[7].count × 100) | sep(rows[7].raw) |
| 0x161 007 Bonus | 8 | rows[8].value | rows[8].count | sep(rows[8].raw) |
| 0x162 Subtotal | — | sep(entry+0x2c) | | |
| 0x164 Difficulty Bonus | — | sep(entry+0x28) | | |
| 0x163 TOTAL | — | sep(entry+0x34) | | |

The pct current is `fptoui(ratio × 100 + 0.5)` = round-half-up [M 0x209938:92/100/106]; the row
TOTALs are `fptoui(raw)` = trunc [M :62/71/…]. The header row binds 0x1000204 (Category) and
fills cells 1/2/3 with 0x10001e1 (Current), 0x10001e2 (Target), 0x163 (TOTAL reused) [M
0x209938:44-56; the same quartet the pause SCORE tab binds, 0x204408:447-498]. Two label-only
rows ride between the data rows: 0x15a "Opponents" (msg 0x10, after Bond Moves [M :66-68]) and
the blank spacer (the static string 0x30d2d8, before Subtotal [M :117]).

## The formatters

- `SeparateNumber(u32, buf)` 0x1dec78: `<1e3` → `%d`; `<1e6` → `%d,%.3d`; else `%d,%.3d,%.3d`
  (millions group printed unpadded) [M the format strings 0x30d090/0x30d098].
- `Timer_Seconds2String(cs, 0)` 0x1deaa8: seconds = `fptoui(cs × 0.01 + 0.5)` (round-half-up);
  hours = (s/3600)%60; hours ≠ 0 → the TWO-field `%.2d:%.2d` with (hours, minutes) [M :27-31;
  ACTION.ELF disasm 0x1deb8c-0x1deba4 — the default arm's sprintf rides $a2 = hours, $a3 =
  minutes, the same fmt 0x305940 case 2 uses]; else `%.2d:%.2d` mm:ss [M case 2]. Negative →
  all zero [M]. The Time row feeds elapsed in 1/100 s and par × 100 [M 0x209938:103-108].

## The formulas (already `Progress::get_score`, corrected here)

- Normal rows (0-3, 5): `raw = ceil₁ₑ₋₅(max × min(count, value) / count)`, ratio =
  round½↑(min/count × 100) × 0.01 clamped [0,1] [M 0x1984b0:171-199 — the `1e-05` bump is the
  ceil]. **The port truncated instead of ceiling — fixed** (the golden's Dispatched
  2/30 must show 3,334, not 3,333.33→3,333).
- Row 4 stealth: detections over par → `raw = ceil₁ₑ₋₅(max(1 − (det − par) × penalty, 0) × max)`
  (penalty = entry+0x14, code default 0.03 [M 0x1984b0:201-224]); within par → raw = max,
  ratio = 1.0 [M :225-233]. Same ceil fix.
- Row 7 time: **`raw = ceil₁ₑ₋₅(max × par / max(elapsed_s, 1))`** — under par the score RISES
  above max [M 0x1984b0:118-170: under-par branch multiplies max × par and divides by
  max(elapsed, 1); the over-par branch divides by elapsed/par — the same expression]. The
  capture's 01:19 / 09:00 → 170,887 = ceil(25000 × 540 / 79) proves it. **The port capped the
  under-par arm at max — fixed.**
- Row 8 bonus pickups: all-or-nothing (count ≤ value → max, ratio 1) [M 0x1984b0:255-268].
- `base = Σ raw` (f32 accumulate, trunc on store) [M :288-293]; difficulty multiplier
  `fptoui` of `ScoringTableDifMul` 0x2bc910: difficulty 1 → [0] = 0.0, 2 → [1] = 1.0, else →
  [2] = 2.0 [M :294-300]; bonus = base × mult [M integer multiply]; TOTAL = base + bonus, and
  with commit: best = max(best, TOTAL) [M :301-315]. Status ≠ 6 zeroes ONLY the per-row points
  (+0x10 — the 0x14-stride loop) and the entry's dif/bonus/subtotal/total plus DoneBetter
  [M :318-333] — the tallies (+0x08) and ratios (+0x0c) survive, so the stats page on a non-6
  status still shows the run's current cells and percentages (the handler's reads at
  0x209938:70/84/101) with every TOTAL cell zeroed.

## The port's surface

- `nf_game::stats::{separate_number, timer_string, StatsRow, stats_rows}` — `stats_rows(&mut
  Progress, base_map, ms, difficulty, Option<&Lang>) -> Option<Vec<StatsRow>>` mirrors the
  handler (order, gating, formats) and returns `None` when the hash is not a `ScoringTable`
  entry (the game's `score_table == 0` arm).
- `FrontEnd::stats_enter(...)` builds the rows into `FrontEnd::mission_stats` when the page
  opens; `Page::Stats` (0x40000037) is reachable from the results page's side action.
- `nf_game::stats::STATS_HEADER` = (0x1000204, 0x10001e1, 0x10001e2, 0x163) for the draw code.

## Tests

`crates/nf-game/tests/mission_stats_rows.rs` drives a headless mission end through the real
`log_*` path and asserts every cell (ids + strings); the second test is the PCSX2 capture
(07000005, Agent: Time 01:19 / 09:00 / 170,887; Stealth 100% / ---- / 25,000; Accuracy 100% /
100% / 50,000; Health 100% / 100% / 50,000; Subtotal = TOTAL 295,887) as numbers only. The
gate test asserts the status ≠ 6 zeroing (a removed gate fails the test). `crates/nf-view/
tests/mission_stats_flow.rs` asserts the page flow (side action opens, △ returns, rows stored).
