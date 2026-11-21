## Your task: `retune-nb-08-18`, 08 whole-mission 00 Agent real-damage walk green again (round 18)

Branch from current origin/master; if origin/s/retune-nb-08-17 is not on master, merge it in first. Read docs/reports/retune-nb-08-17.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Use the round-17 hit list: change the earliest segment that costs the most health first, starting with the 14 pre-fight standing zaps. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-08-18`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-21T23:18Z

---
## Report (done, 2026-11-21T23:59Z)


---
## Report from the worker (done, 2026-11-21T23:59Z)

# retune-nb-08-18 — the 08 whole-mission 00 Agent real-damage walk, round 18

## What's done

- Branch `s/retune-nb-08-18` from `origin/master` (b5278f0d1, the fusebox-pit sight-abort
  tree) with `s/retune-nb-08-17` merged in first (08-17 was not on master; its merge brings
  `docs/WORKLOG.md` and the rounds 5-17 ledgers). Setup from scratch (the sandbox reset):
  clone, work-slim + the 369823203-byte ISO archive verified, rustc 1.100 (rustup), the
  extracted libasound2/libasound2-dev debs into a user prefix (no root; the linker pointed
  at it via `~/.cargo/config.toml`, a sandbox file, never committed), `cargo fetch`.
- **The task's ask, lever (1) THE PRE-FIGHT CROUCH — TRIED and SHIPPED (still red, the
  death moved +724 ticks).** The ride-free shape as the round-17 ledger designed it, both
  halves in `crates/nf-view/tests/walkreal_07000008_live.rs`, both d3-gated:
  1. the crouch press at the anchor arrival — after the `[08 anchorwalk]` walk loop and
     before the fight loop, a `for _ in 0..6 { if crouched break; tick(crouch) }` press, so
     the approach burst (the round-17 ledger's 14 x -2.00 standing rows t27776-27951) lands
     on the crouched capsule and the fight OPENS crouched;
  2. the first-pass hold-off on the round-13 west stand gate — the gate now holds while
     `rockets == 0 && t < 60` (the band's live window t0-t55 [L the round-13 [08 gates]
     dump]). The shipped timeline never reaches that branch standing during the first pass
     (its four in-loop rows are its standing cost), so the hold-off is byte-exact there; in
     the crouch timeline it stops the gate from standing Bond at the dive's first west low
     run and re-opening the t49 dive launch — the ride the lever exists to skip.
- **MEASURED [L the 2026-11-21 NF_DMG_TRACE lever run]:**
  - the ride-free shape WORKED: the [08 gates] t0 row reads line_ok false (the crouched
    eye's ledge blindfold reproduced — every bearing's ray stops 0.02-0.12 u out,
    az0.00:0.08..az5.89:0.09 [L the t0 sweep; the round-4 law]), ZERO launches the whole
    fight, the Hind damage 0.0 of 598, no ride, no frozen input.
  - death t28907 at (113.1,13.1,-179.1) CROUCHED at the anchor — 724 ticks deeper than the
    round-17 baseline t28183 — 61 hits for 100.0, 22 return-fire fights; the fight drained
    its 26.0 entry health over 989 fight ticks vs the shipped 26.0 over 266 (the crouched
    zap trickle + #25's cadence).
  - the fight CANNOT be won from this anchor on this timeline: the west gate did stand Bond
    once (feet 13.47 at t100, after the hold-off lifted) but the machine's pass was already
    y 15.4 — the scoped eye ray stops t0.062 at (110.4,14.2,-180.3) (the west band slot's
    top ~14.25 stops the rising rays; the flat 30-u az ray at the same bearing reads CLEAR
    1.00 — the slot passes FLAT rays only) — and the burst arm re-crouched him on the next
    zap. The machine never came low+west again: the flight samples t0-t900 climb 12.87 ->
    15.4 -> 19.5-20.5 and circle east (x 127-142 at t600-t900, state 2/1); the route itself
    flies y >= 14.6 (the approach dump f27060-27840: y 14.2-22.1), so the ONLY y <= 13.7
    window in the whole fight is the arrival dive. Skip it (this lever) = zero Hind damage
    forever; take it (the shipped shape) = the ride freeze + the post-ride crouch blindfold.
  - #25 was NOT answered: hp 20 INTACT through all 22 return-fire fights — his three rows
    (t28115 -6.00, t28827 -6.00, t28907 -2.00, cadence ~712 ticks) landed with the AT-420
    armed and the walker's return-fire machinery fires the CURRENT weapon; a guided launcher
    cannot fight a 47-u guard.
  - NEW lever material [L the [08 pickups] dump]: THREE un-gone 0x44 dart/tranq pickups sit
    1.6-2.8 u from the anchor at the fight entry (kind 0 rounds 4 at (111.84,13.37,-181.45);
    kind 1 rounds 4 twice at (111.62,13.24,-178.47) and (110.67,13.24,-177.72) [L
    docs/atlas/weapon-data.md: 0x44 = the dart/tranq family]) — a scoop-and-arm lever is
    untried and cuts all of #25's -14.0.
- The refreshed ledger and the next levers are in the ignore message
  (`walk_the_whole_mission_on_real_input_real_damage_00agent`): (1) the fight-order
  upstream savings (the round-17 item (2) — a common/walker.rs change, so the whole
  walkreal_*_live set must run with it) to bank the hall trio's -28.0 on top of this
  lever's savings (the survive-to-second-dive shape needs the entry health ~70+); (2) the
  dart scoop-and-arm; (3) the copter dive-cadence read (Copter_Update 0x17dbb8's kind-2
  selection vs the cannon cycle — does the d3 machine EVER re-dive? The d2 sister's machine
  re-dived five times in 2100 ticks [L the green d2 run]).

## How it was verified

- The lever on its own real run: the whole d3 00-Agent walk with `NF_DMG_TRACE`, from the
  level load, real input only — the run's dumps quoted above (`[08 gates]` t0-t200, the
  `[08 hind]` eye sweeps and flight samples t0-t900, the `[08 pickups]` fight-entry scan,
  the `[dmg]` rows) [L the 2026-11-21 run, `lever18.out` in the sandbox only].
- The whole `walkreal_07000008_live` binary on the shipped tree: **6 passed / 1 failed
  (the target walk), 162.00 s** [L the `--include-ignored --test-threads=1` run] — the d1
  and d2 real-damage sisters, both plain walks and both ally tests byte-exact green,
  matching the rounds 14-17 final-tree numbers. No `crates/*/src/` change and no
  `crates/nf-view/tests/common/` change (both edits live in the 08 test file only, and each
  is inside a `difficulty >= 3` block the sisters never run), so every other binary's
  behaviour is byte-identical to the merged 08-17 state.
- The full suite (tools/test/suite.py): the background runner was started twice in the
  window (23:26 and 23:57 UTC) and died between tool calls both times (suite.out 0 bytes,
  the same kill pattern rounds 13-17 documented), and the 10-minute foreground cap cannot
  hold the four-crate build on this sandbox. What ran: the whole affected binary above
  (6/1, 162.00 s). The final tree (the push above; origin/master did not move) changes only
  the one 08 test file, so the other binaries' behaviour is unchanged from the merged
  08-17 state.

## What's left

- The walk is still red: the machine fight cannot be won from the anchor (the arrival dive
  is the only y <= 13.7 launch window; the route's passes are y 14.6-22). The next window
  works the three levers in the ignore message, in that order, starting with the cheap
  dive-cadence read (no code change) to decide whether the survive-to-second-dive shape can
  ever pay before spending the common/walker.rs fight-order lever.

## Live checks for the main agent

- The copter's re-dive cadence (carried from this round): a gdbwatch on the copter state /
  path during a long-lived fight — does the d3 (00 Agent) machine ever launch a second
  attack dive (y <= 13.7) after the arrival dive, and at what cadence? It decides whether
  the survive-to-second-dive lever family (the fight-order savings) can ever pay.
- The reply cadence (carried rounds 11-17): a gdbwatch on the w62 spawn after the dive
  launch — provoked or scheduled?

## Usage

not visible to me

## ntfy failed

1. curl: `curl: (28) Connection timed out after 20002 milliseconds` (rc 28) — the same
   failure as rounds 5-17.
2. python3 urllib: rc 124 within the 35 s outer timeout — no output, no traceback, the
   request did not return (the round-5-16 pattern).
3. wget: rc 124 within the 25 s outer timeout — no output, the request did not return.

All three failed; the intended text: `retune-nb-08-18: the pre-fight crouch + first-pass
hold-off shipped, death t28183->t28907 (+724), ride-free fight measured siege-only (zero
Hind damage, no re-dive in 989 ticks); whole binary 6/1; next: dive-cadence read, dart
scoop-and-arm, fight-order savings`.
