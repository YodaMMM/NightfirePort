## Your task: `level-16-stuck-overlook`, 16: no way on from the overlook by the control-room glass

On 07000016 at Bond (-17.86, 6.81, -97.16) facing yaw 6.98 (the walkway beside the big control-room window, around tick 3700 of a real run) the player cannot continue the level, and the held rifle's view model visibly clips into the glass/wall. Find what the game expects here [D the route on from this walkway: doors, triggers, the channel that opens the way; M the classes], reproduce on real input from the level start with the port's walker, fix any port gap with the address cited, and pin a walk that passes this spot. Also check the view model's near-clip / wall push-back at that spot against the game [M the view-model draw path]. Run walkreal_07000016_live, interactables_07000016_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-stuck-overlook`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T03:07Z

---
## Report (done, 2026-11-01T05:31Z)


---
## Report from the worker (done, 2026-11-01T05:31Z)

# level-16-stuck-overlook — report

Branch `s/level-16-stuck-overlook` (from master 521d9bb2). Task: on 07000016 at Bond
(−17.86, 6.81, −97.16) yaw 6.98 (the walkway beside the control-room window, ~tick 3700 of a
real run) the player cannot continue the level, and the held rifle's view model clips into the
glass/wall.

## What the game expects there (the answer)

The reported spot is the **SW dead end of the trench hall's upper WEST walkway** (deck
y 5.7..6.8 [D docs/levels/07000016.md §7]), ~3.5 m SW of the **SecurityStation** trigger #597
(kind 0xEA TOUCH_ONCE, out ch 0x24, mask 0x04) at (−15.20, 6.52, −94.56) [D]. The way on is
**back north-east along the walkway to the station, and press E**:

- the E dispatch fires ch 0x24 [M Player_Activate 0x1a5170 → Trigger_Activate 0x1b2318:38-49,
  the mask&4 arm];
- door[0] "Door_1" (1.84, −0.10, −114.23, ch_lock 0x24 [D]) unlocks AND opens on the channel's
  rising edge [M Door_Update 0x1363c8, the lock arm ~0x13660c → Door_Activate 0x135be0];
- downstairs the south corridor to the silo is the route on (the existing walkreal route).

So "cannot continue" at that spot is **route knowledge, not a port gap**: from the dead end the
station is 3 m NE and its E unlocks door[0] 20 m below. The port's doors, trigger and E path
all work (pinned below).

## The pinned walk (reproduce on real input from level start)

New test `walkreal_07000016_live::the_overlook_dead_end_returns_to_the_station_and_door0_opens`
(real input only — no `bond.pos` writes after the spawn; the carried 07000014/15 chain like the
sister tests): arrival → south corridor → east ramp → upper ring west → the station area →
SOUTH past the station to the reported dead-end spot → back NE → the station E → down and
through door[0]. Result: the dead-end spot is reached grounded at (−17.50, 6.84, −96.37) (the
raised south end holds Bond; the 1.2-radius assert covers the reported (−17.86, 6.81, −97.16)),
ch 0x24 rises on the E, door[0]'s lock clears and Bond walks through. **No wedge, no gap** —
the walk pins the whole arc for the interactive player.

## The view-model check at that spot [M the view-model draw path]

The game has **no wall push-back and no special near plane for the gun**: the gun path
(`Player_PositionGun` 0x1a3b40, `Player_SetWeaponAnimObj` 0x1a2e38, `Player_GetHeadPos`
0x195aa0) never queries collision; the anti-clip is **draw-order**: the gun's flags word 0x2000
puts it in draw list 2 [M View_AddObjects 0x1e62d8:57-60] and `Game_Draw` 0x1e6f50:157-163 runs
`psiClearZ` 0x10b9a0 (a fullscreen write-Z-only sprite) before list 2 — world geometry (glass
included; glass leaves are ZMSK=1) can never occlude the view model. The real keep-out is the
0.55-radius player capsule [M Player_Collision 0x194df0]. The port already mirrors the
depth-clear (gpu.rs list-2 VMZ trick; scene.rs VIEW_MODEL_NODE → pass 7), so a rifle overlapping
the glass when Bond stands close is **faithful** — the game does the same at that proximity.

