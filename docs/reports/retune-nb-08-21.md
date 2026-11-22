# retune-nb-08-21 — the 08 whole-mission 00 Agent real-damage walk, round 21

## What's done

- Branch `s/retune-nb-08-21` from `origin/master` (9fff71e1c) with `s/retune-nb-08-20`
  merged first (08-20 was not on master; the merge was clean). Setup from scratch (the
  sandbox reset): clone, work-slim + the verified 369823203-byte ISO archive (py7zr; no
  root, no 7z, no sudo), rustc 1.100 (rustup), libasound2-dev + libasound2t64 extracted
  into a user prefix (`~/.cargo/config.toml` rustflags `-L` + `PKG_CONFIG_PATH`, sandbox
  files, never committed), `cargo fetch`, the nf-queue worktree. ~5.6 GB of pre-seeded
  sandbox caches (`~/.venv` 4.4 GB, `~/.bun/install` 1.2 GB, `~/.cache`, the default
  Next.js `node_modules`) were deleted to fit the four-crate target tree on the 9.9 GB
  rootfs (round 20's disk-full lesson).
- **THE MERGED-TREE BASELINE RE-MEASURED FIRST, as the rounds' law requires: death t25783
  BYTE-IDENTICAL to round 20's measurement** — (113.06,13.47,-179.09), 48 hits for 77.0 +
  other 23.0, 17 return-fire fights [L the 2026-11-22 NF_DMG_TRACE run, /tmp/base21.out].
- **THE DEATH READ IN FULL for the first time.** The "world row" at the anchor is the
  machine's OWN ROCKET BLAST: `[dmg] tick 25783: -2.00 ... from the world (live wire /
  blast / hurt volume); explosion at (110.4,14.1,-181.3) d 3.5 radius 8.0 damage 100.0
  owner -1 kind 44` [L the run]. The machine's high-phase salvos (weapon 61, the [08
  salvo] gates live: line_ok true, slit true, riding true, clip 3) drained the armour
  50.0 → 0 between t24000 and t25783 (the -0.00 world rows at f25643-25785 are the
  armour-soaked blasts), and the fatal blast's residue took the last 2.0 health. Health
  had been 2.0 since t5020 (the secure-room #6 exchange close): 100 − 23.0 (the running
  jump's landing fall) − 60.0 (the deck fight, 15 hits by t3000) − 15.0 (#6's five −3.00
  rows t4886-5020) — the armour carried the walk the last ~20000 ticks [L the same run].
- **THE FALL LEVER (round 20's (1)) CLOSED as accepted stream cost, not a bankable fix:**
  the walk-off is dead by geometry (round 20's sliver read [L]); the step-split landing
  (jump onto the rail step at 49.5, then off east) saves ~4 by the sqrt descent law (the
  two-fall damage ≈ 19 vs 23 [S — the fall law is [M Player_CollisionHandler 0x1949a8:
  97-131, damage = fall_timer − 60, the timer accumulates only while vy < 0 and zeroes in
  every non-walk substate, so a ladder/wire/grapple-interrupted drop never hurts [M
  player.rs fall-timer comment]]); there is NO game ladder at the rail to interrupt the
  drop (the "stuck ladder" mentions in the test's own comments are the walker's
  stuck-mitigation sidesteps, not class-0x2D ladder objects [M]); and the d1/d2 sisters
  run the SAME running jump (the jump call fires for them at the walkway-head leg, the
  test's crossing block) and PASS — the game's own player pays the 23.0 on this route [L
  the round-20 whole-binary 6/0/1 on the same crossing].
- **THE SECURE-ROOM #6 LEVER (round 20's (2)) TRIED TWO WAYS, BOTH MEASURED DEAD on their
  own real runs, both reverted with the lever:**
  1. **The aperture-align stand** (+0.45 south along the perpendicular of the bond→target
     ray, gated by an additive `exposed_part_gun_rays_from` probe — a position-parameterised
     `exposed_impl` variant whose gun stand-ins derive from the AIM yaw): the #6 exchange
     kept its five −3.00 rows (t4886-5085, the same −15.0 — the burst rifle's lane covers
     the whole walkway; the stand-in rays read the shipped spot CLEAR while the real rounds
     still died, so the probe cannot certify any nearby spot) and the stream re-dealt +2351
     ticks deeper (death t28134 at (110.94,13.47,-180.95), 15 fights — re-deal noise, not a
     fix) [L the lever run /tmp/lever21b.out].
  2. **The line-break stand** (−0.45 north, the probe inverted to the first BLOCKED root —
     the fight goes blind and the 60-tick unseen abort ends it): WORSE — the abort left #6
     ALIVE, he ADVANCED (state 0x5e, to (158.7,43.0,-121.2), d 4.9) and shot the resuming
     walk for −3.00/−3.00/−2.00: death t5260 [L the lever run /tmp/lever21c.out].
  Both reverted; the shipped tree carries only the ledger. The exchange itself is WON
  (killed true, #6 hp 0) — the −15.0 is #6's burst cadence over the ~200-tick exchange and
  no nearby stand beats it (the same "no out-of-lane spot" wall the deck trio measured in
  round 17's lever).
- The refreshed ledger (the death law, the two dead #6 variants, the fall closure, the
  next levers) is appended to the target test's ledger comments in the ignore message.

## How it was verified

- Each lever judged on its own real run: the whole d3 00-Agent walk with NF_DMG_TRACE
  from the level load, real input only [L the 2026-11-22 runs: /tmp/base21.out,
  base21rf.out (with NF_RF_TRACE for the fight picks), lever21a/b/c — sandbox scratch].
- The whole `walkreal_07000008_live` binary on the FINAL tree (the reverted lever, the
  ledger comments only): **6 passed / 0 failed / 1 ignored (the target walk), 272.21 s**
  [L the final-tree run].
- The full suite (tools/test/suite.py): started twice in the background (setsid nohup,
  11:26 UTC) and killed between tool calls both times (the rounds 13-20 kill pattern —
  suite.out empty, the process gone). The documented fallback ran instead, chunked in
  foreground calls on the final tree:
  - nf-game lib binary: **1080 passed / 0 failed / 4 ignored** (1.13 s).
  - nf-game's other 155 test binaries: **678 passed / 27 failed / 4 ignored** — every one
    of the 11 failing binaries is the drive/driving family (the driving-disc BIGF content
    the work-slim extraction + onfoot ISO do not carry: `BIGF: OutOfBounds { what: "BIGF
    header", offset: 0, len: 16, size: 0 }`), the same environmental class the task rule
    carves out for the disc_golden trio; the non-driving nf-game binaries: **412 passed /
    0 failed / 4 ignored**.
  - nf-formats + nf-audio (74 binaries): **395 passed / 6 failed / 3 ignored** — the six
    are the disc-content family: the disc_golden trio (the task rule's allowed three) plus
    crp_mesh_disc's two track/collision meshes and identity_disc's full-disc identity
    digest (the same slim-ISO environmental class; carved_dir, round 16's environmental
    failure, PASSED here).
  - nf-view: only the walkreal_07000008_live binary was built and run whole (above); the
    rest of the nf-view binaries were not built — the window closed (see What's left).
  - The chunked totals across what ran: **2153 passed / 33 failed / 11 ignored**, with
    every failure in the documented environmental classes (no nf-game/nf-formats/nf-audio
    code changed this round — the only source-file change is the ledger comment block in
    the 08 test file, comments only).
- `cargo test --offline --release --no-run -p nf-view --test walkreal_07000008_live`
  compiles clean on the final tree. No `crates/*/src/` change at all this round.

## What's left

- The walk is still red at t25783 (the machine-fight anchor, health 2.0 + armour 50.0, the
  machine's kind-44 rocket blast after the armour drain). The next window works the
  machine-fight lever: the 08-19 anchor shape (the pre-fight crouch, the dive-launch band)
  re-pinned on the batch-3 stream, with the death law above as the first read — the anchor
  gates already read `riding true` (the boarding gate OPENS; the salvos close the walk
  first), so the shape's job is surviving the high phase, and the upstream banks that feed
  it are the deck fight's 60.0 (the pre-t3000 bleed — the biggest single bucket) and the
  #6 15.0 (measured unfixable by stand geometry this round).
- The nf-view test binaries other than walkreal_07000008_live were not built/run — the
  next window should start the suite EARLY (the background kill makes chunked foreground
  builds the only reliable path on this sandbox; budget ~25 min of build for nf-view).

## Live checks for the main agent

- The fall (23.0): does the game's own player pay the ~23.0 for the station-hall balcony
  running jump onto the deck (a boot + jump + the health read on landing)? The d1/d2 walk
  evidence says yes; a live boot confirms the route's cost is the game's own.
- The machine fight at the anchor: with the gates reading `riding true` at t25773 (the
  boarding gate open), does the machine's high phase ever drop low enough to board from
  the anchor on this stream, and what stance keeps the salvos off the 2.0-health Bond
  longest? A PCSX2 boot at (112.09,13.56,-179.46) facing the machine answers it.

## Usage

Session context: the driver's task window (2026-11-22 11:19 UTC sandbox start; this report
written ~12:21 UTC). Token/credit figures: not visible to me — the environment shows no
input/output token or cost counters to this worker; the session's own tooling exposes none.
