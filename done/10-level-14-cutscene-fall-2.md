## Your task: `level-14-cutscene-fall-2`, the cut-scene base reset without breaking other levels' scripts

Start from `origin/s/level-14-cutscene-fall` (merge it first; its report: `Script_CameraStart` 0x1d9140 resets the script base SCRIPTINFO+0x00 to identity [M Mat_IdentityT at 0x1d916c], so a script with a camera op keys world space afterwards). Merged with origin/master these go red, each green on master (and green on master + the urn fix):
- interact_07000001_live (the armour pickup + channel 13; the E key on the entry sliding door)
- exit_aim_01_live::the_exit_resolves_to_the_gate_plate_and_e_is_refused_there
- escort_hide_findrun_live::the_hides_exit_reanchors_the_mission_route_before_the_lead_resumes
- zoe_gondola_realpath_live::zoe_rides_the_carriage_and_never_targets_bond (08)
- walkreal_07000008_live (the ally fights only the guards; the gondola ride to the end)
- cutscene_fall_probe_07000014::probe_camera_flags_of_pinned_scripts (the branch's own probe)

1. Find which scripts on 01/08 your change re-keys (camera-flag scripts whose entities/objects were authored as local offsets) and what the game really does for them [M — when exactly the identity reset happens relative to the entity streams' first keys, whether it applies to the entity interps or only the camera interp (case 4 vs case 1 base-concat flags at 0x1da054 / 0x1d9b30), which SCRIPTINFO the reset hits when a player runs several scripts]. Fix the port to the game's exact rule.
2. Everything above green, the 14 cut-scene camera stays in the room and Bond is back on the floor after the hand-back; run the FULL suite on the merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-cutscene-fall-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-10-25T11:19Z

---
## Report (done, 2026-10-25T12:52Z)


---
## Report from the worker (done, 2026-10-25T12:52Z)

# level-14-cutscene-fall-2 — the cut-scene base reset without breaking other levels' scripts

Branch `s/level-14-cutscene-fall-2` = origin/master (34ac31a5) + origin/s/level-14-cutscene-fall
(c2d9fbc8), then merged origin/master again at d8334b70 (batch-1025e, the urn fix) — merge
commit 24faf89f. Window 11:21–13:20 UTC (sandbox expiry); the last ~55 minutes went into the
root cause below, so the re-pin of the six red groups is prepared but NOT executed.

## What's done

**The fall fix's rule is confirmed exact at disasm level — all four links the task asked for:**

1. **When the reset happens**: `Script_CameraStart` (0x1d9140) resets the base as its first
   act, BEFORE the `+0xaa8 & 0x20` flag test — `Mat_IdentityT(SCRIPTINFO)` [M 0x1d916c: the
   disasm shows `lw $v0,0x98($s1)` then `jal 0x1e34c8` with `$a0` still holding param_1 = the
   SCRIPTINFO; the prologue only copies a0→s1/a1→s2 and nothing rewrites a0].
2. **Entity interps vs camera interp**: the base reaches BOTH, through different doors.
   The camera's `Script_GetInterp` call stores the literal 1 into the base-concat flag slot
   [M 0x1da054 `addiu $v0,$zero,1` + 0x1da068 `sw $v0,($sp)`], and `Script_GetInterp`'s tail
   runs `Concat(SCRIPTINFO, out)` when that slot is non-zero [M 0x1da838 `beqz $a3` skip /
   0x1da840 `jal 0x1e4a90`]. An entity stream's own interp gets the flag `(stream+0x25 == -1)`
   [M 0x1d9be4/0x1d9bf4/0x1d9bfc `lb 0x25($s1)` → `nor` → `sltiu` → 0x1d9c0c `sw $v0,($sp)`];
   a PARENTED entity skips the base in its own interp and takes it once at the parent-chain
   root [M 0x1d9cd4 `sw $zero,($sp)` per parent, 0x1d9d34 `Concat($s2=SCRIPTINFO, obj+0x90)`
   after the `bne $s0,$s6` chain loop exits with parent == -1]. So the port's "base always,
   after the chain" composition matches the game's product either way.
3. **Which SCRIPTINFO**: one SCRIPTINFO holds ALL streams of one script (array at +0xa8, 0x28
   each, count +0xaaa, index +0xab3 [M Script_Run 0x1d8548 head]); `Script_Update` 0x1da998
   walks every stream in index order each tick. Sub-scripts (op 0x1b, `Script_SubScriptStart`
   0x1db050) get their OWN SCRIPTINFO whose base the parent's mode-7 stream keys every tick
   [M case 7 at 0x1d9d44], so a camera op inside a sub resets only the sub's base. The port's
   per-`Script` `matrix` and op-0x1b `Script::load(n, self.matrix, …)` already model this.
4. **Stream order**: the reset lands the moment the op is read [M Script_Run: interp first,
   then the op loop], so on a camera-flag script every key sample from the camera op's tick on
   — in any stream of that SCRIPTINFO — composes with identity.

**Why master was green and the merge is red — the epsilon carrier.** On 07000001 and 07000008
every camera-flag script's SP base is identity except for one ulp: the port's placement
`object_matrix` (nf-formats/src/placement.rs) computes m00 = `sinf(π/2)` = 1.0000002 for
rotation (0,0,0) — the PS2 sine [M PS2Sinf3, G-046] — and SP #276/#277 (01), #294/#296/#307
(01/08) all sit at rotation (0,0,0) [D probe dump, base matrices printed]. In the GAME that
epsilon exists only until the script's first camera op — for the 01 intro 0x06000087 the op is
the first op of stream 0, so the base is exact-identity from the intro's first tick [M
0x1d916c]. The pre-merge port composed the whole 2120-tick intro (its glist entities s5
0x02000231 / s9 0x020001D7 — the script-created AI volumes [D decode]) with the 1.0000002
base. The reset removes it, exactly like the game.

**Measured**: the first simulation divergence between master and the merged tree on
07000001 is tick 254 — guard D0 (skin 0x05000036, state 0x74) x differs by 4.8e-7
(`probe_drones_at_divergence`, both trees, 9-decimal dump) — and from there the epsilon
compounds through the AI into macroscopic route differences (Bond's approach to the entry
sliding door lands ~4 cm away, and the kind-2 touch path opens the door the sister expected
shut). This is deterministic chaos, not a rule disagreement: the tests pinned the
epsilon routes.

**Verification runs** (this window): pure origin/master d8334b70 in a scratch worktree —
interact_07000001_live 10/10 GREEN; the merged tree — the same test red on
`the_armour_pickup…` and `the_e_key_opens…sister` (state 4 vs 2), exit_aim_01 red (the
contact within 6.0 of the plate), and the 14-side pair GREEN (cutscene_fall_07000014_live
2/2 — the fall fix itself still works). nf-game's diff vs master is exactly the reset block
(git diff inspected), so the reset alone flips the six groups.

## What's left

- **Re-pin the six red groups** per the task's test rule (the tests contradict [M 0x1d916c +
  0x1d9c0c + 0x1d9d34 + 0x1da068]): interact_07000001_live (armour sister + door sister),
  exit_aim_01_live (the 6.0-band premise), escort_hide_findrun_live (the re-anchor leg),
  zoe_gondola_realpath_live (08), walkreal_07000008_live, and the branch's own
  cutscene_fall_probe_07000014::probe_camera_flags_of_pinned_scripts. Each failing expectation
  needs the current (post-reset = post-epsilon) value with the address citation; none of them
  is a decompile-pinned number, they are route-sensitive behavioural pins.
