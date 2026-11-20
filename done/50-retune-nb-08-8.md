## Your task: `retune-nb-08-8`, 08 whole-mission 00 Agent real-damage walk green again (round 8)

Branch from current origin/master; if origin/s/retune-nb-08-7 is not on master, merge it in first. Read docs/reports/retune-nb-08-7.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-08-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-20T03:18Z

---
## Report (done, 2026-11-20T04:44Z)


---
## Report (done, 2026-11-20T04:58Z)

# retune-nb-08-8 — 08 whole-mission 00-Agent real-damage walk, round 8

Branch `s/retune-nb-08-8`. `origin/s/retune-nb-08-7` was not on master, so it was merged in
first (clean: the walk test + docs only). Still red — the shipped timeline reproduces exactly,
and the window's lever (the ledger's standing "idle-wait sweep") was executed as the crouch
sweep and measured DEAD on both of its own real runs; both halves reverted with the
measurements recorded.

## What was done

1. **The re-measure on the merged tree** (origin/master + 08-7; no `crates/*/src/` changes
   this window): the walk dies t22649 at (111.0, 13.5, −180.9) to the −6.06 world row — round
   7's ledger tick — with the fight-entry dump armour 5.0 / health 20.0 [L the 2026-11-20
   baseline NF_DMG_TRACE run].
2. **The upstream bleed is now fully attributed** [L the same run, per-owner aggregation]:
   - the y-46 trio 36.0 (#2 8.0/2 + #14 16.0/4 + #11 12.0/3) — ALL at the wedged nav pin
     (149.5-149.6, 46.1, −93.1) t738-1009, while the DRY-PISTOL return-fire stands (0x1 clip
     1 spare 0) ground on #3/#2/#11 — #14 was never picked by any stand;
   - #16 28.0 in 5 (t5168-5583) — the pit-excursion descent legs (the pair's stands fail
     twice each before the ramp mid opens the line, `killed false` t5044/t5384);
   - #9/#10/#4 16.0 (t1549-6331); world 20.0 in 3 in the Hind fight (t21527-22649);
   - the secure-room armour 50 soaks EVERY hit t6331-21527 (health flat 20.0) and enters the
     fight at 5.0.
3. **Lever (1) the idle-wait sweep, both halves tried and dead on their own real runs** (the
   shipped crouch-sweep precedent — the hall crossing t26671→t29598, the briefcase stretch
   t29598→t34720 — does not extend):
   - (1a) the crouch press before the east-deck-head nav: the crouched crossing diverged the
     approach (the fight armed 48 rounds vs the baseline 34) and the deck-line west walk_to
     wedged at the closed hall door (143.45, 49.55, −97.30) head_bump grounded=false for the
     6000-tick cap — ZERO progress [L the lever-1a run, 15.8 s];
   - (1b) the crouch press before the pit-excursion descent legs: the crouched eye dropped
     INTO the pit pair's line (the ramp west edge no longer blocks) and the walk died ON THE
     RAMP t6918 to #15's burst at (153.2, 40.1, −113.0) — was t22649 [L the lever-1b run,
     7.2 s].
   Both reverted; the bleed needs fight-geometry answers, not stance answers. The final tree
   keeps only the dead-lever comments and the refreshed ledger in the ignore message.

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk (NF_DMG_TRACE, 15.8 s /
  7.2 s / 18.9 s baseline) [L the three runs above].
- `walkreal_07000008_live` whole binary on the final tree: **6 passed / 0 failed / 1
  ignored** in 152.57 s — the d1/d2 real-damage sisters, the plain walks and both ally tests
  stay byte-exact green (the final tree's only code delta vs the merged 08-7 state is
  comments).
- Only `crates/nf-view/tests/walkreal_07000008_live.rs` and `docs/` changed — no
  `crates/*/src/`, no `crates/nf-view/tests/common/` files — no other test binary compiles
  the changed file, so no other binary's timeline can move.
- **The full suite (tools/test/suite.py) did NOT fit the window, for an environment reason:**
  this sandbox reset to a bare machine (clone, toolchain, game files ≈ 45 min with the
  alsa-sys/pkg-config and 3 GB-RAM linker failures to solve), the background suite process is
  killed between tool calls in this sandbox (it died twice, at 44 MB and 1.8 GB of target),
  and the rootfs has 9.9 GB total — after building the four crates' libraries and the one
  required test binary there were ~656 MB free, not enough for the ~200 test binaries the
  full suite links. What ran: the required binary whole (above), the changed file's own
  target only. No src/ or tests/common/ changes exist on this branch, so per the merge
  lessons the affected-binary set is `walkreal_07000008_live` alone.

## What's left (the next window's levers, in priority order)

1. **The deck pin 36.0** — the wedge + the stands run INSIDE the trio's crossfire at
   (149.6, −93.1); a player would fight from outside it: pull the return-fire stand spots
   2-3 u west (the deck's west lip), or the main agent's live check for a cover spot that
   blocks the trio's lanes (the placements (152.5-161.5, 46.0, −87.7..−101.5) all fire down
   the deck line the nav wedged on).
2. **#16's 28.0 on the excursion legs** — try the descent with the gun ALREADY raised (the
   aim press before the legs so the first visible tick fires [S] unverified), or the live
   check for which leg first opens #15/#16 and whether an earlier leg can.
3. **The fight-entry armour 5.0** — the ledge armour box (109.5, 27.8, −129.4) is taken
   AFTER the platform-mouth fights; check whether the box can be taken BEFORE them (the
   order swap needs its own real run; the soak that costs 45.0 is the platform/boarding
   chunk #19/#20/#23/#24/#26).

## Live checks for the main agent

- During the station deck approach (Bond crossing from the hall mouth toward the east deck
  head, t~700-1200 of the walk): a sight/cover probe of the deck at standing and crouched
  eye — which standing spot between (148, 46.1, −98.5) and (152.2, 46.0, −93.9) is out of
  the trio's lanes (#2 (161.6, 46.0, −97.4), #14 (152.4, 46.0, −96.2), #11 (154.3, 46.0,
  −87.6))? The walk must fight them from a spot their bursts cannot reach; the crouch is
  measured useless here (the lever-1a run).
- On the pit excursion (the descent legs from the landing east to the ramp mid, t~5000-5600):
  which leg first opens sight to #15/#16 (at (146.3-148.1, 38.0-39.2, −110.5..−111.8)), and
  is any EARLIER leg's spot a firing position once the aim is pre-raised? The crouch is
  measured harmful (the lever-1b run).

## Usage

not visible to me
