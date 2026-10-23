# level-16-jetpack-escape-3 — the launch-pit kill timing and the way out

## What the task asked
Re-derive the pit timeline against the fresh-boot live numbers (why the port's ch 243 looked ~730
frames early and why its kill looked instant), read `Hurt_Update` 0x17e5d0 in full, find the
player's real way out of the pit in the ~1600 + 454 frame window, and settle
`walk_the_chasers_defeat_completes_the_mission` (pass with real input, or re-ignore with the exact
remaining blocker) plus a kill-timing sister.

## What's done (all evidence tagged)

### 1. The timeline — the "~730 early" was the round-2 doc's own arithmetic error
The launch chain: the ch-0x81 touch trigger #255 → SP #262 = pack cinema 0x06000631 (length 750
[D]) → stream 8's single key at t747 is the pit-floor pose (3.57, −22.76, −179.55) with stream 9's
EVENT 0x06 snapshot at t747, and stream 7 writes ch 185 = 1 at t748 [D stream dump,
probe_pack_flight] → SP #272 = launch script 0x0600090a, whose stream 6 pulses ch 243 = 1 @1611
and = 0 @2170 — ABSOLUTE stream ticks [M nis.rs:446-451, the Wait blocks until the clock reaches
the value]. So the model is **re-enable → ch 243 ≈ 1611 − 1 = 1610 ticks**, not the round-2 doc's
"~863" (that figure is 1611 − 748 — it subtracts the cinema the script starts AFTER, instead of
counting from the re-enable). The task's "~730 frames early" followed from that line.

Measured: the port's own sim (rerun probe_pit_window, and the new sister test) gives re-enable
t752 → ch 243 t2355 = **+1603**; the live fresh boot gave **+1591** [L]. Model, port and game agree
within 19 ticks. The port's ch 243 is NOT early — nothing to fix in the script/Wait/stream-clock
layer. The one number still open is the live **ch-0x81 → re-enable delta: 553 [L] vs the modeled
747 [D t747]** (see Live checks) — if real, something starts the pack cinema ~194 ticks before the
touch trigger's ch 0x81, and that starter is the only remaining candidate mechanism.

### 2. The kill is not instant — on either side — and the port already models it
`Hurt_Update` 0x17e5d0 read in full [M]: it is a state machine with the gate maintenance live every
tick (channel LOW sets `obj+0xF0 |= 0x20`, high clears it, :26-37), state 0 arms on the raw
channel byte (:98-103), and state 1 with key count 0 **returns immediately — no damage, no delay,
no fail path in Hurt_Update at all** (:106-108). The damage rides the player's object collision:
`Player_DealWithObjHit` 0x194458:95-101 (the class-0x27 arm) calls
`Player_HandlePain(impact + Hurt_GetDamage(obj) = 1000 [D params], type 2)`; types 1..7 give
armourFraction 0 [M 0x1902c8:84-99] so health 100 − 1000 → 0 the same tick [M 0x1902c8:131-134].
The mission fail is NOT health-poled: the death sequence runs `Player_HandleDeath` 0x190bb0:23-33,
which raises ch 0x62 only when the blood pane's state is non-zero, and `HUD_UpdateBloodPane`
0x19a6a8:23-30 grows that pane 1 px/tick to the 448 cover (`RED_FADE_TICKS`, death.rs);
`Mission_Update` 0x182c60:21,35-42 reads ch 0x62 → MissionState 3. The port models every step
(crate::death — RED_FADE_TICKS 448 → ch 0x62 → state 3). The escape-2 probe's "Bond dead t2356"
was that probe's own `health <= 0` break, not the port's mission fail — the "one-tick kill" never
existed in the port's mission layer.

### 3. The sister test (un-ignored, green)
`walkreal_07000016_live::the_pit_pulse_kills_an_idle_bond_about_450_ticks_later_like_the_game` —
Bond placed on trigger #255 (the live session's own setup [L]), no input, the walkreal allowances
off (`top_up_health = false`, `skip_cutscenes = false`), and it asserts:
- re-enable at the pit floor, ch 243 rises (+1603 in the recorded run),
- the contact kills within the pulse's first ticks (+1),
- **MissionState 3 lands at pulse +451 — the game's measured +454 [L] within 3 ticks**,
  asserted in a 440..470 window backed by [M 448] + [L 454],
- ch 240 and ch 80 stay 0 through the fail on both sides [L; D stream ticks 2180/2240 land later],
- the fail rode the death sequence's ch 0x62 [M 0x190bb0:32].
God mode in the live run rewrote health every frame and the mission still failed — consistent with
this chain: the fail is not health-gated, it rides the death sequence.

