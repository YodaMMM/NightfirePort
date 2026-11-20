# retune-nb-08-12 — 08 whole-mission 00-Agent real-damage walk, round 12

Branch `s/retune-nb-08-12`. `origin/s/retune-nb-08-11` was not on master, so it was merged in
first (clean). Still red — the walk dies t26276 (frame 26278) at (113.00, 13.47, −179.31) — but
round 12 landed the two biggest structural wins of the whole retune: the machine took its FIRST
dive-window damage (a rocket landed −101.0 where every earlier dive-window run died with
0.0/598), and the round-11 reply-missile wall is down.

## What was done

1. **The fine launch-gate diagnostic** (`[08 gates]`, every 10 ticks for the first 200 fight
   ticks, d3-gated, eprintln-only — no sim tick, the timeline byte-exact). The t%100 samples
   had left the dive (fight t0-116, f25710-25830) a blind spot. The first lever run's dump
   measured the launch failure exactly: at fight t0 Bond was ONE gate from launch (line_ok
   false on the t0.936 sag stop alone; slit/eye/lane all true), then yaw_err ran −0.78 → −2.44
   through t10-60 — the fight loop's walk-back RELEASES THE AIM ([M player.rs Player_Move's
   aim branch]) and turns toward the walk bearing — and by t70 (aim re-converged 0.004) the
   machine had climbed past the cp[1] ≤ 13.7 launch band. ZERO launches in ~2000 fight ticks,
   damage 0.0/598 [L the lever-1/2 runs].
