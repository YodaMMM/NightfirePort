## Your task: `retune-nb-13-2`, 13 warm chain to the front entrance real-damage walk green again (round 2)

Branch from current origin/master; if origin/s/retune-nb-13 is not on master, merge it in first. Read docs/reports/retune-nb-13.md there and the ignore message on walk_the_warm_chain_to_the_front_entrance_and_out_real_damage. The chain now brings 54 health and 50 armour into 13. Take the report's first What's-left item (the pool lever: win the fights with rounds). Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000013_live binary and interactables_07000013_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-13-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-18T20:15Z

---
## Report (done, 2026-11-18T21:18Z)


# retune-nb-13-2 — the pool lever (round 2 of the 13 warm-chain real-damage walk)

Branch: `s/retune-nb-13-2`, from current `origin/master` with `origin/s/retune-nb-13` merged in
first (fast-forward — nb-13 branched from the same master tip 73822d3a0). Task: the nb-13
report's first What's-left item — **the pool lever: win the fights with rounds** — for
`walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` (`walkreal_07000013_live`, 00
Agent d3, real damage). Real input only, no top-ups, never lowered damage or health, never
changed guard code; every helper change gated to the level and the difficulty.

## Result: still red, substantially further along (death t4849 → t5075, a different failure surface)

The pool lever **landed and is proven**: the re-anchor fill takes the lobby's ammo pool from 17
rounds to 71 and with rounds the fights won every measured floor exchange. The recorded death
(the dry gun, t4849) is gone. What kills the walk now is a different, precisely measured thing:
after the west-mid answers dry the held clip, the fights' `switch_to_gun` cycles to the wheel's
SPAS12 (8 shells, no spare) instead of the pool's deeper guns, and that stand loses to the wave
rifle. The test stays `#[ignore]` with the refreshed ledger and the next lever (the task's law:
red ⇒ ledger + next lever into the ignore message).

## What was done, each judged on its own real run (all [L] = this session's runs, byte-identical repro first)

1. **Repro:** the recorded ledger death reproduced byte-identical on the merged tree — t4849 at
   (−13.41, −368.30, 39.52), clip 0 spare 0, the fetches empty [L the repro run, 24 s wall]. The
   sim runs ~1000 ticks/s, so each lever judgment is a real ~30 s run.
