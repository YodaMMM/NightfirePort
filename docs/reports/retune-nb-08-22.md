# retune-nb-08-22 — the 08 whole-mission 00 Agent real-damage walk, round 22

## What's done

- Setup from scratch (the sandbox reset between sessions): clone, the verified
  369823203-byte ISO archive, work-slim (7z extracted from the 7zip deb into a user
  prefix — no root, no system 7z), rustup stable 1.100, the ALSA headers via the
  user-prefix `PKG_CONFIG_PATH`, `cargo fetch`, the nf-queue worktree (`git worktree add`).
- The branch `s/retune-nb-08-22` already existed on origin (this round's first session
  had pushed it): checked out and continued. It is master 9fff71e1c + `s/retune-nb-08-20`
  + `s/retune-nb-08-21` merged, and already carried the round's first shipped lever —
  the WEAVING fight stands (opt-in `with_real_damage_fight_weave`, damage_log.rs
  `stand_and_fire`): a real ±0.5 strafe with 6-tick stops so the guards' perfect-shot
  roll reads a MOVING target (×0.25 first-moved × 0.75 moving = ×0.1875 [M
  DroneWeap_DoBulletAccuracy 0x179740:52-54,71-73; DroneWeap_DoOpponentTargetting
  0x179a48:33-41]) — the deck bleed 60.0 → 16.0 and the machine-fight anchor reached at
  37.0 health + armour 50.0 (was 2.0). That lever met the round law (health saved BEFORE
  the anchor); the walk was still red at t21154 with the machine phase costing 37.0.
- **The round-22 ledger's item (3) — the excursion legs' stop discipline — implemented
  and JUDGED ON THREE OWN REAL RUNS.** The law pinned from the decompile first: the
  not-moving branch of `DroneWeap_DoOpponentTargetting` clears +0x1DD/+0x1DE and the
  moving branch re-arms +0x1DD only when the +0x1DC/+0x1DD byte pair is still zero [M
  0x179a48:44-58]; the window is `DroneFiring_TargetFirstMoved_TimeToHit` 0x40000000 =
  2.0 s [M 0x1d3a80]; the 6 no-input ticks zero the eased lane exactly (0.0375·0.8^6 <
  0.01 [M Player_Move 0x191dd0:111-141]).
  - **Variant 1** (6-tick stops between the descent legs and on the return legs): WASH —
    the pit pair kept 24.0. Leg 1 paid ×0.75 UNLATCHED (the walk's arrival ease still
    held +0x1DC non-zero at the excursion's first tick — the row t4933 landed on it), one
    row landed DURING a stop (t5269, the full-p standing cost), and the 120-tick window
    expires inside the 330-tick crawl legs. Death t22980 (the machine phase re-dealt
    into three kind-98 missile blasts −12.34/−8.88/−12.78) [L the lever22a run].
  - **Variant 2** (+ the pre-leg stop): the pit bank landed (24.0 → 12.0, leg 1's crawl
    clean) but the ~100-tick pace shift re-rolled the boarding ledge's unseen pair — #27
    burst 28.0 where the shipped stream paid ZERO — fight entry 15.0, death t20785 [L
    the lever22b run].
  - **VARIANT 3 SHIPPED: the PRE-LEG STOP ALONE** (6 no-input ticks between the
    excursion's entry log and the first descent leg; nothing else — the stream shift is
    6 ticks). MEASURED on its own real run [L the lever22c run, NF_DMG_TRACE from the
    level load, real input only]:
    - the pit pair 24.0 → **12.0** (#15 three rows t4933/t5032/t5151 — leg 1's crawl
      LATCHED and clean; #16 ZERO rows; the t4933 round is in flight before the excursion
      starts and is unavoidable);
    - the mid-game transit CLEAN (no #27, no #6 rows);
    - the machine-fight entry **armour 0.0 health 49.0** (the shipped tree: 37.0 + armour
      50.0 — this deal drained the whole armour during the boarding chunk and left the
      health intact);
    - **death t25843** at the anchor (113.1, 13.5, −179.1) — the deepest walk yet
      (+4260 over the merged-tree baseline t25783, +4689 over the shipped weave run
      t21154), "39 hits for 77.0 + other 23.0, 20 return-fire fights".
- The refreshed ledger (the three variant runs, the new state, the next levers) is
  appended to the target test's ledger comments in the ignore message. The walk is STILL
  RED: the last killer is ONE row — the machine's kind-44 rocket blast −45.00 at t25843
  (explosion damage 100.0 radius 8.0 → the falloff puts Bond d ≈ 4.4 off the centre)
  after two −2.00 zaps, armour 0.0, health 45.0 when it landed: death by EXACTLY zero
  margin. Next levers (in the ledger): (1) the kind-44 rocket answer (the dodge package
  does not read this round class; or the stand spot ~1.0 u along the trench → the blast
  −36.25 and the walk survives); (2) the pit pair's remaining 12.0 (a stop earlier than
  the excursion re-latches the approach walk's last leg); (3) the armour-entry timing
  (the ledge armour box vs the boarding chunk's drain).

## How it was verified

- Each lever variant judged on its own real run: the whole d3 00-Agent walk with
  NF_DMG_TRACE from the level load, real input only, no top-ups [L the 2026-11-22 runs:
  lever22a/b/c — sandbox scratch, the numbers are quoted in the ledger].
- The whole `walkreal_07000008_live` binary on the FINAL tree (the shipped variant 3 +
  the ledger comments): **6 passed / 0 failed / 1 ignored (the target walk), 276.59 s**
  [L the final-tree run]. The change is test-file-local and d3-gated by construction
  (every `fight_guards_wide` call site is difficulty >= 3): the d1/d2 sisters and every
  other walk in the binary stayed green.
- No shared code was touched (the whole change is in
  `crates/nf-view/tests/walkreal_07000008_live.rs`), so no other walkreal binary's
  timeline can move; the whole-binary run above covers the file's seven tests.
- The full suite: the background fast runner (`tools/test/suite.py`) was started twice
  and killed between tool calls both times (the rounds 13-21 kill pattern — suite.out
  empty, the process gone). The documented chunked fallback ran on the final tree as
  far as the window allowed; see the "Suite" section below for the totals that fit.

## What's left

- The walk is still red (death t25843, one kind-44 rocket blast past a 49.0-health,
  0.0-armour fight entry). The next levers are in the ignore message's ledger comments:
  the kind-44 rocket answer first (the dodge's round-class read or the stand spot along
  the trench), the pit pair's remaining 12.0, the armour-entry timing.

## Suite

- The chunked fallback on the final tree: nf-game lib binary 1080 passed / 0 failed /
  4 ignored; the walkreal_07000008_live binary 6 / 0 / 1 (above). The remaining crates
  did not fit the window (the sandbox expires ~21:01 UTC; the report and the finish
  steps take priority — the touched-code rule is satisfied: the whole change is the one
  test file, and its whole binary ran clean).

## Usage

not visible to me — this environment shows no token, credit, context-window or cost
figures for the session.
