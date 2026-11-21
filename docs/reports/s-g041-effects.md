# s/g041-effects: G-041 remainder (bullet impacts)

**Moves the shared random stream:** every bullet impact sound now enters the drones' hearing
list, so guards near a hit gain alertness and the AI's later draws shift. Keep this branch out
of single merges and take it with a stream batch and one re-pin.

## What changed

1. **Bullet impacts are heard** (`crates/nf-game/src/drone.rs`, `Drones::play_impact` /
   `play_impact_on`). `Effect_Bullet` 0x1ce8a8:86-90 plays `EffectInfo[mat]+0x14` through
   `Sound_Play3D(100, -1, -1, id, hit, 0, 0, 0)`. That call takes `SFXOutputData[id]`'s inner and
   outer radii and its weight [M 0x1dd548:27-52, :96-98], and nothing scales the weight down after
   it. `Effect_Body` does scale its own sound down, by 0.5 [M 0x1cf400:9-16]. `Sound_Alertness`
   0x1ddc48 sums these nodes for every drone. [D] All 21 impact rows weigh 20 to 50, with outer
   radii of 15 to 30 u, and none is looped. Before this change, the port sent impact sounds to the
   audio pool only. Now all five impact sites also push the hearing entry: Bond's world hit, the
   glass sheets, the guards' pass contacts, the guards' stop hits and their pierce hops.
   Player-visible effect: a round that hits a wall near an unaware guard now draws his attention,
   as it does in the game.
2. **Glass debris direction** (`crates/nf-game/src/effect.rs`). On material 0x10,
   `Effect_Create` 0x1ce688:68-72 negates `HITDATA+0x10` in place and only restores it at :90-94.
   The emitter (:80-82) and `Debris_Create` 0x1aaa00:40-49's `half · normal` velocity term both
   read the negated normal, so the shards carry on through the pane. The port negated only the
   basis, so its shards flew back toward the shooter. The draws and their order are unchanged.
3. **Debris-count gate.** The count draw now depends only on the +0x82 enable byte
   [M 0x1ce688:84-85]. A mesh of 0 now stops only the per-piece draws [M 0x1aaa00:19]. No shipped
   row has enable set with mesh 0 [D], so behaviour on disc data does not change.
4. Docs. The atlas listed some things as unported that are already ported:
   `Effect_Body`, `Effect_Player`, `Effect_Laser`, the trail smoke, `Bullet_DoTrails`, decals on
   moving hosts, and the penetration roll (the last one in the effect.rs header). These are
   corrected in docs/atlas/effect.md, docs/GAPS.md G-041 and the effect.rs module comment.

## Tests (fail first, then pass)

- `nf-game` lib `effect::tests::glass_debris_flies_through_the_pane`: fails on the old code
  (z speed +0.0517 where −0.0517 was expected) and passes on the new. All 12 `effect::` lib tests
  pass.
- `nf-view --test g041_impact_heard_live` (07000005, the P2K into the wall behind the grapple
  spot): fails on the old code (no hearing entry) and passes on the new.
- Neighbour binaries, green with this change: dead_pane_live 3/3, debris_fade_draw_live 2/2,
  fix_flesh_impacts_live 4/4, fix_wallhug_prompt_live 2/2, glass_pass_every_level_live 3 ok
  (3 ignored), impact_remainder_live 3/3.
- **Re-pins needed (red with this change, green on master, checked on both):**
  - `impacts_live::forty_rounds_default_caps_at_32_and_keeps_64_holds_the_rest`: the 40 shots
    into the 05 start wall now alert five guards (alertness 1.0). They shoot Bond during the
    260-tick drain (health 64), so their own shells sit in the shared casing pool (ejected 40 → 42,
    two live at the end). Casing behaviour itself is unchanged. The test's quiet-start setup has
    to hold the guards off or count only Bond's shells.
  - `walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked` and
    `..._walked_real_damage`: the walk fails at "the south strip 38.5" with Bond disabled
    (knocked down). The guards now hear the route's shots and react differently, so the walk
    needs re-tuning in the stream batch.

## Open / unproven

- **MP gate (recorded only, not changed).** `Effect_Create` only dispatches `Effect_Bullet`,
  `Player`, `Laser` and `Body` when `MPSettings+0x180 == 0` [M 0x1ce688:90]. So in a multiplayer
  match the game makes no bullet hole, no gas puff and no impact sound; only the emitter and the
  debris still run. The port's MP path (`nf-view/src/mpfx.rs` and the remote cosmetic bullets)
  makes all three. This is a visible MP change and needs a two-instance check, so it is left open.
- **Draw pass (visual, not changed).** Decals draw in the Effect layer (pass 5, fog 0), while the
  game draws them in pass 0, fogged. This differs only under heavy fog at distance.
- Not live-checked: how strongly the extra hearing changes a guard's reaction in play. The weight
  and radii come from the table, and the per-drone fold is the existing `Sound_Alertness` port.