### 4. The way out — the data says pulse 1 has no real-input escape (re-checked this round)
- The six-band hurt stack (world y −24.08..−21.32) tiles the whole pit footprint with **overlapping
  quads** — 18 quads per band layered over x −30.7..35.3, z −197.3..−133.8; there are no gaps to
  stand in [D tri dump, hurttris16].
- The blast rooms on the pit floor (the obvious shelters; base −23.0, roofs −19.53) are **sealed by
  static BlockingPolys** — obj 117 "BlockingPoly_blastroom" is a 2-tri wall x 19.60..23.22,
  z −174.84..−169.00, y −22.08..−19.53, and obj 118 mirrors it west [D vertices, blockvert16].
  Their roofs are ~4.4 above the grate — no jump reaches, and the pit's only kind-0x2d ladder is a
  0.5×2×0.8 "PlayerStart" box at the rim (place[256]/obj[167]) [D].
- The shuttle-bay lift's LOW point (−21.22 [D keys]) sits **0.10 above the top band (−21.32)** —
  the one safe floor in the pit — but ch 80's ONLY writers are the scripts (0600090a stream 21
  @2240, 06000632 stream 19 @1606 [D]); the full-placement census found **no trigger, AND, blocker
  or door naming channel 80** (ch80writers16), so the platform does not exist until 630 ticks into
  the 559-tick pulse.
- ch 240's writer is the launch script's stream-7 event @2180 [D] — post-pulse; no door placement
  carries it (the doors' locks are 0x24/0x27/0x28/0x94/0xb4/0xb5 [D census]).
- The pit squad (spawners #294-296 on ch 187 @1512 [D]) is dropped into the pit 99 ticks before the
  pulse to die in it — the game's own demonstration that the floor is lethal.
- No pickup/substate/Bond-actor stream exists [D+M, the escape-2 census stands].
So the walk test stays `#[ignore]`d with the sharpened blocker (the doc comment above it carries
the whole chain), and the ONE live lead that could still hide a mechanism is the 553-vs-747
ch-0x81 → re-enable gap.

## Live checks for the main agent (PCSX2, FRESH boot of 07000016)
1. **Nail the ch-0x81 rise tick.** Attach a watch on `switch_channels[129]` (0x26fc90 + 129·4,
    byte) BEFORE placing Bond, then place Bond on trigger #255 at (2.37, 1.04, −176.41) with no
    input. Expected (port-faithful): ch 0x81 rises at placement + ~2 and the re-enable lands 747
    ticks later. If instead ch 0x81 first reads 1 only ~200 ticks after the placement (the round's
    "~687" reading with a ~470 placement), the round's 553-vs-747 gap is a first-observation
    artifact and the timeline is fully closed. If ch 0x81 genuinely rises ~194 ticks after another
    event started the cinema, gdbwatch the write to 0x26fc90+516 to catch the writer — that writer
    is the missing escape candidate.
2. (Superseded — answered this round): the old "watch switch_channels[243] from the touch" and
    "gdbwatch obj+0x30 through 748..1611" checks: nothing script-side moves Bond out — the pit
    squad spawns at +1512 and dies in the pulse beside him; the survival question reduces to
    check 1's chain-start gap.

## What's left
- The main agent's live check 1 (it either closes the timeline or hands the next round the
  cinema's real starter).
- The walk test's un-ignore once that is settled; the route to the chaser/button tail is unchanged
  from the parked test body.

## What I ran
- `cargo test --offline --release -p nf-view --test walkreal_07000016_live the_pit_pulse_kills --
  --test-threads=1 --nocapture` — **1 passed** (the numbers above are its recorded output).
- `cargo test --offline --release -p nf-formats --examples` compile checks for the five probes
  (dump16_packs, pit_census16, blast16, blockvert16, hurttris16, ch80writers16 — all opt-in via
  NF_ISO, no game bytes).
- The fast suite (`tools/test/suite.py`) was started in the background at 22:52 UTC on the merged
  tree (master 8b101249 + escape-2) BEFORE this round's test-file edit; at report time (23:45) it
  had processed 421 binaries with 1390 passed / 3 failed / 106 ignored and the FAILED lines it
  printed (fix_0c_pylon_live ×2, fix_wallhug_prompt_live) plus the mp CRASH lines are the same
  baseline family the escape-2 run measured on the unmodified tree (72 failed there). The ~119-min
  suite cannot finish inside this sandbox's remaining window, so the final-tree full total is left
  to the next session; the crates this round's edit touches (`nf-view` tests) compile clean and the
  new test ran green standalone.

## Usage
not visible to me — this sandbox exposes no token/credit/context-window figures in any readable
surface (no environment variable, log or UI text was found).
