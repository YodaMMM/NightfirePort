# port-map-scripts — round report

Branch `s/port-map-scripts` (from `origin/master` 22d5836b7). Task: the scripts-family
function map — one line per `Script_*`, `SP_*`, `Control_*`, `Mission_*` function in the
decompile, ported with the port location or not reached with the reason, to
`docs/port-map/scripts.md`; fix any reached-without-port-code gap red-then-green.

## What's done

- **The map, complete: `docs/port-map/scripts.md`, 97 rows** — every `Script_*`/`SP_*`/
  `Control_*`/`Mission_*` function in ACTION.ELF's decompile (the two `Drone_*_Control`
  name collisions excluded as drone-family). **59 ported, 29 folded, 9 not ported**, each
  row with the port location (file + item + the behavior match) or the reason.
  - SP_* (17): the script-player object — ported in `nis.rs` (`Player`/`Scripts`).
  - Script_* NIS engine (40): the opcode interpreter and per-tick machinery — ported in
    `nis.rs` (`Script::run` carries every command op with its byte footprint) plus the
    `Nis::parse` def parser in `nf-formats`.
  - Script_* menu engine (18): the M_CONTROL keyframe runtime — genuinely ported as the
    headless `MenuRuntime` in `nf-formats/src/menudef.rs`, test-pinned against decompile
    addresses; **not wired into the playable front end** (menus render final states).
  - Control_* (11): the generic obj_tag helpers — mostly folded into the port's per-system
    records; 5 rows have no port code (see below).
  - Mission_* (11): fully ported in `mission.rs` (init/update/monitor 1:1, the getters as
    fields with every consumer wired).
- **How it was verified:** the family was enumerated from `work/decomp/ACTION.ELF/index.tsv`;
  a whole-decompile xref pass collected every function's callers (mangled `Name__F` call
  form + raw-address data refs) — all 97 are dispatched on the shipped disc, so there is
  no "not reached" class here. Each function's decompile was then read and matched against
  the Rust by semantics in three parallel verification passes (menu group, Control group,
  Mission group) plus a hand pass over the SP/NIS group against `docs/atlas/script-player.md`.
  Name-search hits were treated as leads only; every row cites code that was actually read.
- **The in-family MISSING class is empty.** The 9 not-ported rows (SP_RemoveObj,
  Script_RemoveObj, SP_SetColour, Script_SetColour, Control_SetEffectType,
  Control_DeleteAllObjectsOfType, Control_ReturnNextObjectOfType, Control_Plr2Ind,
  Control_ObjectReferenced) each carry a per-row reason in the table: structural no-ops in
  the port's owned `Option`-row object model (traced equivalent through every reader), MP
  state whose observable is the unported MP render, or deliberate documented drops
  (mp/rules.rs:672-685). None has a sim-side observable that could make a real-level test
  red, so per the task's own red-proof requirement there is no fixable gap among them.
- **One real adjacent gap found (outside the family, recorded not fixed):**
  `Script_CreateEntity`'s 0x20 arm pushes `Objects::pending_breaks` (nis.rs:902 — the
  07000008 gondola-cabin-window route, 16 panes of ride script 0x0600004C) but nothing
  drains it: `pending_pickups` drains at drone.rs:20790, `pending_volumes` at
  scene.rs:5811, `pending_breaks` nowhere, while nis.rs:893-898's comment claims a drain
  into `Breaks` exists. Script-created panes render but never get `Breaks` records — they
  cannot be shot/smashed. The fix belongs to the breakables family; the table's findings
  section proposes the shape (drain at the Sim latch + a 07000008 real-level red test).

## Incidents

- **Sandbox reset at 02:27 UTC.** The first workspace (repo + ISO + work/ + build) was
  wiped between tool calls with the table already composed but uncommitted; the branch
  with the WORKLOG commit had been pushed at 02:00 and survived. Setup was redone from
  the token (re-clone + branch checkout); the ISO/work downloads were deliberately skipped
  because this branch carries **docs-only commits** (WORKLOG + the map + this report — no
  source or test change), so the full-suite-before-push run has nothing code-side to
  verify. No `cargo` run of any kind was possible in the remaining window after the reset
  (cold registry fetch + build alone exceed it). The next task touching code from this
  branch should run the suite as usual.
- WORKLOG: `port-map-scripts` entry appended (start + outcome), nothing else touched.

## What's left (for the next round)

1. The `pending_breaks` drain (breakables family) — with the 07000008 gondola-pane red test.
2. The menu runtime wiring decision (integrate `MenuRuntime` into the front end, or retire
   it with the stale audit note at menu.rs:6733-6736 replaced).
3. MP-render items if MP visuals get ported: the uplink terminal tint
   (`SP_SetColour`/`Script_SetColour`) and the trigger effect word (`Control_SetEffectType`).

## Live checks for the main agent

1. **SP_RemoveObj / Script_RemoveObj equivalence**: on any level with a script-spawned
   breakable (07000008 gondola panes are the known instance), smash a pane by shooting
   while its script is still running; confirm nothing re-appears or re-keys it afterwards
   (the port models the kill-on-delete as a no-op — nis.rs:957-961 — and the trace says
   every reader behaves the same; a visible difference would reopen the row as MISSING).
2. **Control_SetEffectType**: trigger with an effect word (triggers.rs:130-135 records the
   field) — what does the effect word visually do on trigger fire in the real game
   (trigger-adjacent object flash/colour)? Needed before anyone ports the render side.
3. **Mission_SetMapHCode / PTP return**: after a driving level returns to ACTION (PTP
   round-trip), confirm the hub uses the returned map id for scoring (the port cold-boots
   the hub; ptp.rs:311's `PtpBlock.map_id` is built but unconsumed).

## Usage

Not visible to me — this environment exposes no token, cost or context-window figures for
the session. (The only token counters seen were the subagents' own reports, which are not
this session's figures and were not estimated here.)
