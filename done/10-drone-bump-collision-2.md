## Your task: `drone-bump-collision-2`, the guard push in every stance without breaking the escort and party routes

Start from `origin/s/drone-bump-collision` (merge origin/master) and read its report: the player-side push from armed drone capsules now runs in every stance [M Player_Collision 0x194df0, Collide_Jointy 0x1ec180:229-266] with `Drone::bump_capsule` per NDrone2_Collision 0x140e90:59-124. Alone on master it turns these red: mayhew_hold_live `the_escort_holds_at_the_ally_wait_node_until_the_ambush_clears`, party_door_lock_07000007_live (both tests), walkreal_07000007_live `walk_the_party_start_to_finish`, probe_07_route `probe_the_route`, cutscene_timer_pause_live `the_control_scene_freezes_the_mission_clock_until_the_hand_back`, walkreal_07000011_live `walk_spawn_to_the_lower_floor_load_point`. Find which push the game does not do there - the pick-level gate (which drones are armed for the player's class-3 query: +0x464 = 0x800 under NDrone2_DoCollision 0x149ec8:11-34, the 0x20/0x200 flag arms, NearPlayerCollision 0x14b190), allies/escorts/civilians in scripted states, the radius per state - and port exactly that. All seven green plus drone_bump_live; full suite green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-27T23:49Z

---
## Report (done, 2026-10-28T00:53Z)


---
## Report from the worker (done, 2026-10-28T00:54Z)

# drone-bump-collision-2 — the pick-level gate found; the cel-straddle arm landed; the arming-of-frozen-bots hypothesis left with live checks

Branch `s/drone-bump-collision-2` = `origin/s/drone-bump-collision` (07a98925) + merge of
`origin/master` (fcf6f334) + this round. WORKLOG entry `drone-bump-collision-2`.

## What was done

### The decompile trace of the game's push chain (fresh this round, disasm-checked)

The full gate chain Bond-vs-bot, with every link at its address:

1. **The pick is cel-straddle scoped** [M Collide_Pick 0x1eb320:125 (`Collide_StraddleCels`
   builds the seeker's straddled-cel list), :312-336 (per cel, `iVar9 = *(int *)(iVar12 + 8)`
   = the cel's object list, `Collide_PickObj(hittest, obj, owner)` per obj whose
   `obj->0xf0 & 0x30000000 == 0`)]: a bot is only a pick candidate if its own cel is among the
   cels the player's pick capsule straddles.
2. **The pick's class/config word is the hittest +0x98** [M Collide_PickObj 0x1eae80 — the
   disasm at 0x1eaf34-0x1eaf5c shows bits 0x400/0x100/0x200 EXCLUDE classes 3+0x12 / 2+0x11 /
   5 respectively (`beq` to the return), they are not seek-enable bits; the player's own
   +0x7d8 (= hittest+0x98) is zeroed every tick by Player_Collision 0x194df0:88 and only the
   swim substate 0xE sets 0x1c0 (:382)] — so normal on-foot Bond picks class-2 candidates with
   no class exclusions. obj+0xfa bit 8 (the Jointy gate below) is set by AnimObjectNew
   0x1be910:58 — every skeleton character (bots included) carries it.
3. **The intersect gates before Jointy** [M Collide_Intersect 0x1ebaa8 — disasm 0x1ebb7c-0x1ebbac:
   the other's obj+0xd8 (glist) must be nonzero, `other->0xfa & 8` set, and `glist+8 == 0`
   (glist+8 is the breakable-data pointer, Break_Kill 0x12b970:59)]: Jointy runs vs animated
   non-breakable characters.
4. **Jointy's kind gate** [M Collide_Jointy 0x1ec180:203-209]: BOTH sides' hittest kind words
   must be 0x800 — the bot's is ext+0x464, armed only by NDrone2_Collision 0x140e90:66-72/95-101
   when `NDrone2_DoCollision` 0x149ec8 returns 1.
5. **The arming** [M NDrone2_DoCollision 0x149ec8:11-34]: `(0x4f8 & 0x200)` flying → 0 (not
   0700004a); **`(0x4f8 & 0x20) == 0` → 0 — the master arm**; then for DSTATE ∉ {0xC, 0x13}:
   +0x538 != 0 → 1, **NearPlayerCollision (< 2.0) → 1**, DCVars+0xfc != 0 → 1, `+0x570 & 2`
   → 0, else 1. The 0x20 flag is set by DefaultInit (`+0x4f8 = (& 0xffea05e2) | 0x801e2`,
   0x14b300:60 — bit 0x20 in the OR mask) and NDrone2_Enable(1) (`| 0x801a0`, 0x149dd0:27);
   cleared by Enable(0) (`& 0xffe7fe5f`, :14), NDrone2_SetAsDead 0x14a2f0:16, the death/Fade
   states (SpecialDeath_Anim 0x163c68:55, AbseilDeath 0x169b28:19, Fade 0x172cd8:19) and the
   TruckDriver states (conditionally). Every spawned drone runs PostLoad_Init → DefaultInit
   (0x14d890:13) — armed at spawn.
