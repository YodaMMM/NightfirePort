# retune-nb-0d-18 — round 18 of the 0d arrival-to-C5-door real-damage walks: the ledger's levers judged on their own runs, the D20 ladder-top FAST act claims #20's 28.0 and walks to t21471 (the deepest), the conserved-tax law, the scoped-reload law, and the yard-west armour lever blocked — still RED, the ledger refreshed

Branch `s/retune-nb-0d-18`, from origin/master 7562f1e70 with origin/s/retune-nb-0d-17 merged in
first (0d-17 is not on master; the merge was a fast-forward — 0d-17's tip 2f8fc3eb2 sits directly
on master). **The task walk `walk_arrival_to_the_c5_door_real_damage_agent` is still RED — the
ignore stays, refreshed with the round-18 ledger and the next levers.**

## The baseline re-measure on the merged tree

- **18-0, the Agent walk with its committed shape (no env)** [L, the direct test binary]:
  DIED t20234 at (137.69, 5.11, −313.38), 34 hits for 70.6 + 29.4 other (falls), 0 return-fire
  fights — 17-7/17-8b reproduced exactly. The drain map (the round's own reading): #20 28.0 (1
  hit) + #15 28.0 (1 hit) + #22 6.0 (1 hit) = the 62.0 ladder-top tax; #29 5.3 (7 hits) + #7
  2.1 (5 hits) + #2 1.2 = 8.6 of residual chips across the yard→trench legs; the falls 29.4.

## The ledger's levers, each judged on its own real run

- **(c) the S15 head act on #15 before the #17 act — MEASURED DEAD.** 18-1 [L]: the act
  (kill_guard_head_present, her spot box (166.3, −271.4) ± (3,3), knob `NF_0D_S15_HEAD`) picked
  her AT the spawn spot and the head-box line from the ladder-top eye (213.1, 18.1, −234.5)
  reads **full 0.947-blocked** — the same order-dependent ray law as 17-4's pair-first. The
  height law ("the head lines clear where the chest rays die") does NOT hold for this line: the
  obstruction sits on the head line too. Death t4795, 6 hits for 100.0 (the stall fed the
  snipers). Knob stays default OFF.
