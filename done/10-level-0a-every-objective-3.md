## Your task: `level-0a-every-objective-3`, Tower B office floor: the vent crawl into the Q-Worm cubicle, then the rest of the walk

Continue `level-0a-every-objective-2` (on master; read its report). The Q-Worm install's last stand-in remains because no walk-reachable stand has a bullet-clear shot at the cubicle PC; the game's own route is most likely the VENT CRAWL into the cubicle's south side. Port the vent crawl the game uses there [M the crawl/creep path, the vent placements D], walk it on real input, do the real install, then the mission tail (the ring legs and the hatch E) the -2 branch re-derived. Un-ignore `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch` when it is green; keep it ignored only with the exact remaining blocker and its evidence. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-every-objective-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T17:51Z

---
## Report (done, 2026-10-29T18:36Z)

# level-0a-every-objective-3 — the Q-Worm install is REAL (the walked south-band route, the gadget-cycle equip, 1 real shot); the mission test re-ignored with the walked mouth evidence; the crawl found already ported and the 0a vent found empty

## What was asked

Continue `level-0a-every-objective-2`: the Q-Worm install's last stand-in remained because no
walk-reachable stand had a bullet-clear shot at the cubicle PC; the guess was the vent crawl into
the cubicle's south side. Port the crawl the game uses, walk it on real input, do the real
install, then the mission tail; un-ignore `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch`
when green; FULL suite with its total line.

## What the probes found (scratch probe_l0a3_*, this round, not committed)

