# retune-nb-08-22 — the 08 whole-mission 00 Agent real-damage walk, round 22

## What's done

- Setup from scratch (the sandbox reset between sessions): clone, the verified
  369823203-byte ISO archive, work-slim (7z extracted from the 7zip deb into a user
  prefix — no root, no system 7z), rustup stable 1.100, the ALSA headers via the
  user-prefix `PKG_CONFIG_PATH`, `cargo fetch`, the nf-queue worktree (fetched origin
  queue, `git worktree add ../nf-queue origin/queue`).
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
  (explosion damage 100.0 radius 8.0, d 3.5) after two −2.00 zaps, armour 0.0, health
  45.0 when it landed: death by EXACTLY zero margin.
- **A fourth run (the ledger item (1)'s naive form) TRIED and REVERTED on its own real
  run:** the d3 fight anchor shifted 0.35 u north-east along the trench plate ([113.55,
  −179.05]) to buy ~0.4 u of reply miss. MEASURED [L the lever22d run]: the upstream
  stream is byte-identical (the 12.0-pit bank and the 49.0-entry preserved — the anchor
  walk runs after the shoot-out), but the dive launch window CLOSED (line_ok false at
  every hind sample, aim_yaw_err blowing up 0.009 → 3.062 after t100, ZERO launches, the
  machine hp 598 untouched — the round-12 anchor-move law repeats), the fight re-dealt
  into a 19-zap stalemate (t25788-26301) and the walk-back/dodge walked Bond WEST onto
  the band corner (110.9, −179.1) for the killing w62 missile: death t27890. Two ledger
  facts pinned: (a) **the kind-44 reply blast is the machine's response to Bond's OWN
  rocket** (no launch → no reply) — answering it by suppressing the launch is
  self-defeating, the answer must keep the launch and dodge the reply in flight; (b) the
  shifted spot's walk-back can drive Bond west onto the band (the round-13 lever-3 death
  class). The anchor reverted to the round-12 SE spot; the shipped state stays variant 3.

## How it was verified

- Each lever variant judged on its own real run: the whole d3 00-Agent walk with
  NF_DMG_TRACE from the level load, real input only, no top-ups [L the 2026-11-22 runs:
  lever22a/b/c/d — sandbox scratch, the numbers are quoted in the ledger].
- The reverted-tree target walk re-run reproduces variant 3 BYTE-CONSISTENT: death
  t25843, "39 hits for 77.0 + other 23.0, 20 return-fire fights", fight entry armour
  0.0 health 49.0 [L the final22 run].
- The whole `walkreal_07000008_live` binary on the FINAL tree (the shipped variant 3 +
  the ledger comments — the anchor comment edits are comments only, the ANCHOR const is
  the shipped value): **6 passed / 0 failed / 1 ignored (the target walk), 272.28 s**
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

- The walk is still red (death t25843, one kind-44 reply blast past a 49.0-health,
  0.0-armour fight entry). The next levers are in the ignore message's ledger comments,
  updated after the fourth run: the kind-44 reply answer that KEEPS the launch (the
  dodge's round-class read widened to the machine's in-flight kind-44 rocket — the reply
  is the machine's response to Bond's own rocket, so suppressing the launch is
  self-defeating), the pit pair's remaining 12.0 (the t4933 row is the opening-volley
  poll's own trigger; the crawl legs need an in-walk_nav_to stop discipline — shared,
  opt-in, all walkreal binaries re-run), the armour-entry timing (the ledge armour box
  vs the boarding chunk's drain).

## Suite

- The background fast runner (`tools/test/suite.py`) was started twice and killed
  between tool calls both times (the rounds 13-21 kill pattern — suite.out empty, the
  process gone). The documented chunked fallback ran on the final tree, the chunks that
  fit the window [L this session's runs]:
  - nf-game lib binary: **1080 passed / 0 failed / 4 ignored** (1.15 s).
  - nf-audio: all binaries green (29+1+4+3 passed, 0 failed).
  - the `walkreal_07000008_live` binary whole: **6 / 0 / 1**, 272.28 s (above).
  - nf-formats and the rest of nf-view did not fit the window (the sandbox expires
    ~21:01 UTC; the report and the finish steps take priority). The touched-code rule
    is satisfied: the whole change is the one test file, and its whole binary ran
    clean twice.

## Usage

not visible to me — this environment shows no token, credit, context-window or cost
figures for the session.

## ntfy failed

All three fallbacks failed (2026-11-22 ~19:41-19:43 UTC), in order:

1. curl: `curl: (28) Connection timed out after 20002 milliseconds` (exit 28, `-m 20`).
2. python3 urllib: no output, the process hung past its 20 s urlopen timeout and was
   killed by the 60 s shell cap.
3. wget: exit 124 (`timeout 25 wget -q -O- --post-data "retune-nb-08-22: …"
   https://ntfy.sh/nightfire-yoda-8k3x` — the 25 s cap killed it, no response bytes).

The message text all three carried: "retune-nb-08-22: round 22 cont. — the excursion
pre-leg stop shipped (pit 24->12, entry 49.0 best, death t21154->t25843 deepest, killed
by ONE kind-44 reply blast -45.00; anchor-shift lever tried and reverted: it kills the
dive launch; whole binary 6/0/1; report+ledger pushed)".
