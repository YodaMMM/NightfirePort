# gas-rng-draws — the gas puffs now draw from the shared RNG, in the game's order

## What was wrong

The port's `Gas_Init` path (`GasPool::init_ex`/`init`, crates/nf-game/src/gas.rs) consumed **zero**
draws from the shared bit-exact RNG: the spin and the sign came from a per-pool counter hash
(`jitter()`, marked [S]), and `Env_Breath`'s x-ray variant A did not make the game's yaw draw at
all. Every real `Gas_Init` call in the game consumes draws, so every puff the port spawned without
them shifted the whole downstream stream (AI decisions, bullet spread) relative to the game.

## The game's draws, with the addresses

`Gas_Init` 0x1aacc8 draws, in order [M decompile 0x1aacc8:72-80 and disassembly]:

1. the spin: `Rand_FRand(π/2 · REC_FRAME_RATE)` — `jal Rand_FRand` at **0x1ab060** (the argument
   REC_FRAME_RATE·1.5707964 is built before the call; decompile :66-72);
2. for kind 0xb only, the result is quartered: `mul.s $f0, $f1, 0.25` at **0x1ab078-0x1ab088**
   (decompile :74-76);
3. the sign: `Rand_Rand(0x100)` — `jal Rand_Rand; addiu $a0, $zero, 0x100` at **0x1ab08c-0x1ab090**,
   `sltiu $v0, $v0, 0x80; bnez` at 0x1ab094-98 = negate the spin exactly when the draw is ≥ 0x80
   (decompile :77-80);
4. the velocity branch: kind 4 **with a given velocity** consumes `Rand_FRand(0.05)` ×2, ADDED RAW
   to the x/z components (`vel·REC + draw`, no y draw) [M 0x1ab21c/0x1ab248; decompile :91-98 — the
   old port arm centred the jitter with −0.025, which the decompile does not say]; kind 7 **with a
   NULL velocity** consumes `Rand_FRand(0.2)` ×3 (`(wind + (r−0.1), wind + r, wind + (r−0.1))·REC`,
   wind = `Env_GetWindVel` = 0 here) [M 0x1ab160/0x1ab184/0x1ab1a4; decompile :108-116]; every
   other kind takes the (0, REC, 0) default with no draws [M decompile :105-129]. The draws happen
   inside the `cel != 0 && life != 0` gate — a zero life consumes nothing [M 0x1aacc8:31].

