//! sleeper-push: 07000014's drop-slot ambush — Bond dropped onto the patrolling sleeper drone
//! ends BESIDE it, not riding it [M Collide_Jointy 0x1ec180:229-281 via LadderCtx::bot_capsules;
//! the SP asymmetric rule: the class-3 player resolves against the class-2 bot's capsule with the
//! full overlap (MPSettings+0x384 == 0 [M C_SBNFMAP_Handler 0x20b8d8:26], fVar12 = 1.0) and the
//! bot never resolves against the player (the class-3-other return [M 0x1ec180:231-233])].

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_drop_onto_the_sleeper_ends_beside_the_drone_not_on_top() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    // Boot past the intro gate so the drones tick (the ambush bot walks from the boot).
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..3000 {
            sim.step_bond(nf_game::player::Input::default(), false, false, false);
            if !sim.disabled { break; }
        }
        for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    }
    let sleeper = {
        let sim = scene.sim.as_ref().unwrap();
        let i = sim.drones.list.iter().position(|d| d.skin == 0x0500_0090)
            .expect("the sleeper drone #270 (skin 0x05000090) patrols the slot [D]");
        (i, sim.drones.list[i].pos)
    };
    // The drop: Bond enters the slot lane above the sleeper's LIVE spot (the real drop lands
    // x 40.9-41.0 over his patrol line; the teleport stands in for the lid-open drop here —
    // the route test carries the lid legs).
    let live = scene.sim.as_ref().unwrap().drones.list[sleeper.0].pos;
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [live[0], 2.6, live[2]];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = 1.5707964;
    }
    // 180 ticks of the real input path, steering east (the walker's exit direction).
    for _ in 0..180 {
        scene.sim.as_mut().unwrap().step_bond(
            nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    let dp = sim.drones.list[sleeper.0].pos;
    let dxz = ((p[0] - dp[0]).powi(2) + (p[2] - dp[2]).powi(2)).sqrt();
    // The spine-column rule [M 0x1ec180:248-252]: a riding capsule holds |dxz| under the radius
    // sum's XZ projection (r 0.55 + 0.4) while tangent ABOVE the bot (its capsule top 0.8, arms
    // pos ± up·r [M NDrone2_Collision 0x140e90:91-121]); beside it, Bond's root rests on the
    // slot floor (−0.6 [D probe14_mouth4]) at crouched-root 0.05 or stands above it.
    assert!(dxz > 0.95,
        "Bond must NOT ride the sleeper's spine column after the drop (the Jointy push slides \
         him off the wandering carrier [M 0x1ec180:229-281]): bond {p:?} drone {dp:?} dxz {dxz:.3}");
    assert!(p[1] < 1.2,
        "Bond must be DOWN on the slot floor, not tangent above the bot: bond {p:?} drone {dp:?}");
    eprintln!("[sleeper] bond {p:?} drone {dp:?} dxz {dxz:.3} — beside, on the slot floor");
}
