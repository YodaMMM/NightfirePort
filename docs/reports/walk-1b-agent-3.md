# walk-1b-agent-3 — 1b Agent / 00 Agent real-health walks, round 3

Branch `s/walk-1b-agent-3` (origin/master 7562f1e70 with origin/s/walk-1b-agent-2 merged in
first — a fast-forward; agent-2 was 5 commits ahead of master). The sandbox kept the repo,
`work/`, the ISO and the queue worktree from the previous round; the remote token was refreshed,
the branch created and pushed at the merge point.

## What's done

**The Agent walk (`walk_arrival_to_drakes_defeat_on_real_health_agent`) is GREEN — the ignore is
removed.** The round-2 ledger's named lever, the Drake-fight shoot-and-scoot, landed after 11
measured runs (each judged on its own real run of the real-input walk, no top-ups, no
damage/health writes, no guard-code changes):

- The diagnostic (`NF_1B_SCOOT_DIAG`, stderr-only) mapped the fight [L this branch's runs]: the
  Agent fight hold is the east nest (53.2, −10.9, 15.6) with exposure (0,0) — the zero-loss
  shelter the round-2 ledger already proved (1260+ ticks); Drake spawns (−7.3, −1.4, 0.1) 63 u
  west, closes ~0.014–0.018 u/tick while ascending (y −1.42 → −0.1), and the shelter's sight-cut
  never opens his box line (the round-23 law confirmed: `exposed_aim(Drake)` never passes).
  Weapon 0x33's death range `+0x1C` reads 500.0 — no range blocker for the round-23 lever-2
  probe. The wave's first hits arm ~120 ticks after sight gain [M DoBulletAccuracy 0x179740:
  102-109].
- The shipped lever (all inside `crates/nf-view/tests/walkreal_0700001b_live.rs`, gated to
  `drake_first = waves_first && !hard` at the Drake-fight call — the Operative walk (d1) and the
  00 Agent walk (d3) keep byte-identical fights):
  1. **The free wait**: no aim lift, no wave taps while no window is locked (the round-22/23
     climb cost exp 0→14 and three hits; the wait is free).
  2. **The probe**: every 30 ticks without a lock, a 1.5-u grid around Bond for body spots whose
     EYE — at a live Drake box centre's own height (a level view only hits what the eye's plane
     spans) — puts a level ray into a box before the world's first solid contact [M
     0x1ea398:216-229, the bullet pass's own comparison], refused on hurt plates, scored
     near-first; a launch needs the candidate to repeat on consecutive probes (the stability
     gate) — immediate once high (the exposure is already being paid), never mid-dive.
  3. **The two-phase fly**: the dark cavity plane until overhead, then the rise at the spot.
  4. **The TAP hold**: aim at the remembered box offset (projected onto Drake's live position),
     the fire gate is the PREDICTED BULLET's own line (`bullet_on` — the compensated view ray
     points past the box while the round converges onto it [M scene.rs `shot_from`]), the kill
     tap fires what the clip holds.
  5. **The sight-break cycle**: each hold is capped at 100 ticks and forces a dive back under
     the cut plane — every armed shooter's 2-s clock resets on lost sight [M 0x179740:102-109],
     Drake's −14.0 weapon 0x30 at Agent included [D weapon_data 0x30 +0x0c; M 0x1902c8] — then
     the cycle re-launches. Each cycle lands 2-3 clean −20 taps at ~1 wave hit and zero Drake
     hits.
- The measured-red variants banked on the way (all [L] this branch): the long direct flight
  (21 hp, died one tap short), the exposure-dominant score (flipped the best candidate every
  probe — the stability gate never passed), the mid-height exposure term (picked z 21.58 whose
  line closed after one tap), the 3-u grid (missed the stable column x ~49-51, z ~13.9-14.1 by
  1.3 u), the strong chaser repulsion (traded the dark wait for exp 4-13), and the un-capped
  hold (died to Drake's first −14 after his 2-s clock completed). The green run is
  deterministic on the shipped tree: no env gates needed.

**The 00 Agent walk (`..._00agent`) is NOT attempted this round** — the task's "Agent first"
plus the 11 lever runs consumed the window. Its round-2 ignore message (the ledger, the next
levers: the stop-length sweep 7050/7250/7400 at len 30/90/120 at the 42.5 stalls, or the k4
stop) stands unchanged.

## How it was verified

- The whole `walkreal_0700001b_live` binary on the final tree, `--test-threads=1`:
  **18 passed; 0 failed; 1 ignored** (210.19s). The 1 ignored is the 00agent walk (its ledger).
  The Operative walk stays green; nothing else moved.
- The Agent walk individually: green, ~112 s.
- All four crates build clean (`cargo test --offline --release --no-run -p nf-game -p
  nf-formats -p nf-audio -p nf-view`).
- Change surface: the single 1b test file (verified by grep — no `crates/*/src/` change, no
  `crates/nf-view/tests/common/` change; the new helpers `drake_shot_from`/`bullet_on` and the
  modified `fight_opts_y` are file-local; the only other `bullet_on` grep hit is a test NAME in
  `interact_07000001_live.rs`).
- The prescribed full suite did NOT fit the window: background jobs die between tool calls in
  this sandbox (re-verified with two nohup probes — the process was dead within seconds both
  times, matching the round-2 finding), and the foreground window went to the 11 lever runs
  (~90-125 s each) plus the final sweeps. Same justification as rounds 22 and 23: the change
  surface is the single 1b test file and the whole 1b binary ran twice on the final tree.
- `NF_1B_PAUSES2` (the second-stop re-roll hook, the ledger's alternate lever) shipped
  env-gated, defaulting to a no-op; `NF_1B_SCOOT_DIAG` keeps the fight dumps for the next round.

## What's left

- **00 Agent green**: the round-2 ledger's levers are untouched — the stop-length sweep at the
  42.5 stalls (7050/7250/7400 at len 30/90/120) or the k4 stop sweep; `NF_1B_PAUSES3` makes them
  one-line runs. If the hard walk ever reaches Drake's gate, the Agent walk's scoot machinery
  does NOT ride the hard fight (gated `waves_first && !hard`) — that gate can be revisited with
  hard-walk measurements.
- The Drake fight's margins are thin at Agent (the wave's 16 shooters at exp 13-14 during the
  kill cycles); a lower-exposure kill column or a wave-thinning arm would widen them, but the
  walk is green as shipped.

## Usage

not visible to me.

## ntfy failed

All three fallbacks failed (network egress to ntfy.sh unreachable from this sandbox):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "walk-1b-agent-3: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (exit 28)
2. `python3 -c "... urllib.request ..."`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "walk-1b-agent-3: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out (exit 124 after 30 s, no output)

The branch, the WORKLOG entry and `docs/reports/walk-1b-agent-3.md` are pushed
(s/walk-1b-agent-3 e3499a2f6 + the ntfy-failed addendum), and
`finish.sh 50-walk-1b-agent-3 done` reported OK before the notification attempt.
