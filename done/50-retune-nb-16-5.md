## Your task: `retune-nb-16-5`, 16 chasers-defeat 00 Agent real-damage walk green again (round 5)

Branch from current origin/master; if origin/s/retune-nb-16-4 is not on master, merge it in first. Read docs/reports/retune-nb-16-4.md there and the ignore message on walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent. Master changed the guards again (SniperAim ENTER, 07:28), so first re-measure the death on the merged tree. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000016_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-16-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-19T16:51Z

---
## Report (done, 2026-11-20T01:12Z)


---
## Report from the worker (done, 2026-11-20T01:12Z)

# retune-nb-16-5 — 16 chasers-defeat 00 Agent real-damage walk, round 5: GREEN

## What was done

Round 5 of `walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent`
(branch `s/retune-nb-16-5`, from origin/master ccd79b5be — SniperAim ENTER, 07:28 — merged with
`origin/s/retune-nb-16-4` b17f0a6c9, which was not yet on master). The task said to re-measure
the death on the merged tree first, because master changed the guards again.

**The re-measure: the round-4 death reproduces byte-exact.** On the merged tree the walk died at
**t17282** at (−12.62,−21.97,−169.69) with armour 0.0 — the ledger's tick, the ledger's spot.
NF_DMG_TRACE: ONE −19.50 round from squad clone #12 (skin 0x500001e, weapon 0x14, hp 100) at
d 1.4, health 19.5 entering the squad stage, 37 hits for 100.0. The SniperAim ENTER change did
not move this fight; lever (a) from the ignore message stayed the next untried one.

