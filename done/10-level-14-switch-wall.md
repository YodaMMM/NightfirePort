## Your task: `level-14-switch-wall`, what the player sees at Countdown Part 1's sabotage switch

On 07000014 the E-use switch oDroneTrigger1b (touch out 0xA0, mask 4) sits at (-60.30, 0.61, 38.37); the port draws nothing there, so a player cannot tell where to press E. Find what the game shows at that spot [D the placements/objects/glists within a few units, any 0x3b channel-show pairs, ScriptPlayers whose scripts draw there; M the touch's own draw, if any] and the M.Hatch indicator ScriptPlayers #947/#950/#952 (script 0x06000708, triggers 0x85/0x54) - which objects, frames and channels. Port any missing data/sim link (a placement the scene skips, a show channel not applied, a script object not spawned). List exactly what must DRAW (object hash, position, frame) in the report so the main agent can render and compare with the PS2. Test the data/sim side. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-switch-wall`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-28T12:55Z

---
## Report (done, 2026-10-28T14:44Z)


---
## Report from the worker (done, 2026-10-28T14:46Z)

# level-14-switch-wall — what the game draws at 07000014's E-use switch, and the M.Hatch indicator players

Branch `s/level-14-switch-wall` (from origin/master 17aa36ab; origin/master re-merged, "already
up to date"). The task: the E-use touch oDroneTrigger1b #725 (−60.30, 0.61, 38.37; out 0xA0,
mask 4 [D m54 placement 399; M Trigger_Activate 0x1b2318]) "draws nothing, so a player cannot
tell where to press E" — find what the game shows there and at the M.Hatch indicator
ScriptPlayers #947/#950/#952, port any missing data/sim link, list exactly what must DRAW, test
the data/sim side, full suite.

## Headline: the port already draws the switch — the premise is refuted with disc + sim evidence

Nothing at the switch band was missing, so there was no code to port. The port creates, gates
and draws every piece of the switch's furniture the game does, and the new tests pin each link
end to end. What looked like "nothing drawn" was the previous task's unverified observation (its
own report said "the port's draw state unknown"); the disc data shows the game has NO drawn prop
at the touch either — the player's cues are the console screen 2.2 units west and the INFO-bar
hint text, and both work in the port:

1. **The touch itself draws nothing — in the game too.** `Trigger_Create` 0x1b1720:95 sets
   obj+0xF0 = glist+0x44 = **0x50** (bit 0x10 = not drawn [M `View_AddObjects` 0x1e62d8:33]) and
   the placement's object carries **no leaf geometry** [D m54 #399 → glist 0x50, leaves 0].
   The kind-10 bit-clear arm (0x1b1720:99-100, the exit trigger's) does not apply to a 0xEB
   touch. The exit #944 has leaves 0 too — nothing drawn there either; the hatch art is baked in
   the room cels.
2. **The visible "switch" is the 0x3b electricity-console screen trio** at (−62.43, 1.19,
   38.59), 2.13 west of the touch, yaw −1.693 (the draw list below). The port's DynObject sim
   (`worldobj.rs` = `DynamicObject_Create` 0x18dbf8 / `Update` 0x18d3b8 verbatim — re-verified
   against both decompiles this session) and the shared 0x3b draw path (the build's slot
   instances + `lock_slot`'s per-frame pose; the same chain render-proven on 0700000d/0700001b)
   carry them. The build/sim slot bases align on this level (sensors 14 = 14, 0x3b 45 = 45,
   rotors 0 = 0, doors/sss/breaks/locks in the same order).
3. **The player's text cue works**: the Hint Small Plates #897/#900 (kind 0x33, glist 0x50 —
   the plate meshes are not drawn; the hint is TEXT [M `Hint_Update` 0x1b3380:28-30
   `Text_AddMsg(…, Txt_BindLabel(p0), …)`]) fire in the port with the game's own strings, bound
   from USATxt.dat: **0x100020c "Power system safeguards operational"** (off-channel 0x8C —
   before the sabotage) and **0x100020d "Danger: safeguards offline. System critical"**
   (on-channel 0x8C — after). Standing in the band fires the right one each way (test-pinned).

## The M.Hatch indicator ScriptPlayers (m54 placements #947/#950/#952)

The audit doc's #1273/#1276/#1278 are these same three SPs under a different index base. All
three run script **0x06000708** (384 B, 3 streams, 4 keys, length 61 [D decode]):

| placement | position | params | trigger |
|---|---|---|---|
| #947 (west, at the Door_Slide_11A/B frame) | (−52.40, 2.64, 40.14) | p0 0x6000708, p2 0x40, p4 1, p5 1, p6 **0x85**, p10 100 | the sabotage cascade's door-unlock channel |
| #950 (east, exit hatch #944) | (26.90, 2.69, −25.78) | same but p6 **0x54** | Rook's death channel |
| #952 (east, exit hatch #944) | (28.69, 2.69, −25.78) | same but p6 **0x54** | Rook's death channel |

**Mechanism [M]:** mode 0x40 = the preload ([M `SP_LoadScript` 0x1dbc08:46-54 — play + update +
stop, one tick, state 1]); mask p5 = 1 = `SP_Hit`'s bit 0 = **the out channel's own value**
([M `SP_Hit` 0x1dc470:17-19 — `bVar8 = switch_channels[obj+0x30] != 0`]), so the SP fires when
its trigger channel is on ([M `SP_Update` 0x1dbda8 case 1: replay + countdown 5 + write the
channel back 1]); p4 = 1 → after the replay the SP deletes itself ([M 0x1dbda8 case 2 → 5,
obj+0xFE bit 0]); the create clears the out channel ([M 0x1db9a8:72]). The SP's trigger channels
are the delta-exit walk's own milestones: 0x85 rises from the sabotage cascade (the west door's
unlock), 0x54 is Rook's death channel.

