# WORKLOG

## cutscene-handback-anim-prev

Branch s/cutscene-handback-anim-prev (from origin/master @ fcf6f334). Task: the game's event-6 hand-back admits a previous stream of kind 1 OR 2 and snapshots that object's +0x90 either way [M Script_EventHandler 0x1d94c0:73-84, :125-134]; an anim object's +0x90 is its live matrix [M AnimObjectUpdate 0x1befd8:290 -> Control_GetObjMatrix 0x134210 View_RotTransMatrix(obj+0x50 rot, obj+0x30 pos)]. The port's event-6 arm reads only `Obj::Entity`, so four scenes hand Bond back where he was disabled instead: 07000002's 0x0600007f and 0x06000084, 07000003's 0x06000080, 0700004a's 0x060006d3 [D]. Carry the anim object's world matrix on `Obj::Anim`, let the event-6 arm snapshot it, and make those four census rows hard assertions (each on a floor [D]). The census stays green.
