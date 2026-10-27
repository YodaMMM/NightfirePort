# Report: cutscene-kind31-frame-2

Branch `s/cutscene-kind31-frame-2`. The per-script scope from level-14-capture-placement is replaced with the game's general rule: an unparented case-1 stream's keys (and the event-6 hand-back snapshot of its entity) land composed before the script's `Script_CameraStart` op decodes and keys-alone after it — for every kind, not just 0x0600062F's kind-0x31 streams.

## The general rule [M]

- An unparented case-1 stream **always** concats the script base si+0x00: the 10th argument to `Script_GetInterp` is `(stream+0x25 parent == -1)` stored at 0x1d9c0c [M 0x1d9a08 case-1 disasm; case 6 the same at 0x1d9a98, case 7 at 0x1d9d6c; the parent-chain ancestor calls pass 0 at 0x1d9cd4/0x1d9b30], and `Script_GetInterp`'s tail runs `Concat(si+0x00, out)` on that flag [M 0x1da838-0x1da844]. Parented streams get the base via the explicit `Concat(si, obj+0x90)` at the end of the ancestor walk [M 0x1d9d34].
- si+0x00's **value** is the lifecycle that makes world-frame vs composed temporal: `View_RotTransMatrix` of the SP placement at `Script_Load` [M 0x1d8248:148], overwritten with `Mat_IdentityT(si)` when `Script_CameraStart` (op 0x0d) decodes — binary-proven: `$a0` = si is untouched from entry to the `jal` at 0x1d916c — and **never restored** (`Script_CameraEnd` 0x1db348:9-13 doesn't touch it; the only other writers are `Script_SetPosRot`/`SP_SetPosRot` 0x1db920:10-19 and `SubScriptStart`'s zero-vector load 0x1db050:28-38). Camera streams themselves pass the flag as constant 1 [M 0x1da054/0x1da068] — which is why CameraStart zeroes the base first: camera keys are authored world-space.
- **No per-kind or per-hash branch exists in the interp path** [M `Script_CreateEntity` 0x1d8968:35-173 — the entity kind byte only selects the constructor; `Script_Interp`/`Script_GetInterp` branch only on stream type, parent byte, channels flag and si fade flags].

So the port's `keys_alone` collapses to `camera_started && parent < 0` — the 0xff-hash arm and the `nis.hashcode == 0x0600_062f` arm it replaces were class-scoped approximations of exactly this.

## The 01 intro question, resolved by the census [D]

`crates/nf-formats/tests/nis_kind31_census.rs` decodes every archive's kind-7 NIS scripts (the 079* packs — 149 scripts carry kind-0x31 streams) with the runner's timing model (ops fire at the stream clock; Wait/End hold it at their absolute time [M Script_Run 0x1d8548]) and prints per script: the first camera-op tick, each kind-0x31 stream's op tick / key window / parent, and each event-6 tick.

- **0x06000087's camera op ticks at 0** — s5 (hash 0x20006b1, keys from t0) and s11 (hash 0, no keys: first_key 3893 > last_key 3892) both sit after it. Under the rule both are keys-alone. The 01 walks still pass because the hand-back snapshot comes from s11's entity, which **never keys** — its matrix is the create-time identity (the game's `control_create_object` builds script entities at the zero vector [M 0x1d8968 → 0x1344e0 arm 1]) and the case-1 interp returns early on the empty key range, so the snapshot is unchanged by the rule; s5's wire moves to world-frame with no walk pin on it.
- 0x0600062F: camera op tick 0, marker s10 op at t1197, event 6 in s11 at t1197 — world-frame, as level-14-capture-placement already established.
- The census's verdict rows: 27 kind-0x31 streams WORLD-frame (all in scripts whose camera op ticks before them), the rest composed in scripts with no camera op (`inf`) — every script without a camera op behaves identically before and after this change.

The hash-0 kind-0x19 marker skip (level-4a-lift-4's landing-scene pin) is retained unchanged: its pin is [L], and re-deriving its scene's base state is future work.

## What changed

- `crates/nf-game/src/nis.rs`: `keys_alone = self.camera_started && self.streams[i].parent < 0`, with the rule's [M] catalog in the comment. No other code changed.
- `crates/nf-formats/tests/nis_kind31_census.rs`: the disc census (informational, runs in ~0.8 s).

## Verification

- Real-path 01/14 battery, all green with the rule: `capture_placement_07000014_live` 1/1, `probe14_capture_end` 1/1, `interact_07000001_live` 10/10, `exit_aim_01_live` 1/1, `walkreal_07000001_live` 2/2, `walkreal_07000014_live` 2/2, `walk_07000014_live` 3/3, `exit_aim_hint_live` 2/2 (+1 ignored).
- `nf-game` full: 95 binaries, **933 passed / 0 failed / 8 ignored**.
- `nf-formats`: unit tests + census green (bigf's `disc_golden` failure is one of the 3 allowed on the slim ISO).
- Cutscene-sensitive `nf-view` chunk via `tools/test/suite.py` (114 binaries: nis/cutscene/capture/exit_aim/interact/walkreal/walk/lift/marker/scene/sp names): **373 passed / 0 failed / 18 ignored** across the two runs.

## What's left

- `walkreal_07000008_live` times out at the gondola-briefcase leg (450 s cap; no completed baseline run exists in this sandbox's logs, so pre-existing walk length vs a rule stall is undetermined). Next session: run it at the runner's default 600 s; if still stuck, bisect against master — 07000008's kind-0x31 scripts have no camera op [D census], so any stall would come from a non-0x31 unparented stream in a camera script.
- The census covers kind-0x31 streams; a follow-up could extend the same verdict table to every unparented entity stream in camera scripts (the rule applies to all kinds).

## Live checks for the main agent

1. 07000001 intro: after the intro cut-scene ends, Bond should stand where the game puts him (the port hands him back at the scene's stream-11 snapshot — create-time identity per the game's zero-vector entity creation). A fresh-boot [L] of Bond's position at intro end would pin the hand-back independently of the port.
2. 07000001 intro s5 (hash 0x20006b1, a wire/rope object): its world-frame position should visually match the disc's key coordinates (the camera frame), not offset by the intro SP's placement base.
3. 07000014 capture: unchanged from level-14-capture-placement (Bond re-enabled at the t1197 marker (-55.4948, 1.5464, 42.0171)).

## Usage

not visible to me