**Lever (a): the east armour #649 fetch — judged on its own real run.** The pit's second
BodyArmourHeavy #649 (21.62,−22.97,−168.83) [D census m63 #649, the same params row as #648:
kind 0x3, 50] stands 36 u east of the blast room; kind 3 refuses only at pool ≥ 50
[M Pickup_Handler 0x18e650:220-221] and sets the pool flat 50.0 [M 0x18e650:220-224], so the
emptied pool takes it. The fetch (all `difficulty >= 3 && !tops_up()`, file-local to
`walkreal_07000016_live.rs`) runs **after the kill loop, while the pit squad does not exist yet**
(ch 242 wakes spawner #640 @7332 [D]; the squad's own placements #641..#644 sit at z −149..−167,
north/east of the z −169 route [D census]) **and before the pulse-2 leaves meet at 6660**
[D script 1350] — a player straps the spare armour on while the pit is quiet, then shelters for
the pulse. Return fire stays off for the legs (the room loot hop's dash idiom); the spot is
probed with the standing capsule and skipped when a live volume claims it (the loot-hop idiom;
measured `claimed false`). Measured: the round trip lands **zero landed rounds** (the pit
shooters are the y −7.63 ledge class [D census #696..#698], their rounds grazes), the take sets
**armour 10.0 → 50.0** at stage +4619 with health 19.5 untouched, and Bond is back in the room at
+5291 — the shelter assert (`back_at < 6660`) holds with ~1370 ticks to spare.

The fetch alone moved the death **t17282 → t17560**, and the trace named the next blocker: the
squad loop picked the FIRST member present (#13, ch 0x19, at its (11.24,−157.23) spawn, 28.8 u)
and — because `shoot_bounded`'s blocked arm pushes Bond forward [M walker.rs] — chased the
unexposed member ~15 u east of the room for its whole 10800-tick budget while the closers
(#12 hp 100, #14 skin 0x50000b1 hp 10) drained the fetched pool 50.0 → 0.0 and the last 19.5
health in 120 ticks (−2.50/−5.00/−4.50 from #12 at d 1.2, inside the 3.0 always-hit band
[M 0x179740:113-115]).

**Two squad-stage laws (same gate) made the fetched pool survivable:**
1. **The member pick prefers an EXPOSED member** (the same eye-ray the threat answer runs
   [M walker.rs `DamageLog::visible`]); when none is exposed it waits in real ticks (bounded
   2400) and falls back to the nearest member (the old law — the d1/d2/tops-up sisters keep the
   old pick exactly).
2. **The slice loop does NOT chase a far unexposed member** (far > 6.0 u, no exposed aim): it
   holds plain ticks (`wait_ticks`, the tick_until idiom) instead of the chase, and lets the
   members close into the room's door area, where the existing close stands (120-tick, inside
   10 u) answer them. The first wait attempt used `hold_the_band(300)` and measured **death
   t17519** — `hold_the_band` steps ZERO ticks when no hostile stands inside 3.0 [M hold_the_band:
   `let Some(mi) = close else { break }`], so the 36 wait-slices collapsed into the fallback's
   instant 1800-tick chase and the closers drained the pool again. The plain-tick wait is the
   fix. The d1/d2/tops-up sisters keep the chase (their streams are byte-identical).

**Measured green run (NF_DMG_TRACE, --nocapture):** the fetch 10.0 → 50.0 at +4619, the shelter
+5291, the squad phase picked #12 (death ch 0x1a — the skin 0x500001e clone, now closing to the
room) once it was close/exposed, **the squad-phase audit 2 hits, every delta within the game's
chain** [D record 99; M 0x1d4424/0x139fac], ch 0xbf at +8428, trigger #286, ch 0xc0 true, the
lift-out hand-back at +9490, the button, **end health 17.0** (armour pool spent across the
squad's fight — 37 hits for 83.0 across the whole walk, 9 return-fire fights). The mission
completes on real damage, real input only — no top-ups, no damage lowered, no health/ammo/channel
writes, no guard code changed.

The test is **un-ignored**; the GREEN note above it carries the measured round (the round-4
ledger's text retired with it).

## How it was verified

- The re-measure ran first on the merged tree (the task's own order): t17282 byte-exact.
- Lever (a) alone: the fetch run (death t17560) — the fetch's mechanics proven on its own run
  (the take, the timings, zero landed rounds, the shelter assert) before the squad laws were
  added; then the two squad-law iterations (t17519 with the 0-tick hold, then green) — each
  judged on its own real run, each trace kept in the session (not committed).
- **The whole `walkreal_07000016_live` binary: 19 passed / 0 failed / 0 ignored** (171.45 s,
  `--test-threads=1`, NF_ISO the onfoot ISO) on the final tree — the d1/d2/tops-up sisters and
  every replay byte-identical, the gates hold.
- The diff touches ONLY `crates/nf-view/tests/walkreal_07000016_live.rs` (+ the WORKLOG and this
  report): no `crates/*/src/` files, no `crates/nf-view/tests/common/` — so no other crate's or
  level's tests are affected by the diff (rule 10's affected set is this one binary; rule 7 does
  not trigger — nothing under `common/` changed).
- **The full four-crate suite did not fit.** `tools/test/suite.py` was started in the first
  minutes per the task but this sandbox kills background jobs between tool calls (it died with an
  empty `suite.out` before the build finished — the same outcome the round-4 report documented).
  The foreground all-binaries prebuild (`cargo test --no-run` over the four crates) then hit the
  sandbox's 10 GB disk wall at 93% (the ISO + target + toolchain) in two 10-minute chunks; the
  registry caches and `work/` were freed (1.4 GB recovered) but the remaining build + run did not
  fit the window. This is the same outcome the round-1 and round-4 reports documented. What ran:
  the whole `walkreal_07000016_live` binary above — the affected set per rule 10.

## What's left

Nothing for this task — the walk is green and un-ignored. Notes for future rounds, measured on
this tree: the w-0x15 tag in the room loot scan still costs its 66-tick hop (it can never take —
the round-4 suggestion stands); the fetched pool ends the run at 0.0 with health 17.0 (a
~-3-margin finish — the squad laws are what keep it alive, not spare armour); the full suite
should be run on this branch at merge time on a machine with room for the build.

## Usage

not visible to me — this session shows no token, credit or context-window figures (no meter in
the environment).
