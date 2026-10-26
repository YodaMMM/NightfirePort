# level-07-audit — the 07000007 behaviour audit

Branch `s/level-07-audit` = origin/master (b312d959) + this task's commits.

## What's done

1. **The audit table** — `docs/levels/07000007-audit.md`: every placement kind on the level's main
   map (513 records [D fresh parse], all 23 kinds, one row per placement for every kind whose game
   path differs, grouped only where it is identical) with the creator in
   `parsemap_create_dynamic_objects` 0x1d04f0 / the static path (`parsemap_block_map_data_static`
   0x1d1998:93-120; rooms via `parseentity_fixup_entity` 0x1d00c0:122-143), the update, the
   use/damage/event handlers, and the port's create/tick/draw/use status file:line — with a GAP
   column feeding §4.
2. **The scripts** — full stream decode of all three SP scripts, byte-exact against
   `nf-formats::nis`: 0x0600007A (the urn, len 23, 13 streams, no camera/hand-back/channel ops),
   0x06000072 (Drake's reveal, len 3000 — camera spline, 13 anim streams incl. the tux-Bond skin
   0500000D, drone freeze, hand-back marker at the balcony, sounds/sprites), 0x06000073 (the
   control-taking scene, len 4326 — camera, the ONLY script channel write [130←0] at the length
   tick, hand-back inside the control room, fade-in). Data locations pinned [D]: the urn script
   ships in the level archive 07000007.bin, the reveal in **07900007.bin** (corrects the old
   "pack 07800007 holds these scripts"), the control scene in 07800007.bin. The five shared
   castle scripts in the level archive are loaded-into-lib and never played [D + the Script_Load
   caller set]. All 17 event cases and every command op are ported (nis.rs) — checked op by op.
3. **The mission record** — decoded from the snapshot [L ee_start_05.bin @0x2a43a0, record 2]:
   objectives 0x4000002/0x4000003 with complete channels 0x48/0x49, reveals 0x47/0x48, no per-
   objective fail labels; the fail machinery (reasons → 0x4000033 + ch 99 [M 0x13e560]) matches
   the port byte-exact (c07000007.rs). Corrected the level doc: the 0x400005d/e labels are the
   pause HINTS, not titles.
4. **GAP-2 fixed** — the cut-scene mission-timer pause. The game's event-4 arm is
   `Player_Disable` + `PlarStat_LogTimerPause(0)` [M 0x1d94c0:118-121], the accumulator
   `PlarStat_LogUpdateElapsedTime` runs only while the pause byte is 0 [M 0x1995a0:9-14], and
   `Script_Free` unpauses [M 0x1da880:44-48]. The port never paused the clock — both of this
   level's cut-scenes kept it counting. Fix: scene.rs's `PlayerDisable`/`PlayerEnable` arms now
   call `progress.stats[0].log_timer_pause()` / `log_timer_unpause(now)`.
5. **Doc corrections** — docs/levels/07000007.md: SP_Hit bit 2 IS ported (+ `urn_break_live`);
   the urn script contains NO event 0x0A (the "the urn SP uses Break_DoBreak" claim was wrong);
   the pack locations; the title/hint split; the record address 0x2a43a0.

## How it was verified

- New live test `cutscene_timer_pause_live` (nf-view): the real walk to TouchOnce #412 → SP #420's
  event 4 → the clock FROZEN across 300+ ticks of the unskipped 4326-tick window → the hand-back
  restarts it → Bond back at the script's marker (91.52, 43.85, −31.09). **Green, 7.3 s.**
- The audit's [M] chain was triple-sourced: the decomp reads (parsemap dispatch, the drone
  collision/talk/enable functions, the mission record readers), the disc decode (placements
  re-parsed from block 0x1A, the scripts from the ISO archives), and the port grep (every claim
  about "the port does X" carries a file:line).
- What I ran before pushing: `cargo test --offline --release -p nf-view --test
  cutscene_timer_pause_live` (the touched crate's new test — green); then the suite chunks: the
  build warm (809 binaries, `--no-run` EXIT=0 over the four crates) and a 560-s foreground chunk
  of `tools/test/suite.py` that completed **127 binaries with ZERO failures** (../suite.log)
  before the window closed mid-chunk. The sandbox kills background jobs and the tool cap is
  10 min, so the full 13-min run needs two `--only` halves or one uninterrupted foreground run.
- **The remaining suite chunks are the first job of the next window**: run
  `NF_ISO=<iso> python3 tools/test/suite.py` to completion on this tree and put the total line
  here. No total line is claimed in this report.

## What's left

1. The full suite on the final tree (the chunk above).
2. The audit's gap list as its own tasks: **GAP-1** guest/Bond cylinder collision (the game blocks
   Bond [M 0x140e90:66-116 + 0x149ec8:10-37 + 0x14b190 <2.0 + 0x1949a8:168-175; drone-side
   0x13fff0:131-146; NIS freeze clears solid 0x140968:62-64]; the port has nothing — the biggest
   behavioural delta); **GAP-3** the 0x3b draw linkage (sim exact, nf-view never re-reads it —
   #495 never appears, deleted #496 keeps drawing); **GAP-4** guest idle faces; **GAP-5** the tux
   skin 0x050000d (no third-person body in the port; both cut-scene scripts animate it);
   **GAP-6** DoBreak's scan scope (latent on 07 — no event 0x0A in its scripts); GAP-7/GAP-8 are
   notes.
3. The live checks for the main agent are listed in the audit's §5 (the 2.0 collision gate, the
   rope-barrier gate states, three `--bond` render spots for the tux/faces/barriers).

## Usage

My own session's token/context/cost counters are not visible to me (no environment readout this
session). The research subagents reported their own totals: 225,768 + 4,505 (port coverage);
247,808 + 7,647 (script decode); 113,153 + 5,393 (mission records); 42,030 + 5,348 (dispatch
table); 56,622 + 5,915 (collision/talk/timer/freeze). Everything else: not visible to me.