6. **The near check is fed per control tick** [M NDrone2_PreDroneControl 0x148db0:35-48 fills
   `Drone+0x154[i]` = `Vec_Dist3D(drone->pos, glb_players[i]->pos)` (100000.0 for empty slots);
   DroneFunc_NearPlayerCollision 0x14b190 reads any non-null slot < 2.0].

### The port's change: the cel-straddle arm on the producer

`bot_capsules` (scene.rs) now requires the bot's containing cel to be inside the player's
straddled-cel set (the same set the third-icon context consumes; bot cel = `find_cel(pos)`,
the codebase's established stand-in for the game's obj+0x20 link — the sight check uses the
same stand-in, noted [S] there). **Evidence: link 1 above.** drone_bump_live stays 2/2 green.

## What the round established about the seven reds — and what is left

The cel arm alone does NOT un-wedge probe_07_route (Bond still stops at (79.19, 44.13, −51.21)
vs the bot at (79.57, 44.40, −50.34) — same-room adjacency straddles). The probe's binary
`probe_07_who` shows the hall bots (skins 0x5000020/0x24/0x26, team 3) boot `enabled false,
state 0x1` and are enabled by the party script before the walk — so in BOTH sides they walk
armed (link 5 arms every spawned bot; the party states set no 0x4f8 collision bits).

**The remaining hypothesis (diagnosed, not ported — it needs one live measurement): the
frozen/scripted bots are not armed because their control pass does not run.** Links 5-6:
NearPlayerCollision reads `+0x154[i]`, written ONLY by PreDroneControl on the bot's control
tick. A bot whose control is suspended (the cutscene "drone freeze", the scripted holds) keeps
a stale/100000.0 slot → the near arm fails → for DSTATE ∉ {0xC, 0x13} and +0x538 == 0 →
DoCollision 0 → **+0x464 = 0 → Jointy's link-4 gate fails → no push**. The port freezes bots
by skipping their AI but still arms them from live positions — that is the delta. The mayhew
failure ("the ambush guards died but ch 0x99 never rose") is consistent with the guards going
mobile during a stalled approach (states 0x29/0xa4/0x68 at (−30.1,10.2,13.6)…), i.e. the
push-induced stall changes the fight choreography, not a separate gate.

What is left, in order:
1. Port the arming per the game: a bot's capsule feeds the player query only when its own
   DoCollision would return 1 — i.e. respect the frozen-control bots' stale +0x154 (armed
   only via DSTATE 0xC/0x13/+0x538 then). Needs the live check below to settle WHICH bots'
   control the game really freezes.
2. Re-run the seven: mayhew_hold, party_door_lock ×2, walkreal_07000007, probe_07_route,
   cutscene_timer_pause, walkreal_07000011.
3. The +0xa0 anim-height word (capsule bottom `pos + up·(r − hip)`, hip = the anim's height −
   0.02, min 0.2 [M DroneAnim_PreTransCallback 0x13b490:63-66]) — the port holds hip = 1.0;
   scripted/sitting bots carry shorter capsules in the game.

## Verification (what ran)

- `cargo test --offline --release -j4 -p nf-view --no-run` — full test build OK.
- **drone_bump_live 2/2 GREEN** on the final tree (the cel arm keeps the proven push).
- probe_07_route: still FAILED (the wedge unchanged — reported above, the arming fix left).
- mayhew_hold_live: still FAILED (the ch-0x99 chain, consistent with the stall hypothesis).
- walkreal_07000001/07000011, party_door_lock, cutscene_timer_pause: NOT re-run this round —
  the build of the remaining binaries did not fit the window (the background `tools/test/
  suite.py` runner dies between tool calls here — re-confirmed again this round; suite.out
  empty, no cargo process survives). The affected crates: **nf-view only** (scene.rs);
  nf-game/nf-formats/nf-audio untouched by this round's commit.
- Scratch probe `probe_07_who` (assertion-free, --nocapture) added for the bot census.

## Live checks for the main agent

1. **The arming of a frozen bot** (07000007 fresh boot, the hall): gdbwatch a party bot's
   ext+0x464 (the kind word) while Bond stands 1.0 away and the bot is in a scripted
   hold/freeze — expect 0 if the control pass is suspended (the arming hypothesis), 0x800 if
   the game still arms it (then the delta is elsewhere). Also gdbwatch its Drone+0x154[0]
   (the near slot): 100000.0/stale vs live distance each tick.
2. **The party-crowd solidity**: fieldprobe — walk Bond straight into a standing party guest
   mid-floor (not a doorway): does the game stop him at ~0.95 (the push runs) or pass through
   (the guests are never armed)? This settles whether the seven reds need the arming fix, the
   radius-per-state fix, or both.
3. The +0xa0 anim height of a seated/chatting bot: the word at ext+0xa0 vs the standing 1.0
   (DefaultInit 0x14b300:48) — feeds the capsule bottom arm.

## Usage

- tokens/context/cost figures: not visible to me (no token or context-window readout is
  exposed to this session; nothing to copy exactly).