2. **The pool lever (the task's item, landed):** a d3-gated re-anchor fill — up to three
   there-and-back errands (`resupply` with `resupplying` held, the helper's own idiom) while the
   **held** clip+spare < 30, the deep-pocket corner (4.9, −369.4, 22.2 [L the margin-6 run-5/6
   ledgers]) pushed into `no_fetch`, and `resupply_home` now set on 13's d3 walker (the field
   doc's own lever; the measured re-anchor fetch had left Bond 2.4 u off the leg line and the
   guard-repin ledger measured pocket displacement breaking the west-mid line). Measured: pool
   17 → 71 rounds; #4 (the mezzanine-east rifle) killed true mid-fill at t4241 zero-cost, the
   west-mid answers killed #6 (11 rounds), #11 and #10 (7 and 4 rounds) — every exchange the
   ledger's law promised ("with rounds the fights win EVERY measured exchange") [L levers 2/3/5].
3. **The mezzanine law (the levers 2-4 runs taught it):** the plain fight machinery measured dead
   on the elevated guards twice — the unfiltered nearest answer sprayed the whole refilled clip
   at the mezzanine #5 (15 rounds, 0 hits, dxz 13.3, dy 6) and a 900-cap fight on the balcony
   rifle #4 at dxz 37.2 returned killed false costing 42.0 (46.0 → 4.0). Landed, all d3-gated:
   the fights walk on past elevated guards (`level_fights` — the **existing** 07000016 opt-in
   flag, whose ±2.0 filter is exactly this law; only its doc comment was extended, no shared
   code change), the file's d3 sweeps take Bond's floor only (`floor_only` param on the
   file-local `clear_the_visible`; the d2 sister's sweep call passes false and stays
   byte-identical), arm+reload before the aimed-stand watch, and the north-west mezzanine watch
   drain (radius 45, the door drain's own `aimed_stand` machinery — it lands at 40.8 u [L the
   margin-2 note] — one leg earlier, the lever-6 flank's shape).
4. **Measured dead, recorded, not repeated:** (a) the fill's first gate — the inventory-wide
   pool (≥ 40) was already satisfied while the held gun sat at clip 2, death moved EARLIER
   (t4364) [L lever-1]; the gate is the held gun's clip+spare < 30 now. (b) The re-anchor
   sweep's own stand on elevated #4 — killed false, `stop_on_fail` then ended the sweep, and the
   far-stand converged on clip 1 into reload churn — death t4621 [L lever-4]; `floor_only`
   answers it. (c) The ammo-rich pre-arm (a file-local ≥ 10 wheel preference at the west-mid and
   north-west watch ends) measured a NO-OP — the held gun was already at clip 22 at both points,
   byte-identical death t5075 [L lever-6]; the gun mismatch is inside the fights' own
   `switch_to_gun`, after the answers dry the held clip.
5. **The remaining death:** t5075 at (−19.43, −368.30, 37.55) — the west-mid nearest answer on
   the wave rifle #9 (hp 24, 9.6 u) burned 11 pistol rounds and hit out of ammo at +179 (killed
   false, 54.5 → 25.5 under #6's fire), then the north-west leg's fight on the wave rifle #6
   (14.7 u) ran the SPAS12's 8 shells, killed false, and the 900-cap stand ate 46.5 — death
   INSIDE the fight stand, so the north-west watch drain never ran (the leg never arrived).
   **Next lever (in the ignore message):** an opt-in flag (the lesson-7 shape) making
   `switch_to_gun` prefer an owned gun with clip+spare ≥ 10 over the first armed slot; the
   north-west watch drain then runs once the leg arrives, ahead of the door drain.

## Verification

- Byte-identical repro of the recorded death before any change [L].
- Five lever runs, each judged on its own real run, the death line moving t4849 → t4364 →
  t5631 → t5950 → t4621 → t5075 across them (the intermediate steps' ledgers in the commit
  messages and the ignore history) [L].
- `walkreal_07000013_live` (whole, final tree): **3 passed / 0 failed / 1 ignored** — the
  non-real-damage walk and the Agent (d2) / Operative (d1) sisters green. The sisters are
  byte-identical by construction: every lever is inside `real_damage && difficulty == 3` gates
  (the `floor_only=true` sweep args are only on the d3 callers; the d2 call passes false).
- `interactables_07000013_live` (whole, final tree): **10 passed / 0 failed**.
- `walkreal_07000016_live` (the other `level_fights` user) — SIGKILL'd in this sandbox on the
  first attempt (memory pressure while a cargo build ran concurrently); on the second attempt
  the same SIGKILL. The change to the shared file is a **doc comment only** (no code path), so
  its behaviour cannot differ; listed here honestly rather than claimed as run.
- Files touched: `crates/nf-view/tests/walkreal_07000013_live.rs` (all levers), the
  `level_fights` doc comment in `crates/nf-view/tests/common/damage_log.rs`, `docs/WORKLOG.md`
  (appended), this report. No shared helper code changed; `no_fetch`/`resupply_home`/
  `level_fights`/`resupplying` are all pre-existing opt-in surfaces.
- The fast full runner (`tools/test/suite.py`) was started in the first minutes as required and
  again on the final tree; on this 9.9 GB disk with the repo + ISO + target resident it cannot
  finish (the nb-13 report measured the 1112-binary nf-view set needs ~2 GB past what remains).
  The first attempt's build died with ENOSPC; the final-tree attempt's output is `suite.out`
  next to the repo root. The binaries the task names were run whole instead (above), plus the
  level_fights user documented honestly.

## What's left (the next session's first items)

1. The `switch_to_gun` spare-preference flag (the lesson-7 opt-in shape in `walker_b2.rs`), set
   by 13's d3 walk only — the fights then land on the pool's deep guns instead of the wheel's
   dry 0x1c; judge on one real run.
2. Then the north-west watch drain (already in place, gated behind the leg's arrival) runs
   before the door drain; the door legs at ~50 hp are previously proven (the nb-13 report).
3. Un-ignore when green; run the whole `walkreal_07000013_live` + `interactables_07000013_live`
   again; the sisters stay byte-identical (verify by run).

## Live checks for the main agent

- The wave-rifle drops: when a group-1 respawner rifle (skin 0x5000006, weapon 0x18) dies, does
  its gun drop rest on the lobby floor (y −369.4) reachable from the lane, and what weapon-id
  ammo does the wheel's 0x1c SPAS12 actually consume — is the "0x18-type rounds" the fill's
  pass-2 drop (5.67, −369.43, 24.31) carried, and could a fresh-boot player arm it from the
  wheel (the cycle order)?
- The mezzanine respawn cadence: lever-2 measured killed t4241 → first volley t5358 (~1100
  ticks) for the re-fed mezzanine post. Does the group-1 respawner's 32000-wave stream really
  re-feed the mezzanine east end forever, or does the post exhaust?

## Usage

Not visible to me — this session's environment shows no token, credit or context-window
figures to copy.

---
## ntfy failed (2026-11-18T21:20Z)

All three fallbacks timed out from this sandbox: curl (28) Connection timed out after 20002 milliseconds; python3 urllib.request (20 s urlopen timeout elapsed, no connection); wget exit 124 (timeout).
