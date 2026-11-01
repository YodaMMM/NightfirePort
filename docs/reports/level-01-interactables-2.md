# level-01-interactables-2 — every player-touchable kind of 07000001 behaves like the game

## What's done

The census in `docs/levels/07000001.md` maps every placement kind; this task closed the three
kinds a player touches that no test drove through the real path yet (the first pass's
`interactables_07000001_live.rs` had 7 tests; `interact_07000001_live.rs` and `walk_07000001_live.rs`
already pinned the doors, shoji, breakables, armour/ammo/monitor-E/shooter/muxes/TouchOnce/fans/
exit/spawner). Three tests added to `crates/nf-view/tests/interactables_07000001_live.rs`, each
driving the real walk/E/shoot input, each with a sister that fails on the wrong result:

1. **The comms console's shot arm** (0x30 #105, p1 = 0): the tag's health starts at 1.0 [D #105
   params; M `Monitor_Create` 0x1b08b0:16-21 — the `param_3+0x30 == 0` arm stores 0x3f800000];
   `Monitor_Update` 0x1afcc8:23-27 drains the frame's class-5 contact chain
   (`Collide_GetDamageNObjects` 0x1ee220 sums every HITDATA+8) off the health; health < 0 →
   state 3 + the 0x0600027 explode in the same arm (0x1afcc8:32-45); state 3 never leaves
   (0x1afcc8:58-59) and the E feed is dead with it (`Monitor_Activate` 0x1b0058 gates on state 0).
   Sister: a fresh load's console sits idle at health 1.0.
2. **The security camera's sight latch** (0x28 #280 at (−29.19, 7.87, 1.59), p1 = 5 cone, p2 = 6
   range, p3 = 0 out, p6 = 1 monitor group [D]): `Sensor_InCone` 0x1af4f0 (cone sphere +
   `Collide_LineOfSight`); 31 consecutive seen ticks (`0x1e <`, [M 0x1af618:211-216]) flip the
   state-0 sweep into the state-1 track. The 2-s sight arm's `switch_channels[p3] = 1` is gated
   by `+0x58 != 0` [M 0x1af618:154-157] and #280's p3 = 0, so being seen writes nothing — the
   test pins the latch AND the silent channel board. Sister: Bond at the spawn (~30 m away)
   holds the sweep with no cone hit.
3. **The radio fax's E restart** (0xD9 SP #79 "radioroom_FaxMachine", scripts 0x06000016/17,
   modes 64/8, mask 4 = the E [D params; U-049]): the mode-64 prime's chain leaves the machine
   body standing with bc as the last page and NO bd [M `SP_LoadScript` 0x1dbc08:41-55 — Play +
   Update + Script_Stop + the trailing Update; D 0x06000016 stream 6's bc/bd alternation every
   0xa ticks]; the E is the class-0 entity pick [M `Player_Activate` 0x1a5170:121-122] →
   `SP_Activate` 0x1dc338:79-89 (mask & 4 → the +0x2C flag, the SCRIPTINFO+0x9C un-hold) →
   `SP_Update`'s state-1 arm restarts the script (`Script_Play`, state 2 [M 0x1dbda8 case 1]) —
   the page cycle re-runs to its bd op. Sister: no bd page before the E.

**No port gap needed fixing.** All three paths were already ported and wired: the monitor's
bullet arm in `drone.rs`'s bullet pass (`Monitors::bullet_hit` on `b.damage`), the sensor's tick
in `sensor.rs` (`Sensor_Update`'s state machine with the 0x1e counter), and the SP use path in
`useobj.rs::player_activate` (mask-4 SPs pick on their live entity spheres only, [M
0x1a5170:121-122]). The tests only added the missing kind-level pins. One test-side correction
during the work: the fax sister first assumed the prime leaves the pages unspawned — wrong
against the port's own load state; the [M 0x1dbc08] prime-chain reading (stop point mid-stream-6)
is what the load state reflects, and the sister was rewritten to pin bd's absence instead. No
existing test contradicted the decompile.

## How it was verified

- `interactables_07000001_live`: **9 passed; 0 failed; 1 ignored** (the ignored sibling is the
  first pass's bonus walk-over, its named floor blocker unchanged), `--test-threads=1`, slim
  on-foot ISO.
- `walkreal_07000001_live`: **6 passed; 0 failed** (198 s).
- `nf-game` (lib + integration): **1015 passed; 0 failed** across its bins.
- Full four-crate suite via per-binary chunks (`--test-threads=1` per binary, the suite.py
  method; this sandbox kills background jobs between tool calls, so the run went through
  foreground chunks): **283 of the 1058 test binaries run — 822 passed; 4 failed; 49 ignored.**
  - The 4 failures are environmental, on master's own code, none from this branch (which adds
    one nf-view test file + docs only): the 3 `disc_golden` tests the slim ISO is allowed to
    fail, plus `identity_disc` — the slim on-foot ISO's 518,435 sectors vs the full retail
    identity's 1,191,072 (docs/build-identity.md), the same slim-ISO class.
  - The chunk covered the nf-game + nf-formats + nf-audio binaries (all of them) and nf-view's
    alphabetical range through `doublecross_transitions_live`; the remaining nf-view range
    (roughly `door` onward alphabetically) did not fit the window. The touched binaries all ran
    green above; three nf-formats bins the first build pass missed (`goggles_panes`,
    `night_quarter_census`, `xray_only_census`) were built explicitly and ran green (2+2+2).

## What's left

- The rest of the nf-view suite range on this tree (the ~775 binaries past the chunk boundary) —
  mechanical time, no code pending.
- The first pass's `#[ignore]`d bonus walk-over (the west-hall bonus floor off the walker's
  reachable set) — unchanged, named in the test.
- A live check would still strengthen the fax pin (below).

## Live checks for the main agent

1. **The fax pages** — fieldprobe Bond E-ing the fax at (17.56, 6.21, −0.96) in the radio room:
   expect the printed-page art at the fax's out-tray to cycle (bc/bd entities) after the E; the
   port pins the entity spawns, the visible page motion is the render layer.
2. **The camera sweep stand** — the port latched the camera from the gallery stand at
   (−28.86, 6.40, 8.05) [L port walk]; a real-game stand there should show the camera turning to
   track Bond within ~0.5 s and no HUD/channel effect (p3 = 0).

## Usage

not visible to me

## ntfy failed

All three fallbacks failed from this sandbox (egress blocked):
- `curl -sS -m 20 -H "Title: NightfirePort" -d "…" https://ntfy.sh/nightfire-yoda-8k3x` →
  `curl: (28) Connection timed out after 20000 milliseconds`
- python urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
- `wget -q -O- --post-data "level-01-interactables-2: done" https://ntfy.sh/nightfire-yoda-8k3x` →
  timed out after 30 s with no output (killed)