**1. The crawl/creep system is ALREADY in the port — and 0a cannot use it.** The game's crawl is
substate 7: `Player_Creep` 0x1969d0 (the per-tick channel hold, the axis snap, the end-exit rule)
entered via `Player_Activate` 0x1a5170:66-97 on the use key when the third-icon context `BL+0x95F`
is 6, through `Player_CollCreepWall` 0x196758 (camera 0xB, the floor ray, the holster) [M]. The
port carries it line-for-line: `nf_game::player::SS_CREEP`, `Bond::start_creep`/`creep_tick`
(player.rs:1319/1353), `creepwall.rs` (the 0x3F data model), the scene's ctx-6 arm and channel
hold [M port]. **But 0700000a has NO creepwall (0x3F), NO 0x3B blocker, and NO type-6 icon** [D
docs/levels/0700000a-audit.md §3, the full 800-placement census; the level doc's own census line
agrees] — there is no rail for the crawl to run on, so the game's crawl cannot engage on 0a at
all. The level's vent furniture is the ladder #686 `Vent_StoreRoom_Ladder` (0x22, centre
(−13.156, −2.014, −9.549), bottom −2.05, top −0.13 [D + the port's ladder dump]), the E-openable
mezzanine grate SS #663 (0xE2 at (−4.6, 4.6, 0.4)), the graphic SP #685, and the static room
`office-b_ventilationpipe` (one of the 30 room cels, loaded [D audit §3]).

**2. The under-floor vent space is an EMPTY VOID in the port's collision.** Standing at the
ladder bottom (the ladder test's own stand), every horizontal ray at y −1.8/−1.3/−0.7 in all 8
directions is clear, the down-ray to y −6 is clear, the up-ray to y 1 is clear, and
`floor_below` finds no floor at any of 25 grid points around the ladder [D probe rays]. Walking
south from the void, Bond rises onto the office-floor slab top at (−13.6, 1.04, −12.4) and wedges
there — the shaft mouth exists but the crawl space the ladder serves does not (as collision).
Whether the REAL game has a floor/walls there is exactly the -2 round's live check #1 — still
open, still listed below.

**3. THE INSTALL DOES NOT NEED THE VENT — the south band is walk-reachable around the diagonal
wall's east end.** The -2 round's "south stand blocked at z ≈ −31.5" was an under-probed grid:
it probed x −18..−15 only. This round: the office block's south wall runs z −25.2..−27.1 (north
faces n (0.26,0,0.97), x −18..−6); the pocket south of it is walled from the band behind the
cubicle by the diagonal wall whose north face (n (0.30,0,0.95)) spans x −18..−15 at
z −29.8..−31.5 — **but its east end stops at x ≈ −15**: the column at x −13.5 runs south into
the band (northward rays from z −34.4 clear at x −13 and x −9 [D probe rays]). The band's floor
(y 1.10) holds bullet-clear stands: `segment_bullet` CLEAR from every stand x −16..−17.8,
z −33.5..−34.4 to the PC [D probe rays; the sight ray hits the desk lip — the known
bullet/sight pass-flag difference]. Walked end-to-end (probe_l0a3_install): every leg arrived,
the stand (−15.89, 1.10, −34.24) is 2.40 from the PC.

**4. The real install lands, and it alarms nobody.** With the Q-Worm equipped through the REAL
gadget cycle (9 cycles, mode-1 bounds 0x4A..0x5E owned-only [M player.rs Input::gadget_cycle]),
**1 real shot from the walked south stand → ch 0x48 ON** — the PC's own writer
[PCQorm_Update 0x1aee20: a weapon-0x58 BULLET in the PC's contact list writes
switch_channels[0x48]]. The -2 round proved the old legs' 112 blocked shots were the spawn
loadout; this round the equip + the clear stand close it. 1200 ticks of watching after the
install: ch 0x61 off, fail 0, exit None, zero guards seen or near [probe_l0a3_alarm].

**5. The walk-back needs the ring's alarm-runner watch.** The prologue's nav-route walk-back
passes the alarm-panel corridor; a guard challenged en route ran for the panel and his press
landed mid-keyboard-leg (deterministic "level_exit reached at +342 ticks" into "round the wall
end"; ch 0x61 still off at the walk-back's end — the runner was mid-run). Fixed with the ring's
own pattern: the walk-back legs split and walked with `walk_leg_watched` (the watch between the
legs kills the runner before the press) [M the runner 0x168ea8's follow]. The mission test's
ring re-derivation then walked and wedged at the mouth — see below.

## What changed

- `crates/nf-view/tests/walkreal_0700000a_live.rs` only:
  - The prologue's install leg: the walked south-band route (5 legs), the real gadget-cycle
    equip, `shoot_at` from the walked stand, and the assert on ch 0x48 from the REAL writer —
    **the unconditional `set_channel(0x48)` stand-in is deleted** (this was the task's last
    stand-in).
  - The walk-back: split into per-node legs with `walk_leg_watched` (the alarm-runner catch) +
    a hard state gate after it (`!ch 0x61`, `level_exit.is_none()`).
  - The ring: the -2 42-leg delta re-applied (master never carried it), then this round's
    walked corrections — the (−11.5, −13.5) leg replaced (it hugs the pillar's south face:
    the r-0.55 capsule's north edge z −12.95 overlaps the face z −13.10 [D probe rays]; it
    wedged at the 6000-tick cap) — and re-ignored with the walked evidence (below).
- `docs/WORKLOG.md`: this task's entries. No port-code change: the crawl system and the
  collision behaved as documented; every prior blocker reading that fell, fell to test-side
  route work.

## Why the mission test is re-ignored (the exact remaining blocker, walked this round)

The prologue (arrival → the REAL install → the keyboard → muxes) is green. The ring's mouth
crossing wedged on THREE walked routes: (1) the -2 leg (−11.5, −13.5) — inside the pillar's
south face's capsule band; (2) east along z −15 — the NS wall's west face x −10.6955 runs
continuously z −17.5..−11.0 [D probe rays]; (3) around the wall's south end at z −18.6
((−12.0,−18.6)→(−8.5,−18.6)) — wedged, and the wall line past z −18.6 is unprobed. The -2
round's 42-leg ring had never actually been walked; this round walked it to the corner and
pinned the three wedge points. Next round's first probe: eastward rays z −18.4..−19.5,
x −12..−8, and the pocket's southward columns x −11..−9, then re-derive the join to node 130.

## How it was verified (what I ran)

- `cargo test --offline --release -p nf-view --test walkreal_0700000a_live` (full file,
  --test-threads=1, final tree): **6 passed / 0 failed / 3 ignored** (22.7 s) — the mission
  test is one of the 3 ignored (re-ignored with the walked evidence; the two glide pins are the
  other two, unchanged).
- `cargo test --no-run` builds over nf-view's walkreal + the four-crate release build ran clean
  through this session's iterations (the last full `--no-run` pass had no errors; only the
  pre-existing warnings).
- The five scratch probes (not committed): the ray grids and walks cited above.

## The FULL suite — honest status: NOT RUN to a total line this window

The driving message's background start (`nohup … &`) was launched in the first minutes and died
instantly — twice verified (empty `suite.out`, no process; the same sandbox behaviour the -2
report documented, where the chunked foreground fallback was used). This round the remaining
time went into the route/probe work above, and the chunked foreground sweep did not fit before
the stop-new-work line. **The suite's total line is NOT quoted because it was not produced.**
The next message should start the chunked sweep from `bins.tsv` (repo root, untracked, from the
prior session — regenerate for this tree) and run it until ALL DONE before merging.

## What's left

1. The ring's mouth crossing (the three wedge points above; the probe list is in the ignore
   reason) — then the un-ignore for real: the prologue is already green with the real install.
2. The suite's chunked sweep on the final tree (the total line for the merge).
3. The vent questions for the record (live checks below) — the install no longer depends on
   them, but the port's missing under-floor collision (if the real game has it) is a real GAP.

## Live checks for the main agent

1. (carried from -2, now sharper) PCSX2, fresh 0700000a boot: at the vent ladder #686
   (−13.16, −9.55) — is there a FLOOR at y ≈ −2 under it in the real game, and can the player
   crawl south under the office floor toward the cubicle? The port's collision there is an empty
   void (probe evidence above); `office-b_ventilationpipe`'s cel either has no collision tris or
   is art-only.
2. The mouth: eastward rays z −18.4..−19.5 x −12..−8 in the port decide the ring's crossing
   around the NS wall's south end; in the real game, walk the pocket (−14.9, −29.2) east to the
   ai lane x −8.5 and record the actual crossing latitude (the wall line x −10.6955, the pillar
   x −12.98..−10.9 / z −13.1 faces, and anything at z −18.6..−19.5).
3. The south band: confirm in the real game that the band behind the cubicle (x −16..−18,
   z −33..−35) is walkable from the x −13.5 column (the port's walked route) — if the real game
   walls it (the vent being the only way in), the port's diagonal-wall east end is missing a
   blocker and the collision owner should compare the cels there.

## Usage

not visible to me (this environment exposes no token, cost or context-window figures to the agent).
