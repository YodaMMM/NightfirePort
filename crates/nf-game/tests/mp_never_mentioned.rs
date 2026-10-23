//! 868 unported-multiplayer-1 — the 8 never-mentioned `MP_*` functions. Three are ported here
//! (`MP_HasEsponage` 0x18b468, `MP_ObjTeamHasGEObj` 0x18b4a0, `MP_isObjectOnTeam` 0x18b720),
//! five are cited as the game's own empty stubs / the port's existing equivalents —
//! docs/systems/mp-never-mentioned.md is the evidence table. Tests run the real state chain
//! (`MpMatch::init` → `start` → `register_mp_object` → `create_mode_objects` → `touch_*`), the
//! same chain the game's HUD pane and tick read; every assertion carries its address and each
//! pairs with a sister that fails on the wrong rule.

use nf_game::drone::Rng;
use nf_game::mp::hud::{MpHudSnapshot, MpScorePane};
use nf_game::mp::{scenario, MpMatch, MpSettings};

const DT: f32 = 1.0 / 60.0;

fn make(sc: u32) -> MpMatch {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.scenario = sc;
    s.gametype_mask = sc;
    s.human_count = 2;
    s.player_count = 2;
    s.total_players = 2;
    s.frag_limit = 100;
    s.time_limit = 600;
    let mut m = MpMatch::init(s);
    m.start(&mut Rng::new(1));
    m
}

fn join(m: &mut MpMatch, slot: usize, team: u32) {
    m.settings.players[slot].team = team;
    m.set_slot(slot, [10.0 + 10.0 * slot as f32, 0.0, 0.0]);
}

fn pane(m: &MpMatch, viewer: usize) -> MpScorePane {
    let snap = MpHudSnapshot::build(m, viewer, true);
    MpScorePane::build(&snap, m.settings.team_game, m.settings.bit30, m.settings.players[viewer].team)
}

