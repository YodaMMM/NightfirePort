# retune-nb-14-7 — the 14 infiltration to the delta exit 00 Agent walk, round 7

Branch `s/retune-nb-14-7`, cut from origin/master ccd79b5be with `origin/s/retune-nb-14-6`
merged in first (it was not on master; the merge was clean — the round-4/5/6 reports, the
ledgers and the WORKLOG entries carried over; WORKLOG appended only).

## What's done

**The 00agent walk is still `#[ignore]`d — still red — but the round-6 ignore message's lever
list is now fully measured, and the walk got its first real armour bank.** The task's ordered
first step (the round-6 report: "START with the ray probe from the candidate hold spots") ran
as a read-only, env-gated probe pass (`NF_KIKO_PROBE`) inside the walk's own corridor-hold
gate, and its verdict reshaped the ledger:

- **Lever 1 (the hold-spot ray) is MEASURED DEAD — by the probe the ledger itself ordered.**
  At the corridor-hold entry (tick 26224, bond (−50.23,1.05,38.81), health 1.9) the probe
  replicated the walk's own fight gate (the box loop over `d.hit_boxes` vs
  `segment_bullet_scoped` — the bullet pass; GLASS_SIGHT is off on this walk) from a grid of
  candidate spots, x −53..−41, z 30..40, both standing heights: #20 (0x5000090, 0x14, hp 10,
  then at post (−51.33,1.00,32.42), d 6.49) reads EXPOSED with **infinite world-stop margin**
  from the hold spot itself and ~30 grid spots, including the whole corridor line. The sight
  gate was NEVER the blocker — the round-5 "rounds die on the corridor's own geometry" shape
  is a real-round-flight asymmetry the gate cannot see. The lever-1 run (the pocket-line hold:
  walk the east lane's west lip (−48,34), kiko_hold 900) died the SAME way: t26386, bond
  (−48.9,1.2,34.8), #20's post shot −1.93 at d 3.1 with his hp 10 FULL — the fights' rounds did
  not land from the lip either [L both runs 2026-11-20].
- **Lever 2 (the Kiko-corridor armour fetch (−41.29,1.53,36.48)) is MEASURED BLOCKED.** The
  north band east of the corridor mouth wedges at (−46.82,1.06,42.38) — head_bump=true,
  segment_to_target 0.099, cel 42 — and burns the 6000-tick cap, on BOTH attempts (the v2
  fetch run and the v3 run's trace) [L]. The wedged stands then drained the armour before the
  pocket. Both fetch legs are REMOVED from the tree; the wedge facts live in the ignore
  message.
- **Lever 3 is REAL and BANKED: the round-4 census's untried loop-mouth armour
  (32.26,0.87,31.14)** — ~2 u south of the measured low-floor spot (32.58,1.35,33.08), the
  census's "the only one west of the 0x85 door", never attempted by rounds 4-6. A d3-gated
  `walk_to(Wp::new(32.26, 0.9, 31.14).r(0.9))` inserted at the descent foot — real input, the
  take on contact, no teleport — **landed the take**: the door-face arrival went from health
  17.5 (rounds 5-6, the 1.9-health corridor shape) to **health ~87.6** (the trace's −17.19
  #33 grenade at t24066 left 70.3; every 0x14 bullet before it absorbed into the armour)
  [M Player_HandlePain 0x1902c8:84-104 — bullet pain (type 0) is the ONLY armour-absorbing
  pain type; apply(): absorbed = (d·frac).min(armour)]. The one health-drain hit in the whole
  ledger is the weapon-0x0 grenade [L NF_DMG_TRACE].
- **The walk now dies at the pocket with the armour DRAINED** (the v4 run: the shipped lane
  crossing restored after the v2 legs were removed): the bill t25236-25389, #20 (post
  (−51.1,1.0,32.6), hp 8) −5.00 per hit ×10 then −2.50 ×3, health 50.3 → 0.0, armour 0.0,
  41 hits for 100.0, 24 fights [L]. Measured correction to the round-5/6 ledger: **#20's
  per-hit damage is 5.00** (the "1.93" was the final partial). The door-face stands (the mute
  legs + the scans) plus the 4-fight loop chew the 50 armour dry before the crossing — the
  binding constraint is now the armour budget, not the corridor ray.

## How it was verified

- **walkreal_07000014_live (whole binary, the final tree, --test-threads=1)** [L 2026-11-20]:
  **9 passed / 0 failed / 1 ignored, 167.60 s** — the ignored one is the 00agent sister (its
  refreshed ledger in the ignore message). Includes the un-ignored Operative d1 walk, the
  plain/tops-up walk, the d2 agent walk, the vent-drop, grill, E-cascade and tripbomb tests —
  the sisters stay green with the probe hook and the d3-gated fetch on the tree (the gates
  keep their streams byte-identical, and the whole-binary run proves it).
- **interactables_07000014_live (whole binary)** [L 2026-11-20]: **10 passed / 0 failed,
  15.62 s** — matches rounds 5-6's counts.
- The probe pass ran twice (the initial dump + the lever-1 run re-measure); every lever was
  judged on its own real run (the v1 lip hold, the v2/v3 fetch attempts, the v4 banked run),
  each with NF_DMG_TRACE ledgers captured.
- **The fast suite (`tools/test/suite.py`)**: started in the background in the first minutes
  as ordered, and the sandbox killed it between tool calls (suite.out stayed empty — the same
  failure rounds 4-6 recorded). The foreground four-crate build exceeds the window on this
  2-core box; the two required binaries' direct runs above are the same evidence. NO total
  line this round, same as rounds 4-6. The whole-binary run above also caught and fixed a
  real parse error in the new ignore message (the inner quotes), so the shipped tree compiles
  clean.
- Only `crates/nf-view/tests/walkreal_07000014_live.rs` changed (the test file's own d3-gated
  blocks + the env-gated probe); no file under `crates/*/src/` changed, so no other test
  binary is affected; the two required binaries were run whole anyway.
- Facts tagged [L] come from this window's runs on the real ISO with NF_DMG_TRACE /
  NF_KIKO_PROBE; code citations are [M] with the decomp address; no [S] items entered
  pass/fail; no test was weakened; the game bytes, the token and the URLs stayed out of the
  commits.

## Still red / what's left (the round-8 levers, in the ignore message too)

1. **The server-room armour (−27.29,0.87,30.96)** — the census's 4th, untried, ~4.5 u NE of
   the escape touch (−33.1,26.8): +50 more covers the door-face-stands + loop + pocket bullet
   bill (the v4 math: the bill from the door face to the pocket ≈ 100 total; the loop-mouth 50
   covers half). MIND the escape touch's 0x4A cut-scene timing (the touch is a cut-scene
   trigger; a fetch that crosses its volume fires it early) — probe the approach with the
   NF_KIKO_PROBE idiom first, or find a mid-route pass point on the server-room west corridor
   (nav nodes 94-97/150/591-600 [D]).
2. **The door-face legs' armour drain law** — the mute legs + scans stand in the door squad's
   bursts (~2000 ticks) and the drain measured ~45 armour; a shorter stand shape (the
   slide-off fights' own law, fewer re-parks) banks more of the 50 for the pocket.

## Live checks for the main agent

- Trace Bond's fired-round stops vs #20's incoming-round stops in the Kiko corridor at
  t25200-25400 (the v4 pocket bill): the asymmetry's blocker has NO static gate shadow (the
  NF_KIKO_PROBE grid reads margin inf both ways at those spots). gdbwatch the Bullet_update
  flight paths both directions, the muzzle spawn offsets, and the 0x14 weapon record's gravity
  bit (+0x6C 0xa00, bit 0x8) — if the player's 0x14 rounds carry gravity and the AI's aim
  compensates, the walker's crosshair aim needs the drop lead, which would explain "his rounds
  land at d 2.5-6.5, Bond's taps burn dry at the same ranges" from BOTH the corridor and the
  lip.
- The loop-mouth armour take (32.26,0.87,31.14): confirm the armour pane's value on the HUD at
  the door face (~t24000) reads 50→draining, to pin the take's bank visually.

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session; nothing to copy.
