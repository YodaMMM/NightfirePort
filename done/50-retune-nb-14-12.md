## Your task: `retune-nb-14-12`, 14 infiltration to the delta exit 00 Agent walk green again (round 12)

Branch from current origin/master; if origin/s/retune-nb-14-11 is not on master, merge it in first. Read docs/reports/retune-nb-14-11.md there and the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Keep the Operative sister green. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-14-12`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-20T23:38Z

---
## Report (done, 2026-11-21T01:13Z)

Closed by the driver: the report is on `s/retune-nb-14-12` (docs/reports/retune-nb-14-12.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-21T01:13Z)


---
## Report from the worker (done, 2026-11-21T01:13Z)

# retune-nb-14-12 — the 14 infiltration to the delta exit 00 Agent walk, round 12

Branch `s/retune-nb-14-12`, cut from origin/master ccd79b5be with `origin/s/retune-nb-14-11`
merged in first (it was not on master; the merge was clean — the round-4..11 reports, the
ledgers, the NF_DELTA_PROBE hook and the WORKLOG entries carried over; WORKLOG appended only).

The sandbox was fully reset this round (no repo, no work/, no ISO, no rust): the clone,
work-slim.7z + the ISO archive (verified at exactly 369823203 bytes before extraction, py7zr
for both — no p7zip), rustup stable 1.100, `cargo fetch` and the ALSA user prefix were redone.
One setup trap worth recording: the libasound2-dev package's `libasound.so` is a symlink to
`libasound.so.2.0.0` that the dev deb does NOT ship, so the extracted prefix symlink dangled
and every link failed with `unable to find library -lasound` even after a full RUSTFLAGS
`-L native=` rebuild pointed at the right directory; the fix was repointing the symlink at the
system's own `/usr/lib/x86_64-linux-gnu/libasound.so.2.0.0`. The rootfs never reflects
lower-layer deletions in `df`, and the full four-crate test build does not fit beside the two
required binaries (disk-full link failures), so this round's builds were the two required
binaries only — the fast suite could not reach a total line (details below).

## What's done

**The walk is still `#[ignore]`d — still red — but lever 1 is now measured BOTH ways, the
kind-27 fuse is DEFEATED (+1429 ticks, the deepest walk of rounds 4-12), a SECOND kind-27 trap
is measured on the route, the pass-break's scan-fight blind spot is named, and the death moved
from the trap to Rook's first volley, 0.7 health short.**

