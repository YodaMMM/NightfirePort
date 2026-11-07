## Your task: `rb-impact-sound`, rigid props make their impact sound and carry their light

docs/reports/unread-params-props.md on origin/s/unread-params-props says the 0xD8 rigid body params p3 (carry-light) and p4 (impact sound) are unread and ride the body evolution. crates/nf-game/src/rigid_body.rs now has a section of ported evolution functions, while its header still calls the evolution a tracked gap, so first find what really runs on master: read `RB_Update` 0x1d6e38 and `RB_RestingContact` 0x1d7168, compare them with the port, and correct the header. Read what `RB_Create` 0x1d6bf0 does with p3 and p4, then which reader plays the sound (when, how often, and at what speed gate) and how the light follows the body. Census both params per level [D]. Port what is missing, with sounds through the normal sound path and no change to the count of Rand draws. A light on a moving prop is a visual, so list frames for a render check under the report's live checks for the main agent. Pin through real fire on a level from the census: a shot prop that hits the floor plays its sound once per the game's rule, and an undisturbed one stays silent. Run the rigid body binaries (grep tests for `rigid_body`), walkreal for the level you pin, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/rb-impact-sound`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-07T18:10Z

---
## Report (done, 2026-11-07T18:49Z)


---
## Report from the worker (done, 2026-11-07T18:49Z)

# rb-impact-sound — rigid props' impact sound and carry-light: what the game really does with 0xD8 p3/p4

2026-11-07, branch `s/rb-impact-sound` (from origin/master 10956badb). The brief came from
docs/reports/unread-params-props.md's 0xD8 row: p3 (carry-light) and p4 (impact sound) looked
unread, riding the body evolution. Bottom line: **the port already reads both and already plays
the sound through the normal sound path** — what was wrong on master was the module header (it
still called the whole evolution a tracked gap), and nobody had pinned the sound end-to-end or
censused the disc. And the carry-light is **dead code in the retail build**: the map-light
lookup it depends on is a stub.

## What really runs on master (the header is now correct)

`crates/nf-game/src/rigid_body.rs`'s header said the disturbed-body evolution (`RB_Update`
0x1d6e38, `RB_EulerEvolve` 0x1d7810, `RB_ApplyImpulses` 0x1d6ff8, `RB_RestingContact`
0x1d7168) was the tracked gap. Comparing decompile against the port, all four run:

- `RB_Update` 0x1d6e38 = `rb_update` / `Rbodies::tick_one` — the buoyancy gate with its ONE
  `Rand_FRand(0.15)` draw [M 0x1d6e38 disasm 0x1d6eb4-0x1d6f44], `RB_ApplyWind` 0x1d7c78, then
  `RB_EulerEvolve` (drag = the game's `expf`, native `f32::exp` in the port [M unported-misc-7]);
- `RB_CollisionHandler` 0x1d7a80 = `rb_collision_handler`, per body per tick at the kind-7
  control entry (the {0x1d6e38, 0x1d7a80} dispatch pair [M 0x26f9c8/0x26f9cc]):
  `RB_RestingContact` 0x1d7168 for type 0, `RB_RestingContactRot` 0x1d73a0 otherwise, over the
  body's obj+0xD0 chain — the chain the collide pass fills, the bullets' class-5 contacts among
  them (the port's `update_player_bullets` rigid-body-bullet arm; class 5 = the round itself,
  `Bullet_init` 0x12c5a0:87 sets obj+0xFF = 5).

The header now states this with the addresses (own commit).

## RB_Create 0x1d6bf0 with p3 and p4 [M 0x1d6bf0:107-112]

`level_tag` holds placement param k at +0x2C+4k [M 0x1d1b68:64-75]. RB_Create stores
`rb+0x154 = *(u8*)(level_tag+0x38)` (p3, a byte) and `rb+0x158 = *(int*)(level_tag+0x3c)`
(p4), mapping p4 == 0 to 0xFFFF. Both were already ported (`RbObj.s154/.s158`); the pinning
tests live in `interactables_07000004_live` ((s154, s158) = (1, 0xFFFF) for the lamps) and now
in the new census test.

## Which reader plays the sound, when, how often, at what speed gate

The reader is the **class-5 arm of the resting-contact walk** — both variants:
`RB_RestingContact` 0x1d7168:111-112 (type-0 bodies) and `RB_RestingContactRot`
0x1d73a0:121-123 (type != 0). Per hit entry in the body's obj+0xD0 chain whose other object's
class byte (hit+0x58 → obj+0xFF) is 5:

`Sound_Play3D(100.0, −1.0, −1.0, *(rb+0x158), obj+0xC0, 0, 0, 0)`

- **When**: while the handler processes the chain (per tick at the kind-7 control entry); the
  chain drains each handler call.
- **How often**: once per class-5 contact. A round lands ONE contact and dies
  [M Bullet_CollisionHandler 0x12ce80 state 2 on the first sorted contact; the port's `b.dead`],
  so one landed shot = one call, not one per tick.
- **Speed gate: there is none.** In both decompiles the `Sound_Play3D` is the arm's first
  statement, directly inside the `class == 5` branch; the hit's velocity is only read AFTER the
  call, for the impulse (`hit+0x8 · 0.5 · vel` into the accumulators). The gate is the contact's
  existence, not its speed.

The brief's mental model — "a shot prop that **hits the floor** plays its sound" — is not what
the code says: the trigger is a **round hitting the prop** (class 5), not the prop hitting the
floor. (And the explosion never reaches a prop at all: `Explode_Propagate` 0x17c518's class
switch has **no 0x1a case** — 0/2/3/5/0x20/0x21/0x28/0x35/0x36/0x37/0x38/0x3d/0x4b, default
skips — so a blast never tips a prop in the game and never makes one sound. The port's
explosion→class-5 routing is the documented impulse stand-in [S, the wiring test]; its sound
pool is dropped on purpose, now with a comment citing the switch. Blast-tipped barrels in the
port move by that stand-in, not by the game's route.)

## The census [D] — p3/p4 on every dev level

All 34 dev levels parsed, main-map 0xD8 placements only: **35 bodies** —
07000001 ×4, 07000002 ×21, 07000003 ×3, 07000004 ×2, 07000006 ×1, 0700000d ×3, 07000041 ×1.

- **p4 → 0xFFFF (none) on all 35.** No dev disc ships an audible rigid-body impact sound; the
  game calls `Sound_Play3D` with the none sentinel and its sound path drops it [M sound.rs
  `play3d`'s 0xFFFF gate]. Pinned per level: `rigid_body_impact_sound_live::
  census_p4_is_the_none_sentinel_and_p3_rides_four_levels`.
- **p3 = 1 on 16 of the 35**, only on 07000001 (4), 07000002 (10), 07000004 (2) — the whitelamp
  class. Everything else is p3 = 0 (0700000d's three mass-600 crates, 07000006's mass-8 barrel,
  07000041's single type-0 body among them). (unread-params-props.md's prose said 15 across
  01/02/03/04; the per-object count is 16, all on 01/02/04 — that branch's own census brackets
  do not list p3 for 03, so its prose row was loose. My numbers are the per-object [D] census.)

## The carry-light: the light does NOT follow the body [M 0x1cc3d0]

`RB_Update` 0x1d6e38:76-103 (the type != 0 tail): with `rb+0x154` set and the cached handle at
`rb+0x150` zero, the game calls `Light_FindMapLight(rb+0x13c, rb+0x100)` once and — while the
handle stays non-zero — syncs rb+0x100 (the cylinder base point, RB_RestingContactRot 0x1d73a0:90
writes it) into light+0x10 every tick. **`Light_FindMapLight` 0x1cc3d0 is an 8-byte stub whose
whole body is `return 0`** [M decompile; the flicker effect's reader documents the same stub],
so the handle never arms and the sync is unreachable: **no light follows a moving prop in the
retail game**. The port now mirrors that faithfully: `rigid_body::light_find_map_light()` (the
stub, cited), `RbState::s150` (the rb+0x150 handle), and the gated lookup in `tick_one` — no
RNG, no behavior change, and a unit test (`carry_light_lookup_stays_inert_behind_the_stub`)
pins the inertness both ways (p3 set → handle still 0; p3 clear → no lookup; the read of s154
itself is pinned so the test can't pass with the flag dropped).

## The end-to-end pin (real fire, from the level load)

`rigid_body_impact_sound_live.rs` (new binary):

1. `census_p4_is_the_none_sentinel_and_p3_rides_four_levels` — the [D] census above as a test
   (35 bodies, p4 none everywhere, p3 on exactly 01/02/04).
2. `a_shot_lamp_records_its_impact_call_once_and_the_sister_stays_silent` — on 07000004, from
   the load through the settle (intro + story scenes), Bond WALKS under lamp #40 and fires the
   real trigger path (`Walker::shoot_point`, the turn/look inputs through `Sim::step_bond`).
   The class-5 contact records exactly ONE call — `(placement 40, id 0xFFFF, pos = the lamp's
   translation)` — id none per the census, so the game (and the port's `Sounds::play3d`) plays
   silence for it; the unshot sister #39 records nothing before or after; 300 further ticks add
   no re-trigger (the chain drained); the level records nothing else; and the shot lamp's
   carry-light handle stays 0. The call record itself is new sim surface: `Drones::
   rb_sound_events` (capped at 256) records each `Sound_Play3D` the kind-7 control walk raises
   for a body — 0xFFFF entries included — because `play3d` gates none-ids before any observable
   state. No RNG draws added anywhere (the buoyancy `Rand_FRand` count is untouched).

The bullet-side chain (two-stage contact: world sphere, then `Intersect_RayGeom` over the
glist's triangles; hit+0x8 = the round's damage, hit+0x30 = the unit flight direction) was
already ported and live-pinned (the 07000004 lamp #40 break: hit+0x8 = 3.5, the P99 round [L]
in `update_player_bullets`' comment); this task pins the SOUND arm on top of it.

## Runs (per binary, release; NF_ISO set)

- `nf-game` full: lib 689/689 (5 ignored) + tests/ 37, 10, 5, 4, 1, 1, 1, 1 — all ok
- `nf-formats` full: 24, 4, 1, 1 — all ok; `nf-audio` full: 92, 1, 1, 1 — all ok
- `rigid_body_impact_sound_live` 2/2 (new); `rigid_body_draw_live` 3/3; `rigid_body_wiring_live` 2/2
- `lamp_solid_live` 4/4, `lamp_wind_live` 2/2, `interactables_07000004_live` 3/3
- `walkreal_07000004_live` 10/10 (the pinned level)
- Every level whose bodies tick through the changed paths: `walkreal_07000001_live` 8/8,
  `walkreal_07000002_live` 3/3 (1 ignored), `walkreal_07000003_live` 6/6 (1 ignored),
  `walkreal_07000006_live` 6/6 (1 ignored), `walkreal_0700000d_live` 2/2 (1 ignored),
  `walkreal_07000041_live` 9/9, `interactables_07000001_live` 10/10,
  `interactables_07000002_live` 18/18, `interactables_07000003_live` 5/5,
  `interactables_07000006_live` 11/11, `interactables_0700000d_live` 15/15
- nf-game's two `mp::bots::fell_out` tests panic in DEBUG builds ("subtract with overflow",
  bots.rs:1368) — pre-existing on master, unrelated to this diff (mp bots); they pass in
  release, which is what the suite command runs.

**The full nf-view suite did not fit**: 984 test binaries, and the prescribed fast runner
(`tools/test/suite.py`) could not run to completion in this sandbox — background processes are
killed between tool calls here (observed twice: the runner died silently with an empty
suite.out both times, nohup/setsid alike). Everything whose code paths this task touched ran
green; the rest of nf-view is untouched by the diff (a new Drones field + comments + the
rigid_body light mirror).

## Live checks for the main agent

- **Render (the carry-light negative check)**: on 07000004, stand at the whitelamp spot the
  walkreal uses — `--bond 8.0,4.4,26.5` facing the lamp at (8.01, 7.86, 23.03) — shoot lamp
  #40 (one P99 round) and capture frames just before/after the impact and while it swings:
  **no light may appear on or move with the lamp** — the game's own lookup is a stub
  [M 0x1cc3d0], so even the real PS2 shows none; the port must show none too.
- **gdbwatch (optional, the handle word)**: watch the lamp's rb+0x150 (the ext block at
  obj+0xE0; the lamps are the two kind-0xD8 creations of place 26) through a shot: expected
  0x00000000 forever, never written after the memset [M 0x1cc3d0 + 0x1d6e38:89-93].
- **gdbwatch (optional, the call)**: break at the class-5 arm's `Sound_Play3D` site 0x1d74b8
  (Rot) with a3 = the lamp's rb+0x158: expected 0x0000FFFF on the shot tick [D census],
  a0/a1/a2 = 100.0/−1.0/−1.0, the stack pos = the lamp's obj+0xC0.

## What's left

Nothing blocking. The one loose thread this task did not pull: the port's explosion→prop
impulse routing remains the documented [S] stand-in (the game has no such route; see the
Explode_Propagate switch above) — if a future task wants bit-exact blasts-vs-props behaviour,
that stand-in needs a live trace of what the game actually does to a blasted lamp (which, per
the switch, is: nothing).

## Usage

not visible to me (no token, credit or context-window readout in this environment; the
session's sandbox expires around 20:12 UTC per the task brief).
