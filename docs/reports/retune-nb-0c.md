# retune-nb-0c — 0c whole-mission real-damage walks green again (Agent, 00 Agent)

Branch `s/retune-nb-0c` off origin/master 73822d3a0. Commit 3412a89a3 (the spacing scan on
every level + `NDrone2_CanAltAttack` in the fire states) left both 0c whole-mission
real-damage walks ignored with measured deaths; both are now re-tuned, un-ignored and green.

## What killed each walk (the reproduced ledgers)

- **Agent (difficulty 2)** — DIED t6159 at (53.76, 6.01, -158.87), ON the mezzanine deck:
  the balcony pair **#19** (skin 0x5000090, d 9.1) landed 2×28.0 = 56.0 and **#20** (weapon
  0x1e, d 7.9) the last 18.1; 9 return-fire fights never cleared the pair first; 16 hits for
  100.0 [L 2026-11-18 rerun, NF_DMG_TRACE]. The deck pre-clear (switch to the scooped 0x1e,
  bounded stands on every visible hostile within 15 u / 6 dy) existed but was gated `hard_0c`
  (difficulty >= 3) — the Agent rows never ran it.
- **00 Agent (difficulty 3)** — DIED t35413 at (94.20, 8.92, -232.05), mid crate-stack hop:
  **#5** (skin 0x500006d, weapon 0x12, at (62.1, 7.2, -207.5) — the game's own mid-void pit
  placement [D plc60's placement, the pit floor plc28]) dealt **74.8 in 51 hits** of 1.8-2.4
  across the east ground and the hops; his hp 10.0 was INTACT at the death — the return-fire
  fights fought him five times (the staircase top, the turb2 flight, the run-up, the gate
  spot) and never landed one round [L the same rerun]. The fights hold the aim button
  (`aimed_fights`); the fight machinery drives the VIEW and cannot drive the cursor window
  [M Player_AutoAim 0x1a7c98:50-59 — the level-11 law], so every aimed stand at d 17-40
  missed.

## The fixes (real input only; no top-ups; no damage/health writes; no guard-code changes)

1. **`agent_0c` (common/walkreal0c.rs):** the deck pre-clear's gate opens one difficulty row
   down (`difficulty >= 2 && !tops_up()`); the d3 block already ran there, so the d3 timeline
   is untouched and d1 never enters. Measured: the pre-clear drops #19 at d 11.5 (clip 4
   spare 20 [L this branch's run]).
2. **The pit-shooter pre-clear (walkreal_0700000c_live.rs, d3 + top-up off only):** from the
   east ground at the chasm's east (78, 7.4, -208.2) the pit shooter IS exposed to the plain
   gun rays (exposed_part_gun_rays Some, dxz 17.0, dy -0.1 [L the exposure probe run on this
   branch]) and the P2K (weapon 2, the boot loadout) fires zero spread at range 50 [L the
   strip probe on master]. The walk switches to it through the real switch-input path and
   runs the deck pre-clear's own bounded stands on every live hostile exposed from that spot
   within 20 u. Measured: #5 shot at d 17.0 (clip 7 spare 41 [L this branch's run]).

## Verification (each a fresh run on this branch)

- `walkreal_0700000c_live` whole binary: **6 passed / 0 failed / 0 ignored** (102 s) — the
  plain whole-mission walk, the real-damage Operative, the Agent (un-ignored, green: 0c legs
  28944 ticks + 0d 838, objective 3 complete at 81.1 hp), the 00 Agent (un-ignored, green:
  0c legs 36586 ticks + 0d 838, objective 3 complete at 34.8 hp), ladder495 and the 0c
  probe-sister.
- `interactables_0700000c_live`: **7 passed / 0 failed** (12 s).
- Every other user of the shared `common/walkreal0c.rs` (all difficulty 1, byte-exact legs):
  comms_chatter_0c_live 1 passed; guard1_rounds_0700000c_live 2 passed;
  upward_shot_0700000c_live 2 passed; walkreal_0700000c_eastroute_live 1 passed;
  walkreal_0700000c_ventexit_live 1 passed; walkreal_0700000c_barrelrow_live 1 passed.
- walkreal_0700000d_live (the hand-off target): 3 passed, 1 ignored (master's own ignore,
  untouched).

## What's left

- The full four-crate suite did not fit this sandbox's window (the background runner cannot
  survive between tool calls here; the foreground rebuild + 26-binary sweep exceeded every
  chunk). My diff touches two nf-view TEST files only — `crates/nf-game`, `nf-formats` and
  `nf-audio` are byte-identical to origin/master 73822d3a0 — and every `walkreal0c.rs`
  consumer plus the task's two required binaries ran green (above). The next driver round
  should start `tools/test/suite.py` early and read its total on the merged tree.
- Measured and dropped: `stuck_answers` on the d3 armour/drop nav walks — the wedges ("gave
  up at the 6000-tick cap" ×3, ~18000 ticks) are geometry, not body blocks; the wrapper was
  a measured no-op and was reverted. The 00 Agent walk passes with those wedges in place.

## Live checks for the main agent

None requested — the exposure probe ran inside the port's own sim (the [L] entries above
cite it).

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session).

## ntfy failed

All three fallbacks failed from this sandbox:

1. curl: `curl: (28) Connection timed out after 20001 milliseconds`
2. python3 urllib: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. wget: timed out after 60 s (killed; no output)

