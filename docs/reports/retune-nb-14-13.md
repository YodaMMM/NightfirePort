# retune-nb-14-13 — the 14 infiltration to the delta exit 00 Agent walk, round 13

Branch `s/retune-nb-14-13`, cut from origin/master 502f62fae with `origin/s/retune-nb-14-12`
merged in first (it was not on master; the merge was clean — the round-4..12 reports, the
ledgers, the probe hooks and the WORKLOG entries carried over; WORKLOG appended only).

The sandbox was fully reset this round (no repo, no work/, no ISO, no rust): the clone,
work-slim.7z + the ISO archive (verified at exactly 369823203 bytes before extraction, py7zr
for both), rustup stable 1.100, `cargo fetch` and the ALSA user prefix were redone. The ALSA
prefix from the earlier session was complete this time (the `libasound.so` symlink resolved —
the round-12 dangling-symlink trap did not recur).

## What's done

**The walk is still `#[ignore]`d — still red — but the round-13 lever (1(b), the crossbow
switch at the room position) is now MEASURED DEAD SEVEN WAYS on its own real runs, the run
discovered a BLIND POCKET on the chase route (a stop spot where every guard reads
eyefalse/gunfalse and nobody shoots), and the ignore message carries the round-14 levers with
the pocket-bombardment completion design.**

