# level-13-warm-chain-margin-8

Branch `s/level-13-warm-chain-margin-8`, cut from origin/master 385ebb66d with
`origin/s/level-13-warm-chain-margin-7` merged (it was not on master; the merge was a
fast-forward to 194df6378). Task: take the margin-7 report's pool-abort lever in
`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`
(00 Agent, real damage, still ignored) and carry the warm chain out the front entrance.

## What's done

1. **The rule-7 pool-abort lever landed in the shared fights' stand.**
   `crates/nf-view/tests/common/damage_log.rs` gained one opt-in field,
   `DamageLog::pool_abort_stands` (its own line, `#[derive(Default)]` false — the only
   `DamageLog` literal is `new()`'s, which uses `..Default::default()`), and
   `stand_and_fire` reads `pool0` (health + armour) at stand entry — before the settle
   loop, so a burst landing inside the raise counts — and aborts (the give-up return: the
   240-tick cooldown, the legs resume) when the stand has eaten the bar. The bar is
   **max(6.0, half the entry pool)** — the fraction is the measured law, see below.
2. **The file-local `aimed_stand`'s run-12 flat 6.0 became the same fraction law**
   (the drain's own bar), same evidence.
3. **The glass-band drain skips #4's post** (d3-only, by position (16.2, -362.4, 43.3),
   r 3.0): the run ledger below says the stand picked #4 first by index, his return
   rounds cost 13.5-57 pool for 0.5 landed, and its `!killed` end meant NO other guard
   was ever drained either. A player does not take a fight he cannot hold from a spot he
   cannot hold.
4. **The run ledger (five real runs, all [L] /tmp/d3_run13..17.log, traces
   NF_DMG_TRACE + NF_RF_TRACE):**
   - **Run 13** (the ledger's literal lever: a flat 6.0 abort in `stand_and_fire`): the
     fights aborted at t1 on #4's single -5.5/-7.0 rounds — Bond carried **40.5 hp +
     36.0 armour** at the door (the instrumented trace settles the pool accounting: the
     13.5 "took" was all armour; run 14's fight#7 "took 44.0" = 21.5 health + 22.5
     armour) — so the fights never answered the balcony sniper. Death t4383 INSIDE the
     raw door-crossing loop (no fight machinery there).
   - **Run 14** (abort gated on the aim bit unset): the same shape, t4383 — the bit law
     still forbids the trade.
   - **Run 15** (the fraction bar): fight#4 traded to 54% of entry (took 41.5) before
     the abort; Bond entered the door fights with **clip 0 spare 9** — the 13 rounds
     burned on the #5/#6 killed-false fights and the drain stand's four taps (0.5
     landed) — then the reload window ate a full burst; #4 alive; death t4383 at the
     crossing again.
   - **Run 16** (the fraction law in `aimed_stand` too): the drain stand on #4 held to
     the 38.25 bar and STILL killed false (his hp moved 14 → 13.5 across five runs of
     stands) while costing ~57 pool; Bond entered fight#4 at 19.0/0.0, aborted at 14/19,
     death t4315.
   - **Run 17** (the #4 skip): the drain picked the mover #6 instead, burned its quiet
     waits, and #4's free sniping killed Bond AT the band, t4234.
5. **No other walk changes.** The flag is set only by 13's d3 real-damage block
   (`walkreal_07000013_live.rs`, `w.dmg.pool_abort_stands = true`); the sisters and every
   other walk keep the master behaviour (the default is false and the code checks the
   flag). No health/ammo/position writes anywhere; the only inputs are the view drives,
   the trigger taps and the legs.

## How it was verified

- The d3 walk runs to the door phase deterministically; the runs 13-17 logs are the
  evidence for every claim above. The test stays `#[ignore]`d with the update-6 ledger
  (the walk is not green). The file compiles (`cargo test -p nf-view --test
  walkreal_07000013_live --no-run` clean after every edit and after the ignore-message
  rewrite).
- Whole binaries run this session (`--test-threads=1`, release, NF_ISO set), all green:
  - `walkreal_07000013_live`: **3 passed, 0 failed, 1 ignored** (the d2 agent sister,
    the d1 operative sister, the plain walk; the ignored one is the still-red d3 walk) —
    every new block is inside `real_damage && difficulty == 3` gates those tests never
    enter;
  - `interactables_07000013_live`: **10 passed, 0 failed**.
- **The full battery did not fit.** Two 10-minute foreground build windows on
  `cargo test -p nf-view --no-run` (all nf-view test binaries, the rule-7 sweep) did not
  finish linking the ~100 binaries on this 2-core sandbox, and `tools/test/suite.py`
  could not run at all: background processes are killed between tool calls in this
  sandbox (verified twice with nohup + the wrapper), so the "start it in the background
  in your first minutes" plan had no background to start into. What ran instead is the
  required binary pair above. The added field is a `derive(Default)` bool read only
  behind the new flag; the 13 sisters' byte-identical green runs are the direct evidence
  the flag-off path is untouched, but the other levels' walkreal binaries are NOT
  re-verified this window — the next window should run them (rule 7/10) before merging.

## What's left (the next session's levers, also in the ignore message)

1. **The #4 line itself** (the sharp next item): his box OPENS from the band (aim Some
   in every run) but the stands' zero-spread taps at d 37-42 land ~0.5 TOTAL (runs
   15-17). The live probe (the margin-7 report's #5/#6 exposure probe, extended to #4)
   must answer WHY — the root motion beats the flight [M Bullet_update 0x12e7b0]? the
   world stop eats the round? the eased cursor offsets — and from WHICH spot his line
   actually lands.
2. **The door-crossing raw loop**: the r13/r15 deaths landed INSIDE the crossing loop
   (no fight machinery there) — once #4 is answerable the crossing needs its own answer
   for any surviving sniper.
3. The clip state: the door fights started at clip 0 spare 9 (the #5/#6 killed-false
   fights burned 13 rounds); the #5/#6 exposure map (margin-7's lever 2) remains open.

## Live checks for the main agent

1. `tools/trace/gdbwatch.py` on drone #4's root position during a player stand at the
   glass band (-20.5, -368.4, 39.5): address = #4's pos (the drone struct's pos fields),
   access R/W, expected: whether his root MOVES between the aim and the round's impact
   (the run-15..17 taps land ~0.5 at zero spread — if the root steps during the flight,
   Bullet_update 0x12e7b0's per-tick step explains it).
2. `tools/trace/fieldprobe.py`: from the band spot, the world stop of the straight ray
   eye→#4 chest (16.2, -362.4, 43.3) and the same ray from the door spot (-20.6, 39.5)
   and from (-25.5, 38.8) — expected: which of the three lines CLEAR (the port's own
   segment reads them clear; the live probe confirms the disc agrees) and where the
   round dies if not.
3. What a fresh-boot player's aimed P99 taps at d 37-42 land on #4 (does the port's
   zero-spread model match the disc at that range).

## ntfy failed

(written after the send attempt — see below if it failed)

## Usage

not visible to me (no token, context-window or cost figures are exposed in this
environment; nothing to copy exactly).