**The script's objects and frames [D decode]:**
- s0 (keys 0-1, both at the local origin, t 0.00 / 30.00 — the light never moves): op 7
  **Entity 02000724 "M.Hatch Indicator (locked)"** (obj_type 0x31 → class 0 via
  `Script_CreateEntity` 0x1d8968's fallthrough [M :170-173]) at t0; **Entity 02000725
  "M.Hatch Indicator (unlocked)"** at t31 (the stream object re-points to the new glist —
  the light swaps mesh IN PLACE at the SP position); End(61).
- s2: **Entity 02000a6b "GLOW_Light_1"** parented to stream 0 at t31.
- s1: Event 0x0c at t54 → SCRIPTINFO+0xaa8 |= 0x10 ([M `Script_EventHandler` 0x1d94c0:159-161])
  — the KEEP flag, so the spawned light survives the SP's death (the port's `Script::kill` keep
  arm).
- The pose: both s0 keys at the origin, so each light draws AT ITS SP's placement matrix (the
  SP base concat [M `Script_GetInterp` 0x1da158 tail]). The unlocked dummy #948's position
  (−52.23, 1.49, 38.38) is a resource-declaration reference only — the light never sits there.
- 02000a6b resolves only through the session hashtable: it lives in **0700000a.bin**, not this
  archive [D archive scan over every 07xxxxxx.bin], and the port's by-hash map is level-local —
  the sim still spawns the Obj::Entity (test-pinned), the draw correctly skips it. Its PS2_GFX
  block is **EMPTY on the disc** (boxes 0 / leaves 0 [D probe], vs the real coronas' boxes 1 /
  leaves 1 / batches [1]), so the game's spawned object draws no pixels either — no visible
  gap; the only deviation is the (unobservable) gfx resolution through the session.
- The resource dummies #946/#948/#949/#951 (flags 0x2031 — the dispatcher's param_7 gate skips
  them [M 0x1d04f0]) are declarations, correctly skipped by the port's object creation and
  without static cels (no 0x8000).

## Exactly what must DRAW (the main agent's render list)

At load, before any trigger:
1. **CONSOLE_Electricity_Active** — m54 object #293 (by index, no hash [D]), glist 0x0, 1 leaf /
   5 batches, sphere r 0.42 — at (−62.43, 1.19, 38.59), yaw −1.693. Drawn from load: p0 = 0 →
   no hide word ([M 0x18dbf8:27-29 — the |0x30 hide applies only when p0 ≠ 0]).
2. **02000724 "M.Hatch Indicator (locked)"** at (−52.40, 2.64, 40.14) — SP #947's preload.
3. **02000724** at (26.90, 2.69, −25.78) and at (28.69, 2.69, −25.78) — SPs #950/#952.
4. The generator-room SCREENS_* 0x3b pairs (world-space glist centres) — drawn as always.