Two faithful gaps were found and fixed in the port's view-model draw path:

1. **The delta-repeater bolts** were the one fp attachment built on the default node
   (world-occludable) instead of the depth-cleared view-model pass — now VIEW_MODEL_NODE [M
   View_AddObjects 0x1e62d8:57-60; Game_Draw 0x1e6f50 psiClearZ 0x10b9a0] (scene.rs).
2. **The camera near plane** was a port choice 0.05; the game's is `Ps2Near2 = 0.1` [L
   ee_start_05.bin; docs/systems/render.md — PS2WorldViewVU1Kick2 0x10b2b0's depth mapping,
   f = 2^24, the ZBUF range saturating ≈ 0.114] — now 0.1 (lib.rs).

## Tests run (final tree of this branch) — the quoted totals

- `walkreal_07000016_live`: **10 passed, 0 failed** (the 9 existing + the new dead-end walk).
- `interactables_07000016_live`: **8 passed, 0 failed**.
- `nf-game` (all 97 of its test binaries, direct run): **964 passed, 0 failed, 9 ignored**.

Full suite (fast runner): the sandbox kills nohup background jobs between tool calls (verified
twice), so the runner's 1013 prebuilt binaries ran through an untracked 2-worker foreground
chunk runner with resume state. State at the window's end: **126/1013 binaries, 426 passed /
0 failed / 26 ignored**; two CRASH entries (`walkreal_07000014_live`, `interact_0700004a_live`)
were re-run solo and pass (**6/0** and **6/0/1** respectively — the 4 GB sandbox OOMs
concurrent heavy walkreal pairs). One chunk ran without the per-shell env exports and its 489
instant ISO-test failures were purged from the state and re-queued. The remaining ~890 binaries
are code paths my diff does not touch (a new test in one nf-view binary + two nf-view
render-path lines), but they have not been re-run on this branch in this window.

## What's left

1. **origin/master moved 521d9bb2 → babf9247 mid-session** (140 files, walker edits among
   them). NOT merged: the merge would invalidate every quoted run with no window to
   re-validate. Next continuation: merge, rebuild, re-run the three quoted targets + the
   suite tail (the chunk state format makes that resumable).
2. **Render checks for the main agent (GPU-less sandbox here):**
   - `--bond -17.86,6.81,-97.16` facing yaw ≈ 6.98: the rifle should draw OVER the control-room
     glass (list-2 depth-clear) and the raised dead end should look solid; with the near-plane
     change, geometry within 10 cm of the camera clips like the game's.
   - Any view with the Delta Repeater equipped: the three magazine bolts should ride the gun
     and stay visible when the gun is against a wall.
3. **Live checks for the main agent** (fresh-boot PCSX2):
   - At the dead-end spot: Bond's head→glass distance (expect the 0.55 capsule's ≈0.3–0.55
     stand-off [M Player_Collision 0x194df0]) — settles whether the interactive "clip"
     proximity is the game's own.
   - PINE-measure BLData+0x7e8 → gun obj+0xc0 (the gun world pos) at that spot and diff against
     the port's `Sim::gun_matrix()` output.
   - Watch switch_channels[0x24] (0x26FC90+0x24) rise on the station E during a real run —
     game-side confirmation of the route answer above.

## Usage

Not visible to me: this session exposes no token, credit or context-window figures for itself.
The only token figures I can see are the research subagents' per-call counts printed in their
results (e.g. "Tokens -> prompt:186398 completion:4993", "prompt:124370 completion:5829",
"prompt:115029 completion:2990"); I cannot see my own session's totals, cost or quota.