- **(d) the #20 ladder-top act — the DRAINED variant dead, the FAST variant the round's win.**
  18-2 [L]: the drained act (kill_guard_head_present on her deck box (198.4, −260.7) ± (4,4))
  LANDED the kill — **#20 down at +212: the DOWN-hill ray from the ladder-top eye to her deck
  spot (198.4, 14.0, −260.7) CLEARS** (unlike the pre-climb base act's up-hill ray, which the
  crane kills at 0.040 [M the 512 comment]) — but her own 28.0 was already in flight (frame
  3803 vs the kill at ~3912: the 160-tick aim-off drain lost the race ~110 ticks) and the +212
  stand re-rolled the south-roof windows (#9 28.0 + #19 38.0 + the killing 10.0, death t4348).
  18-3 [L]: the FAST variant (kill_guard_head_present_fast, no drain) **WON the race — the kill
  at +52, #20's 28.0 GONE from the hit list** — and the walk reached **t21471 at the plate,
  +1237 ticks past 17-7 (the deepest any round has reached)** — but the bill is conserved:
  #19's window opened (+28.0) and #29's roof shooter chipped 8.6 in 13 hits. Knob
  `NF_0D_D20_TOP` stays default OFF (not green).
- **THE CONSERVED-TAX LAW (the round's finding).** The south-roof pair's 2 × 28.0 + #22's 6.0
  land on the walk's longest stand WHEREVER it is (the ladder top in 17's dice, the
  yard-south-1 stand in 18-9's); the pair ray clears only AFTER the pair's own triggers
  (17-4 + 18-1), so the tax cannot be killed pre-trigger. Every re-roll this round conserved
  ~62.0.
- **THE SCOPED-RELOAD LAW (new, [M]).** `scene.rs` 4763 (`let reload = reload && !(self.aiming
  && rd0(0x68) & 0x40 != 0)`) — the port's Player_WeaponFiring R-key gate: **the reload is
  REFUSED while AIMING a scoped weapon** (flags & 0x40). The hand at the yard phase is the
  yard's picked-up sniper 0x1e (the pre-state print: clip 1 spare 8, weapon 0x1e). Every
  in-act reload pulse is swallowed while the act holds the aim bit.
- **The roof-shooter lever (`NF_0D_D2_ROOF`) — the act converges, the ammo blocks, the
  exposure kills.** 18-4 [L]: the act's PLATE→roof ray converges (cross [0.0, −0.0018], full
  CLEAR — the run-14-b law holds at d2) with the clip DRY; the in-act pulses are swallowed
  (18-5/18-6: the clip 0 through +283). 18-7/18-8 [L]: the pre-act aim-off reload_full fills
  the clip (clip 5 spare 4 after two passes — the shell loop is slower than one 100-tick pass,
  reload_state 6 at the first) but **200 aim-off ticks at the plate (pool ~0.4) were the fatal
  exposure** — death t21315 DURING the passes, the act never ran. 18-9 [L]: the reload moved to
  the yard-south-1 stand (pool ~13-15) — the pair's 2 × 28.0 landed INTO the reload window,
  death t21042 at the yard-south-1 stand. Every aim-off stand in the yard phase is a tax
  window. Knob stays default OFF.
- **The yard-west armour lever (`NF_0D_WARMOUR`) — a real +50 blocked by #4's live park.** The
  round's pickup scan (18-12 [L, NF_0D_PICKUP_SCAN on the 00agent path]): the level carries
  FOUR body armours (kind 3 weapon 0x6 rounds 50) — the committed annex grab (192.82, 10.79,
  −272.03), plus (147.0, 21.76, −252.77) on the roof level, (205.41, 0.11, −205.31) far north,
  and **(107.75, 6.84, −275.16) ~37 u west of the exit stand**. A second +50 armour would
  absorb the trench/return chips (the bullets hit armour first; only the falls skip it [M
  Player_HandlePain 0x1902c8:84-104]). MEASURED BLOCKED: the detour lane passes **#4's LIVE
  park (132.2, −273.4)** — the drift law (the early-catch box (129.6, −282.1) ± (4,4) missed
  him by 8.7 in z) — and the head act from the exit stand **cannot converge the pitch on his
  flat line** (the cross y pinned −0.2257, 5 rounds high with the FILLED clip [L 18-16], death
  t21210 mid-detour; the unfilled-clip run died t21336 [L 18-13]; the plate-pool detour
  t21336 [L 18-13's predecessor 18-13 itself]). Knob stays default OFF.

## How it was verified

- 18 measured runs this round [L, the direct test binary, --test-threads=1, each its own
  roll]: 18-0 the committed shape t20234 (= 17-8b); 18-1 the S15 head act t4795; 18-2 the D20
  drained act t4348; 18-3 the D20 FAST act **t21471** (the deepest); 18-4/5/6 the roof act's
  dry-clip runs (t21708 ×2, t21042); 18-7/18-8 the reload diagnostics (t21315, the act never
  ran); 18-9 the yard-south-1 reload t21042; 18-12 the pickup scan (log-only); 18-13 the
  armour detour t21336/t21338; 18-15/18-16 the #4 head-act runs (t21338/t21210); 18-17 the
  final no-env byte-identity **t20234 exact**.
- **Whole binaries on the final tree**: `walkreal_0700000d_live` **2 passed / 1 failed / 2
  ignored** in 70.3 s (the plain walk + the d1 Operative walk GREEN — the d1/plain
  byte-identity holds; the failure is the 00agent batch-3 regression, pre-existing [L 16-0],
  signature t3995 / 4 hits for 100.0 identical; the Agent walk ignored, its own runs above).
- The suite (`tools/test/suite.py`): this sandbox kills background processes between tool
  calls (the round-13..17 signature — the nohup attempt produced no output), and the
  four-crate build + run did not fit the remaining window after the 18 measured runs. The
  round's substitute, per the 0d-16/17 precedent: the touched binary whole (above); the
  round's diff touches ONLY `crates/nf-view/tests/walkreal_0700000d_live.rs` (no
  `crates/*/src/`, no common helper, no other test file — the merge added no conflicts), so
  the nf-game / nf-formats / nf-audio crates' suites are unaffected by construction — 0d-16
  measured them 2153 passed / 71 failed / 11 ignored on this same merged line, and nothing
  this round touched can change them.
- Lesson-10 grep: the touched file is `crates/nf-view/tests/walkreal_0700000d_live.rs` only.
  All the round's knobs are gated `difficulty == 2 && real_damage` plus an env read defaulting
  0; the d1/d3/plain runs are byte-identical (the whole-binary evidence: the plain + d1 green,
  the d3 failure signature unchanged from 16-0).
- No guard code changed, no damage/health constants changed, no top-ups: the armour detour is
  a REAL level pickup taken through the real touch path (the walk's own nav legs — the same
  class as the committed annex grab), the acts are the test's own real-input act machinery,
  the crouch/stand presses are real input edges [M Player_HandleJump's crouch arms, 0x190630].
- Live checks for the main agent: none required (no visual work; the deaths are fully
  instrumented by the test's own damage log).

## What's left

- The Agent walk is still red. The refreshed ignore message carries the round-18 ledger and
  the next levers: (i) the D20-TOP + WARMOUR composition still has a live path — the #4
  pitch-pin law needs a DIFFERENT act class on his flat line (a chest shoot_far_aim from the
  exit stand, or the #4 kill from the yard-south-1 arrival with the detour run FIRST) — the
  +50 armour at (107.75, 6.84, −275.16) then absorbs the trench/return legs; (ii) the tax's
  only other reclaim is the FALLS 29.4 (the hop grid measured dead — the deck_crouch rides
  ~2-3 MARGINAL); (iii) the photograph posture is MOOT (the pool dies before the photograph
  phase runs); (iv) the #22 6.0 needs an act during the climb (unbuildable this round).
- The 00agent walk needs its own re-green on the batch-3 tree (pre-existing, unchanged by
  this round).
- The full four-crate suite still needs a bigger window/rootfs (the rounds-13..18 wall); the
  per-binary evidence above is this round's substitute.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no gauge, no billing panel, no API counters are exposed to the agent).

## ntfy failed

All three transports timed out (the round-17 blackout signature, re-measured 2026-11-23 ~01:38Z):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (exit 28).
2. `python3 -c "... urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', ...), timeout=25)"`
   → no output, killed at the 40 s tool timeout.
3. `timeout 20 wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x` → exit 124
   (timed out); a second `curl -m 15` retry → exit 124.

The queue entry (`done/50-retune-nb-0d-18.md` on origin/queue) carries the full report; the
notification could not be delivered from this sandbox.
