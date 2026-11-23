# walk-06-00agent-4 — the 06 castle-interior-to-the-party 00 Agent real-damage walk, round 4

Branch `s/walk-06-00agent-4`, from origin/master 7562f1e70 with origin/s/walk-06-00agent-3 merged
in first (fast-forward — no conflicts; it brought the round-3 report, the WORKLOG entries and the
round-3 ledger). **Still red** — the ignored walk stays
`walk_the_castle_interior_to_the_party_real_damage_00agent` in `walkreal_07000006_live` — but all
three round-4 levers from the ignore message are now MEASURED DEAD on their own real runs, the
stall premise they were built on is corrected, and the exact survival margin is measured. The tree
ends comments-only: every walk stays byte-identical to round 3's merge, the verdicts live in the
ledger comments and the ignore message.

## What the round did (setup from scratch; the sandbox had been reset)

Repo clone, `work-slim.7z`, the ISO (size verified 369823203 before extraction), the queue
worktree fetch, rustup stable + the ALSA headers in a user prefix, `cargo fetch`. Branch created,
WORKLOG opened and pushed as the first checkpoint.

## The four probe runs, each judged on its own real run

1. **Lever (a1) — the #754 fetch on the quieter lane: MEASURED DEAD.** The fetch was attempted
   where the round-3 ledger asked (AFTER the descent's fight_through): Bond stood at the descent's
   end (4.41, 42.04, −47.26) at health 40.0 armour 0.0, exactly the ledger's strip-exit state.
   BOTH staged nav legs wedged on the SAME wall face — the 3000-tick give-ups at (−1.45, 42.81,
   −37.03) d 14.40 and (−0.52, 42.10, −49.39) d 19.77 [L the a1 run] — and round 3's wedge
   (−1.45, −47.47) d 17.9 sits on the same line. **Three wedges on one line x ≈ −1.45: the
   courtyard floor band has no west exit toward the shelf at all.** The lane is geometrically
   dead, not a shooter problem: the run cost 27.5+ health while wedged (40.0 → 12.5) and Bond died
   t20693 mid-fetch without reaching the corner. The return-fire-ON and east-first variants share
   the same wall, so the whole quieter-lane family is dead. The probe code was removed; the
   verdict is recorded in the #754 ledger comment.
2. **Lever (b)+(b2) — the stall levers: MEASURED DEAD, and the ledger's stall premise corrected.**
   The strip duel's exit is NOT a ~100-tick stall: it exits at taps 41 "no recent hitter" ~2
   tapless taps after the 0.2 drop, and at that moment #20 is already dead by return fire (the
   a1 run's death summary: hp 0). His burst 2 (16.5, ticks 7112-7120, health 51.0 → 40.0) landed
   DURING the grind's final taps — the duel's own close print reads health 40.0, i.e. post-burst-2.
   So the 32.5 is two bursts of a grind already at the fire-gated pace floor (4 drops in 52t). Two
   surgical arms were built and run, each inert: a one-tap-stall bail (hp ≤ 2.0, 5 tapless taps —
   gated to keep the round-2 premature-bail lesson intact) and an active-shooter re-engage arm
   (a 300-frame recent-hit scan without the enabled gate — the game's bullets hit ANY present
   drone [M Drone_CollisionHandler 0x140968:85-95]). Both runs were byte-identical to the a1
   baseline (every exit reason identical), so both were removed. Whether the +132..+184 grind
   labels "#20" are the skin-0x500002f guard or a shifted index stays open (the round-5 lever γ).
3. **Lever (c) — the east-floor deck answer before the corner: MEASURED DEAD.** The detour walked
   the route's own east-floor ground early (the proven 0.59-rad anchor (25.3, −40.6)), then
   returned to the corner. Bond died EARLIER than the baseline: t8211 at (17.33, 41.13, −36.17) on
   the corner-return leg — the detour's double crossing of the #25/#5 stream cost more than the
   20.75 splash it prevents. Removed; the verdict is in the ignore message.
4. **The margin is now exact.** The strip exit state (health 40.0 armour 0.5) and the SE-corner
   death (t8556, (12.97, 41.83, −36.04), 27 hits for 100.0) reproduce round 3 exactly on the
   untouched tree. The tail cost is #24 16.5 + #5 2.75 + the splash 20.75 = 40.05 — **death by
   0.05 health. Any ≥1-health trim anywhere flips the sign.**

## What's left (the round-5 levers, in the ignore message)

- (α) the court trio's 27.5 — the biggest un-answered chunk, dealt before the #755 fetch (armour 0
  there); the court crossing's stand/duel placement has never been swept at 00.
- (β) #24's 16.5 in 3 — measure whether he is a recent hitter when the tower duel's re-pick runs
  (his patrol (29.2, −36.0) is inside the 45u band) and whether the high-post pick takes him.
- (γ) pin the grind-target identity first (one run with a temporary env-gated skin print in the
  duel drop lines) before spending more levers on the pair.

## Live checks for the main agent

- **The west shelf's reachability**: a collision/walkability scan of the band x −2..−16, z −38..−50
  at y 41-44 (the x ≈ −1.45 wall face) — is there ANY gap, door or climbable step from the
  courtyard floor band to the shelf at (−15.85, 43.16, −36.92)? Three independent lanes wedge on
  the same line; if the scan shows no gap, #754 is set dressing on this floor and the lever dies
  for good.
- **The drones.list corpse semantics**: does the list remove dead drones (shifting indices) or
  keep them as corpses? This settles whether the duel grind labels "#20" can drift across drones
  (the round-5 lever γ could then be settled by reading the save state instead of a probe run).
- **#20's enabled state at ticks 7100-7130** (fieldprobe/gdbwatch on the drone's enabled flag):
  the b2 evidence (burst 2 firing after the pick lost him) suggests a disabled-but-firing window;
  a live measurement would say whether the game really does that or the exit timing is different.

## How it was verified

- Every lever above ran on its own real run of the real input path (the load → the truck spawn →
  the walked route; no top-ups, no teleports, no health/ammo writes; NF_DMG_TRACE traces cited per
  run; runs are 4-12 s wall each on this sandbox).
- Whole-binary runs on the final comments-only tree: **walkreal_07000006_live 8 passed / 0 failed
  / 1 ignored** (the ignored one is the ledgered 00 walk), **interactables_07000006_live 5 passed
  / 0 failed**. `tests/common/` and `crates/*/src/` are untouched this round (the probe code lived
  only in the walkreal_07000006_live.rs test and was removed).
- The fast suite (`tools/test/suite.py`) was attempted in the background twice in the first
  minutes; this sandbox kills background jobs between tool calls (the first launch also lacked
  cargo on PATH and died instantly with FileNotFoundError), and the full suite cannot fit the
  window regardless — the touched-binary runs above are the verification set, as in rounds 2-3.

## Usage

- Input/output tokens, context used, cost: not visible to me. The session's environment shows no
  token or cost figures anywhere; the WORKLOG entries for this task are the progress record.

## ntfy failed

All three fallbacks timed out against ntfy.sh egress at finish time (the same failure mode as
round 2's report recorded):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "walk-06-00agent-4: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 urllib.request` with `timeout=20` → the tool call killed at 30 s with no response.
3. `wget -q -O- --timeout=20 --post-data "walk-06-00agent-4: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → the tool call killed at 40 s with no response (no output, non-zero exit).

The push, the report and the queue `done` marker all landed before this; the notification text is
reproduced in the commands above.