- **The full four-crate suite** on the final merged tree (tools/test/suite.py) and the total
  line — did not fit this window; the background runner is killed between tool calls here, so
  it needs a foreground slot.
- The two probe files (`base_reset_probe_01_08.rs`, `door_trace_01.rs`) are untracked
  diagnostics — commit or drop them next window.

## Live checks for the main agent

1. **The epsilon wipe in the real game**: fresh boot of 07000001 (not a save-state resume),
   break at `Script_CameraStart` 0x1d9140 during the intro (SP #277, script 0x06000087), and
   dump SCRIPTINFO+0x00 BEFORE the Mat_IdentityT call (expect m00 ≈ 1.0000002 if the game's
   PS2Sinf3(π/2) rounds the same way) and AFTER (expect exactly 1.0). gdbwatch on
   SCRIPTINFO+0x00, access type write, one hit.
2. **The door approach**: fresh boot, walk Bond to (7.888, 5.35, 4.47) — 1.6 in front of door
   #13 "SlidingDoor" — WITHOUT pressing E. Expected per the post-reset port: the door opens
   (the kind-2 touch path). If the real game keeps it shut, the divergence is NOT the epsilon
   and the re-pin above is wrong — the route difference would then be a real port bug to hunt.
3. **Guard D0's walk**: fresh boot, watch the skin-0x05000036 guard's x at frame ~254 of the
   intro (expected ≈ −7.3970, the two candidate values differ by 5e-7 — either value confirms
   which side the game's epsilon lands on).

## Usage

not visible to me — this environment shows no token, context-window or cost figures anywhere
(no status line, no /usage output available); nothing to copy, and per the task's rule I did
not estimate.
