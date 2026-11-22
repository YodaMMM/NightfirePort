## Your task: `walk-06-00agent-2`, 06 castle interior to the party 00 Agent real-damage walk green (round 2)

Branch from current origin/master; if origin/s/walk-06-00agent is not on master, merge it in first. Read docs/reports/walk-06-00agent.md there and the ignore message on walk_the_castle_interior_to_the_party_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Keep the Operative and Agent sisters green. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000006_live binary and interactables_07000006_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/walk-06-00agent-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-22T22:51Z

---
## Report (done, 2026-11-22T23:53Z)


---
## Report (done, 2026-11-22T23:53Z)

# walk-06-00agent-2 — the 06 castle-interior-to-the-party 00 Agent real-damage walk, round 2

Branch `s/walk-06-00agent-2`, from origin/master 7562f1e70 with origin/s/walk-06-00agent merged
in first (three conflicts: README and `interactables_gadget_census_live.rs` taken from master's
newer side — the branch's copies were stale duplicates of pre-7562f1e70 master; the walk test's
ignore message keeps both the master sweep texts and the round-1 ledger). The ignored walk is
`walk_the_castle_interior_to_the_party_real_damage_00agent` in `walkreal_07000006_live`.
**Still red — but the measured state moved from "dies at the strip approach ONE-TO-TWO TAPS
short, #11 at hp 2.4" to "dies at the SE corner with #11, #20, #24 and #25 all dead, the killing
blow now ONE grenade"**, with four new file-local levers landed, gated, and ledgered.

## What the round started from

The baseline reproduced round 1's ledger exactly: death t7212 at (5.97, 42.04, −32.77), #20
ground 20 → 15.6 then the duel's swap, #11 ground 13.4 → 2.4, 30 hits for the whole pool. An
NF_DMG_TRACE run split the pool for the first time: 27.5 pre-fetch (the dock + guardroom trio),
then the 50 armour soaked 9 un-answered hits across the mouth/balcony phases (round-1 lever (b)'s
stream — they record as zero-delta soak hits, invisible to the per-hit trace), then the strip
duel's 72.5 health (#11 40.0 in 11 + #20 32.5 in 9). The port's armour law is decompile-backed:
only pain type 0 (the bullet arm) absorbs — `Player_HandlePain` 0x1902c8:84-104, the port's
`PlayerHealth::apply` — so the fetch's 50 armour was doing real work until the stream emptied it.

## Landed levers (each judged on its own real run, all file-local, gated `difficulty >= 3 && !w.tops_up()`)

1. **`duel_target`'s fixed arm no longer reads `enabled`.** #11 stands dormant (en0) through the
   balcony/landing phases [L the mouth pick census: "#11 team1 en0 hp20 y55.5 d32.7 rays0"], and
   the enabled-gated fixed duel bailed 0 taps. A player aims at any visible guard; the game's
   bullets hit ANY present drone [M Drone_CollisionHandler 0x140968:85-95]. The recent-hitter
   pick arm keeps the gate (it mirrors the shooters' own activity).
2. **The blocked-line early-out in `duel_target`: 25 consecutive TAPS without one hp drop bail
   the duel.** Measured from both sides: the first balcony tower duel burned 206 taps for ZERO
   damage (the whole load, bailed DRY clip 0 spare 0) and the walk then fought the mouth starving
   (cycling weapons, clip 1 spare 0); an earlier 40-TICK window measured PREMATURE — it bailed
   the strip-approach tower duel with #11 at hp 9.0 (4 taps short) and the strip duel with #20 at
   6.8. The tap-count window holds a peeking guard's ~20-tap pauses and still caps a fully
   blocked line at ~100 ticks / ~25 rounds.
3. **The #24 finder 3.0 → 6.0.** #24 patrols (29.2, −36.0) [L the lever-a run's death trace],
   3.9 u off the old finder's (32.0, −38.7) — the stand skipped and his 56.0 in 12 shredded the
   rest of that walk. The same patrol widening round 1 measured for #25. On the final tree the
   solo-spot stand VANISHES #24 at +43.
4. **Lever (a) ANSWERED: the FIXED tower duel from the strip approach** (cap 600), the ledger's
   next untried lever at its only proven angle. Three placements were judged on their own runs:
   the balcony's 0.26-rad up-angle (0 of 206 taps — the tower's own geometry eats the shallow
   line; retired in the file), the sight edge's 0.51-rad line (0 of 10 — the ledger's prescribed
   stop, measured dead there; retired in the file), and the strip approach's 0.57-rad line — the
   angle the baseline's own duel taps proved (13.4 → 2.4). From the strip approach #11 grinds
   20.0 → 9.0 behind the soak, dies mid-duel (the fixed arm then re-picks #20 and grinds
   6.8 → 0.2 — both high-post shooters neutralized).
