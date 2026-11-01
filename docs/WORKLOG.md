# WORKLOG

## gas-rng-draws — started (2026-11-01 21:58 UTC)

Branch cut from origin/master 6042ee53. Task: the port's gas puffs take no draws from the shared
RNG; the game's do — x-ray breath variant A draws `Rand_FRand(2*pi)` from the shared stream
[M Env_Breath 0x17be00, disasm 0x17bf8c-0x17bfa0] and `Gas_Init` 0x1aacc8 draws its spin and
sign. Port the draws in the game's order with the addresses, check determinism
(cross_process_determinism, fps_determinism_live), re-pin any test whose seed-dependent outcome
moves (say which, and why the new value is the game's). Run the gas/breath/smoke binaries,
ai_live and nf-game; quote the totals.

## gas-rng-draws — done (2026-11-01 23:15 UTC)

The port's `Gas_Init` sites consumed no shared-RNG draws; the game's do. Ported, in the game's
order: the spin `Rand_FRand(π/2·REC_FRAME_RATE)` [M 0x1ab060], quartered for kind 0xb
[M 0x1ab078-0x1ab088], the sign `Rand_Rand(0x100)` negating when ≥ 0x80 [M 0x1ab08c-0x1ab098],
the kind-4 given-vel jitter `Rand_FRand(0.05)` ×2 ADDED RAW [M 0x1ab21c/0x1ab248; decompile
0x1aacc8:91-98 — the old hash arm centred it], the kind-7/NULL-vel wind `Rand_FRand(0.2)` ×3
[M 0x1ab160/0x1ab184/0x1ab1a4], and `Env_Breath` variant A's yaw `Rand_FRand(2π)` before
Gas_Init [M 0x17bf8c-0x17bfa0; disasm]. `init_ex`/`init` now take the rng and draw for every
real game site (breath, water ripple, ripples-update splash, Effect_Bullet entries, the flesh/
laser puffs, the muzzle smoke scene+MP, the flashbang); the port stand-ins (swoosh ribbon, GT
destroy burst) stay on `init_hash` — the game makes no Gas_Init call there. The flashbang puff
got its real args [M 0x12dd60-0x12ddb8: mesh 0x020001f, rgba 0xffffff7f→0x80808000, life 0x7f];
the ripple splash got kind 1 / end 6 [M 0x1ac028:44-46]. Tests: gas.rs replay test (bit-for-bit
per kind + exact draw counts), breath A/B draw tests, the ripple twin; re-pins: the ricochet
sister (the kind-9 puff's 2 draws are the game's) and ai_live's cover seed 9→4. Runs: nf-game
lib 649/0, ai_live 37/0, blast_smoke 4/0, laser_gas_mesh 1/0, weather_env 1/0, rng_live 1/0,
grenade 2/0, stun_grenade 1/0, guard_drops_flashbang 6/0, searchlight 3/0; nf-view lib 100/0,
models_misc 10/0, rocket_trail_gas 2/0, walkreal_07000005 12/0, cross_process_determinism 3/0,
fps_determinism_live 4/0; the bulk nf-game --tests sweep (all 98 binaries) reported no FAILED
line.
