# retune-nb-08-20 — the 08 whole-mission 00 Agent real-damage walk, round 20

## What's done

- Branch `s/retune-nb-08-20` from `origin/master` (b60b1ff79, stream batch 3) with
  `s/retune-nb-08-19` merged first (08-19 was not on master; the merge was clean).
  Setup from scratch (the sandbox reset): clone, work-slim + the verified 369823203-byte ISO
  archive (py7zr; no root, no 7z), rustc 1.100 (rustup), the extracted libasound2-dev +
  runtime libasound2t64 debs into a user prefix (the linker pointed at it via
  `~/.cargo/config.toml`, a sandbox file, never committed), `cargo fetch`, the nf-queue
  worktree.
- **THE MERGED-TREE BASELINE RE-MEASURED FIRST, as the round's task required.** Master's
  stream batch 3 (guards' rounds hit Bond's body boxes, guards aim at his body, the
  one-sided wall push, room walk gate, guided rocket hit size, burst latch) re-dealt the
  walk long before the machine fight:
  1. The batch-3 running jump (master's d1/d2 balcony crossing, added because the
     one-sided push broke the old rail climb) fires before the d3 east-deck-head leg,
     lands Bond ON the deck (143.86,46.05,-90.48), and the shipped lip walk + in-place
     hop then had to climb back UP 4 u — the walk wedged the 6000-tick cap at the lip
     (bond (142.56,46.13,-91.36), target (141.00,50.10,-92.50) d 1.93, tick 6540,
     health 77.0) [L the merged-tree NF_DMG_TRACE run].
  2. With the jump gated off for that leg, the hop itself failed the other way: the
     in-place hop jump is DEAD under the one-sided push — Bond reached the lip
     (140.54,50.10,-92.37) at +79 ticks and then sat ON the lip step at
     (141.406,50.275,-92.503) for the whole 300-tick hop loop (the running-jump
     comment's own law: the step top reads as walkable ground, apex y 50.27) [L the run].
- **SHIPPED FIX (1): the d3 east-deck-head crossing is the running jump itself.** The
  lip-walk + in-place-hop block (walks-1121-08-0c) is removed — on the 08-19 stream the
  two-sided push let the in-place jump clear the edge; on the batch-3 stream it does not.
  The d1/d2 sisters skip this leg before the jump call (difficulty < 3 `continue`), so
  their timelines are byte-exact by construction. The deck-head nav runs from the jump's
  deck landing.
- **RE-MEASURED BASELINE (after fix 1): the walk reaches the LOWER LEVEL EAST and dies
  t16372** at (107.62,28.28,-120.42), health 4.0 — 29 hits for 77.0 + other 23.0 (the
  running jump's landing: the summary's fall/drown/hurt-volume bucket), 11 return-fire
  fights [L the 2026-11-22 run]. The merged-tree upstream ledger: the deck fight 52.0
  (six 0xa shooters #3/#2/#14/#13/#9/#10 at 4.1-11.5 u, rows t717-1570, every fight
  KILLED), #6 -3.0, the y33 transit -18.0 (#18 skin 0x500002f weapon 0x15: -2/-6/-6 in
  8 ticks at t14767-14775 from (115.2,41.5,-101.8) d 21.5, plus #16 -4.0), then the lower
  level east unseen pair: the stand on #23 killed false and the point-blank #19 burst
  (-4.00 d 0.8) closed the walk. 13000 ticks short of the machine fight — the round-19
  machine levers are moot on this stream.
- **LEVER TRIED (1st variant) — the wide-fight ring pick most-hits-first (the ledger's
  round-17 item (2)): BYTE-IDENTICAL no-op on its own real run** (death t16372 unchanged,
  same fight order — the newest hitter was already the worst shooter at every pick; #13
  died before #2's window grew). Reverted; recorded in the ledger.
- **LEVER SHIPPED (2): the MOST-HITS-FIRST RETARGET** (opt-in thread-local
  `with_real_damage_most_hits_first` in `crates/nf-view/tests/common/damage_log.rs`; the
  00 Agent walk only — the d1/d2 sisters and every other walk byte-exact): the fight
  retarget samples EVERY tick instead of the 30-tick grid, and reads the worst shooter
  over the WHOLE shooters set, not the worst VISIBLE one (the unseen creep opens the
  line). The y33 #18 burst (14.0 inside 8 ticks) fell whole between two 30-tick grid
  samples — with the flag the switch fires mid-burst.
- **MEASURED [L the 2026-11-22 NF_DMG_TRACE run]: death t16372 -> t25783 (+9411, the
  deepest merged-tree walk).** The lower level east pair answered, the master ledge-armour
  leg scooped ~50 armour, and the walk REACHED THE D3 MACHINE-FIGHT ANCHOR ("the d3 fight
  anchor: arrived (112.09,13.56,-179.46)") before a world row (live wire / blast / hurt
  volume) closed it at (113.1,13.5,-179.1) — Bond entered the fight at 2.0 health (the
  secure-room #6's five -3.00 rows at t4886-5020: the re-dealt fight walked him into #6's
  lane at the creep; was -3.0) + the armour. 48 hits for 77.0 + other 23.0, 17 return-fire
  fights [L the run].
- The refreshed ledger and the next levers are appended to the target test's ledger
  comments (`walk_the_whole_mission_on_real_input_real_damage_00agent`): (1) the fall —
  the "other 23.0" is the running jump's landing [M Player_CollisionHandler 0x1949a8:108-131:
  the damage is fall_timer-60 with the apex reset; the jump apexes ~51 over the 46.05 deck];
  (2) the secure-room #6 re-deal — the unseen switch needs the round-19 #17 range-gate law;
  (3) the machine fight from the anchor at ~2.0 health + ~50 armour — the 08-19 anchor
  shape re-pinned on the batch-3 stream (the world-row death at the anchor is the first
  read).

## How it was verified

- Each lever judged on its own real run: the whole d3 00-Agent walk with `NF_DMG_TRACE`,
  from the level load, real input only [L the 2026-11-22 runs: /tmp/baseline20.out,
  base20b, base20c, lever20a, lever20b — sandbox scratch].
- The whole `walkreal_07000008_live` binary on the shipped tree: **6 passed / 0 failed /
  1 ignored (the target walk), 276.90 s** [L the final-tree run] — and again with the
  retarget lever: **6 passed / 0 failed / 1 ignored, 272.69 s** [L]. The d1 and d2
  real-damage sisters, both plain walks and both ally tests byte-exact green.
- The full suite (tools/test/suite.py): started in the background twice (08:19, 08:41 UTC)
  and died between tool calls both times (the rounds 13-19 kill pattern), and the second
  start's build filled the sandbox disk: the rootfs hit 100% (9.9 GB — the release target
  tree for four crates is 5.9 GB of it), `cargo test` failed with `No space left on device`
  on four not-yet-built nf-view test binaries (level_carry_live, fix_objective_banner_live,
  bath_floor_chain_02_live, rigid_body_impact_sound_live). Cleanup freed ~1.3 GB (the
  sandbox's browser caches) but the space was re-consumed by the suite build before the
  kill; 67 MB free at report time. The full suite therefore DID NOT RUN this round — the
  same documented fallback as rounds 13-19. What ran: the whole affected binary above
  (6/0/1 twice, 276.90 s and 272.69 s), plus the two lever runs. The final tree changes
  only the one 08 test file and the opt-in helper in common/damage_log.rs (flag-gated,
  default off — no other test sets it), so the other binaries' behaviour is unchanged from
  the merged 08-19/master state.
- `cargo test --offline --release --no-run -p nf-view --test walkreal_07000008_live`
  compiles clean on the final tree (the only crate touched is nf-view's test code plus the
  opt-in helper in common/; no `crates/*/src/` change at all this round).

## What's left

- The walk is still red at t25783 (the machine-fight anchor, 2.0 health + ~50 armour, the
  world row). The next window works the three levers in the ledger: the fall (23.0), the
  secure-room #6 range gate, and the machine fight from the anchor on the batch-3 stream.
- The full suite did not run (the sandbox disk filled; see the verification section) — the
  next window should start it EARLY and check the four test binaries that never compiled
  this round.

## Live checks for the main agent

- The fall: does the game's own player pay the ~23.0 for the station-hall balcony running
  jump onto the deck (a boot + jump + the health read on landing)? It decides whether the
  d3 route should chase a softer crossing or accept the stream cost.
- #18 (skin 0x500002f, (115.2,41.5,-101.8), weapon 0x15, hp 20): is he visible from the
  y33 hall floor (a boot at (118.3,34.6,-109.2) facing him)? His rounds reach Bond but no
  fight ever opened on him.

## Usage

not visible to me
