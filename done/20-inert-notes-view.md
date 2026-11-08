# Port notes that say not ported: nf-view sim code

The same grep over crates/nf-view/src/scene.rs and lib.rs, gameplay arms only. For each note: is the arm reachable on any on-foot level? Close with a count, or port it.

Read the game code first, count on every level with `Level::parse` over `nf_view::dev::LEVELS`, and test on the real level, red on the old code. A count of zero on every level closes an item with that count as proof. Do not trust a port comment that says inert. Values measured at load can differ from the same global in play (FRAME_RATE reads 30 while a level builds, 60 after). Branch from origin/master, push to `s/inert-notes-view`, report in docs/reports/inert-notes-view.md. No driving levels. Draw work: leave a stub in the hand-off list and say so.

---
claimed-by: sandbox1 at 2026-11-08T00:28Z

---
## Report (done, 2026-11-08T01:07Z)


# inert-notes-view — the port notes that say "not ported": nf-view sim code

The grep over `crates/nf-view/src/scene.rs` and `lib.rs` for `not ported` / `inert` / `unported-*`
notes, gameplay arms only. For each: is the arm reachable on any on-foot level? A count of zero
closes it; a non-zero count demands a port or a hand-off. No driving levels; the draw-side notes
are hand-off stubs.

## The inventory (the subagent's read of both files, spot-verified)

30 occurrences in scene.rs, 10 in lib.rs. Gameplay-relevant open items:

| Note | Claim | Verdict |
|---|---|---|
| scene.rs L10243 (fix-safe08b block) | `build_LinkToRoom` 0x1c9680 links by `build_FindCel` 0x1c8cb0's raycast tie-break, "which the port has not ported (the next session's item, with the live check)" | **STALE — already ported** |
| scene.rs L6332 (objective drain) | `Text_AddMsg` 0x1b0c30's wrap + life scaling "not ported" | **PORTED this branch** (red first) |
| scene.rs L5530 (SS/gunimp use pick) | the game's glist refine `Intersect_PointGeom` 0x1e92d8 "is not ported — a mesh veto of a sphere hit is left to the live checks" | **REACHABLE (7 objects) — hand-off** |
| scene.rs L5918 (`Sim::guard_breath` doc) | Bond's own call "needs BLData+0x950 ≠ 0, a non-first-person camera mode, which the port's on-foot play does not enter outside the cut-scenes BLData+0x94b already excludes; it is not wired" | closed as written (unreachability argument, [M 0x193600:370-377]) |
| scene.rs L3862 (`Sim::ptp_handover` doc) | "The DRIVING program itself is not ported (a separate ELF; only its exit contract is [M psiLaunchDriving 0x120af0:12])" | open **by design** — driving is out of scope (the task letter: no driving levels) |
| lib.rs L6531 (KeyK debug arm) | "drones waiting for one (WaitSwitch) start (**triggers are not ported**)" | **STALE** — the aim-ray class-0x1D LOAD_LEVEL trigger arm is ported at scene.rs L5672 with its stand-in pick; comment corrected |
| scene.rs L909 (`Msg::image`) | "no decompiled caller passes a non-zero id (the 36-call-site sweep) — the arm is inert until a future caller sources an id" | already closed with its proof (count-0 style) |
| scene.rs L6370 / L13203 / L172, lib.rs L478 | historical / provenance notes ("missing glue before 2026-10-15", the game's own empty stub mirrored, render-build provenance, deci2 mapping) | already closed / docs |
| scene.rs L2450 (Script_SetColour tint), L11537 (blast billboard spin), L12132 (corona alpha G-033), L861-868/10573/15314-15388 (shear/swing view-matrix concat halves) | draw-side gaps | **hand-off stubs** (no draw changes made, per the letter) |

## The census (`crates/nf-view/tests/inert_notes_census_view.rs`)

All 34 on-foot levels (`nf_view::dev::LEVELS` [D dev.rs:830]) through a real `nf_view::load`:

- **962 of 4767 generic cels have ≥2 room candidates** through `link_owner_room`'s radius+bbox
  gates (02×94, 05×65, 08×62, 0b×83, 11×88, 0a×89, 12×67, 01×26, 03×39, 06×37, 0c×52, 0d×11,
  09×9, 16×8, 13×5, 14×5, 07×2, 4a×…) — the raycast tie-break's real population. 4650 chains
  link.
- **7 SS pick objects** (state ≠ 0, mask & 4): 07000002×3, 0700000a×1, 0700000b×1, 07000011×2.
- **1 objective label wraps at 480** across every level: 07000007's 0x0400_0002 ("Rendezvous
  with undercover agents and maintain cover as party guest" — the [L] PCSX2 case fix-hud-parity
  cites).