- **v0 baseline [L 2026-11-21]**: the merged tree reproduced the round-12 death TICK-EXACTLY —
  t38906 at (26.99,1.13,−8.62), Rook's first 0x6e volley −10.70 at d 9.8 into the 10.7-health
  pool, 58 hits for 100.0, 48 return-fire fights — the stream carried over the merge clean.
  New facts: the attempt-loop aim prints read Rook at hp 30 (the round-12 ledger's 22 was its
  own stream's wake-time value), and the wake-legs loadout dump holds the crossbow at
  0x11(c2+0 dmg60.0) — two bolts, no spare.
- **Lever 1(b), each form its own real run, the tree reverted to the shipped state after
  each [L 2026-11-21]**:
  - **v1** (the press AFTER the shipped close-on): the close-on's own stretch is the death's
    home — the walk died at t38906 tick-exact BEFORE the press ran. And a press
    BEFORE/DURING the chase hands the armed crossbow to the fights (`switch_to_armed` keeps
    an armed gun in hand at every fight start [M walker.rs]) — the bolts die on the escorts,
    the lever's own law.
  - **v2** (the stand-off gate on the route SNAPSHOT): Rook runs south THROUGH the route (his
    volley fired from (27.6,1.2) while the snapshot target stood far north) — the stale gate
    never broke, the run replayed v0 tick-exact.
  - **v3** (the LIVE gate at 14.0, the press at the stop): the aim broke at +0 — the
    went-silent pass-break (walker_b2.rs:754, opt-in on `switch_on_first_blood &&
    finish_fights`, both set by this walk) reads Rook bloodless in the 240-frame window while
    #53's t38595 volley is still inside it — a law built for the legs' determined pass, fatal
    to a first-volley aim. The stand also sat in #57's line ((28.1,2.7) — a guard v0's moving
    Bond never drew): FOUR standing-target volleys while Rook crossed his ~9.8 engage bubble
    ~94 ticks after the stop; the draw (~90, the shipped switch blocks' budget) + the aim
    lost the race — death t38881.
  - **v4** (a 2-tick press): the switch did NOT take — the aim opened with the GL (clip 5
    spare 79) [L the aim's +0 print]: the switch needs the HELD input (the shipped switch
    blocks' own idiom).
  - **v5** (the LIVE gate at 10.0, the HELD press): **the press WORKS** (the aim opened with
    the crossbow, clip 2) — but **both bolts MISSED** (out of ammo at +144, Rook untouched at
    hp 30): the 0.02-rad tap gate is ±0.22 m at d 10.9 vs the ~0.3 m hit boxes. **THE GOLD**:
    the stop spot was a **BLIND POCKET** — the pre-clear census read EVERY guard
    eyefalse/gunfalse (Rook at d 9.9 included) — nobody shot during the whole aim and the
    pool held 21.2.
  - **v6** (the gate at 6.5, to shrink the miss band): walked INTO the ~9.8 volley bubble and
    died at t38906 tick-exact vs v0 — the bubble is real and distance-shaped.
  - **v7** (the POCKET GL BOMBARDMENT — no switch at all, the grenade blast covering the
    ±0.22 band): the aim from THAT pocket spot was BLOCKED (aim None — the ray to Rook's
    boxes clips the room's geometry; the clearance flips on Rook's live pos — v5's spot
    cleared, v7's did not) and the blocked stand's own slide law dragged Bond east into the
    eyes — death t39025. The fights still answered (Rook died hp −22, #53 hp −27) but the
    pool ran out first.
- The ignore message is refreshed with the full round-13 ledger and the ROUND-14 LEVERS in
  order: (1) the POOL at Rook's first volley (the deficit 0.1-7.0): (a) #58's pre-emption
  (his −7.0 burst at d 15.7 — the chase's fights never reached him; the round-13 lever 1(c)
  carried over), (b) #57's pre-emption (the v3 discovery — his standing-target volleys eat
  any stop in his line, the shoot-stage stand included), (c) THE BLIND-POCKET BOMBARDMENT
  COMPLETION — at the LIVE-10.0 stand-off break, read `exposed_aim(rook)` BEFORE standing (a
  read-only visibility check, zero draws): bombard with the GL where the aim is Some (the
  blast covers the ±0.22 tap band; the v5 pocket measured blind and quiet, pool 21.2), step
  ONE more nav leg and re-read where None — never STAND in a blocked spot (the slide law
  walks Bond into the eyes). (2) the second kind-27 trap's trigger — the live check (carried
  from round 12). (3) the round-9 levers 2/3 (carried, lower priority).

## How it was verified

- **walkreal_07000014_live (whole binary, FINAL tree — the shipped state, `--test-threads=1`)**
  [L 2026-11-21]: **9 passed / 0 failed / 1 ignored, 179.07 s** — the same counts as rounds
  9-12; the sisters and the other walks stay byte-identical.
- **interactables_07000014_live (whole binary, FINAL tree)** [L 2026-11-21]: **10 passed /
  0 failed, 13.17 s** — matches rounds 5-12's counts.
- The final tree's 00agent single run reproduces the v0 death TICK-EXACTLY (t38906 at
  (26.99,1.13,−8.62)) — the ignore-message edit is stream-neutral, certified.
- Every lever form was judged on its own real run (v0 plus v1-v7, nine runs total), all with
  NF_DMG_TRACE (and NF_KILL_TRACE where the kill path mattered) ledgers captured
  (v0_base13.log, lever1b_v1.log … lever1b_v7.log, final_cert13.log, whole_binary13.log,
  interact13.log — untracked scratch outside the repo).
- Changed files: `crates/nf-view/tests/walkreal_07000014_live.rs` (the refreshed ignore
  message only — the lever code was measured and REVERTED; the final file differs from the
  round-12 state in the ignore attribute text alone) and `docs/WORKLOG.md` (appended).
  **No file under `crates/*/src/` changed; the shared test helpers
  (`crates/nf-view/tests/common/*`) carry ZERO change this round** — the other walks'
  byte-identity is by construction, and the whole-binary counts above match rounds 9-12.
- The fast suite (`tools/test/suite.py`) was started in the background on the final tree per
  the task order; its tail is read before the final push (the round-12 disk lesson stands:
  the four-crate test build may not fit this rootfs — if no total line exists, the two
  required binaries' whole runs above are the evidence, as in rounds 4-12).
- Facts tagged [L] come from this window's runs on the real ISO with the NF traces; code
  citations are [M] with the addresses in the test comments; no test was weakened; the game
  bytes, the token and the URLs stayed out of the commits.

## Still red / what's left

The 00agent sister at t38906 — Rook's first volley into the 0.7-short pool. The ignore
message carries the round-14 levers: the pool (the #58/#57 pre-emptions, then the
blind-pocket bombardment completion), the second trap's trigger (the live check), the
round-9 levers 2/3.

## Live checks for the main agent

1. **The second kind-27 trap's trigger** [D/script] (carried from round 12): at
   (−4.4,1.7,−25.3) r 4.0, what arms/fires the blast — answer ONE-SHOT vs CYCLICAL.
2. **Rook's 0x6e volley cadence when chased** [L is ours; D for the answer] (carried): his
   first volley fired at d 9.8 (t38906 on the v0 clock); the next volley's delay decides
   whether a +10.7 pool buys the shoot stage or one more volley. The v7 run adds: his 0x6e
   volleys read −3.74 and −6.96 at d 9.7 — the per-round damage varies (the accuracy law?),
   not the single −10.70 of the v0 death.
3. **NEW — the pocket's sight geometry** [D/script]: at the chase's stand-off spots
   (Bond ~(27.26,−9.97)), the v5 census read EVERY guard eyefalse/gunfalse while the v7 spot
   (12.3 u off Rook's live pos) read the aim ray BLOCKED on the room's geometry. What
   script/geometry owns the room-mouth sight line — can a pocket spot be named where the GL
   aim ray to Rook's boxes clears AND every guard stays blind?

## Usage

not visible to me
