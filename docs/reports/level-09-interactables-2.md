# level-09-interactables-2 — report

Branch `s/level-09-interactables-2` (cut from `origin/master` e4550f2e; WORKLOG commit
`1b14b4e1`). This is the -2 continuation of `level-09-interactables` (first pass merged as
`c1414f03`, report `30f51334`): the first pass built the per-kind battery and found no port
gap, but its sandbox could not deliver the full-suite total line (no background survival, disk
exhaustion). This pass re-verified everything against current master independently and
delivers the suite total.

## What's done

1. **The census re-run from the disc, this session** [D]. `census_07000009 --ignored --nocapture`
   (2 passed) re-dumps the placements; the main-map (m39) kind table matches the first pass's
   census exactly: 160× 0x19 scenery, 23× 0x31 FuseBoxes (no params → no channels → nothing a
   player can drive), 17× 0x20 breakable glass, 14× 0x47 CEL rooms, 12× 0x32 rain, 12× 0x29
   Switch (11 `alarmswitchplayer` on ch 0x7C/0x7D + `SCRIPTPLAYER_Lasers` #328 at the origin,
   unreachable by the game's own use), 11× 0xE9 AI infra, 6× 0xDB doors, 6× 0x0F guards, 4×
   0xF9 SoundTriggers, 3× 0xF0 pickups (armour kind-3 + two kind-6 BOND_BONUS), 3× 0xEA touch
   plates, 3× 0xD9 ScriptPlayer, 3× 0x2A skylines, 2× 0xE2 SS, 2× 0x33 hint markers, 1× each
   0xE8 Load Point / 0xEC / 0xEF / 0xF5 / 0xDF / 0x71 / 0x24. **No player-touchable kind is
   missing a live test**; the zero-placement kinds the task names (locks 0x2E, monitors 0x30,
   grapple 0x3A, gadget targets 0x35–0x38, copters 0x2F, turrets/sensors) have no m39
   placement in the dump — monitors 0x30 in no map at all.
2. **The kind → test map re-verified green on the current tree** (master e4550f2e — which
   moved since the first pass's bdb8c6e0: the 0b door-pick semantics, the 14 guard-AI probes,
   the 04 rigid-body params all changed; none of it touches the 09 family's results):
   - `interactables_07000009_live` (the battery): **4 passed, 0 failed** — the kind-6 bonus
     pair (medal gate + walk-over take + the no-medal negative), the 0x33 hint markers
     (register + walk-in INFO pane + unfired sister), the LIFTDOORS SS positive arm.
   - `interact_07000009_live`: **16 passed, 0 failed** — the alarm switches, the glass fire
     path + 0x30-filter refusal, the E-doors + locked shutter, the armour take, the Load
     Point E-refusal, the ch-0xAB plate, the guard kill.
   - `walkreal_07000009_live`: **4 passed, 0 failed** — the arrival plate ch 0x51, the panel
     activation chain, the exit chain ch 0x32.
   - `walk_07000009_live` **3/0**, `garage_lift_calls_live` **2/0** (the script-lift call
     stations), `lift_indicators_09_live` **2/0**, `security_alarm_09_live` **2/0**,
     `census_07000009` **2 passed** (the [D] dump mode).
   - **nf-game (the crate the battery drives): 588 passed, 0 failed, 5 ignored.**
3. **The full suite total line** — see "How it was verified".
4. **No port gap found this pass either**: every kind the census finds is driven by a green
   test through the real input path (walk/E/shoot) with a sister, on the current port; no
   test needed weakening; no decompile contradiction was met.

## How it was verified

The fast runner (`tools/test/suite.py` binary set) on the final tree (branch = origin/master
e4550f2e + the WORKLOG commit). This sandbox kills background jobs between tool calls, so the
same runner's chunked form (`tools/test/chunk_suite.py` with a shared flock-protected state
file, 2 workers) ran it in foreground chunks; every binary's result lands in the state file as
it finishes.

The only hard failures: `probe_pylon_render` (1) and `fix_0c_pylon_live` (2) — both panic in
wgpu adapter creation (`NotFound { active_backends: Backends(0x0) }`): this sandbox has no GPU
and no active render backend. Environment limitation, not a regression: the branch changes
nothing in render code, and the panic is in adapter enumeration before any game code runs.
The heavy mp/net/walk binaries that the OOM killer took (rc −9) when two ran concurrently were
re-run solo at the end; their final results are in the tally.

## What's left

- Nothing on the task: the census is re-verified, every kind is driven live and green on
  current master, and the suite total line is delivered (quoted in the WORKLOG outcome entry).
- The two wgpu-adapter failures need a GPU-capable environment (the main agent's), as the
  merge lessons already prescribe for anything visual.

## Live checks for the main agent

None required: no visual work shipped, and no sim-vs-game divergence was left unresolved.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
