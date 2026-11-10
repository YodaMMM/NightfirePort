## Your task: `level-13-warm-chain-margin-2`, Chain Reaction can be walked from the warm chain out of the front entrance on real damage

Branch from origin/s/level-13-warm-chain-margin, merge current origin/master, and read docs/reports/level-13-warm-chain-margin.md. Round 1 bisected the death to 7876b7feb (the 12 keypad now takes its typed code, which re-times the chain) and proved the new bleed is game-correct: at frame 1647 a shotgun guard (skin 0x5000008, weapon 0x1c) from the spawner wave on the lobby's east lane unloads 7 pellets for 36.5 at d 2.6. Re-route the lobby leg so Bond is not on the east lane when that wave lands (wait out the wave from cover, or take the wave first from range), then finish the walk with a real margin and un-ignore walk_the_warm_chain_to_the_front_entrance_and_out_real_damage. Real input only, no top-ups, never lower damage or health; helpers gated to 07000013. Run walkreal_07000013_live and interactables_07000013_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-13-warm-chain-margin-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-10T11:10Z

---
## Report (done, 2026-11-10T12:30Z)


## Report (done, 2026-11-10T12:30Z)

# level-13-warm-chain-margin-2 — the d3 warm-chain walk un-ignored

**Branch** `s/level-13-warm-chain-margin-2` (pushed as itself; the finish line's
`10-level-13-warm-chain-margin-2` spelling is the queue name). Cut from
`origin/s/level-13-warm-chain-margin` 76608679d, merged current origin/master (clean; the merge
touched only `crates/nf-view/tests/walkreal_07000015_live.rs` — its binary re-proven below).

## What was done

`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` is
**UN-IGNORED and green** on this tree, all changes inside the level's own test file
(`crates/nf-view/tests/walkreal_07000013_live.rs`); no shared helper, no src, no damage or health
number touched; real input only.

The round-1 state died at tick 2242 inside the north-west sweep's failing #9 stand. This session's
ledgers (all [L], NF_DMG_TRACE runs on this branch, repro byte-identical to round-1) established
the mechanism before the fix:

