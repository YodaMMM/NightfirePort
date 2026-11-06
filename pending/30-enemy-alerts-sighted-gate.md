# Sound alerts: the sighted gate on the fall-through

`enemy_alerts` lacks AlertSound's sighted gate for the SOUND fall-through [M 0x177800:48-50], while the ALARM path keeps its own gates [M 0x177c18 case 0x1e]. Make the fold message-aware, fail-first test, run every live binary that pins hearing (grep msg::SOUND and alert_pending in crates/nf-view/tests).
