# WORKLOG

## gas-rng-draws — started (2026-11-01 21:58 UTC)

Branch cut from origin/master 6042ee53. Task: the port's gas puffs take no draws from the shared
RNG; the game's do — x-ray breath variant A draws `Rand_FRand(2*pi)` from the shared stream
[M Env_Breath 0x17be00, disasm 0x17bf8c-0x17bfa0] and `Gas_Init` 0x1aacc8 draws its spin and
sign. Port the draws in the game's order with the addresses, check determinism
(cross_process_determinism, fps_determinism_live), re-pin any test whose seed-dependent outcome
moves (say which, and why the new value is the game's). Run the gas/breath/smoke binaries,
ai_live and nf-game; quote the totals.
