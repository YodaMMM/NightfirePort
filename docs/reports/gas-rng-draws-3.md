# gas-rng-draws-3 — finishing the gas RNG draws (the open reds)

Branch `s/gas-rng-draws-3` (cut from `origin/s/gas-rng-draws-2` 3083a2ca, then `origin/master`
merged clean). The base branch proved the gas-init draws ride the SHARED RNG stream
[M 0x1aacc8:72-80; Rand_FRand 0x1e3780 / Rand_Rand 0x1e3738] and re-landed them; its report
(docs/reports/gas-rng-draws-2.md) diagnosed the remaining reds. This session worked that list
until the window closed.

## Done this session (verified green on this tree)

- **sight_frustum_live — 2/2 GREEN.** Both tests died on "guard placement 261 missing or not
  present": on the shared stream the guard finishes his route during the intro and the game
  deletes him — prop 0x59 guards are DeleteMe on GO_TO_GOAL arrival [M 0x165640:47-82]. The
  census is re-derived onto the staying guards **158 and 161** by a live probe of the real path
  (120 idle ticks at the 07000001 start): 158 at 6.4 u / 17.4° off the aim, 161 at 2.4 u / 19.9°,
  both bullet-clear, in-frustum and drawn; after the 40-tick look-up (pitch 0.40) both are still
  bullet-clear and inside the old pitch-blind cone but below the tilted bottom plane — undrawn.
  They are the only other present+clear+in-frustum guards at the start view (every other
  candidate is occluded). The world-sphere-slack coverage that 261 carried (0.7° outside the raw
  half-angle) left with his deletion; the slack term itself is unchanged code [M 0x133070
  obj+0x8C] and the header note says so.
- **hostage_killer_07000002_live — 5/5 GREEN.** The opener-law re-check [M 0x144340] the task
  asked for: the monitor runs on the captor and step 0 self-sends 0x1A, so the captor's line
  opens [M 0x144340:19-21] — but his pool roll is 0x29+Rand(6) [M 0x14a8b8:8-21] and the pool's
  0x2E carries sfxId 0 [D], which SpeechSFX no-ops [M 0x141b00]: no voice, the step-1 busy test
  advances at once [M 0x144340:10-24], his 1→2 send reaches the hostage and HER expected reply
  opens the conversation. On the re-landed stream the ch-0x19 captor's first roll is silent —
  the first heard line is legitimately the hostage's. The pin now accepts exactly that case with
  the same evidentiary standard the repeat rule already used (no captor voice at any census tick
  + her voice arrived expected, i.e. off his 1→2 send edge); every other law (≥8 periods, strict
  alternation, no overlap, the 2→1 wrap, recurring hostage-directed sends) is untouched.

Checkpoint commits: d9266f48 (both fixes + WORKLOG), 67fbd51f (WORKLOG diagnosis below).

## Diagnosed further this session (for the next message)

- **fix_blast_scripts_live — the type-0 leg is geometry-dead, not stream-unlucky.** Probed on
  the real path: the teleport spot (1.9717, −0.4, −2.256) is a **void** — no floor within 35 u
  on any ground ray around the origin (Bond falls forever, `grounded` stays false); destroy[10]
  (placement #470, the boundless bigdoor01 leaf) parses to a **zero-extent box**
  (lo == hi == origin, hit tolerance ±0.1 [M breakable.rs seg_box_t lo==hi arm]); the P2K record
  carries **spread 0.0** [L ee_snapshot.bin weapon_data @0x2bf150 + 2·0x10C: +0x24 = 0.0,
  +0x98 = 0.0, flags68 = 0x1004] so the shot dir is exactly `look` — and the test's `aim_at`
  helper treats pitch as radians while the game's pitch is HALF-TURNS [M player.rs look():
  p = pitch·π/2, RotMatrixX 0x1e46a0], so the shot dives 34.3° instead of the intended 21.8°
  and its closest approach to the origin is 1.45 u — no spread roll can close that (there is no
  spread to roll). The type-1 sister passes because its box is large. Fix shape: re-derive the
  leg onto a real floor spot (or compensate the aim for the half-turn convention) and cite the
  placement data; the type-0 no-script law itself [M 0x12bc50] is untouched by all this.
- The remaining reds keep the gas-rng-draws-2 report's diagnoses (unchanged this session):
  walkreal_07000001_live (3 — the contact-route pin + the skin-0x5000036 friendly's forced hit
  under 3 u [M 0x179740:113-115]; the fixed route gets unlucky on the correct stream),
  walkreal_07000002_live (1 — `walk_the_castle_all_objectives_to_the_exit`, re-confirmed red),
  walkreal_07000003_live (2 — the friendly death + the east-lane wedge), walkreal_07000014_live
  (4 — one wedge: the corridor door at (30.3, 35.4) whose opener shifted with the stream),
  walkreal_07000016_live (2), walkreal_0700001b_live (1), fix_finale_live (1 — the 16 shelter
  pulse), enemy_accuracy_live (1 — the per-setup hit counts; its 8-minute run never produced the
  bad list in the last session).

## Verification runs (this session, direct binary runs, `--test-threads=1`)

- `sight_frustum_live` 2/2 (was 0/2). `hostage_killer_07000002_live` 5/5 (was 4/5).
- Re-confirmed red on this tree: `fix_blast_scripts_live` 2/3, `fix_finale_live` 3/4,
  `walkreal_07000002_live` 2 passed / 1 failed / 1 ignored (324 s — the walks cost minutes each
  on this box).
- Full workspace test build (`cargo test --offline --release --no-run`, all four crates):
  Finished clean.
- **The full suite did not fit**: this sandbox allows ~2 h and the suite's run phase needs ~3 h
  here (the gas-rng-draws-2 report measured 1089 binaries at ~10 s each; the six walkreal
  re-runs alone cost 10+ minutes). The affected binaries that could run were run whole, above.
  Rule-10 note: no `crates/*/src/` code was changed this session (two test files + WORKLOG +
  this report), so no src-consuming test families were invalidated; the two fixed binaries and
  the re-confirmed reds above are the complete affected set of this session's diff.

## What's left (priority order for the next message)

1. The walk re-routes: 01 (contact route + the friendly-death stretch), 02 (to_the_exit), 03
   (east-lane wedge + friendly death), 14 (the corridor-door chain at (30.3, 35.4) against
   [M]), 16 (punch pin + defeat walk), 1b (Drake's defeat) — all re-routed the way a player
   would adapt, never by forcing state.
2. fix_blast_scripts' type-0 leg per the diagnosis above.
3. fix_finale's 16 shelter pulse and enemy_accuracy's per-setup re-pin (the run takes 8 min —
   start it first).
4. Every walkreal_*_live whole, then the full suite total line.

## Live checks for the main agent

None required for the two landed fixes (both pinned through the real input path in-sim). For
the blast leg, if convenient: in PCSX2 at 0700000d, confirm the bigdoor01 leaf (#470) has no
collision bounds and that no walkable floor exists near the world origin — predicted: Bond
placed there falls out of the world (matches the probe).

## Usage

not visible to me — this environment exposes no token, credit or context-window figures.