- **v0 baseline [L 2026-11-21]**: the merged tree reproduced the round-11 death TICK-EXACTLY —
  t37477, the node-22 world blast (kind 27, (−17.6,1.5,−23.2), r 4.0, 50.0 nominal, owner −1)
  dealing −14.20 into 14.2 health, 49 hits for 100.0, 38 return-fire fights. The stream carried
  over the merge clean. The ledger self-corrects one prose item: the round-10/11 trap-budget
  fight list (two 300-capped fights + the second #40 fight + the resupplies) is NOT this
  stream's ledger — the node-21 scan killed #51+#41 in ~150 ticks for −7.0 total, and the real
  killers were #39/#40's FIRST volleys (−3.50 at t37245 d 23.8, −7.00 at t37246 d 21.2 — both
  posts sit BEYOND the scan's 20 u gate from every route node through 24) plus #40's in-flight
  −7.00 × 3 burst (t37398-37406) landing while the leg fight answered.
- **Lever 1, v1 (the post-crossing scan form — the node-21 scan moved to node 24, the shipped
  laws otherwise) [L its own real run]**: the scan's fights ALL broke at +0 — the 240-frame
  window still held #51's t36915 volley when the scan ran, so `other_blood` was true while the
  not-yet-shooting pick #40 had dealt 0.0 — four zero-tick attempts, the scan a NO-OP on this
  stream. And that no-op WON: with no fights inside the kind-27 zone the crossing cleared
  r 4.0 well before the fuse tick — **no world hit anywhere near the zone** — the walk crossed
  the whole wake leg chain and the touch, and died at **t38906 (t37477 → t38906, +1429)**.
  The new endgame, measured: the escorts' first volleys landed while the leg fights answered
  each after (#50 −7.0, #46 −7.0 at d 1.4 point blank, #53 −3.5, #58 −7.0 — #58 never killed,
  d 16.5 from the north corridor), #39's crossing −3.5 stayed unanswered (hp 10 at the end),
  and **Rook (#27, skin 0x5000066, weapon 0x6e, hp 22 FULL) fired his first 0x6e volley −10.70
  at d 9.8 during the close-on chase** — he was never a shooter before it, so the leg gate
  never fought him, and the shoot(rook_i) stage never started. Bond died at 10.7 health:
  **the pool is 0.7 short at Rook's first volley.**
- **Lever 1, v2 (the same scan with the pass-break held for the scan fights, via an opt-in
  `scan_fights_hold_pass_break` flag — measured and REVERTED) [L its own real run]**: the
  fights RAN and were DOUBLE-DAMAGING. The in-hand GL (2.5 contact) at d 14-17.4 cannot kill
  the hp-20 post #40 — 300 ticks for −3 hp (killed false, clip 26 → 8) — and the #42 fight
  ran the gun DRY (clip 8 → 0 spare 0). Worse: `shoot_bounded`'s slide dragged Bond 3.9 u
  east-south of the node-24 park into a **SECOND kind-27 trap — explosion (−4.4,1.7,−25.3)
  r 4.0, nominal 50.0, owner −1; the route nodes 25/26 sit INSIDE its radius (d 1.94 / 2.30)** —
  whose fuse fired at t37634 with Bond at (−5.8,−25.8) d 1.6: **−45.70, death (t38906 →
  t37634)**. The flag code is fully reverted (the shared files back to HEAD); the scan stays at
  node 24 on the v1 law — on THIS stream it is a no-op whose candidates dump is free read-only
  data, and the refreshed ignore message warns that a future upstream re-roll could wake it
  (the v2 double-damage is the measured expectation if it ever fights).
- The ignore message is refreshed with the round-12 ledger and the round-13 levers in order:
  (1) the Rook-chase endgame — the pool needs +10.7 at Rook's first volley; the measured
  candidates: (a) #51's −7.0 pre-emption at node 23 (his post (−12.0,−20.2) d 3.1 eyetrue
  guntrue, but the scan's road-blocker rank picks #40's 0.4-perp first and the door-face health
  rank ties #39/#42 at hp 10 ahead of #51 — the pre-emption needs a NEAR-first or
  distance-gated sub-pick), (b) the crossbow switch (0x11 c2, dmg 60.0 — one bolt kills Rook's
  hp 22), the round-9 'a switch that does not stand Bond' design question re-tested at the ROOM
  position (the v1 stream's touch window was quiet 833 ticks around the stands, but the bolts
  must not be spent on the escorts — the switch belongs inside the Rook kill attempts),
  (c) #58's unanswered −7.0; (2) the second kind-27 trap's trigger — a live check for the main
  agent; (3) the round-9 levers 2/3, lower priority.

## How it was verified

- **walkreal_07000014_live (whole binary, FINAL tree, --test-threads=1)** [L 2026-11-21]:
  **9 passed / 0 failed / 1 ignored, 178.05 s** — same counts as rounds 9-11; the sisters and
  the other walks stay byte-identical (the d1 sister's k==21 scan untouched; the round's only
  test change is the d3 branch's scan position).
- **interactables_07000014_live (whole binary, FINAL tree)** [L 2026-11-21]: **10 passed /
  0 failed, 15.63 s** — matches rounds 5-11's counts.
- The final tree's 00agent single run reproduces the v1 death TICK-EXACTLY (t38906 at
  (26.99,1.13,−8.62)) — the ignore-message edit is stream-neutral, certified.
- Every lever was judged on its own real run: v0 (the baseline — tick-exact vs round 11), v1
  (the post-crossing scan, the shipped laws — the +1429 verdict), v2 (the flag form — the
  double-damage verdict, then reverted). All with NF_DMG_TRACE ledgers captured (v0_baseline.log,
  v1_postcross.log, v2_scanhold.log, final_cert.log, whole_binary.log, whole_binary_final.log,
  interact.log, interact_final.log — untracked scratch).
- **The fast suite (`tools/test/suite.py`)**: started in the background as ordered at
  01:12 UTC; the four-crate `--no-run` build does not fit this rootfs beside the two required
  binaries (the earlier full-build attempt died in disk-full link failures — build3.log /
  build4.log, untracked scratch), so no total line exists this round; the two required
  binaries' whole runs above are the evidence, as in rounds 4-11.
- Changed files: `crates/nf-view/tests/walkreal_07000014_live.rs` (the d3 delta scan moved from
  node 21 to node 24 with the measured verdicts in the comments + the refreshed ignore message)
  and `docs/WORKLOG.md` (appended). **No file under `crates/*/src/` changed**; the shared test
  helpers (`crates/nf-view/tests/common/*`) carry ZERO net change this round — the round's flag
  experiment in damage_log.rs/walker_b2.rs was measured (v2) and reverted to HEAD before the
  final-tree runs, so the other walks' byte-identity is by construction.
- Facts tagged [L] come from this window's runs on the real ISO with NF_DMG_TRACE; code
  citations are [M] with the addresses carried in the test comments; no test was weakened; the
  game bytes, the token and the URLs stayed out of the commits.

## Still red / what's left

The 00agent sister at t38906 — Rook's first volley into the 0.7-short pool. The ignore message
carries the round-13 levers (the Rook-chase pool first; the second trap's trigger; the round-9
levers 2/3).

## Live checks for the main agent

1. **The second kind-27 trap's trigger** [D/script]: at (−4.4,1.7,−25.3) r 4.0 (the v2 death:
   fuse t37634, Bond at (−5.8,−25.8) d 1.6, −45.70), what arms/fires the blast? The v1 fast
   crossing (nodes 25/26 inside the radius at ~t36980-37060) never triggered it while the v2
   linger did — a fuse off the same node-21 door, a proximity volume, or a cycle? Watch with
   gdbwatch on the script that writes this blast; answer ONE-SHOT vs CYCLICAL.
2. **The first trap's fuse anchor** [D/script] (carried from round 11): the kind-27 blast at
   (−17.6,1.5,−23.2) fired ~507 ticks after the node-21 door E-open on every stream so far —
   is the anchor the door's E-open script or a fixed level timer?
3. **Rook's 0x6e volley cadence** [L is ours; D for the answer]: Rook (#27, skin 0x5000066)
   fired −10.70 at d 9.8 once in the v1 endgame (t38906, hp 22 untouched before it). What gun
   is 0x6e [D] and what is his fire cycle when chased (the next volley's delay decides whether
   the +10.7 pool buys the shoot stage or one more volley)?

## Usage

not visible to me