`Env_Breath` 0x17be00, variant A (env gate true AND cam mode 2), draws the puff's yaw
`Rand_FRand(2π)` **before** its `Gas_Init` call and stores it as the quaternion z
[M 0x17bf8c-0x17bfa0: `lui $at, 0x40c9; ori $at, $at, 0xfdb; jal Rand_FRand`, the result stored
into the quat quad's z slot; the `jal Gas_Init` follows at 0x17bfe4]. Variant B (null quaternion)
makes no caller draw [M 0x17bf00-0x17bf68]. So a variant-A puff consumes 3 draws, a variant-B puff
2.

The muzzle smoke's shape matters and is now cited in place: `Player_MuzzleFlash` 0x1a4298 stores
its own `Rand_FRand(2π)` as the quaternion z (sp+0x88) and passes a **NULL velocity** (a3 = 0 at
0x1a4428) [M disasm 0x1a43d4-0x1a443c], so its kind-7 puffs take the wind branch — 5 draws inside
Gas_Init on top of the caller's 2π. `Effect_Bullet` 0x1ce8a8 likewise passes a3 = 0 [M 0x1cea80].

## What the port does now

- `GasPool::init_ex(rng, …)` and `init(rng, …)` consume the game's draws in the game's order
  (spin → kind-0xb quarter → sign → velocity draws), **before** the pool-cap check (the game has
  no cap; a skipped draw would shift the stream) and after the life gate (the game draws nothing
  at life 0). The kind-4 jitter is added raw per the decompile.
- `env_breath(rng, …)`: variant A makes the 2π yaw draw first, then init_ex's spin/sign; variant
  B no yaw draw. `env_water_ripple(rng, …)` unchanged shape, now drawing.
- Every real game site passes the sim's rng: the breath (nf-view scene.rs `guard_breath`), the
  water ripple, the ripples-update splash (world_fx.rs), the `Effect_Bullet` gas entries
  (effect.rs), the flesh puffs (drone.rs, mpfx.rs), the laser impact puff (drone.rs, mpfx.rs),
  the muzzle smoke (scene.rs, mpfx.rs — the caller's 2π draw was already consumed there), and the
  flashbang puff (drone.rs).
- The port stand-ins that are **not** game `Gas_Init` calls stay on the old counter hash via
  `init_hash`: the swoosh ribbon puff (`Swoosh_Update` 0x18d200 has no gas/RNG path [M
  0x18d200:49-82]) and the GT destroy burst (the port's presentation of `Emitter_Create` [M
  0x180498:249-257]). The game consumes no draws for them, so the port must not either.

## Bugs the work exposed and fixed (with evidence)

- **The flashbang puff's parameters were misread.** [M disasm 0x12dd60-0x12ddb8, decompile
  0x12d890:288-290]: mesh **0x020001f** (the port had SMOKE_QUAD), rgba start **0xffffff7f**
  (the port had [0x00,0x80,0x80,0x80]), rgba end **0x80808000** (the port had 0), life **0x7f =
  127** (the port had 30 — `sw $v0, ($sp)` with v0 = 0x7f at 0x12dd98, the same dropped-11th-arg
  pattern the searchlight's 0x80 was pinned with). Kind 0xb is right, and it now quarters the spin.
- **The ripple-splash puff's kind and end were misread.** [M `Ripples_Update` 0x1ac028:44-46]:
  kind **1**, rgba end **6** (the port had kind 6, end 0). Both hit the same no-motion update arm,
  so no behaviour moved; the record now matches the call.
- **The kind-4 jitter was centred** (−0.025) where the game adds the raw draw [M 0x1aacc8:91-98].

## Tests

- `gas.rs` `init_ex_consumes_the_games_draws_in_order`: replays each kind's draws on a twin stream
  bit for bit (kind 0: 2 draws; kind 7: 5; kind 4: 4 with the raw add; kind 0xb: the quartered
  spin) and asserts the stream advances by exactly the game's count. `init_hash` is covered by the
  existing deterministic-pool test.
- `env.rs`: the breath tests replay variant A's 3 draws (2π then spin/sign) and variant B's 2, and
  assert the gated-out paths consume nothing; the ripple test replays its 2.
- Re-pins (seed-dependent outcomes that moved, and why the new values are the game's):
  - `effect.rs` `ricochet_draws_once_and_reflects` sister: the old pin read **zero** total draws
    for an N=0 impact; the record's kind-9 gas puff now consumes `Gas_Init`'s spin+sign [M
    0x1aacc8:72-80], so the sister replays those 2 draws and asserts no *ricochet* draw on top.
    Effect_RicochetProb itself still draws nothing [M 0x1cf290].
  - `ai_live.rs` `guard_takes_cover_like_the_game`: seed 9 → **4** (every pin passes: the state
    ladder, both duration windows, cover node 1, the fire position, and the sister). Why the stream
    moved: the guard's wall impacts spawn the Effect_Bullet gas entries through `Gas_Init` [M
    0x1cea98], whose spin/sign (+wind for kind-7 rows) now draw. The seed history comment records
    the sweep (0/2 pick another first move, 1/3 hold the 90-tick lean).

## Verification (what I ran, with totals)

`NF_ISO=$NF_ISO cargo test --offline --release -j2 … -- --test-threads=1`:

- nf-game lib: **649 passed / 0 failed / 5 ignored** (includes the new replay tests);
- nf-game binaries: ai_live **37/0**, blast_smoke_live **4/0**, laser_gas_mesh_live **1/0**,
  weather_env_07000014_live **1/0**, rng_live **1/0**, grenade_live **2/0**, stun_grenade_live
  **1/0**, guard_drops_flashbang_live **6/0**, searchlight_live **3/0**, mp_lockstep 0/0 (all
  gated); plus a bulk `cargo test -p nf-game --tests` sweep over all 98 binaries — **no FAILED
  line** in the whole run;
- nf-view: lib **100/0**, models_misc_live **10/0**, rocket_trail_gas_live **2/0**,
  walkreal_07000005_live **12/0** (the combat-level walk — the closest canary for impact-driven
  stream shifts), **cross_process_determinism 3/0** (both sim builds and both child processes
  bit-identical with the new draw counts folded into the state hash), **fps_determinism_live 4/0**.

The full-suite fast runner (`tools/test/suite.py`) could not run: this sandbox kills background
jobs between tool calls (three starts, all dead within minutes — same as the door-scan-skip-bit
session recorded). The affected-crate runs above are the reported coverage.

## What's left

- The `Bullet_DoTrails` bit-**0x80000** smoke site (0x12db50: every 2nd tick, kind 7 NULL-vel,
  quat z = `Rand_FRand(2π)` [M decompile :177-188]) is not ported at all — when it is, it must
  consume its caller's 2π + Gas_Init's 5. Records carrying the bit need a [L] sweep.
- `Flicker_Update` 0x1ab960:122 spawns a kind-4 puff (4 draws per spawn) behind the stubbed
  `FindMapLight` — the port spawns no flicker gas, so those draws are missing; porting it is
  visual work (needs the main agent's render).
- `AnimProcessScriptCmds` 0x1c1b38:453 (NIS script smoke, kind 7) and `RB_Update` 0x1d6e38:58
  (rigid-body dust, kind 1) are real `Gas_Init` sites the port does not reach yet.
- The muzzle smoke sites' shared `rgba_end 1` visual (alpha 0x80 → 1) and the MP flesh puff's
  rgba end (the game packs 0xffffff00, the port passes 0 — the rgb stays 255 in the game) are
  presentation-level and need the main agent's render to confirm.

## Live checks for the main agent

None required — no visual claim rides on this change; the drawn values are consumed from the
stream and only the spin reaches the billboard. If convenient: a `--bond` spot at a breathing
guard in a snow level (e.g. 07000014 outside) before/after this branch, to confirm the breath
puff's look is unchanged apart from its spin direction now being stream-driven.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures.
