## Your task: `retune-nb-14-11`, 14 infiltration to the delta exit 00 Agent walk green again (round 11)

Branch from current origin/master; if origin/s/retune-nb-14-10 is not on master, merge it in first. Read docs/reports/retune-nb-14-10.md there and the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Keep the Operative sister green. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-14-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-20T19:38Z

---
## Report (done, 2026-11-20T20:34Z)


---
## Report from the worker (done, 2026-11-20T20:34Z)

# retune-nb-14-11 — the 14 infiltration to the delta exit 00 Agent walk, round 11

Branch `s/retune-nb-14-11`, cut from origin/master ccd79b5be with `origin/s/retune-nb-14-10`
merged in first (it was not on master; the merge was clean — the round-4..10 reports, the
ledgers and the WORKLOG entries carried over; WORKLOG appended only).

The sandbox was fully reset this round (no repo, no work/, no ISO, no rust): the clone, work-slim.7z
(the ISO archive verified at exactly 369823203 bytes before extraction), rustup stable 1.100,
the ALSA user prefix (/tmp/alsa-root, the alsa.pc prefix rewritten, PKG_CONFIG_PATH + RUSTFLAGS
in env.sh) and `cargo fetch` were all redone. py7zr did both extractions (no p7zip on the box).
The two required test binaries built from scratch in 4m14s; an incremental test-crate rebuild is
~5 s.

## What's done

**The walk is still `#[ignore]`d — still red — but both round-10 levers are now measured dead
on their own real runs, the ledger self-corrects one misread, and the death's TRAP GEOMETRY is
measured: the killing world blast is a FUSE anchored to the node-21 door E-open, and the
round-12 lever 1's pre-door form is answered dead by a new read-only probe.**

- **v0 baseline [L 2026-11-20]**: the merged tree reproduced the round-10 death TICK-EXACTLY —
  t37477, (−13.77,1.04,−24.24), the world blast (kind 27, (−17.6,1.5,−23.2), r 4.0, 50.0
  nominal, owner −1) dealing −14.20 at the radius edge, 54 hits for 100.0, 39 return-fire
  fights. The stream carried over the merge clean.
- **Lever 1(b) (the node-21 scan's fast-tap cadence) MEASURED NET-NEGATIVE (v1, its own real
  run)**: the #51 fight (d 6.2, clip 29 spare 0) sprayed 15 rounds (clip 29 → 14) with ZERO
  landed — the door-face family's no-slide park cannot find the live line the shipped
  16-tick+slide fight finds (the shipped fight killed #51 from the same park in one stand) —
  and the round-10 pass-break then broke the dead fight at +120 (other_blood: #41's volley sat
  in the 240-frame window), leaving #51 alive at hp 10 to converge to d 2.9 and deal 45.7 in
  7 hits: death t37477 → t37122 (**−355**). Removed.
- **Lever 1(c) (skip the ≤ 1.0-hp tail at d > 10) MEASURED NO-OP (v2/v3 + a pick-time
  diagnostic)**: its trigger NEVER EXISTS on the measured stream. The round-10 ledger's
  "0.4 remaining" was a `{:.0}`-display misread — "shooting drone 40 at +0 (health 0...)" was
  the determined pass's POST-STAND state; the diagnostic read the pick-time healths: #40 was
  **20.00 at the first fight's pick and 11.60 at the second's**. The −21.0 was spent fighting
  a REAL 11.6-hp shooter (the pass killed him from ~0.4 with the GL switch). The first fight's
  killed-true with 11.6 left is `stand_and_fire`'s return semantics (true = corpse OR health
  ≤ 0 OR dart-asleep [M the helper's own code]) — accurate at the time; something (a script —
  his final hp reads −400) restored #40 afterward. Removed; the code reverted; the ledger
  self-corrects.
- **The trap geometry, measured [L this round]**: the blast fired at t37477, ~507 ticks after
  the node-21 door E-open ((−19.3,1.2,−22.8), ~t36970) — a fuse. The route nodes 20-24 ALL sit
  inside r 4.0 of the centre (node 20 d 2.6; node 21 d 0.57 — ON the centre, the scan's own
  spot; node 22 d 3.93 — the edge; node 23 d 2.2; node 24 d 4.3): the crossing itself spends
  ~300 ticks inside the zone, so the fuse leaves only ~200 ticks of fight budget — the 00agent
  scan (~300) + the second #40 fight (~157) + the resupplies blow it. The d1 sister opens the
  SAME door, stands on the SAME centre, and crosses green with NO world hit anywhere near the
  zone (the v4 sister run) — her stream has no d3 scans and near no fights there. The edge hit
  deals 14.2 — survivable above ~15 health.
- **The round-12 lever 1's first step SHIPPED as the read-only `NF_DELTA_PROBE` hook**
  (env-gated, zero ticks, byte-identical when unset, modeled on NF_SRV_PROBE): at node 20's
  arrival it dumps every live guard's eye/gun visibility. Measured: the posts are #51
  (0x500008c, 0x1a, hp 10.0) at **(−11.87,1.00,−20.11)** d 8.2 and #41 (0x500008c, 0x1a, hp
  10.0) at **(2.92,1.00,−23.99)** d 22.9 — **BOTH eyefalse gunfalse from the pre-door spot**,
  both EAST of the door leaf — the pre-door scan relocation is dead (a no-op scan buys
  nothing), and the probe run's death reproduced t37477 tick-exactly (the probe is
  stream-neutral; the hook stays for the next rounds).
