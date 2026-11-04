
## real-health-14-00agent-5 (2026-11-04)
Task: get the 14 00 Agent walk past the mezzanine west lip (Bond stops at x 45.134 when placed at (45.6, 3.05, 39.87) facing west [L]); find how the Agent walk gets past without the lip (push-tail rule from s/real-health-14-00agent-4 dropped, do not reuse), then land the d2 with_real_damage wrap and the d3 un-ignore. Branch created from origin/master (690e8fefb).
**Outcome (same window):** the lip question closed by probe (probe14_lip_route): the fresh-boot
hold (x 45.134 [L]) is the GAME's own behaviour — the port's fallback holds 45.1339, the same
number; the walk's route gets past WITHOUT the lip by crossing west at the flush lid (z ≈ 42.37,
arm B, all three legs reached) or the south opening (z ≤ 39.4, arm C passes to the #301 rim at
x 42.16); the hold band is only z ≈ 39.9..42.2; the jump does not clear it (arm D) and the
stance does not matter (arm E standing holds identically). The -4 honest_ref stays dropped. The
d2 with_real_damage wrap MEASURED on master's tree: it wedges at the 1.72 landing hover (tick
25740, (33.90, 0.68, 35.38) d 0.81 short, grounded=false, health 32.5 — 36 hits for 67.5, the
budget survives) — the wrap cannot land until the wedge is fixed; the d3 un-ignore is blocked by
the same wedge. The wedge mechanism identified: a ground-flicker HOVER DITHER (grounded true on
single ticks zeroes vel; the float re-accumulates ~0.003; an up-facing push gives it back — pos
never descends, the recovery hop never fires) and 60+ quiet no-input ticks do NOT break it (a
tried float_settle walker flag was measured ineffective at the pocket and harmful at the trench's
own recovery — reverted). The d3 note carries the full record + the ordered next levers (the
scoped sim-side push rule at the pocket first). Tree parked at master's byte state: the whole
walkreal_07000014_live binary re-run green 9/0/1 (189.4 s).
