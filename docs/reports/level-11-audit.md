# level-11-audit — the 07000011 behaviour audit

Branch `s/level-11-audit` (base: origin/master b604242f, which never moved during the session — the
final tree is the branch tip). Everything below is also in `docs/levels/07000011-audit.md` with one
row per placement.

## What's done

1. **Census [D].** New probe `crates/nf-view/examples/census_07000011.rs` dumps archive
   `07000011.bin`: 44 maps; main map #43 "010000e9" = **195 objects, 319 placements, 0 unresolved,
   0 script parse failures**; full-fidelity params (raw +0x24..+0x47 words) dumped outside the repo.
   Kind histogram: 0x0f×37, 0x19×137, 0x47×20, 0xdb×12, 0xea×22, 0xf0×13, 0xe9×12, 0xe6×24, 0xf1×7,
   0xf9×6, and 25 more kinds of 1–3 each; **0 placements of the kinds the port doesn't handle**
   (switches, monitors, sensors, breakables, destroyables, rigid bodies, searchlights, inverters,
   OR-muxes, ladders, lifts, clouds, copters, cars, GT, wires, mines, cam subjects — checked against
   the dispatcher 0x1d04f0's full case list).
2. **Audit table** `docs/levels/07000011-audit.md`: per-kind Game [M] block (creator / update / use
   / damage / event handler with addresses — dispatcher switch, static path 0x1d1998 + fixup
   0x1d00c0, Drone_Create 0x136be0, Door_Update 0x1363c8, Lock 0x1ae3d8/0x1b0650, PCQWorm
   0x1b0840/0x1aee20, SP_Update 0x1dbda8, SoundTrigger_Create 0x1b2628, Hint 0x1b3248/0x1b3380,
   DroneSpawner 0x1378d0/0x137e68, Trigger_Update 0x1b1a78, rotor 0x18f568, Mission
   0x182270/0x1827d8/0x182c60 — all re-read in work/decomp/ACTION.ELF this session), the port's
   handling with file:line, and a GAP column; one row per placement (interactive kinds row-by-row,
   the 160 static decor/light/sky placements as the 0c audit's compressed tables with every id).
3. **Scripts [D].** All 7 kind-7 entries decoded stream-by-stream. Only two are placement-
   referenced: **0600098a** (the atrium-fall death NIS: camera stream with cuts at 1/0x13/0x2a/
   0x48/0x65, ev4 Player_Disable, ev5 drone freeze, ev 0x0D [0x62,1]@103 = the mission-kill
   channel, sound 1242, **no ev6 hand-back** — Script_Free 0x1da880 re-enables Bond only on the
   ev6 flag 0x100, so a death NIS never hands back; the port's nis.rs:617/628-631/923/945/956/892
   is that law verbatim, and `is_death_nis` covers 0x0600_098a) and **060008c0** (Q-Worm console
   NIS: one entity + delay-sync, no Sound op — the doc erratum). The other five (060006f via
   sub-maps; 060007c4/0600004f/06000052/06000027) have no reference anywhere on the level —
   editor leftovers, noted, not a gap.
4. **Mission record [L]** `work/ee_start_05.bin` @0x2a4350+15·0x28 (record #15): part 0, obj0
   "Escape from the tower" = ch 0x64, obj1 "Gain access to lower office floor" = ch 0x48, both
   init 0 / enable 0 / no fail conditions; `mission/c07000011.rs` matches exactly. Channel-writer
   scan over every param and every script Event 0xD/0x11: **nothing on 07000011 writes 0x48, 0x64
   or 0x61** — the objectives complete cross-level by design (the walk doc credits 07000012's
   #679 with 0x48); the level's own end path is the death NIS's ch 0x62.
5. **The symptom list from the task, resolved:**
   - *Characters walking through Bond* — game-faithful: NDrone2_PreDroneControl 0x148db0 only
     stores player distances, NDrone2_Collision 0x140e90 / Drone_CollisionHandler 0x140968 never
     write a player. No push in the game, no push in the port. Not a gap.
   - *Guests that never talk* — the level has no Text/Sprite/SubScript ops at all; the "voices"
     are the six 0xf9 SoundTriggers (all gated exactly per 0x1b2628, now pinned).
   - *Doors that never lock* — the door "lock" channel is a **lock-release** channel: rising edge
     = label 0x2000003 (180 ticks) + Door_Activate (opens) + ch_lock:=0 + flags|=0x10 once-only
     [M 0x1363c8:96-107]. Wiring verified: TouchOnce #283 → 0x5a → door #213; Lock #177 burn →
     0x12 → door #192. Port matches (door.rs:618-631).
   - *Consoles whose action does nothing* — the Q-Worm chain is intact end-to-end: worm #216 /
     SP #263 both write ch 0x38 → mux #256 (0xed) → out 0x35 → spawners #220/#259; PCQWorm_Activate
     0x1b0840 → state 1 → PCQWorm_Update 0x1aee20 writes switch_channels[p1] one tick later; port
     useobj.rs:455-463/:249-256.
   - *Cut-scenes with broken cameras* — the level's one camera NIS is the death NIS above,
     verified op-for-op; PlungeTriggers #314-318 fire it via ch 0x7c with volumes from the linked
     objects' bounds (triggers.rs:683-687).
6. **Small gaps fixed/pinned [M] with tests (all pass):**
   - `door::tests::a_lock_channel_rising_edge_labels_once_and_latches` — the edge's label +
     once-only latch (the ch_lock halves were already pinned by `a_locked_slide_door_only_thuds`).
   - `difficulty_gates::soundtrigger_difficulty_gate_law` — 0x1b2628's p6 gate: threshold vs the
     5/6/7-exact arms (07000011 rows #231/#237/#238 p6=7, #289 p6=6).
   - `worldobj::rotor_tests::rotor_tick_accumulates_and_gate_stops` — rotor_update 0x18f568's
     accumulate/freeze law (the sim side was untested; the draw side is the gap below).
   No existing test contradicted the decompile; nothing was weakened.
7. **Doc errata** (six, in the audit doc): civilians' death channels (only #130 has 0x63; #126 has
   0x8), no drone waits on 0x0E and the 0x10-waiters are stairwell, #289 is gate-6 not gate-7,
   060008c0 has no SFX op, PlungeTriggers sit at the origin (link-bounds), and this task's own
   spawner-p0 correction (all seven are p0=3, mode 3).

## Large gaps (future tasks)

- **Rotor draw pose (kind 0x71) — DRAW-ONLY.** Game: rotor_init 0x18f4b8 / rotor_update 0x18f568
  accumulate obj+0x50 rotation; the port's sim side ticks `Rotor.angle` (worldobj.rs:411-412/
  :452-453) but **nf-view never reads it** — scene.rs keeps 0x71 in MISC_KINDS:8062 (static
  instance sweep; rotor only in comments :7977/:8122), so the fans render at their placement
  rotation forever. Fix shape: in scene.rs's per-frame instance pose, compose the placement matrix
  with the rotation from `WorldObjects.rotors[i].angle` for the 0x71 placements. Instances: #199
  'rotating blades' → mesh obj#128 @(20.83,25.29,4.45) rz=0.3491; #205/#206 'fanblades-large' →
  obj#134 @(13.42,20.88,-0.11/-1.15) rx=0.3491. Needs the main agent's render (lesson 6) — spots
  under "Live checks".
- Carried from the 0a audit, unchanged here: Pickup kind-6's PlrStats_HasGoldMedal gate
  (0x199a40) — 07000011's two 'Pickup 007' rows are the only kind-6 rows (bonus-log pickups).

## What's left

- The rotor draw fix + the main agent's render (above).
- A complete suite total line: the sandbox kills background jobs and each `suite.py` chunk restarts
  its log, so I ran foreground chunks against the wall clock instead. Best chunk: 820 test binaries
  built in 2 s (cache warm), 60–72 binaries run per ~9-min chunk, **0 failures except
  `probe_pylon_render::probe_render_pylon_only`** — a wgpu `request_adapter().expect("adapter")`
  probe, the known no-GPU environmental failure class (the 0c report's "no-GPU render probes"),
  untouched by this branch (nf-view render code unmodified). No total line: the run never completed
  in-sandbox.

## How it was verified

- Every game-behaviour claim in the audit doc cites a decompile address re-read this session, disc
  bytes via the census probe, or the FRESH-boot EE snapshot for the mission record. [S] items are
  marked and assert nothing.
- Tests: the three new pins above (run: `cargo test --offline --release -p nf-game --lib --
  a_lock_channel_rising_edge_labels_once_and_latches rotor_tick_accumulates_and_gate_stops` and
  `cargo test --offline --release -p nf-game --test difficulty_gates -- soundtrigger_difficulty_gate_law`
  — 3 passed). Crates touched by code: **nf-game only** (door.rs, worldobj.rs,
  tests/difficulty_gates.rs + the nf-view census *example*, which is not a test target).
- Suite: chunked `tools/test/suite.py` on the final tree (merged with origin/master — master never
  moved), as reported above.

## Live checks for the main agent

None required for the pinned tests. One render request for the rotor gap once a pose fix lands:
`--bond 20.8,24.3,4.5` looking up at the atrium fan (#199/obj#128, y 25.29) and
`--bond 13.4,21.9,-0.6` at the stairwell fans (#205/#206/obj#134) — blades should visibly rotate
frame-to-frame; today they hold their placement rotation (rz=0.3491 / rx=0.3491).

## Usage

My own session's token/credit/context figures: **not visible to me** — this environment exposes no
token, cost or context-window meter to the agent. The only such figures I could see were the final
lines of my three research subagents' reports (read where noted): "Tokens -> prompt:153820
completion:11161 total:164981" (engine-chains agent), "prompt:221977 completion:15183 total:237160"
(census agent), "prompt:121510 completion:3437 total:124947" (port-sweep agent). Copied exactly as
they appeared; I cannot verify what they measure.