- The ignore message is refreshed with the round-11 ledger and the round-12 levers in order:
  (1) the trap budget — the walk must clear r 4.0 of (−17.6,−23.2) within ~507 ticks of the
  door E-open; the pre-door relocation is measured dead (above); what remains is the
  POST-crossing scan form (node 24+, d 4.3+, outside the zone — #51/#41's convergence through
  the zone answered only by the shipped fights' first-hit law, the first volleys land — judge
  on its own real run); (2) the fight-stall budget — the first #40 fight's killed-true was
  accurate (a script restore), so the 240-cooldown idea is dead; the remaining shape is the
  fight TICKS themselves (the second #40 fight's ~157-tick GL-switch stand inside the zone);
  (3) the round-9 levers 2/3 (the door-face bank drain, the lane-crossing wedge walk_nav_to)
  stand untried, lower priority.

## How it was verified

- **walkreal_07000014_live (whole binary, FINAL tree, --test-threads=1)** [L 2026-11-20]:
  **9 passed / 0 failed / 1 ignored, 172.01 s** — same counts as rounds 9-10. The ignored one
  is the 00agent sister (its refreshed ledger in the ignore message).
- **interactables_07000014_live (whole binary, FINAL tree)** [L 2026-11-20]: **10 passed /
  0 failed, 15.49 s** — matches rounds 5-10's counts.
- Every lever was judged on its own real run: v0 (the baseline — tick-exact vs round 10), v1
  (the fast-tap lever — the −355 verdict), v2/v3 (the tail-skip + the pick-time diagnostic —
  the no-op verdict), v4 (the d1 sister's crossing — the trap-free comparison), v5 (the
  NF_DELTA_PROBE run — the sightline answer + the stream-neutral proof). All with
  NF_DMG_TRACE ledgers captured (v0_baseline.log, v1_fasttap.log, v2_tailskip.log,
  v3_tailskip_trace.log, v4_d1_sister.log, v5_probe.log, whole_binary.log, whole_binary2.log,
  interact.log, interact2.log — untracked scratch).
- **The fast suite (`tools/test/suite.py`)**: started in the background in the first minutes
  as ordered (`NF_ISO=… nohup python3 tools/test/suite.py > suite.out 2>&1 &`); it was killed
  between tool calls before suite.out wrote a line (the same background-kills failure mode
  rounds 4-10 recorded — `ps` confirmed 0 processes). NO total line this round, same as
  rounds 4-10; the two required binaries' whole runs above are the evidence.
- Changed files: `crates/nf-view/tests/walkreal_07000014_live.rs` (the refreshed ignore
  message + the NF_DELTA_PROBE hook — env-gated read-only) and `docs/WORKLOG.md` (appended).
  **No file under `crates/*/src/` and no shared helper changed this round** — the round-10
  levers in walker_b2.rs/damage_log.rs stand untouched; the round-11 lever code (the fast-tap
  plumbing, the tail-skip field) was fully reverted before the final-tree runs, so the
  other walks' byte-identity is by construction (the final tree's only test-file change is the
  env-gated probe, which consumed zero ticks — proven by the tick-exact t37477 reproduction
  under the probe env).
- Facts tagged [L] come from this window's runs on the real ISO with NF_DMG_TRACE / the probe
  env; code citations are [M] with the addresses carried in the test comments; no test was
  weakened; the game bytes, the token and the URLs stayed out of the commits.

## Still red / what's left

The 00agent sister — the ignore message carries the round-12 levers (the post-crossing scan
form first; then the fight-tick budget; then the round-9 levers 2/3).

## Live checks for the main agent

1. **Name the trap's trigger** [D/script]: at the node-21 door E-open (door 51 at
   (−19.3,1.2,−22.8)), what arms the kind-27 blast at (−17.6,1.5,−23.2) r 4.0 owner −1?
   Expected: a scripted explosion ~507 ticks (~8.45 s) after the door's E-open (the v0 stream:
   door ~t36970 → blast t37477). Watch with gdbwatch on the script that writes the blast;
   answer ONE-SHOT vs CYCLICAL (does it re-arm?).
2. **#40's restore** [D/script]: drone #40 (skin 0x50000b2, post (7.4,1.0,−23.6), the delta
   approach) read ≤ 0/corpse during his first fight (t37271) and came back at hp 11.6 for the
   second fight; his final hp reads −400. Which script writes his health/state? A restore
   script (respawn band?) changes what a "kill" is worth on this approach.
3. **The blast's difficulty gate** [L is ours; D for the answer]: the d1 sister crossed the
   zone green with no world hit near it — is the kind-27 volume difficulty-gated
   (GameState+0x28 → Drones::difficulty at load [M 0x2a3790]) or purely a race the d3
   stream's stalls lose?

## Usage

not visible to me