/// `MP_HasEsponage` 0x18b468 — the `BluePrint` ext's `+0x1c` carrier pointer [M]. Real path:
/// the blueprint registers from the map (type 5), `create_mode_objects` picks it
/// (`MP_Start`'s Rand_Rand [M 0x183a50]), the touch sets the carrier exactly like
/// `MP_BluePrintUpdate`'s pick rule [M 0x1882d8:124-145]. Sister: the other slot and the
/// pre-touch state both answer false.
#[test]
fn esponage_getter_follows_the_carrier() {
    let mut rng = Rng::new(7);
    let mut m = make(scenario::BLUEPRINT);
    join(&mut m, 0, 0);
    join(&mut m, 1, 1);
    // the blueprint item (type 5) and the two delivery bases (type 4, team word 0/1)
    m.register_mp_object([40.0, 1.0, 40.0], [0.0, 0.0, 1.0], 5, [5, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], 4, [4, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([41.0, 1.0, 41.0], [0.0, 0.0, 1.0], 4, [4, 1, 0, 0, 0, 0, 0], 0);
    m.create_mode_objects(&mut rng);
    assert!(m.objects_rt.blueprint.is_some(), "the item pool was non-empty");
    // before the pick nobody carries it [M 0x18b468: BluePrint != 0 but +0x1c == 0]
    assert!(!m.mp_has_esponage(0), "free blueprint: no carrier");
    assert!(!m.mp_has_esponage(1), "sister: the other slot either");
    m.touch_blueprint(0);
    assert_eq!(m.objects_rt.blueprint.as_ref().unwrap().carrier, Some(0));
    assert!(m.mp_has_esponage(0), "the carrier answers true [M 0x18b468]");
    assert!(!m.mp_has_esponage(1), "sister: only the carrier's pointer compares equal");
}

/// The pane arm `MP_HasEsponage` feeds (`HUD_MPUpdatePane` 0x19f0a0:75-82): the carrier's
/// sprite 1 re-textures to 0x300018a at layer 0x1e [M :80-81]; the create's else-arm
/// (everything but 0x400/0x60000008) moves sprites 1/2 down 0x18 and no x [M 0x19ed10:72-76].
/// Sister: the non-carrier viewer's pane has no 0x300018a icon at all.
#[test]
fn esponage_pane_icon_shows_for_the_carrier_only() {
    let mut rng = Rng::new(7);
    let mut m = make(scenario::BLUEPRINT);
    join(&mut m, 0, 0);
    join(&mut m, 1, 1);
    m.register_mp_object([40.0, 1.0, 40.0], [0.0, 0.0, 1.0], 5, [5, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], 4, [4, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([41.0, 1.0, 41.0], [0.0, 0.0, 1.0], 4, [4, 1, 0, 0, 0, 0, 0], 0);
    m.create_mode_objects(&mut rng);
    m.touch_blueprint(1);
    let p1 = pane(&m, 1);
    let bp: Vec<_> = p1.icons.iter().filter(|i| i.tex == 0x0300_018a).collect();
    assert_eq!(bp.len(), 1, "the carrier's pane shows the blueprint glyph [M 0x19f0a0:80-81]");
    assert_eq!(bp[0].sprite, 1, "sprite 1 re-textures [M 0x19f0a0:80]");
    assert_eq!(bp[0].layer, 0x1e, "layer 0x1e [M 0x19f0a0:80]");
    assert_eq!(bp[0].rgba, 0x7f7f_7fff, "the template colour stands [D ACTION.ELF 0x2be888 = 7f 7f 7f ff]");
    // [S] unverified, out of pass/fail: the icon's absolute rect (16, 376). The game's chain is
    // the pane origin + the MPScoreInfo record (sprite x/y at record+0xc/+0xe [M Sprite_Create2
    // 0x1de3d8]) + the create's −0x18 else-arm [M 0x19ed10:72-76], but the disc record 1 reads
    // (0, 448) and the origin path (HUD_ValidateXY 0x19a250) did not resolve to (16, 392) here.
    // A live read of the on-screen rect settles it (open).
    let _ = (bp[0].x, bp[0].y);
    let p0 = pane(&m, 0);
    assert!(
        p0.icons.iter().all(|i| i.tex != 0x0300_018a),
        "sister: the non-carrier's pane never re-textures [M 0x19f0a0:77-78 early return]"
    );
}

/// `MP_ObjTeamHasGEObj` 0x18b4a0 — item k's `GoldenEye` ext `+0x1c` carrier's team word ==
/// the viewer's [M]. Real path: the two items register as types 6/7, the touches pick them
/// (`MP_GoldenEyeUpdate`'s pick [M 0x187568]). Sister: the opposing team and the uncarried
/// item both answer false; the raw word compare has no != 2 guard — pinned by an unjoined
/// slot's default word never matching here.
#[test]
fn obj_team_has_ge_obj_follows_the_carriers_team() {
    let mut rng = Rng::new(7);
    let mut m = make(scenario::GOLDENEYE);
    join(&mut m, 0, 0);
    join(&mut m, 1, 1);
    m.register_mp_object([40.0, 1.0, 40.0], [0.0, 0.0, 1.0], 6, [6, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([44.0, 1.0, 44.0], [0.0, 0.0, 1.0], 7, [7, 0, 0, 0, 0, 0, 0], 0);
    m.create_mode_objects(&mut rng);
    assert!(m.objects_rt.ge[0].is_some() && m.objects_rt.ge[1].is_some());
    // nothing carried yet: no team has an item [M : item +0x1c == 0 → false]
    assert!(!m.mp_obj_team_has_ge_obj(0, 0) && !m.mp_obj_team_has_ge_obj(0, 1));
    m.touch_ge_item(0, 0); // team 0 takes item 0
    assert!(m.mp_obj_team_has_ge_obj(0, 0), "the holder's team reads true [M]");
    assert!(!m.mp_obj_team_has_ge_obj(1, 0), "sister: the other team's word differs");
    assert!(!m.mp_obj_team_has_ge_obj(0, 1), "sister: the uncarried item stays false");
    m.touch_ge_item(1, 1); // team 1 takes item 1
    assert!(m.mp_obj_team_has_ge_obj(1, 1));
    assert!(m.mp_obj_team_has_ge_obj(0, 0) && !m.mp_obj_team_has_ge_obj(0, 1), "viewer 0's split");
    assert!(m.mp_obj_team_has_ge_obj(1, 1) && !m.mp_obj_team_has_ge_obj(1, 0), "viewer 1's split");
}

/// The pane arms `MP_ObjTeamHasGEObj` feed (0x19f0a0:115-127): item 0 → sprite 1 → 0x300018b,
/// item 1 → sprite 2 → 0x300018c [M :119/:124], both at the shared else-arm rect (the shipped
/// layout stacks them when one team holds both — the create moves sprites 1/2 by the same
/// −0x18, no x [M 0x19ed10:72-76]). Sister: nothing carried → no icons.
#[test]
fn goldeneye_pane_icons_track_the_team_items() {
    let mut rng = Rng::new(7);
    let mut m = make(scenario::GOLDENEYE);
    join(&mut m, 0, 0);
    join(&mut m, 1, 1);
    m.register_mp_object([40.0, 1.0, 40.0], [0.0, 0.0, 1.0], 6, [6, 0, 0, 0, 0, 0, 0], 0);
    m.register_mp_object([44.0, 1.0, 44.0], [0.0, 0.0, 1.0], 7, [7, 0, 0, 0, 0, 0, 0], 0);
    m.create_mode_objects(&mut rng);
    let p = pane(&m, 0);
    assert!(
        p.icons.iter().all(|i| i.tex != 0x0300_018b && i.tex != 0x0300_018c),
        "sister: an empty board draws no GE glyphs [M 0x19f0a0:115-127 all-false]"
    );
    m.touch_ge_item(0, 0);
    m.touch_ge_item(1, 0); // team 0 holds both — the stacked-rect case
    let p = pane(&m, 0);
    let ge: Vec<_> = p.icons.iter().filter(|i| i.tex == 0x0300_018b || i.tex == 0x0300_018c).collect();
    assert_eq!(ge.len(), 2, "both held items draw [M 0x19f0a0:115-126]");
    assert_eq!((ge[0].sprite, ge[0].tex), (1, 0x0300_018b), "item 0 rides sprite 1 [M 0x19f0a0:119]");
    assert_eq!((ge[1].sprite, ge[1].tex), (2, 0x0300_018c), "item 1 rides sprite 2 [M 0x19f0a0:124]");
    assert_eq!((ge[0].x, ge[0].y), (ge[1].x, ge[1].y), "the create's else-arm moves sprites 1/2 by the same −0x18 and no x, so one team holding both stacks them [M 0x19ed10:72-76]");
    // [S] unverified, out of pass/fail: the absolute stacked rect (16, 376) — the same open
    // origin-path link as esponage_pane_icon_shows_for_the_carrier_only (live check pending).
    let _ = (ge[0].x, ge[0].y, ge[1].x, ge[1].y);
    let p1 = pane(&m, 1);
    assert!(
        p1.icons.iter().all(|i| i.tex != 0x0300_018b && i.tex != 0x0300_018c),
        "sister: team 1's viewer holds nothing [M]"
    );
}

/// `MP_isObjectOnTeam` 0x18b720 — the gate (`MPSettings+0x18c` team game or `+0x1a4 == 0x400`
/// Assassin), the word compare, and the word-2 sentinel [M]. Real path: a team match's roster
/// (`MPSettings + slot·0x30 + 0x20` = the join team). Sisters: the other team, the 2 sentinel,
/// and the FFA gate (word equal but gate closed).
#[test]
fn is_object_on_team_gates_and_sentinel() {
    let mut rng = Rng::new(7);
    let mut m = make(scenario::CAPTURE_THE_FLAG); // team game: (0x20000004 >> 29) & 1 == 1
    join(&mut m, 0, 0);
    join(&mut m, 1, 1);
    assert_eq!(m.settings.team_game, 1, "CTF is a team game [M 0x183438:41]");
    assert!(m.mp_is_object_on_team(0, 0), "own team [M 0x18b720 word compare]");
    assert!(!m.mp_is_object_on_team(0, 1), "sister: the other team's word differs");
    m.settings.players[2].team = 2; // the unassigned sentinel word
    assert!(!m.mp_is_object_on_team(2, 2), "the 2 sentinel answers false [M 0x18b720 == 2 arm]");
    let mut ffa = make(scenario::DEATHMATCH); // team_game 0, not assassin
    join(&mut ffa, 0, 0);
    assert!(!ffa.mp_is_object_on_team(0, 0), "sister: the FFA gate is closed even on equal words");
    ffa.settings.scenario = scenario::ASSASSIN;
    assert!(ffa.mp_is_object_on_team(0, 0), "the 0x400 scenario reopens the gate [M 0x18b720 gate]");
}

/// The reachability pins of the non-ported five, as far as a headless rig can pin them:
/// the KOTH scorer gates on the `MP_isPlayerOrBotDead` stand-in (a dead slot drips no score
/// [M 0x186ae0:80-86 + 0x18abc0]), and the restart stub leaves the round objects to the next
/// `create_mode_objects` (the game's `MP_ObjectDelete` slot is empty [M 0x18b2c0]).
#[test]
fn koth_dead_slots_drip_nothing_and_restart_recreates() {
    let mut rng = Rng::new(7);
    let mut m = make(scenario::KOTH_FFA);
    join(&mut m, 0, 0);
    join(&mut m, 1, 0);
    // the hill: register type 9 and let create place it
    m.register_mp_object([10.0, 1.0, 10.0], [0.0, 0.0, 1.0], 9, [9, 0, 0, 0, 0, 0, 0], 0);
    m.create_mode_objects(&mut rng);
    let on_hill = [true, true, false, false, false, false, false, false, false, false, false, false, false, false, false, false];
    for _ in 0..60 {
        m.update_objects(DT, &on_hill, &mut rng);
    }
    let alive = m.game.slots[0].score;
    assert!(alive > 0.0, "the hill drips 0.2/s [M 0x186ae0:118-121]");
    m.player_killed(0, Some(1), false, false, &mut rng);
    m.game.slots[0].score = 0.0;
    for _ in 0..60 {
        m.update_objects(DT, &on_hill, &mut rng);
    }
    assert_eq!(m.game.slots[0].score, 0.0, "sister: a dead slot drips nothing [M 0x186b44 gate]");
    assert!(m.game.slots[1].score > 0.0, "the live slot keeps dripping");
    // the restart path: the objects are re-picked, the stub delete does nothing to them [M]
    m.restart_scenario(&mut rng);
    m.objects_rt.hill = None;
    m.create_mode_objects(&mut rng);
    assert!(m.objects_rt.hill.is_some(), "the next create re-files the hill (the empty 0x18b2c0 slot)");
}