After the sabotage cascade arms **0x8C**:
- #1 is DELETED ([M 0x18d3b8:26 — mode 0, armed B]) and **CONSOLE_Electricity_Inactive**
  (m54 object #295) shows at the same spot ([M 0x18d3b8:16-18 — A on → obj+0xF0 = base]);
  the INFO bar reads "Danger: safeguards offline. System critical".
- When the cascade reaches **0x85** (the west door unlock): SP #947 replays — at its t31 the
  west light **swaps to 02000725 "M.Hatch Indicator (unlocked)" at the same (−52.40, 2.64,
  40.14)**; the locked mesh is gone (the same stream object re-pointed). The east pair stays
  locked until **0x54** (Rook's death) swaps both the same way.
- On **0x5B** (no writer found in the level's channel tables [D audit] — likely dead):
  Inactive deletes, **CONSOLE_Electricity_Static** (m54 object #331) shows.

Render spots: `--bond -60.3,0.9,38.4` facing west at the console (the Active screen + the hint
text on entering the band), `--bond -52.2,1.2,40.1` facing the door frame up-west (the locked
light; after the cascade the unlocked one), `--bond 27.79,1.3,-25.32` facing up at the two east
lights (locked; after Rook's death unlocked).

## What was ported

No code change was needed — every link already existed. What this session added is the proof
(`crates/nf-view/tests/switch_wall_probe.rs`, all green):

- `the_m_hatch_lights_spawn_at_load_and_the_console_states_match_the_disc` — the three locked
  lights live at the three SP positions after the preload; no unlocked light/third object yet;
  the console trio's channels/modes/flags match the disc (Active visible, Inactive/Static
  created hidden |0x30); the 0x8C arm deletes the Active screen and shows the Inactive one.
- `the_m_hatch_draw_slots_pose_and_the_0x85_replay_swaps_to_the_unlocked_light` — the draw
  slots pose at all three light positions; the ch-0x85 replay swaps the west light to 02000725
  in place, the east pair stays locked, the west SP deletes itself after the replay, the kept
  light outlives it, and the parented 02000a6b is live on stream 2.
- `the_switch_band_fires_the_safeguard_hints` — the band fires 0x100020c, and with 0x8C armed
  0x100020d (checked in the HUD's info_msgs; the labels bound from USATxt.dat exactly as the
  game's Hint_Update does). The reach-a-spot hop is the tests' documented teleport allowance.
- Assertion-free dumps (opt-in `--ignored`): the near-switch census, the session archive scan
  for the script entity hashes, the 0a object details, the empty-gfx comparison against the
  real coronas, the console object indices, the hint-label binds.

## Verification

What ran (release, NF_ISO set): `cargo test -p nf-view --test switch_wall_probe` — **3 passed /
0 failed / 5 ignored** (the opt-in dumps).

**Full suite: INCOMPLETE — the sandbox kills detached processes, and the run phase does not fit
the session window.** The resumable runner (`scripts/suite_chunk.py`, the suite's own build +
per-binary `--test-threads=1` runs with state) completed **118 of 878 binaries before the
deadline stopped new work: 464 passed, 0 failed, 30 ignored, 0 bad binaries** (the state file
travels with the sandbox; the next session resumes with `python3 scripts/suite_chunk.py` and
prints the full total). No failure of any kind so far. The 07000014-relevant binaries
(walkreal_07000014_live, walk_07000014_live, panels_07000014_live, fix_fusebox_cut_live) are in
the unrun set — the next session should run them first (`--only`), then the rest.

## Live checks for the main agent

1. `--bond -60.3,0.9,38.4` facing west: the Active console screen drawn on the console at
   (−62.43, 1.19, 38.59) and the INFO bar "Power system safeguards operational" on entering the
   band (once). After the sabotage: the screen swapped to the Inactive mesh + "Danger:
   safeguards offline. System critical".
2. `--bond -52.2,1.2,40.1` facing up-west: the locked indicator light at (−52.40, 2.64, 40.14)
   before the cascade; after 0x85 the unlocked light at the same spot (the west-door report's
   live check 2).
3. `--bond 27.79,1.3,-25.32` facing up: the two east locked lights at (26.90, 2.69, −25.78) and
   (28.69, 2.69, −25.78); after Rook's death both unlocked. Also confirms the game really
   renders both east panels (the disc has two SPs + two locked dummies and no east unlocked
   dummy — the unlocked mesh comes from the shared script).
4. The PS2 at the indicator replay: whether ANY glow shows from 02000a6b (the port reads its
   gfx block as empty; the game's spawned object should be invisible — confirm no glow appears
   next to the unlocked mesh on the PS2).

## Usage

Token/credit/cost figures: not visible to me (no usage readout is exposed to this session).
Context window: not visible to me (the session runs from a continuation summary).
