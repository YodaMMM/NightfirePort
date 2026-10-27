## Your task: `level-16-jetpack-escape-4`, 07000016: what the real mission order changes before the launch pit

On master the pit timeline is pinned and matches the live game (`walkreal_07000016_live::the_pit_pulse_kills_an_idle_bond_about_450_ticks_later_like_the_game`): touch trigger #255 -> the pack cinema 0x06000631 -> re-enable at the pit floor (3.57, -22.76, -179.55) -> ch 243 at +~1603 -> the death sequence fails the mission ~450 ticks later. The live run's touch-to-re-enable gap (553 observed vs 747 modelled) was a measurement artifact: Bond was placed on the trigger around frame 470, not at the logger's first row (687), so the gap is ~770 [L].

The data shows no way to survive pulse 1 from the pit floor (the six-band hurt stack tiles the pit, the blast rooms are sealed by BlockingPolys 117/118, the shuttle-bay lift's ch 80 and the pit door's ch 240 are written after the pulse [D]). Yet the shipped mission is completed by players. The live run skipped everything before the bridge (it placed Bond on #255 directly): the chasers, the Kiko death button and their channels.

1. List every channel and script state the mission's real order sets before #255 is touched (the chasers' defeat, the Kiko button, objectives) and find any branch they select: an alternate SP or script for the cinema or the launch (another SP gated on those channels), a different hand-back pose, a gated hurt volume (its channel low), or a script that moves Bond [D script dump + M SP_Update/SP_Hit].
2. Run the port in the real order (`walk_the_chasers_defeat_completes_the_mission`'s legs) through the bridge and compare the cinema, the hand-back pose and ch 243 with the teleported run.
3. If the order changes nothing in the data, name the live check precisely (fresh boot, play the real order, what to watch) and stop there; the walk test stays ignored until it is settled.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-jetpack-escape-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-24T01:07Z

---
## Report (done, 2026-10-24T02:49Z)

# level-16-jetpack-escape-4 — what the real mission order changes before the launch pit

## What the task asked
1. List every channel and script state the mission's real order sets before the ch-0x81 bridge
   trigger #255 is touched (the chasers' defeat, the Kiko button, objectives) and find any branch
   they select: an alternate SP or script for the cinema or the launch, a different hand-back
   pose, a gated hurt volume (its channel low), or a script that moves Bond [D script dump +
   M SP_Update/SP_Hit].
2. Run the port in the real order (`walk_the_chasers_defeat_completes_the_mission`'s legs) through
   the bridge and compare the cinema, the hand-back pose and ch 243 with the teleported run.
3. If the order changes nothing in the data, name the live check precisely and stop there; the
   walk test stays ignored until it is settled.

Both the census (step 1) and the port run (step 2) are **negative**: the real order arms nothing
that can touch the launch chain. The live check from escape-3 stands, refined for the real-order
play-through (§4).

## 1. What the real order sets before #255 (all [D], probes `order16`/`order16b` + [M] param layouts)

The probe closes over the mux family with the [M]-verified param map — out = p0, inputs = p1..p6
+ p8, mask = p9 for the 0xEC/0xED/0xEE/0xEF wrappers (Trigger_Create 0x1b1720:80-91 via the
0x1b2c68 slot shift; Trigger_Hit 0x1b1a78), and the SP map of SP_CreateScriptPlayer 0x1db9a8:18-61
(p0/p1 scripts A/B, p2/p3 modes, p4 stay, p5 mask, p6 out, p9 gate).

**Carried in** from 07000014/15 (the walkreal carry [L the tests' own RamSave]): ch 0x47, 0x48,
0x4a, 0x4b, 0x54, 0xce. **No channel-reader placement in 07000016 reads any of them** [D
order16b: the trigger/AND/SP/door/switch/hurt kinds' channel params name only 0x24/0x27/0x28/0xb4
among these] — the carried state is dead here except through the mission monitor's own records.

**Set in level, in walking order** (each writer is a [D] placement):

| ch | writer | what |
|---|---|---|
| 0x8e | TouchOnce #252 (out p0=142, mask p1=8) at (5.17, 0.84, −65.67) | the arrival spawn plate; completes the carried record 21, announces obj 1 |
| 0x24 | TouchOnce #250 (mask 4 = the E path [M Trigger_Activate 0x1b2318:38-49]) at (−15.20, 6.52, −94.56) | the SecurityStation panel; unlocks door[0] (ch_lock p3=36) |
| 0x27 / 0x28 | TouchOnce #337 / #338 (mask 4) at (3.36, −141.50)/(3.34, −151.95) | door[3]/door[2] unlock panels (ch_lock 39/40) |
| 0xb4 | TouchOnce #270 (mask 4) at (3.40, −165.72) | door[4]'s panel (ch_lock 180) |
| 0x8f | TouchOnce #253 at (2.53, 0.84, −153.89) | the silo plate; completes obj 1, reveals obj 2 |

**AND closure**: the only mux consuming a real-order channel is AND #342 (out p0=51 → 0x33;
inputs p1=50 (0x32 — cold: no pre-#255 writer), p2=36 (0x24 — on)) → **0x33 stays cold** [D —
the probe's `[mux-partial]` line]. No AND/OR/OUT turns anything new on before the bridge.

**The Kiko button** #346 (kind 0x29 switch, hash 020008c1): out 144 (0x90), gate 141 (0x8d),
script 060008ba [D]. Gate 141 is cold before #255 — its only script writer is 06000632's s16
`EV0x0d[141,0]` (a clear). The button cannot fire before the chain and does not feed it.

**Objectives**: 0x8e/0x8f are read only by the mission monitor [M 0x1827d8]; no placement
(trigger, mux, SP, door, switch, hurt volume) keys anything on them.

## 2. The branch hunt — negative on every mechanism the task names

**a. No script conditional exists anywhere in the level.** The kind-7 scripts across the level-16
bins (07000016.bin ×14, 07800016 0x06000631, 07900016 0x06000632, 07A00016 0x06000633; 07F00016
carries none) contain **zero event-0x11 commands** [D order16 dump]. Every channel write in every
script is an unconditional event 0xd [M Script_EventHandler 0x1d94c0: case 0xd writes
`switch_channels[p0]`, case 0x11 is the conditional — absent here]. There is no data path from
any channel to a script branch.

**b. No alternate SP or script for the cinema or the launch.** The only camera scripts are
06000631/0632/0633 [D]. SP#262 is the only placement carrying 06000631 and SP#272 the only one
carrying 0600090a [D order16b param scan]. Their triggers are mask 1 = SP_Hit's bit 1 = "their own
out channel is on" [M SP_Hit 0x1dc470: `if (*(ushort*)(tag+0x30) != 0) bits =
switch_channels[out]`; SP_Update 0x1dbda8:82 `(bits & mask) != 0`]. The **only writer of ch 0x81
in the whole level is the TouchOnce #255** (out 129, mask 8 — the player's body on the slab at
(2.37, 1.04, −176.41)) [D order16b channel-reader census for 129: place[255] + SP#262's own out].
Ch 185 (0xb9), SP#272's out, is written only by the cinema's s7 `EV0x0d[185,1]` @748 [D]. So the
chain has exactly one starter (the touch) and one order (cinema → launch), in both orders.

**c. The SPs the real order DOES arm** (mask 1 with their out in the pre-#255 on-set):
- the four keypad SPs #84/#85/#154/#155 (out 0x27/0x28; script 06000999 — two ENTITY display
  anims 02000a6f/02000a70, no keys on the player, no channel writes) — the corridor panels'
  display flash [D];
- **SP#348** (out 0x8e — armed the moment the spawn plate fires; script **0x060009f6**: two
  streams, NO keys, NO events — one waits ~5.7 script-seconds then ENDs) — a no-op timer [D];
- SP#347 (060009e9, the shuttle-bay lift platform) is mode 1 — it plays at load in both orders
  [D], not a branch.

None writes a channel, moves Bond, or plays a camera [D].

**d. No different hand-back pose on any reachable path.** The event-6 hand-backs [M
Script_EventHandler case 6 — the previous stream's entity matrix, else the player's own]:
- 06000631 s9 @747 → stream 8's ENTITY-hash-0 pose **(3.57, −22.76, −179.55)** [D keys] — the pit
  floor; the launch path's pose in both orders;
- 06000632 s13 @1057 → (5.39, 1.34, −158.31) [D] — the corridor above the pit, but 06000632 is
  armed only by SP#263's out 0x82 ← AND #286 [0xbf, 0xc0] — the pit squad's death channels
  23..26 → 0xbf [D AND #264] — post-#255;
- 06000633 s14 @1888 (mode 1 → the fallback pose) — **no placement references 06000633 at all**
  [D order16b: no SP/switch/trigger carries the hash] — dead data on this level [S: a live path
  outside the placements is not excluded by a dump; see the live checks]. Notably its s15 writes
  181 (0xb5) = 1 — the only possible unlock for door[47] (−3.10, 1.04, −156.68, ch_lock 181 [D]),
  which no other placement writes.

**e. No gated hurt volume arms early.** HurtPoly #309 gate 243 (the pulse) and #352 gate 203 [D
order16b]. Ch 243's only writers are the launch script's streams (0600090a s6/s9) [D]; 203 is
launch-drone #259's death channel (post-#255). Neither gate is in the pre-#255 on-set.

**f. Scripts that move Bond.** The ENTITY-hash-0 streams (the player actor) are exactly:
06000631 s8 (the pit pose @747) and 06000632 s12 (the corridor pose @1057, post-#255) [D]. The
launch script 0600090a carries no Bond entity — its entities are the shuttle (0200098f
"shuttle2", 0200098c, 02000992, 02000993), the platform (02000ab7/8/9a) and the four pit-side
actors (02000994-997) [D]. The chasers and the button set no state the chain reads.

**g. The one channel-state difference the order makes**: the cinema's s13 `EV0x0d[180, 0]` @747
clears ch 0xb4 at the hand-back [D]. In the real order 0xb4 was 1 (the panel E that opened
door[4]); the clear ends the lock condition either way (a door is locked while `ch_lock` is
non-zero AND its channel is on [M triggers.rs:115 — the lock arm]), so door[4] ends unlocked in
both orders — the leaf's open state differs, nothing else does.

## 3. The port run in the real order (step 2) — the chain is order-independent

New test `walkreal_07000016_live::the_real_order_launch_chain_matches_the_teleported_one` (green;
opt-in via NF_ISO like the file). Phase A walks the full real order — arrival → spawn plate → the
station detour → the door panels → the silo plate → door[4] → a manual real-time approach onto
the bridge slab (skip off from the staging point, so the touch, the cinema's disable, the
hand-back and ch 243 are all observed from the exact tick). Phase B is the sister's teleported
setup (Bond placed on #255, no input). Recorded (deterministic, the test's eprintln):

| run | ch0x81 | re-enable | touch→re-enable | hand-back pose | ch 243 | ch243−re-enable | tail |
|---|---|---|---|---|---|---|---|
| REAL (walked) | +29 | +750 | **721** | (3.57, −23.18, −179.55) | +2353 | **+1603** | 0xb4 cleared +750, chaser +2922 |
| TELEPORTED | +0 | +752 | **752** | (3.57, −23.18, −179.55) | +2355 | **+1603** | fail +2806 (+451 after ch243) |

- The hand-back pose equals the [D] stream-8 key in x/z exactly (3.57 / −179.55); y −23.18 is the
  port's capsule resting ~0.42 below the key [S, the sister's existing note].
- The chain clock is **bit-identical across orders**: ch243 − re-enable = 1603 both (the
  [M nis.rs absolute-Wait + D stream] model is 1610; live 1591 [L]).
- The 721-vs-752 touch→re-enable spread is the observation shape, not a chain difference: the
  slab fired 29 ticks into the manual loop and the cinema starts on the next SP tick, so phase A's
  reference tick trails the physical touch. The invariant that excludes clock drift
  (re-enable → ch 243) is identical.
- The cinema's stream-13 ch-0xb4 clear is observed in the real order at the hand-back (+750) [D].

So the port confirms the census: the real order changes the pre-chain channel set, the door
states and the objective records — and nothing about the cinema, the hand-back pose, ch 243, or
the chaser.

## 4. Live checks for the main agent (PCSX2, FRESH boot of 07000016)

The walk test stays `#[ignore]`d (pulse-1 survival has no disc-visible real-input escape —
escape-3's census stands). The one open number is still the live **ch-0x81 → re-enable 553 [L]**
vs the modelled 747 [D stream-8 t747] (the port: 752). This round's census closes every
disc-visible candidate starter — only #255 writes 0x81, only SP#262 carries the cinema, no event
0x11 exists, no pre-chain script moves Bond — so the check is now a pure measurement-artifact
discriminator, and it should be played **in the real order** (no Bond placement):

1. Attach a byte watch on `switch_channels[129]` (0x26fc90 + 129·4) and a position watch on
   Bond's `obj+0x30`, from the level load, then play the real order from the arrival through the
   bridge (the walkreal route: the station panel, the door panels, the silo plate, door[4], the
   bridge slab).
2. Expected if the timeline is closed (the port-faithful shape): ch 0x81 first reads 1 within ~2
   ticks of the slab crossing; the cinema's re-enable (Bond at (3.57, −22.76, −179.55)) lands
   **747** ticks after that first rise; `switch_channels[243]` (0x26fc90 + 243·4) rises ~1610
   after the re-enable.
3. If instead ch 0x81 first reads 1 only hundreds of ticks after the slab crossing (the round-2
   log's first-row-687 shape), gdbwatch the **writer** of 0x26fc90+516 at the first 0→1
   transition: the disc data names no object that can do it, so the writer would be code outside
   the placement census — the missing escape candidate.
4. (Cheap rider, same session): while playing the real order, confirm SP#348's script 0x060009f6
   produces nothing visible after the spawn plate (the port's dump says two empty streams), and
   note whether door[47] (−3.10, 1.04, −156.68, lock 0xb5) has any live unlock path — its only
   candidate writer is the unreferenced 06000633.

## What I ran
- `cargo test --offline --release -p nf-view --test walkreal_07000016_live -- --test-threads=1` —
  **6 passed / 1 ignored** (the parked `walk_the_chasers_defeat_completes_the_mission`; the
  un-ignore condition is §4's check, unchanged).
- the new `the_real_order_launch_chain_matches_the_teleported_one` (inside that binary; the
  numbers above are its recorded output).
- the probes `nf-formats/examples/order16.rs` and `order16b.rs` against NF_ISO — the [D] census
  above (committed; opt-in, no game bytes).
- the fast suite (`tools/test/suite.py`) — the chunks below (recorded at report time).

## Suite
The sandbox kills background processes between tool calls (two `nohup`/`setsid` attempts died
right after their build step — `suite2.out` holds only `build ok`), so the fast suite ran as
foreground `--only` chunks with a reduced per-binary cap (100-150 s vs the default 600):

| chunk | binaries | result |
|---|---|---|
| the level-16 family (walkreal/walk 16, census, interactables, pack_cinema, spurious_complete, fix_finale, kiko_button, fix_kimiko_handback, hurt_08) | 10 | **26 passed, 0 failed**, 3 ignored (1 min) |
| walkreal_0700000{1..d},11..15,1b,41,4a + 11_route | 22 | **46 passed**, 12 ignored, 1 TIMEOUT (walkreal_07000003_live at the 150 s cap — the cap, not a failure) (7 min) |
| the fix_ family (portal/mission/npc/objects/player/weapons/guards/death/level/script/cutscene/hud/progress/audio/drone/ladder/swim) | 33 | **138 passed, 0 failed**, 5 ignored (2 min) |
| AI/guards/doors/triggers/objectives/rng/sensor/spawn/escort/civilian/alert | 30 | **106 passed, 0 failed**, 1 ignored (1 min) |
| menu/flow/census/formats goldens (incl. disc_golden, identity_disc) | 23 | **140 passed**, 4 FAILED + 2 TIMEOUT (below) (4 min) |

**Total: 456 passed / 4 failed / 3 TIMEOUT / 21 ignored over 118 of the 769 binaries.** The 4
failures: the 3 `disc_golden` tests — the documented slim-ISO allowance — and
`identity_disc::the_real_disc_passes_the_identity_check`, which fails in 0.01 s on this slim ISO
immediately after a successful `DiscInfo` read (environmental; this diff adds one test file and
two opt-in examples and touches no shared code). The 3 TIMEOUTs (walkreal_07000003_live,
campaign_flow_live, levels_sweep_live) are my reduced caps; a re-run at the default 600 s cap was
cut by the sandbox's own shell timeout before its summary — they are unverified at the default
cap, not red. 651 binaries (the mp_*, movie/menu-feature, net_*, probe/dump and remaining live
families) did not run in this window.

## Usage
not visible to me — this sandbox exposes no token/credit/context-window figures in any readable
surface (no environment variable, log or UI text was found).
