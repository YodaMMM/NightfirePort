# Explosion debris fades out

Debris_Update steps the piece's alpha byte obj+0x106 down as it dies, but the port draws debris at full alpha until it vanishes. Read the fade (start, step, end) in the decompile and port it into the debris pieces' sim state so the draw can use it; pin the per-tick alpha. Name the draw hook needed (the draw side is the main agent's).

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/debris-alpha-fade`, report in docs/reports/debris-alpha-fade.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-09T14:52Z

---
## Report (done, 2026-11-09T16:14Z)


---
## Report from the worker (done, 2026-11-09T16:14Z)

# debris-alpha-fade — report

Branch `s/debris-alpha-fade` from `origin/master` @ ea11f1312. Task: read the debris fade
(start, step, end) in the decompile, port it into the debris pieces' sim state so the draw can
use it, pin the per-tick alpha, and name the draw hook. No draw changes (mine or otherwise).

## Headline

**The fade is already in the port's sim state, exactly per the game — on all three piece lists.
The symptom ("debris draws at full alpha until it vanishes") is a draw-side gap: nothing in the
renderer reads the byte.** This report pins the sim contract with new real-input tests, corrects
two doc lines that claimed otherwise, and names the draw hook precisely. No sim fix was needed;
no draw code was touched.

## The game's fade, with evidence

Per class-0x10 debris object (`record` = `obj+0xE0`, the object's motion record):

- **Start**: `obj+0x106 = 0xff` at create — `Debris_Create` 0x1aaa00:31 and `Debris_CreateEx`
  0x1ac808:27 [M] (and every object's init, `control_init_object` 0x134630:21 [M]).
- **Step**: `record+0x32 = 1` on both creators [M 0x1aaa00:35, 0x1ac808:32]; `Debris_Update`
  0x1aac00:18-24 subtracts it from `obj+0x106` each tick and stores the result **only while the
  result stays > 0** [M].
- **End**: the piece dies (`obj+0xFE |= 1`, deleted by `control_movement_object_handler`
  0x1336f8's post-update pass [M]) the tick the subtraction would store ≤ 0 — the byte keeps its
  last stored value (≥ 1) until then [M 0x1aac00:18-24, 56-63] — or when its tick count runs out
  (`record+0x30 < 1`, 0x1aac00:14 [M]). Blast pieces (`Explode_Update`'s arm, kind 5/6 metal)
  live `Rand(90)+90` ticks [M 0x17c350:63-67 over param 5 = 180, 0x1ac808:29-31] → death by
  life, last stored alpha 76..165. Breakable pieces live `(0x81>>1) + Rand_Rand((0x81>>1))` [M
  0x1aaa00:34-37] — 27..54 on table entry 0x10 [D ACTION.ELF 0x3037c0] → last alpha ≥ 201. No
  real record reaches the alpha kill; the rule is pinned at unit level anyway (test below).
- **Draw hook (game)**: `Game_Draw`'s pass 0 walks `View_DrawObjList`; per object it calls
  `View_SetupRenderModes_Tweak(viewer, obj)` (0x1e69a0:77 [M]), which passes
  `psiSetTweakARGB(A = obj+0x106, R = obj+0x103, G = obj+0x104, B = obj+0x105)` (0x1e5888:46-49
  [M]; debris tint bytes are 0xff and never written afterwards — census over every 0x106 writer,
  no class-0x10 writer touches +0x103..0x105). `psiSetTweakARGB` 0x124770 stores
  `TAmbient = (R, G, B, A) / 255` @0x2650d0..dc [M disasm], and `FillMatrixChainRot` 0x122310:241
  feeds it to `sceVu0LightColorMatrix` — the **A byte is the ambient alpha of the VU1 lighting
  matrix** every frame the piece draws [M]. A debris object carries `obj+0xFA = 0x20` only (set
  per tick by 0x1aac00:51), so it draws through `View_DrawObjectGlist` 0x1e6780 [M].
  `obj+0xF0` bit 0x20000 (set at create) halves-NO — QUARTERS the byte (`>>2`) under nightvision
  only (`viewer+0x236 == 1` [M 0x1e5888:13-18]).

## The port's sim state — verified, not changed

- `shatter::Debris.fade` (breakable + blast pieces): `crates/nf-game/src/shatter.rs` —
  `spawn_debris` stores `fade: 0xff` (:270), `update_debris_list` steps it −1/tick and kills at
  `fade <= 1` or `life < 1` (:296-310) — equal to the game's `alpha − 1 > 0` rule for step 1.
- Blast pieces spawn with `fade: 0xff` (`drone.rs:21164`) and tick through
  `Shatter::update_debris_list(&mut self.blast_debris)` (`drone.rs:21148`), newest-first like
  the game's list walk.
- `effect::Debris.alpha` (impact pieces): ticked in `Effects::update`
  (`crates/nf-game/src/effect.rs:573-582`).
- The old-code red test the driver asked for does not exist **because there is no sim-side
  defect to redden**: the per-tick pins below are green on old code and that is the point — they
  fix the sim contract the draw must consume. The wrong code is in the draw (next section).

## Tests added (all green)

- `crates/nf-game/tests/debris_alpha_fade_live.rs`:
  - `a_shot_vases_pieces_step_their_fade_byte_down_one_per_tick_and_die_by_life` — a real
    bullet-kill on real 07000001 data (the merged break_debris_live harness): every piece 0xff at
    create, fade == 0xff − k after tick k, life runs down with it, the population after tick k is
    exactly the create rows with life > k, all gone by their tick count (≤ 54), last alpha ≥ 201.
  - `the_fade_byte_kills_the_piece_when_its_step_would_reach_zero` — the alpha-kill arm at unit
    level: fade 3 → 2 → 1 → dead; the byte never holds ≤ 0 while the piece draws [M 0x1aac00:18-24];
    life-1 and life-0 entries die without a fade step [M 0x1aac00:12-14].
- `crates/nf-view/tests/debris_alpha_fade_live.rs` (real input: level load, Bond at the level's
  own MGL pickup on 07000011, one contact round through the real fire control — the
  grenade_bounce_debris_live harness):
  - `blast_debris_fade_steps_one_per_tick_from_ff_and_dies_by_life` — every spawned blast piece:
    fade 0xff at spawn [M 0x1ac808:27], fade == 0xff − k at age k, trail length in the
    Rand(90)+90 window, every observed fade > 1 (death by life), pool empty after tick 179.
  - `a_late_piece_keeps_the_same_per_tick_fade_rule` — later spawn ticks (the debris arm's
    modulo gate) follow the same rule from their own spawn tick.

## The draw gap — what is wrong, and the hook (main agent's work)

- The view's per-frame debris pose reads only pose fields: `scene.rs:11970-11993` (kind-2 arm,
  `debris_slot_map` → piece → matrix). `effect::Debris.alpha` is read once — as a **spin-age
  clock** (`scene.rs:11973`, `age = 255 − d.alpha`), never as transparency.
- `shatter::Debris.fade` is read by **nothing** in nf-view.
- The debris colour path is hard-coded white: `scene.rs:13543`
  (`if matches!(a.effect, Some((2, _, _))) { return one(); }`), pinned by
  `debris_shading_live.rs:17` (keep that test's RGB pin — the game's debris RGB tweak is 0xff;
  only A carries the fade).
- The debris pool instances bake their vertex rgba at build (`pool_record`/`pool_replay`,
  `scene.rs:7837-7870`, `add_instance_tagged`'s RGB-only mult at :8069-8080); per frame only the
  attachment matrices move. **The hook**: feed each live debris slot's piece alpha into the
  drawn instance every frame — the same job `shard_draws` (`scene.rs:8469`) already does for
  glass shards via `shard_alpha` (the game uses `psiSetTweakA` for shards [M 0x12bef8:39] and the
  full ARGB tweak for debris). Data source: the piece the slot map already names
  (`debris_map` at `scene.rs:11592` → `.fade`/`.alpha`); value = the byte as-is (1..255; the
  piece is deleted before it could reach 0). The game's exact rule to reproduce: A = obj+0x106
  every frame, RGB = 0xff, through the object's own glist draw (0x1e6780), quartered under
  nightvision (0x1e5888:13-18).

## Doc corrections (evidence-forced, surgical)

- `docs/atlas/effect.md`: "The game applies no per-particle alpha at draw (the byte is
  bookkeeping only here)" — **refuted**; replaced with the tweak-A chain above.
- `docs/systems/break-debris.md`: "no draw-side alpha read found for class-0x10 objects" —
  **found**; replaced, plus the byte-keeps-its-last-value death detail.
- `docs/atlas/gs-leaf.md`: `>>2` is a quarter (nightvision dim), not "halved"; +0x106's writers
  are not init-only (Debris_Update, Corona_Update, Leaf_Update, …).

## Verification (binaries and pass counts)

- `nf-game`: 118 test binaries, all ok (754-pass lib suite included). My new file: 2/2.
- `nf-formats`: only the 3 `disc_golden` failures — the documented slim-ISO allowance.
- `nf-audio`: ok.
- `nf-view`: the 7 debris-guard binaries all ok — `debris_alpha_fade_live` 2/2,
  `blast_debris_draw_live` 1/1, `debris_shading_live` 2/2, `grenade_bounce_debris_live` 2/2,
  `impacts_live` 1/1, `impact_remainder_live` 3/3, `copter_death_debris_live` 5/5.
- No `crates/*/src/` file changed, so no `walkreal_*` binary's behaviour can move; the full
  nf-view set did not fit the session (108 binaries). The suite command otherwise ran with the
  documented slim-ISO result.

## Live checks for the main agent (PCSX2)

1. `gdbwatch.py` on a live blast debris piece's `obj+0x106` (find one via the class-0x10
   objects after a barrel/launcher blast): expect −1 per tick from 0xff while the piece lives,
   no store below 1, then `obj+0xFE |= 1` and deletion the tick `record+0x30` (s16) hits 0.
2. `fieldprobe.py`: poke a live piece's `obj+0x106 = 0x40` mid-flight — the piece should dim
   immediately (TAmbient A path), back to full on a poke of 0xff. That settles the VU1/GS side
   the decompile cannot (the ambient matrix is proven; the leaf's final blend is not).
3. Nightvision: goggles mode 1 (viewer+0x236 == 1) with a live piece — expect the drawn piece at
   a quarter of its byte (0x1e5888:13-18).

## What's left

- The draw-side change itself (main agent): the hook above; the new tests + the sim contract
  pins are its safety net (`debris_shading_live`'s white-RGB pin must survive).
- Optional: a render check (`--bond` spots) once the draw consumes the byte — e.g. 07000011
  at the MGL shelf (`--bond` at the pickup, facing yaw 0.35) while a round bursts: pieces
  should visibly dim toward ~30-65% alpha over their last second, not pop out at full alpha.

## Usage

- My own session's token/context/cost figures: not visible to me.
- Research subagents (from their final messages): decompile-audit agent — prompt 155,269 /
  completion 9,248 / total 164,517 tokens; port-audit agent — prompt 64,699 / completion 6,278 /
  total 70,977 tokens. No cost figures shown by either.