5. **Lever (a') — the #23 grenade answer — landed at the corner, measured to the thin-pool
   wall.** Round 2's killing blow is ONE grenade: 20.75 splash at d 27.4 (38.5, 51.0, −32.2)
   took Bond's last 20.8 [L the lever-a5 run's death trace]. The answer's placements: the
   east-floor anchor NEVER RAN (the corner phases precede that leg on the route — Bond died at
   the corner twice before reaching it); the first 8.0-wide finder matched #22 instead
   (38.44, 51.0, −37.59) and displacing the #25 stand let his 0x16 stream deal 19.2 in 7; the
   landed order is #25 stand (VANISHED at +51) → the #23 answer (finder tightened to 3.0,
   matched (38.48, 51.0, −32.22) exactly) → the corner duel. Bond still dies ~10 ticks into the
   #23 grind: the pool is 20.8 at the corner and the grind needs 150+ ticks.

## Measured state (the red that remains)

- Death t8556 at (12.97, 41.83, −36.04) — the SE corner, 27 hits for 100.0. The route now kills
  #11 (the tower duel), #20 (its re-pick), #24 (the solo-spot stand at +43), #25 (the corner
  stand at +51), plus the mouth stands' #30/#28/#12; the descent phases ran clean in every
  late run.
- The deciding lever, ledgered for the next round: **pool-trim**. The tower duel's own cost is
  32.5 health (the armour was already 0.5 going in — the un-answered mouth/balcony stream of
  round-1 lever (b) soaked it). With the pool at ~70 instead of 20.8, the #23 answer (hp 10, five
  taps) and the walk's tail win; or answer that stream so the 50 armour carries the grind.

## How it was verified

- Every lever above ran on its own real run of the real input path (the load → the truck spawn →
  the walked route; no top-ups, no teleports; the death positions/ticks, dealer lists and duel
  grind traces cited are from these runs — /tmp baselines leverA…leverB3 on the branch).
- The sisters stay green and byte-identical: every lever is gated `difficulty >= 3 &&
  !w.tops_up()` inside the 06 test file only; no shared helper under `crates/nf-view/tests/common/`
  changed; no src/ changed.
- Whole-binary runs on the final tree: **walkreal_07000006_live 8 passed / 0 failed / 1 ignored**
  (the ignored one is the ledgered 00 walk), **interactables_07000006_live 5 passed / 0 failed**.
- The fast suite (`tools/test/suite.py`) was started twice in the background (pids 2265 and 6523)
  and the sandbox killed it between tool calls both times (this sandbox kills background jobs —
  the driver's own warning; round 1 documented the same plus the ENOSPC disk wall). The
  touched-binary runs above are the round-2 verification set: only the one test file changed
  (`walkreal_07000006_live.rs`), so per the merge lessons the walks' binary + the level's
  interactables are the tests that use the changed code.

## Live checks for the main agent

1. **The armour soak on guard bullets** — walk armoured Bond into one guard volley and record
   the armour/health words per hit: the port splits via the pain-type switch [M Player_HandlePain
   0x1902c8:84-104 — bullet type 0 absorbs], and the round-2 trace arithmetic (9 zero-delta
   recorded hits ≈ 49.5 armour, then the first health-delta hit at tick 6983) matches full
   absorb-while-armour > 0. A PCSX2 watch on BL+0x894/+0x8B0 across one volley settles it.
2. **#23's throw cycle at the corner** — park Bond at the corner (14.0, 41.9, −36.7) with #23
   alive at his perch and record when the first grenade leaves (the port's measured first throw
   came at frame 8566, ~7100 ticks after the load; the throw cadence in the fight ledgers is
   ~90 ticks once started). If the game's first throw is also minutes late, the corner answer has
   all the time the grind needs once the pool carries it.
3. **#11's line openings from the strip approach vs the sight edge** — the port measured the
   0.57-rad strip line landing (13.4 → 2.4) and the 0.51-rad sight-edge line blocked (0 of 10):
   a gdbwatch trace on #11's box from both spots for ~600 ticks shows whether the tunnel arch's
   top face is the blocker (it decides whether any closer western spot could carry the tower
   duel).

## Usage

- Input/output tokens, context used, cost: not visible to me. The session's environment shows no
  token or cost figures; the WORKLOG entries for this task are the progress record.
