# mine-shield-by-guard

Branch `s/mine-shield-by-guard`, from origin/master (d43c28ce), merged origin/master
(6042ee53, batch-1101v) before the final push.

## What's done

`update_player_bullets` (crates/nf-game/src/drone.rs) now lets the mines, the drones, the stuck
tripbombs and the world geometry compete by hit distance — the game's first-hit rule — instead of
the fixed pass order that tested the 0xFE mines before the drones.

The game's rule, from the decompile:

- `Bullet_update` 0x12e7b0 arms the flight HITTEST per tick (mode 0x101/0x201) and moves the
  round; the pick collects EVERY candidate along the flight segment — `Collide_Pick` 0x1eb320
  walks the straddled cels, `Collide_PickObj` 0x1eae80 admits each object and stores the hit's
  distance at HITDATA+0xc.
- `Collide_Update` 0x1ea398:119-158 then QuickSorts the object's hit list by +0xc and re-links it
  nearest-first (entries still at 1e8 are dropped); the walk at :216-229 consumes the sorted list
  and stops at the first solid node.
- `Bullet_CollisionHandler` 0x12ce80 (disasm 0x12cff0-0x12d004) state-2 kills the round on that
  first solid node — nothing farther along the path applies.

Port changes (all in `update_player_bullets`):

1. The drone candidate query (`best`, the flight-trace block with the cel-scoped world stop `tw`)
   moved ahead of the mine and stuck-gadget passes, so a guard nearer than scenery/mine/gadget is
   known before any take is applied. This also makes the pre-existing breakable gate live — its
   comment already said "a guard nearer than the scenery takes the round instead
   [M 0x1ea398:216-229]" but `best` was still None at that point.
2. The stuck-gadget and mine candidates (`gadget_t`, `mine_t`) are computed once per round
   (pure queries) before the takes.
3. Gates (tie rules: the mine and the gadget beat the guard, the gadget beats the mine —
   mirroring the pre-existing `t <= mt` gate):
   - the gadget take now also requires the world (`tw`), no nearer mine and no nearer guard;
   - the mine take now also requires the world (`tw`), no nearer gadget and no nearer guard;
   - the guard arm in the match now also requires no nearer mine and no nearer gadget.
   The grenade block's duplicate in-block world/mine queries were replaced by the shared ones.

Side effect of the same rule: a round that detonates a mine can no longer ALSO hit a guard behind
it in the same tick (the old code applied both), and a round that hits a guard can no longer also
detonate a mine behind it.

## How it was verified

Every run on the final merged tree unless noted:

- `cargo test --offline --release -j2 -p nf-game` — **1028 passed, 0 failed** (all targets,
  including `ai_live` 37/0 — the guards' combat runs against the same pass).
- `walkreal_07000015_live` — **3 passed, 0 failed**: the 07000015 walk shoots the 0xFE mines from
  range on the real routes (#462, #383/384/385, #526, #447/448/449), the strongest existing pin
  that the mine take still works with the new gates.
- `tripwire_shot_live` — 4/0; `interactables_07000015_live` — 15/0; `remote_mine_detonate` — 3/0;
  `walk_07000015_live` — 2/0; `census_07000015` — 0 passed / 3 ignored (the census prints only).
  (All but walkreal ran on the pre-merge tree; the merge touched no bullet-pass code — the
  drone.rs diff after the merge is exactly this task's.)
- The merged tree compiles clean (`cargo check -p nf-game`).

The full `tools/test/suite.py` run did NOT produce a total: the background runner was reaped by
the sandbox between tool calls both times it was started (empty `suite.out`, no process). The
targeted runs above are the quoted totals; the suite should be re-run on the branch.

## What's left

- Re-run the fast suite on the merged tree (see above).
- The new-behaviour pin (a guard in front shields the mine; aside detonates) is NOT landed as a
  test — see "Live checks" for the exact scenario and the two harness findings below. A test
  binary was built and got leg A (guard shields) green, but leg B kept missing the mine through
  `Mine::segment_t` — see the second finding.
- The other round-takers (breakables, monitors, destroy leaves, the copter, the rigid bodies) do
  not yet compete four-way with the mines/gadgets; the guard does (via the moved query) and the
  breakables gate against the guard, but e.g. a monitor console or a rigid-body lamp nearer than
  a mine still applies independently. Same first-hit rule, next increment.
- Live-verified bullet-vs-mine distances on 07000015 (the port's `Mine::segment_t` t against a
  PCSX2 breakpoint value).

## Harness findings (for whoever lands the live pin)

1. The P99 (weapon 0x2a) carries `weapon_data+0x70 = 0x00000004` — the contact-blast flag, radius
   +0x08 = 6.0, damage +0x0C = 75.0 [L work/ee_snapshot.bin @ weapon_data 0x2bf150 + 0x2a·0x10C].
   Any P99 round that hits a guard explodes at the contact, and `Explode_Propagate` 0x17c518 case
   0x4B chains a mine within 6.0 u. A shield test must pin the guard MORE than ~6 u in front of
   the mine or the guard hit's own blast detonates it — that is game behaviour, not a port bug.
   The spawned guard's own AI grenades also chain mines (walkreal_07000015's door[32] note); the
   test fixture has to drain `drones.grenades`/`pending_blasts` per tick.
2. [S] unverified, observed once in the harness: `Mine::segment_t` returned None for a ray aimed
   straight at the mine's posed world-sphere centre (`Mine::centre`) on #462, and also for a ray
   at the beam's mid-span z — while walkreal's real-route shots (from the west nav lane) hit
   mines fine. Worth a look when the mine pick is next touched: the posed sphere centre may sit
   off the beam's own triangles for the crossing beam.

## Live checks for the main agent

1. Guard shields a laser mine. Level 07000015, the silo-crossing beam #462 (x 46.8, z −2..3.2).
   Stand Bond ~10 u east on the crossing lane (x ≈ 57, z ≈ 0.6), aim at the beam, and place a
   live guard on the fire line ~7 u in front of the beam (x ≈ 53.8, z ≈ 0.6). Fire one P99 round:
   EXPECT the guard takes the hit (the AI flinches/attacks) and the mine stays up. Step the guard
   aside (z + 1.5) and fire again: EXPECT the mine detonates (the bullet cause, not a blast —
   keep the first round's contact blast out of reach per finding 1).
2. Trace check on the same scene: watch HITDATA+0xc (the sort key) for the guard's and the mine's
   entries during the flight — the guard's should sort first when it is nearer
   [M Collide_Update 0x1ea398:141 QuickSort, :216-229 walk].

## Usage

Not visible to me: this session exposes no token, credit or context-window figures (no input /
output token counts, no cost display). The sandbox clock at the start of the finish step read
2026-11-01 ~19:15 UTC, about 115 minutes after the task message arrived.