## What was ported (red first)

**`Text_AddMsg` 0x1b0c30's wrap + per-line life scaling** — the game queues ONE node PER WRAPPED
LINE ([M 0x1b0c30:189-269: the wrap loop re-enters at `LAB_001b0e80` while the wrap cursor is
non-NULL], each node's life the scaled store [M :228-239]: `round(ticks · strlen(line) /
strlen(orig))`, ticks < 1 keeping the raw value at :230-234):

- Red: `crates/nf-view/tests/inert_notes_objective_parts_live.rs` (07000007's own announce
  through the real start scripts) — on the old code the queue held ONE full-text entry with the
  raw 300 ticks: `got [("Rendezvous with undercover agents and maintain cover as party guest",
  300.0)]`, left 1 ≠ right 2.
- Fix: the drain (scene.rs, `Event::Objective` arm) now wraps with `crate::menu::font_word_wrap`
  (the codebase's `Font_WordWrapString` 0x1ef178 port) at TextMsgFormats[2]'s 480 [L
  0x2bf108+0x14 = 0x1e0] and pushes one `Msg` per line with the scaled life. The Sim carries the
  font (`Sim::font`, a clone of the draw side's `HudTables::font` — one new struct field on its
  own line, set in the single `Sim { }` literal). strlen is char-count (one char per CP1252
  byte — the game's strlen; the old draw-side share used UTF-8 byte len).
- The draw side keeps `line_part` (hud.rs): a drained entry IS one line, so it returns the whole
  text with the fade by the entry's own time; direct queue pushes (the tests' synthetic
  full-text entries) still re-slice there. `drop_objective_lines`, the SFX scheduling and the
  flash-window rules are unchanged — each part is its own display, which is the game's per-node
  case-2/case-3 behavior [M 0x19c310:148-164].
- Green: the red test passes — the queue holds one node per wrapped line with the scaled lives
  (the 300-tick split across the rendezvous text's two lines), node texts equal the wrap parts
  exactly.

### Tests (every binary that touches the changed code; all green)

`grep -l` over the objective/msg surface + the draw: **52 passed / 0 failed** across 10
binaries — `fix_objective_messages_live` (5), `fix_objective_banner_live` (3),
`objective_banner_timing_live` (3), `objective_opening_live` (3), `objectives_0b_start_live`
(3), `hud_level_start_live` (7), `level06_start_subtitles_live` (6), `spurious_complete_16_live`
(9), `fix_party_mission_live` (4), `walkreal_07000004_live` (10, includes the three real-damage
full walks). Two assertions updated to the per-part semantics, each citing [M 0x1b0c30:189-269]:
`fix_objective_messages_live::direct_07000007_opens_with_its_own_objective` (first announce =
first wrapped line) and `fix_objective_banner_live::every_objective_banner_shows_the_objectives_
own_line` (every part of the bound line shows). The full `cargo test` suite did not fit the
sandbox window; the shared `crates/nf-view/tests/common/` helpers are untouched (no opt-in rule
needed).

## What is left / hand-offs

1. **`Intersect_PointGeom` 0x1e92d8's mesh veto** (scene.rs L5530): the pick's population is 7
   objects on 4 levels. The veto needs the object-glist triangle walk (COLLDATA/COLLBOX tree +
   the 1/16384 vertex blocks, `vecutil_Dist2Tri_PointGeom`) behind `Collide_PickObj`. Until then
   the port's sphere pick can over-select and orders by sphere distance, not mesh distance.
   **Live checks for the main agent**: at the 07000002 SS objects (the census's 3), put Bond's
   use-pick sphere centre across each object's sphere surface and record whether the real game
   activates (a mesh veto rejecting a sphere-only hit shows as no-activation where the port
   activates); access type read, expect `Player_Activate` 0x1a5170's class walk to skip the
   vetoed object.
2. **The draw-side gaps** (unchanged, per the letter — no draw work): the `Script_SetColour`
   character tint (L2450), the blast billboard's per-def spin (L11537), the corona alpha byte
   G-033 (L12132), and the shear/swing view-matrix concat (the sim half computes the phases; the
   Camera has no roll/shear slot — L15314/L15329).
3. **Driving**: the DRIVING.ELF port remains the exit-contract stub (by design; the handover
   `driving_result` stays the dev choice).
4. The `Sim::cel_at` ambient tie-break remains the documented smallest-volume *deviation*
   (fix-render2's [L] re-measurement stands; it is not one of this task's notes).

## Usage

not visible to me (this sandbox shows no token/context-window/cost figures for the session).
