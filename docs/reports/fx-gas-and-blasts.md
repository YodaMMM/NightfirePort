# fx-gas-and-blasts — cloud emitters, the Apocalypse flash, the MP blasts

Ported rows 5, 9 and 10 of `audit-effect-calls`'s "Unported calls" table. Merged origin/master
first (the branch is cut from it, 17a784980). Every value tagged; the decomp files under
`work/decomp/ACTION.ELF/` are the disassembly-backed source.

## Row 5 — the gas kinds 9/10 hidden cloud emitters

`Gas_Init` 0x1aacc8 [M]:
- kinds 9/10 take the HIDDEN bit (`LAB_001ab1e4: obj+0xf0 |= 0x10`); kind 10 also HALVES the life
  at creation (`pfVar1[0xe] = pfVar1[0xe] * 0.5` — +0x38 is the life field).
- kinds in `8 < kind < 0xb` store Gas_Init's OWN args into the object's data table (+0x40..+0x58:
  rgba, end rgba, glist, p1, p2, life) — the child-spawn record.
- the spin `Rand_FRand(π/2·REC)` + the sign `Rand_Rand(0x100)` head runs for EVERY kind (the port
  keeps its counter-hash stand-in for these, consistent with init_ex's other kinds).

`gases_smoke` 0x1ab2d0 [M case 9 = :146-162, case 10 = :164-186]:
- case 9 (the cloud emitter): no motion; a 20-TICK creation warmup (`GameState - obj[0xec] <
  FRAME_RATE_MUL·20.0` blocks — FRAME_RATE_MUL = 1.0 at 60 Hz, so 20 ticks, not seconds) — then a
  kind-8 child on every tick where `GameState & 3 == 3`, with the parent's recorded creation args
  and the parent's rgb re-combined with the parent's LIVE alpha byte
  (`(rec & 0xffffff00) | (u8)obj[0x106]`). The kind-8 NULL-vel store is (0, REC, 0)
  [M 0x1aacc8's kind-8 arm].
- case 10 (the emitter): a kind-7 child on every tick where `GameState & 7 == 7` (the low 3 bits
  all set), with the GIVEN velocity (0, 2·REC, 0) through the generic given-vel store (·REC again
  [M 0x1aacc8:89-92] — so 2/3600 per tick).

Port: `Gas::hidden/born/child` (+ `ChildSeed`), the kind-9/10 arms in `GasPool::update(frame)`
(the pool now carries the GameState tick for the `born` stamp), `push_child` (the Gas_Init
re-call), and the nf-view gas draw skips hidden puffs. **No game caller in the port's scope
creates kind-9/10 parents yet** (the audit's census said the same) — every Gas_Init caller in
ACTION.ELF passes a literal kind ≠ 9/10, so the mechanics land here for the callers that do.

## Row 9 — the Apocalypse flashbang

`Apocalypse_Update` 0x18d4f0 [M]: the channel latches (data+0xe), the countdown decrements per
tick, and while alive: the whiteout flag (data+0xc) writes `viewer+0x200 = remaining·5/total` (the
camera-shake store — audit row 1's consumer, render work for the main agent; the port already
carries the value as `ApocalypseFx.fade`), and the flashbang flag (data+0xd) calls
`Player_SetFlashBang(remaining, glb_blokes, min(remaining·382/total, 255))` EVERY armed tick
[0x196040: BL+0x963 = the byte, BL+0x918 = BL+0x91C = the duration].

Port: `WorldFx::apocalypse_flash` ((remaining, alpha 0..1) per tick, None = no write) set in the
Apocalypse loop, applied in `Drones::tick` right after `fx.tick` to
`health.flashbang/flash_total/flash_alpha` — the same state the stun grenade's
`Player_SetFlashBang` writes (drone.rs's flash arm), so the renderer's existing whiteout model
drives off it unchanged.

## Row 10 — the MP chain and self-destruct blasts (the task text's scope)

- **The GT_Create chain** [M 0x17ffa0:150-166]: after a turret enters the object list, every OTHER
  class-6 object with the same ext+0xb8 owner self-deletes (`obj+0xfe |= 1`) with
  `Explode_Create(5.0, 5.0, 50.0, …, 0x6000052, …, 100)` at its own +0x30. WHO OWNS WHAT [M
  0x17ffa0:80-90]: ext+0xb8 = param_6 (the parent) or the turret itself when param_6 = 0 — and
  `parsemap_create_dynamic_objects` 0x1d04f0 case 0x27 (the level placements) passes param_6 = 0,
  so PLACEMENT turrets own themselves and the chain can never fire between them; only
  `GT_DeployMiniGun` 0x181560 passes a real parent (the deployer, param_3). Port:
  `GTurret::create_chain` runs on the DEPLOY path alone — a redeployed minigun detonates the
  previous same-owner one. (The first cut also chained the placement walk; gt_los_live's 0700000a
  fixture guns went red — placement guns share the port's `owner: 0` stand-in — and the decomp
  above shows the game can't fire it there either. Fixed and re-run green.)
- **The wreck self-destruct** [M 0x180498 case 6 = :427-441]: MP-gated (`MPSettings._384_4_ != 0`)
  10 s after wrecking (`facing != 1` guard) — the emit side was already gt.rs's `DESTROYED` arm
  (`GTEvent::Exploded`, `ctx.mp`); the drain used to drop the event as draw-side data. Now it
  pushes the Explode_Create into `pending_blasts` (5.0 radius / 50.0 damage / script 0x6000052 /
  scale 5.0, at the wreck's own +0x30) so the blast propagates in the sim.
- **Reported, not ported** (outside the task text's "chain and self-destruct" scope):
  - `MP_DemolitionProtectionUpdate` 0x1870a0:144 [M] — the demolition target's death fires
    `Explode_Create(5.0, 5.0, 500.0, …, script 0x600004f, …, 100)` at obj+0x80 plus the MPGame
    round-end flow (`MPGame+0x188 = 6`, the status sprites, `MP_sendTeamBotMessage(0x41)`). The
    port has no MP demolition-mode flow; the blast values are here for whoever wires the mode.
  - `Drone_Delete` 0x136ce8:92 [M] — deleting a drone kills + detonates every class-0x36 object
    whose ext+0xb8 equals the drone (5.0/5.0/50.0, 0x6000052). SP-reachable in principle; the
    port's drone deletion folds the object sweep ("no port object keeps a dangling id"), so the
    site needs the drone→GT owner link modelled first (the port's `GTurret.owner` is the u32
    stand-in).

## Pins

- `gas.rs::emitter_tests` — kind 9: the hidden bit, the 20-tick warmup, the &3==3 child cadence,
  the child at the parent's pos with the recorded mesh and the LIVE alpha byte, the (0, REC, 0)
  kind-8 vel; kind 10: the halved life, the &7==7 cadence, the (0, 2·REC·REC, 0) child vel.
- `world_fx.rs::apocalypse_flashbang_carries_the_per_tick_write` — the clamp at the head
  (99·382/100 = 378 → 255 → alpha 1.0), the linear decay (21·382/100 = 80.22 → 80/255), no write
  after the floor.
- `gt.rs::create_chain_blows_the_same_owner_turrets` — the same-owner sibling dies, the other
  owner and the new one stay, a delete-flagged sibling is skipped.
- Through the real path: `world_fx_live` 2/0, `gun_turrets_live` 3/0, `gt_los_live` 4/0 (red with
  the misplaced placement chain, green after the deploy-only fix), `ronin_live` 9/0 (the deploy
  path incl. the chain site), `mp_hud_real_live` 5/0.

## Changed-code run list (release, --test-threads=1)

- `nf-game --lib`: 666 passed / 0 failed (5 ignored) — includes the three new pin groups.
- nf-view: world_fx_live 2/0, gun_turrets_live 3/0, gt_los_live 4/0, ronin_live 9/0,
  mp_hud_real_live 5/0.
- **The full suite did not fit the window** (the Apocalypse/emitter/chain work + the gt_los
  regression + the fix consumed it). NOT run whole: the remaining nf-game/nf-view binaries (incl.
  every `walkreal_*_live`, `npc_doors_live`, `fix_guard_misc_live` — drone.rs changed: the
  fx.tick consumer, the GT chain sites, the gas.update(frame) call), nf-formats, nf-audio. The
  gas.update signature change touches the sim's internal call only (grep: no test binary calls
  `gas.update` directly); the Gas struct's three new fields are constructed only inside gas.rs.

## Live checks for the main agent

1. Row 5 [M→L]: no kind-9/10 Gas_Init caller is visible in ACTION.ELF — a gdbwatch on the
   Gas_Init entry (0x1aacc8) during long-play smoke-cloud moments (the fuel tanks? the mission
   scripts) would settle whether a disc caller exists with a variable kind; expect the 8th arg
   (the kind byte) to never read 9/10 if the census holds.
2. Row 9 [M→L]: poke a 0x4a placement's latch channel (world_fx parse) in a level with one —
   expect the HUD whiteout to decay linearly over `total` ticks and `viewer+0x200` to read
   `remaining·5/total` while the whiteout flag stands (the same store the shake uses).
3. Row 10 [M→L]: in an MP match, wreck a gun turret and leave it — at 10 s expect the
   `Explode_Create` blast (radius 5, script 0x6000052) at the wreck; redeploy a minigun twice —
   expect the first one to detonate on the second deploy.

## What's left

- The full-suite run (the next message should start there; the not-run list is above).
- The two reported-not-ported row-10 sites (the demolition flow, the drone-owned-GT link).
- The kind-9/10 parents have no port creator yet — a future caller task owns the level-side wiring.

## Usage

not visible to me — this sandbox exposes no token, context-window or cost figures.