- **The spawner chain is effectively endless and re-feeds on every kill.** The audit's census
  [D docs/levels/07000013-audit.md]: spawner #323 = group 2, mode 3, max 2 alive, **3 waves**,
  start ch 0x29; spawner #327 = group 1, max 4 alive, **waves 0 = 32000**, start ch 0x2A; the
  start channels are the load-created guards' death channels (#319/#320 → 0x29, #317/#318 →
  0x2A) [D p12; M 0x13e660]. Mode 3 fills a whole wave then waits for it to die [M 0x137e68:
  176-210]. So every static window (a drain's quiet wait, a failed stand, the door E + crossing)
  invites a fresh wave, and every drain kill re-arms a post: iter-2 took the mezzanine guard #4
  and the spawner raised #12 on the same post (15.7,-362.4,43.1) for 79.0 in 12 hits; iter-3
  bled inside the quiet waits themselves (#6new/#10new, frames 4995-5121) [L both ledgers].
- The round-1 ledger's re-anchor cost (34.0 in 10 hits from drone #3 at (6.8,-368.4,38.3)) was
  the spawner wave's LATE guard met mid-leg — not a route geometry problem.
- The iter-1 run (the round-1 tree + stop-on-fail + the east-lane drain) died ONLY to the room
  guard #8's point-blank blast (16.5 hp left, -16.5 at d 2.9, tick 4386) — its door crossing was
  CLEAN because the glass-band fights answered the wave and no stands followed [L].

## The pass shape (what ships)

1. **The spawn-pocket sweep** (unchanged, `stop_on_fail=false` — measured green in round-1).
2. **The east-lane wave drain** — the task's named lever ("take the wave first from range"):
   `drain_the_wave(&mut w, "13: the lobby east lane", 24.0, 12, 6, 180)` stands at the east-lane
   leg end, waits out the spawn cadence (the round-1 guards land ~300 ticks apart) and takes each
   guard whose boxes the gun-ray set exposes, from the already-taken spot at d ~12. The late
   guard #3 died there for 0.0 (round-1: 34.0) [L iter-1 vs round-1 ledgers].
3. **`clear_the_visible` gained `stop_on_fail`** (file-local fn, per-site): a stand that cannot
   open the guard's box ENDS the sweep — the round-1 death happened inside the post-failure
   stands (the tick-2242 frames 1769-1891 are #11's -3.5s while #9's box never opened) [L].
   Sites that measured green keep the old retry.
4. **NO stands at the west-mid / north-west / glass band.** Every drain variant there (iter-2/3/4)
   converted Bond's static time into respawn fire; iter-1's clean door crossing is the measured
   winner [L all ledgers]. A moving Bond is a smaller target than a statue; the legs' return
   fire answers on the move (the glass-band fights killed both shooters, killed true).
5. **A door-mouth drain with a health-watch** (`drain_the_wave(... 24.0, 8, 3, 150)`): its quiet
   wait breaks on the FIRST hit (a player being shot from an unopenable angle moves; the
   iter-3 lesson — the waits themselves were the exposure).
6. **The room drain INSIDE the security room** (from (−27,35), radius 12): #8 — the iter-1
   killer, posted at (−26.9,-368.4,29.5), never exposed from the doorway threshold — is taken
   from inside the room together with #9, both killed true for 0.0 [L iter-7 ledger].
7. **The room legs re-routed through the pickups test's proven take spots**
   (`interactables_07000013_live::the_pickups_take_on_the_walk`, green): the Spas12 #429
   (−20.48,-368.86,25.38) — the owned bit lands on the walk-over [M Pickup_Handler 0x18e650] —
   then the BodyArmourLight #414 from the EAST approach (−27.66,25.57, r 1.2): the armour float
   to 50 [M 0x18e650:220-225]. The old r0.6 east-face stand's take ray died inside the shelf
   unit's own mesh (armour 0 [L the 9840-tick run]) — the approach spot was the bug. The
   non-real-damage callers keep their byte-identical legs (the `else` branch).

**The pass (three identical runs, [L] iter-7/7b/7c on this tree)**: walked out in 8840 ticks,
0 stuck, 0 falls; end health 25.5 + armour 50.0 (75.5 standing at the exit latch); 15 hits for
57.0 on 13's legs + the 12 leg's 17.5; 6 return-fire fights. The margin is real: the armour and
the Spas12 are the level's own grants on the walked route [D 0xF0 placements #414/#429], taken
by the game's own walk-over.

## Verification (what ran)

- `walkreal_07000013_live` WHOLE: **4 passed, 0 failed, 0 ignored** (the cold walk, the
  un-ignored d3 walk, the Agent sister, the Operative sister — the sisters' legs byte-identical
  via the else branch, both green) [L /tmp/wr13final.log].
- `interactables_07000013_live`: **10 passed, 0 failed** [L /tmp/ia13.log].
- The master-merge-touched file's binaries: `walkreal_07000015_live` **8 passed, 1 ignored**
  (its own pre-existing ignore), `interactables_07000015_live` **19 passed, 0 failed** [L].
- The fast runner (`tools/test/suite.py`) **could not build in this sandbox**: the writable
  overlay layer holds under ~700 MB total (the base image fills the rest), and the nf-view test
  links died with `os error 28` (No space left) / linker SIGBUS, not code errors. In place of
  the total line, the existing current test executables were run directly (no src changed
  anywhere in this branch except the two test files named above): **102 binaries executed to a
  result line, 1034 tests, 0 failed** — nf_game 773 passed (its 2 first-run failures were the
  profile tests' own temp files hitting the same full disk; green on the rerun with space),
  nf_audio 27, nf_formats (no executable present in this sandbox), and the nf-view battery up
  to `assassin_07000004_live` alphabetically plus the four level-13/15 binaries above. The loop
  was killed by the session clock before the remaining nf-view binaries; the full-suite total
  line is owed to the next session on a sandbox with a writable disk.

## What is left

- The full `tools/test/suite.py` total line on a sandbox that can link nf-view's ~35 binaries.
- The BodyArmour take-ray live check from round-1 is MOOT on this route (the east approach
  takes it — the interactables test asserts it green); the spawner-cadence live check (where
  the wave lands, how long after the first kill) is answered by this session's ledgers: the
  wave arrives ~300 ticks after the raising kills and re-feeds a post on each death.

## Live checks for the main agent

1. Fresh boot of 07000013 at 007: stand at the east-lane end (5.0,-368.4,26.5) after the spawn
   pocket kills and hold — confirm the spawner wave lands on/near the east lane ~300 ticks
   after the desk kills and that a player holding there can take the late guard (#3-class,
   (6.8,-368.4,38.3)) from d ~12 (`gdbwatch.py` on the group-2 spawner's wave latch +0x1C,
   0x137e68:176-210).
2. Fresh boot: confirm group 1's spawner (#327, start ch 0x2A) keeps re-feeding while any of
   its wave is alive in the lobby — i.e. a static player is bled indefinitely (the audit's
   waves-0=32000 row); the walk's shape (never standing west of the re-anchor) rides on it.

## Usage

not visible to me (no token/credit/context figures are exposed in this environment).
