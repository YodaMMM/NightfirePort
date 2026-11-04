# gas-rng-draws-4 — report (2026-11-04)

**PARTIAL** — two of the report-3 list items landed (the enemy_accuracy re-pin, the blast type-0
leg), the 01 walk got its doorway play at every difficulty (three later reds remain with fresh
ledgers), and the window closed before the 02/03/14/16/1b re-routes, the finale leg, and the
full-suite pass. Branch `s/gas-rng-draws-4` (cut from `origin/s/gas-rng-draws-3` b6dd3a47e,
origin/master b21edd082 merged clean).

## Done this session (each verified on this tree)

- **enemy_accuracy — the per-setup re-pin: GREEN.** The full 21-cell measurement ran (217 s):
  exactly TWO 0x12 cells moved outside the old ±max(want/5,2) tails — diff 1 d 5 = 8 (was 15),
  diff 2 d 5 = 25 (was 17); same p, the gas-init draws shifted the burst timings [M
  Gas_Init 0x1aacc8:72-80 → Rand_Rand/FRand 0x1e3738/0x1e3780, the ONE shared seed pair]. The
  two PINS re-pinned, the tolerance law and all 19 other cells untouched, the doc comment
  carries the cite. Binary whole: **5 passed / 0 failed / 4 ignored** (401 s). The sister total
  law held (95 vs the old chain's 60).
- **fix_blast_scripts — the type-0 leg: DONE in the honest shape.** Two changes:
  1. `aim_at`'s pitch is now the game's HALF-TURN convention — `(d1/l).asin() / (π/2)` [M
     player.rs look(): p = pitch·π/2 before RotMatrixX 0x1e46a0; report-3's diagnosis — the
     radian read dove the shot 34.3° instead of 21.8°]. All three aim-using legs still pass.
  2. The type-0 sister is split into its own `#[ignore]`d test with the measured geometry
     ledger: destroy[10] (placement #470) is the point box lo == hi == (0,0,0), hit as ±0.1 [M
     breakable.rs seg_box_t lo==hi arm]; a scan of ±56 u around the origin at 2-u steps found
     NO floor at all (report-3 said 35), and the P2K's record gives range 50.0 / spread 0.0 [L
     ee_snapshot.bin weapon_data 0x2bf368 (+0x1C 50.0, +0x20 5.0, +0x24 0.0, flags68 0x1004)] —
     so the shot must start inside 50 u of a point nothing can stand near. The 0x12bc50
     no-script law stays pinned by the type-1 leg (the same `if (cVar4 != 0)` branch decides
     both). Binary whole: **3 passed / 0 failed / 1 ignored**.
- **walkreal_07000001 — the doorway play opened to every difficulty (partial).** On the
  re-landed stream the ch-0xE crowd bunches at the east-door mouth the tick the meet scene
  writes 0xE: the d1 stage walk met three of them at 0.4-0.7 u and died at tick 1930 (26 hits
  for 100.0; the under-3-u forced hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115] + the
  TooClose ×2 [M Drone_ModBulletDamage 0x139f50]). The d2/d3 sound-lure doorway fight (hold
  line, the guards cross one at a time) now runs at d1 too — the AND now fires at 1974 on the
  d1 run — and the lip-lure block (the 0x5000038 shotgun on the aperture's lip), the doorway
  release band and the west-corridor route pre-clear opened from d3/d2-only to d1 as well
  (their old gates kept a byte-exact Agent timeline that this stream already broke). A
  skin-0x500003a far-field pre-clear was added after the hall AND (the south-lower-hall pair's
  lanes cover the door mouth). Remaining reds, all with fresh ledgers:
  - d1: died tick 7712 at (-20.13, 6.38, 14.48) — the west corridor; #3 (0x5000036 SMG) in its
    ch-0x26/0xF pocket sprayed 71.0 in 94 hits from 10.8 u during the follow (he starts AFTER
    the pre-clear pass).
  - d3: died tick 4163 at (7.60, 6.46, 11.84) — the far 0x500003a pair (19.7-19.8 u) covers
    the door mouth during the aperture fight (84.0 in 6 + 14.4 in 8); they are not yet started
    when the post-AND pre-clear runs, so the skin filter passed them by.
  - d2: died tick 8502 at (-31.31, 6.46, 5.15) — the stairwell-head pocket, unchanged from the
    report-2 diagnosis (#3 31.5 in 15 at 7.1 u).

## Not done (the next session's list, in order)

1. The walk re-routes: 01's three remaining reds (ledgers above), 02 (to_the_exit — not run
   this session, the walk alone costs ~6 min), 03 (4/2+1i — the friendly death + the east-lane
   wedge), 14 (not run this session — the corridor-door chain at (30.3, 35.4)), 16 (12/2+2i —
   the punch pin + the defeat walk), 1b (15/2+2i — Drake's defeat + one more).
2. fix_finale's 16 shelter pulse — fresh ledger this session: Bond died tick 6493 at
   (-14.50,-22.00,-170.18) BEFORE pulse 2's window, to a 13.3 "the world" hit (a live wire or
   the pit's lethal-floor edge) plus 29.6 of chaser plinking accumulated on the walk back —
   the chaser-fight block's loose timing (+2600 gate, grenade retries) and the INTO_THE_SHELTER
   route need a phase-accurate re-derivation.
3. Every walkreal_*_live whole (this session ran only the six task levels + 01), then the full
   suite via tools/test/chunk_suite.py (the reports measure its run phase at ~3 h on this box —
   start it early).

## What was run (direct binary runs, --test-threads=1, slim ISO)

- `enemy_accuracy_live` 5/0+4i. `fix_blast_scripts_live` 3/0+1i. `fix_finale_live` 3/1.
- `walkreal_07000001_live` 2/3. `walkreal_07000003_live` 4/2+1i. `walkreal_07000016_live`
  12/2+2i. `walkreal_0700001b_live` 15/2+2i.
- `walkreal_07000002_live` and `walkreal_07000014_live`: built, not run (the window closed
  mid-sweep; 02's walks alone cost ~6 min).
- nf-view lib test builds clean (an earlier ALSA-1.2.16 glibc-2.43 sqrtf link failure was the
  sandbox's user-prefix being too new; 1.2.11 fixed it — no repo change involved).

## Live checks for the main agent

1. 01's far pair: in PCSX2 at 00 Agent, gdbwatch the two 0x500003a guards at ≈(-7.9, 6.3,
   -0.4) — WHEN do they start (the switch channel) relative to the hall AND? The port needs
   their enable tick to place the pre-clear.
2. 16's shelter: the 13.3 world hit at (-14.50, -22.00, -170.18) — which volume wrote it (the
   live wire or the lethal floor)? fieldprobe the hurt-poly gate 0xf3's neighbours at that spot
   during pulse 2's approach.

## Usage

not visible to me (no token/credit/context-window figures are exposed to this session; nothing
in the sandbox reports them).