2. **SHIPPED lever — the d3 anchor moves to the SE pane-hole spot (113.3, −179.3), plus a d3
   pre-fight anchor walk** (after the shoot-out, before the fight loop; the d2 slit walk's own
   bounded pattern). The geometry: both the launch line and the dive reply run chest↔machine
   (the reply's dir is fixed at ITS spawn through the chest [L the diagnostic runs]), so ONE
   chest fixes both crossings — from (113.3, −179.3) each line crosses the west band's x-plane
   110.4 at z ≈ −181.6, pane 2's shot-out centre, ~0.4 u clear of the mullion where the
   round-11 reply detonated (−26.00 at (110.4, 14.1, −181.0), d 3.4, MID-RIDE). The pre-fight
   walk puts Bond at the anchor BEFORE the loop opens so the walk-back (whose 0.35 hold is 0.22
   inside the stand press's re-bridge resting spot (113.08, −179.02)) never walks during the
   dive. MEASURED [L the SE run + the final run, byte-identical]: **rocket 1 LAUNCHED mid-dive
   (fight t49, machine (72.97, 13.41, −208.87) dist 49.7) and LANDED −101.0** — the retune's
   first dive-window machine damage — and **ZERO reply-blast rows**: the round-11 −26.00 wall
   is down. Death t26276 (was t25822), 454 ticks deeper, the machine at 497/598, clip 3
   rockets intact.
3. **The remaining killer, measured and named — the terminal hurt volume.** The standing
   capsule RE-BRIDGES onto the hole's rim at this whole latitude (feet 13.470367 at every
   standing spot including the park — the lever-4 run), and 22 × −2.00 rows (the approach
   burst f25702-25801 + a sustained burst f26198-26278) drained the WHOLE 44.0 entry: 64 hits
   for 100.0, all zaps, no explosions in the death window [L the final run]. Every zap row in
   every run printed feet 13.5 (the rim band); the crouched plate walk never took one.
4. **TRIED and DEAD on their own real runs** (each judged on its own run, then reverted):
   - **The north strip (113.08, −178.1)** — the round-11 lever-1 candidate, tried verbatim
     with and without the pre-fight walk: the reply blast DID land 0-damage there (the
     explosion (110.3, 14.3, −177.8) at d 4.3 — outside the 4.0 falloff; round 11's was −26.00
     at d 3.4), but the launch line CANNOT open from the north spot: launch_line_clear +
     machine_lane_clear read false at EVERY dive tick (the crossing z −180.3..−180.5 rides the
     doorway gap's south jamb), zero launches in ~2000 fight ticks, damage 0.0/598, and a
     SECOND dense zap window (12 −2.00 rows f26657-26871) sits on the spot [L the lever-1/2
     runs].
   - **The park-plate anchor (113.08, −179.24)** — the "zap-free plate" idea: byte-identical
     to the round-11 reply death. The stand press's re-bridge resting spot (113.08, −179.02)
     is 0.22 inside the walk-back's 0.35 hold, so Bond fights at −179.02, the crossing returns
     to the −181.25 mullion, and the −26.00 reply blast returned at t25824 [L the lever-5
     run]. This also disproved the "plate standing spot" premise: the standing capsule
     re-bridges at the park too.

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk (NF_DMG_TRACE): the north
  strip without the walk (zero launches, death t27775), the north strip with the walk (zero
  launches, the gate dump above), the SE anchor (launch t49, −101.0, death t26276), the
  park-plate anchor (the round-11 reply death t25822), and the final tree re-verified
  byte-identical to the SE run [L the five runs, 14-21 s each].
- `walkreal_07000008_live` whole binary on the final tree: **6 passed / 0 failed / 1 ignored**
  in 149.84 s — the d1/d2 real-damage sisters, the plain walks and both ally tests stay green
  (the sisters' timelines are untouched by construction: the anchor is the d3 arm of the const,
  the pre-fight walk and the gate dump are difficulty ≥ 3-gated, the diagnostics are eprintln
  only) [L the final-tree run].
- The final tree's only code delta vs the merged 08-11 state is
  `crates/nf-view/tests/walkreal_07000008_live.rs`. No `crates/*/src/`, no
  `crates/nf-view/tests/common/` change, so the affected-binary set is `walkreal_07000008_live`
  alone, and it ran whole.
- The full suite (tools/test/suite.py) did NOT fit the window, for environment reasons: the
  sandbox was freshly reset this window (all setup ran from scratch — clone, the 370 MB ISO
  download, work-slim, rustup), and the background suite start gets killed between tool calls
  (the same failure as rounds 8-11); the foreground build of the ~200 nf-view test binaries
  does not fit the remaining window after the five lever runs. What ran instead, on the final
  tree: the whole `walkreal_07000008_live` binary above (6/0/1, 149.84 s). No `crates/*/src/`
  change this window, so every other binary is byte-identical in behavior to the merged 08-11
  state, whose numbers are in docs/reports/retune-nb-08-11.md. `nf-game --lib` also ran on
  the final tree: **796 passed / 0 failed / 4 ignored** — the same numbers the merged 08-11
  state reported, confirming the no-src-change byte-identity [L the final-tree run].

## What's left (the next window's levers, in priority order)

1. **The zap-free standing spot OFF the hole's rim** — the round-6 note's (111.99, −179.51)
   class, a full planar unit off the hole centre (the lower plate runs north-east of the hole;
   the standing eye 13.77 is the proven flat-ray launch window [L the round-6 ledger]). The
   whole west strip (x 112.9-113.6, z −179.0..−179.5) re-bridges to the rim standing; a spot
   off the hole centre may hold feet 13.086 standing — judge the launch line and the reply
   crossing from there on its own run.
2. **The crouch-window plan** — stand only for the launches. The t49 launch fired AFTER the
   approach burst ended (f25813 vs f25801), so a crouched hold through the bursts is
   shape-compatible; the stand lockout is 60 ticks [M Player_ChangeSubState 0x190630:48] and
   the dive's launchable band closes f25830, so the walk+stand+converge must fit ~30-70 ticks.
3. **A live check for the machine's reply cadence** (carried from round 11) — the reply
   arrived ~74 ticks after the launch in round 11; provoked or scheduled? That decides whether
   a delayed launch dodges the reply entirely.

## Live checks for the main agent

- The machine's reply cadence (unchanged from round 11): after Bond's rocket launch (any),
  when does the Hind spawn its w62 reply — immediately (provoked) or on a fixed phase? A
  gdbwatch on the missile spawn during a dive-window launch answers it.
- The terminal hurt volume's bounds: the zaps hit feet 13.47 at (113.0-113.1, −179.0..−179.3)
  and (111.99, −179.51)-class spots read zap-free crouched — a fieldprobe of the volume's
  actual lo/hi (and whether ANY standing spot inside the cabin clears it) would settle lever 1
  before the next run.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed from this sandbox (ntfy.sh unreachable — the same failure as rounds
5-11; each attempt at 2026-11-20 ~00:43-00:45 UTC):

1. curl: `curl: (28) Connection timed out after 20002 milliseconds` (rc 28)
2. python3 urllib: killed by the outer 25 s timeout (rc 124) — no response within the 20 s
   urlopen timeout, no traceback, nothing written
3. wget: killed by the outer 25 s timeout mid-connect (rc 124, quiet mode, no output)

The notification text that could not be delivered:
`retune-nb-08-12: rocket 1 landed 101.0 in the dive window (first machine damage; was
0.0/598), the round-11 reply wall is down; still red at t26276 to the hurt-volume zaps;
whole binary 6/0/1`
