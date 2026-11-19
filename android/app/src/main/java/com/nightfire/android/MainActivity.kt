package com.nightfire.android

import android.os.Bundle
import android.view.Choreographer
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Gamepad
import androidx.compose.material.icons.filled.MyLocation
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import java.io.File
import java.util.zip.ZipInputStream
import kotlin.math.hypot

class MainActivity : ComponentActivity(), Choreographer.FrameCallback {
    private var surfaceReady = false
    private var initialized = false

    // Shared input state (written from UI / controller, read on Choreographer frame)
    @Volatile private var moveX = 0f
    @Volatile private var moveY = 0f
    @Volatile private var lookDx = 0f
    @Volatile private var lookDy = 0f
    @Volatile private var touchButtons = 0
    @Volatile private var padButtons = 0

    private var statusText by mutableStateOf("Booting...")
    private var activeLevel by mutableStateOf("07000048.bin")
    private var inMenu by mutableStateOf(true)
    private var showSheet by mutableStateOf(false)
    private var showTouchControls by mutableStateOf(true)
    private var overlayAlpha by mutableFloatStateOf(0.80f)
    private var lookSensitivity by mutableFloatStateOf(1.2f)
    private var toggleAds by mutableStateOf(true)
    private var adsToggledOn by mutableStateOf(false)
    private var dataReady by mutableStateOf(false)
    private var dataPath by mutableStateOf("")

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        hideSystemBars()

        val extDir = File(getExternalFilesDir(null), "carved")
        val intDir = File(filesDir, "carved")
        val obbFile = obbDir?.listFiles()?.firstOrNull { it.name.endsWith(".obb") && it.isFile }
        val obbStamp = obbFile?.let { "${it.length()}:${it.lastModified()}" } ?: ""
        val stampFile = File(intDir, ".obb_stamp")
        val intUpToDate = File(intDir, "ACTION.ELF").isFile &&
            (obbStamp.isEmpty() || (stampFile.isFile && stampFile.readText().trim() == obbStamp))

        val chosen = when {
            extDir.isDirectory && File(extDir, "ACTION.ELF").isFile -> extDir
            intUpToDate -> intDir
            else -> null
        }

        if (chosen != null) {
            dataPath = chosen.absolutePath
            statusText = "Loading Front End..."
            Thread {
                val ok = NativeBridge.nativeInit(chosen.absolutePath)
                runOnUiThread {
                    initialized = ok
                    dataReady = ok
                    inMenu = if (ok) NativeBridge.nativeIsInMenu() else true
                    statusText = if (ok) NativeBridge.nativeGetStatus() else "nativeInit failed"
                }
            }.start()
        } else if (obbFile != null) {
            dataPath = intDir.absolutePath
            statusText = "Unpacking OBB (${obbFile.name})..."
            Thread {
                unpackObb(obbFile, intDir)
                if (obbStamp.isNotEmpty()) stampFile.writeText(obbStamp)
                runOnUiThread {
                    if (File(intDir, "ACTION.ELF").isFile) {
                        dataPath = intDir.absolutePath
                        initialized = NativeBridge.nativeInit(intDir.absolutePath)
                        dataReady = initialized
                        inMenu = NativeBridge.nativeIsInMenu()
                        statusText = NativeBridge.nativeGetStatus()
                    } else {
                        statusText = "OBB unpack failed: ${obbFile.name}"
                    }
                }
            }.start()
        } else {
            dataPath = intDir.absolutePath
            statusText = "No carved data or OBB at ${obbDir?.absolutePath}"
        }

        setContent {
            NightfireTheme {
                NightfireHarnessScreen()
            }
        }
    }

    private fun unpackObb(obbFile: File, destDir: File) {
        destDir.mkdirs()
        val canonicalDest = destDir.canonicalPath + File.separator
        ZipInputStream(obbFile.inputStream().buffered()).use { zis ->
            var entry = zis.nextEntry
            while (entry != null) {
                val outFile = File(destDir, entry.name)
                if (!outFile.canonicalPath.startsWith(canonicalDest)) {
                    throw SecurityException("Zip path traversal: ${entry.name}")
                }
                if (entry.isDirectory) {
                    outFile.mkdirs()
                } else {
                    outFile.parentFile?.mkdirs()
                    outFile.outputStream().buffered().use { out -> zis.copyTo(out) }
                }
                zis.closeEntry()
                entry = zis.nextEntry
            }
        }
    }

    private fun hideSystemBars() {
        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).apply {
            hide(WindowInsetsCompat.Type.systemBars())
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
    }

    override fun onResume() {
        super.onResume()
        Choreographer.getInstance().postFrameCallback(this)
    }

    override fun onPause() {
        super.onPause()
        Choreographer.getInstance().removeFrameCallback(this)
    }

    override fun doFrame(frameTimeNanos: Long) {
        if (initialized && surfaceReady) {
            val dx = lookDx * lookSensitivity
            val dy = lookDy * lookSensitivity
            lookDx = 0f
            lookDy = 0f
            val aimBit = if (toggleAds && adsToggledOn) NativeBridge.BTN_AIM else 0
            NativeBridge.nativeSetInput(moveX, moveY, dx, dy, touchButtons or padButtons or aimBit)
            NativeBridge.nativeStepAndRender()
            if ((frameTimeNanos / 16_000_000L) % 10L == 0L) {
                inMenu = NativeBridge.nativeIsInMenu()
                statusText = NativeBridge.nativeGetStatus()
            }
        }
        Choreographer.getInstance().postFrameCallback(this)
    }

    override fun onGenericMotionEvent(event: MotionEvent): Boolean {
        if (event.source and InputDevice.SOURCE_JOYSTICK == InputDevice.SOURCE_JOYSTICK &&
            event.action == MotionEvent.ACTION_MOVE
        ) {
            val lx = deadzone(event.getAxisValue(MotionEvent.AXIS_X))
            val ly = deadzone(event.getAxisValue(MotionEvent.AXIS_Y))
            val rx = deadzone(event.getAxisValue(MotionEvent.AXIS_Z))
            val ry = deadzone(event.getAxisValue(MotionEvent.AXIS_RZ))
            moveX = lx
            moveY = -ly
            lookDx += rx * 14f
            lookDy += ry * 14f
            val rt = event.getAxisValue(MotionEvent.AXIS_RTRIGGER)
            val lt = event.getAxisValue(MotionEvent.AXIS_LTRIGGER)
            padButtons = if (rt > 0.35f) padButtons or NativeBridge.BTN_FIRE else padButtons and NativeBridge.BTN_FIRE.inv()
            padButtons = if (lt > 0.35f) padButtons or NativeBridge.BTN_AIM else padButtons and NativeBridge.BTN_AIM.inv()
            return true
        }
        return super.onGenericMotionEvent(event)
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        val bit = keyToButton(keyCode)
        if (bit != 0) {
            padButtons = padButtons or bit
            return true
        }
        return super.onKeyDown(keyCode, event)
    }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        val bit = keyToButton(keyCode)
        if (bit != 0) {
            padButtons = padButtons and bit.inv()
            return true
        }
        return super.onKeyUp(keyCode, event)
    }

    private fun keyToButton(keyCode: Int): Int = when (keyCode) {
        KeyEvent.KEYCODE_BUTTON_R2, KeyEvent.KEYCODE_BUTTON_R1 -> NativeBridge.BTN_FIRE
        KeyEvent.KEYCODE_BUTTON_L2, KeyEvent.KEYCODE_BUTTON_L1 -> NativeBridge.BTN_AIM
        KeyEvent.KEYCODE_BUTTON_A -> NativeBridge.BTN_JUMP
        KeyEvent.KEYCODE_BUTTON_B -> NativeBridge.BTN_CROUCH
        KeyEvent.KEYCODE_BUTTON_X -> NativeBridge.BTN_RELOAD
        KeyEvent.KEYCODE_BUTTON_Y -> NativeBridge.BTN_USE
        KeyEvent.KEYCODE_DPAD_RIGHT -> NativeBridge.BTN_WEAP_NEXT
        KeyEvent.KEYCODE_DPAD_LEFT -> NativeBridge.BTN_WEAP_PREV
        else -> 0
    }

    private fun deadzone(v: Float, dz: Float = 0.15f): Float =
        if (kotlin.math.abs(v) < dz) 0f else v

    @OptIn(ExperimentalMaterial3Api::class)
    @Composable
    private fun NightfireHarnessScreen() {
        Box(modifier = Modifier.fillMaxSize().background(Color.Black)) {
            if (dataReady) {
                AndroidView(
                    modifier = Modifier.fillMaxSize(),
                    factory = { ctx ->
                        SurfaceView(ctx).apply {
                            holder.addCallback(object : SurfaceHolder.Callback {
                                override fun surfaceCreated(holder: SurfaceHolder) {
                                    surfaceReady = NativeBridge.nativeSurfaceCreated(
                                        holder.surface,
                                        width.coerceAtLeast(1),
                                        height.coerceAtLeast(1)
                                    )
                                }

                                override fun surfaceChanged(
                                    holder: SurfaceHolder,
                                    format: Int,
                                    width: Int,
                                    height: Int
                                ) {
                                    NativeBridge.nativeSurfaceChanged(width, height)
                                }

                                override fun surfaceDestroyed(holder: SurfaceHolder) {
                                    surfaceReady = false
                                    NativeBridge.nativeSurfaceDestroyed()
                                }
                            })
                        }
                    }
                )
            } else {
                MissingDataCard(modifier = Modifier.align(Alignment.Center))
            }

            // Touch Controls Overlay (auto-switches between Menu & Gameplay)
            if (dataReady && showTouchControls) {
                Box(modifier = Modifier.fillMaxSize().alpha(overlayAlpha)) {
                    if (inMenu) {
                        MenuOverlayControls(modifier = Modifier.fillMaxSize())
                    } else {
                        GameplayTouchOverlay(modifier = Modifier.fillMaxSize())
                    }
                }
            }

            // Top telemetry + harness quick bar
            Row(
                modifier = Modifier
                    .align(Alignment.TopCenter)
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 10.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Surface(
                    color = Color(0xAA0D1117),
                    shape = RoundedCornerShape(12.dp),
                    tonalElevation = 2.dp
                ) {
                    Text(
                        text = statusText,
                        color = Color(0xFFE6EDF3),
                        fontSize = 11.sp,
                        fontFamily = FontFamily.Monospace,
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp)
                    )
                }

                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    if (!inMenu) {
                        FilledTonalButton(
                            onClick = {
                                adsToggledOn = false
                                if (NativeBridge.nativeLoadLevel("07000048.bin")) {
                                    activeLevel = "07000048.bin"
                                    inMenu = true
                                }
                            },
                            colors = ButtonDefaults.filledTonalButtonColors(
                                containerColor = Color(0xBB1B222C),
                                contentColor = Color(0xFFE6EDF3)
                            ),
                            contentPadding = PaddingValues(horizontal = 12.dp, vertical = 6.dp)
                        ) {
                            Text("Menu", fontSize = 12.sp)
                        }
                    }
                    FilledTonalIconButton(
                        onClick = { showTouchControls = !showTouchControls },
                        colors = IconButtonDefaults.filledTonalIconButtonColors(
                            containerColor = Color(0xBB1B222C),
                            contentColor = Color(0xFFD4AF37)
                        )
                    ) {
                        Icon(Icons.Default.Gamepad, contentDescription = "Toggle Touch HUD")
                    }
                    FilledTonalIconButton(
                        onClick = { showSheet = true },
                        colors = IconButtonDefaults.filledTonalIconButtonColors(
                            containerColor = Color(0xBB1B222C),
                            contentColor = Color(0xFFD4AF37)
                        )
                    ) {
                        Icon(Icons.Default.Settings, contentDescription = "Harness Menu")
                    }
                }
            }

            if (showSheet) {
                ModalBottomSheet(
                    onDismissRequest = { showSheet = false },
                    containerColor = Color(0xFF141923),
                    contentColor = Color(0xFFE6EDF3)
                ) {
                    HarnessSheetContent()
                }
            }
        }
    }

    @Composable
    private fun MissingDataCard(modifier: Modifier = Modifier) {
        ElevatedCard(
            modifier = modifier.widthIn(max = 460.dp).padding(24.dp),
            colors = CardDefaults.elevatedCardColors(containerColor = Color(0xFF141923))
        ) {
            Column(
                modifier = Modifier.padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Text(
                    "007: Nightfire Port (Carved OBB)",
                    style = MaterialTheme.typography.titleMedium,
                    color = Color(0xFFD4AF37),
                    fontWeight = FontWeight.Bold
                )
                Text(
                    "Data path: $dataPath",
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    color = Color(0xFFB0BAC5)
                )
            }
        }
    }

    @Composable
    private fun MenuOverlayControls(modifier: Modifier = Modifier) {
        val haptic = LocalHapticFeedback.current
        Box(modifier = modifier.padding(horizontal = 24.dp, vertical = 22.dp)) {
            // Left D-Pad
            Column(
                modifier = Modifier.align(Alignment.BottomStart),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                MenuPadBtn("Up", Icons.Default.ArrowUpward) {
                    haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                    NativeBridge.nativeMenuAction(NativeBridge.MENU_UP)
                }
                Row(horizontalArrangement = Arrangement.spacedBy(26.dp)) {
                    MenuPadBtn("Left", Icons.AutoMirrored.Filled.ArrowBack) {
                        haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                        NativeBridge.nativeMenuAction(NativeBridge.MENU_LEFT)
                    }
                    MenuPadBtn("Right", Icons.AutoMirrored.Filled.ArrowForward) {
                        haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                        NativeBridge.nativeMenuAction(NativeBridge.MENU_RIGHT)
                    }
                }
                MenuPadBtn("Down", Icons.Default.ArrowDownward) {
                    haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                    NativeBridge.nativeMenuAction(NativeBridge.MENU_DOWN)
                }
            }

            // Right PS2-style action cluster
            Column(
                modifier = Modifier.align(Alignment.BottomEnd),
                horizontalAlignment = Alignment.End,
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedButton(
                        onClick = {
                            haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                            NativeBridge.nativeMenuAction(NativeBridge.MENU_SIDE)
                        },
                        colors = ButtonDefaults.outlinedButtonColors(contentColor = Color(0xFFB0BAC5))
                    ) {
                        Text("□ Side", fontSize = 12.sp)
                    }
                    OutlinedButton(
                        onClick = {
                            haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                            NativeBridge.nativeMenuAction(NativeBridge.MENU_EXTRA)
                        },
                        colors = ButtonDefaults.outlinedButtonColors(contentColor = Color(0xFFB0BAC5))
                    ) {
                        Text("○ Extra", fontSize = 12.sp)
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    FilledTonalButton(
                        onClick = {
                            haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                            NativeBridge.nativeMenuAction(NativeBridge.MENU_BACK)
                        },
                        colors = ButtonDefaults.filledTonalButtonColors(
                            containerColor = Color(0xCC2A1E24),
                            contentColor = Color(0xFFFF8A80)
                        )
                    ) {
                        Icon(Icons.Default.Close, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(4.dp))
                        Text("△ Back")
                    }
                    Button(
                        onClick = {
                            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
                            NativeBridge.nativeMenuAction(NativeBridge.MENU_CONFIRM)
                        },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = Color(0xFFD4AF37),
                            contentColor = Color(0xFF0D1117)
                        )
                    ) {
                        Icon(Icons.Default.Check, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(4.dp))
                        Text("✕ Select", fontWeight = FontWeight.Bold)
                    }
                }
            }
        }
    }

    @Composable
    private fun MenuPadBtn(
        label: String,
        icon: androidx.compose.ui.graphics.vector.ImageVector,
        onClick: () -> Unit
    ) {
        FilledTonalIconButton(
            onClick = onClick,
            modifier = Modifier.size(52.dp),
            colors = IconButtonDefaults.filledTonalIconButtonColors(
                containerColor = Color(0xBB1B222C),
                contentColor = Color(0xFFE6EDF3)
            )
        ) {
            Icon(icon, contentDescription = label)
        }
    }

    @Composable
    private fun GameplayTouchOverlay(modifier: Modifier = Modifier) {
        var crouchActive by remember { mutableStateOf(false) }

        Box(modifier = modifier) {
            // Right-half full look pad (underneath right-side buttons)
            Box(
                modifier = Modifier
                    .align(Alignment.CenterEnd)
                    .fillMaxHeight()
                    .fillMaxWidth(0.55f)
                    .pointerInput(Unit) {
                        detectDragGestures { change, dragAmount ->
                            change.consume()
                            lookDx += dragAmount.x
                            lookDy += dragAmount.y
                        }
                    }
            )

            // Left side: Quick Fire + Virtual Thumbstick
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(start = 24.dp, bottom = 20.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                CircleActionButton(
                    size = 54.dp,
                    label = "FIRE",
                    bg = Color(0xBBD4AF37),
                    activeBg = Color(0xFFFFD54F),
                    contentColor = Color(0xFF0D1117),
                    bit = NativeBridge.BTN_FIRE,
                    allowDragLook = false
                )
                VirtualStick(
                    onChange = { x, y ->
                        moveX = x
                        moveY = -y
                    }
                )
            }

            // Right side: Weapon cycle, utility arc, Aim + Primary Fire (supports drag-to-aim while firing!)
            Column(
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(end = 22.dp, bottom = 18.dp),
                horizontalAlignment = Alignment.End,
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                // Weapon switch pill
                Row(
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    HoldChip("◀ WEP", NativeBridge.BTN_WEAP_PREV)
                    HoldChip("WEP ▶", NativeBridge.BTN_WEAP_NEXT)
                }

                // Utility row: USE, RELOAD, CROUCH, JUMP
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    HoldChip("USE", NativeBridge.BTN_USE)
                    CircleIconHoldButton(
                        size = 46.dp,
                        icon = Icons.Default.Refresh,
                        label = "Reload",
                        bit = NativeBridge.BTN_RELOAD
                    )
                    HoldChip(
                        label = if (crouchActive) "STAND" else "CROUCH",
                        bit = NativeBridge.BTN_CROUCH,
                        highlighted = crouchActive,
                        onPressEdge = { crouchActive = !crouchActive }
                    )
                    HoldChip("JUMP", NativeBridge.BTN_JUMP)
                }

                // Primary Combat Pair: AIM (ADS) + Large FIRE (drag-to-aim enabled!)
                Row(
                    horizontalArrangement = Arrangement.spacedBy(14.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    if (toggleAds) {
                        FilledTonalIconToggleButton(
                            checked = adsToggledOn,
                            onCheckedChange = { adsToggledOn = it },
                            modifier = Modifier.size(58.dp),
                            colors = IconButtonDefaults.filledTonalIconToggleButtonColors(
                                containerColor = Color(0xBB1B222C),
                                contentColor = Color(0xFF90CAF9),
                                checkedContainerColor = Color(0xFF1E88E5),
                                checkedContentColor = Color.White
                            )
                        ) {
                            Icon(Icons.Default.MyLocation, contentDescription = "Aim")
                        }
                    } else {
                        CircleActionButton(
                            size = 58.dp,
                            label = "AIM",
                            bg = Color(0xBB1E3A5F),
                            activeBg = Color(0xFF1E88E5),
                            contentColor = Color.White,
                            bit = NativeBridge.BTN_AIM,
                            allowDragLook = true
                        )
                    }

                    CircleActionButton(
                        size = 82.dp,
                        label = "FIRE",
                        bg = Color(0xCCD4AF37),
                        activeBg = Color(0xFFFFD54F),
                        contentColor = Color(0xFF0D1117),
                        bit = NativeBridge.BTN_FIRE,
                        allowDragLook = true
                    )
                }
            }
        }
    }

    @Composable
    private fun CircleActionButton(
        size: Dp,
        label: String,
        bg: Color,
        activeBg: Color,
        contentColor: Color,
        bit: Int,
        allowDragLook: Boolean
    ) {
        var pressed by remember { mutableStateOf(false) }
        val haptic = LocalHapticFeedback.current
        Box(
            modifier = Modifier
                .size(size)
                .clip(CircleShape)
                .background(if (pressed) activeBg else bg)
                .border(2.dp, Color(0x66FFFFFF), CircleShape)
                .pointerInput(bit, allowDragLook) {
                    awaitPointerEventScope {
                        var lastPos: Offset? = null
                        while (true) {
                            val event = awaitPointerEvent()
                            val active = event.changes.firstOrNull { it.pressed }
                            val down = active != null
                            if (down && !pressed) {
                                haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                                lastPos = active?.position
                            } else if (down && allowDragLook && active != null) {
                                val prev = lastPos
                                if (prev != null) {
                                    val delta = active.position - prev
                                    lookDx += delta.x
                                    lookDy += delta.y
                                }
                                lastPos = active.position
                            } else if (!down) {
                                lastPos = null
                            }
                            pressed = down
                            touchButtons = if (down) touchButtons or bit else touchButtons and bit.inv()
                        }
                    }
                },
            contentAlignment = Alignment.Center
        ) {
            Text(label, color = contentColor, fontSize = 13.sp, fontWeight = FontWeight.ExtraBold)
        }
    }

    @Composable
    private fun CircleIconHoldButton(
        size: Dp,
        icon: androidx.compose.ui.graphics.vector.ImageVector,
        label: String,
        bit: Int
    ) {
        var pressed by remember { mutableStateOf(false) }
        Box(
            modifier = Modifier
                .size(size)
                .clip(CircleShape)
                .background(if (pressed) Color(0xFF37474F) else Color(0xBB1B222C))
                .border(1.dp, Color(0x44FFFFFF), CircleShape)
                .pointerInput(bit) {
                    awaitPointerEventScope {
                        while (true) {
                            val event = awaitPointerEvent()
                            val down = event.changes.any { it.pressed }
                            pressed = down
                            touchButtons = if (down) touchButtons or bit else touchButtons and bit.inv()
                        }
                    }
                },
            contentAlignment = Alignment.Center
        ) {
            Icon(icon, contentDescription = label, tint = Color(0xFFE6EDF3), modifier = Modifier.size(20.dp))
        }
    }

    @Composable
    private fun HoldChip(
        label: String,
        bit: Int,
        highlighted: Boolean = false,
        onPressEdge: (() -> Unit)? = null
    ) {
        var pressed by remember { mutableStateOf(false) }
        Box(
            modifier = Modifier
                .clip(RoundedCornerShape(14.dp))
                .background(
                    when {
                        pressed -> Color(0xFF455A64)
                        highlighted -> Color(0xCC2E4A62)
                        else -> Color(0xBB1B222C)
                    }
                )
                .border(1.dp, Color(0x44FFFFFF), RoundedCornerShape(14.dp))
                .pointerInput(bit) {
                    awaitPointerEventScope {
                        while (true) {
                            val event = awaitPointerEvent()
                            val down = event.changes.any { it.pressed }
                            if (down && !pressed) onPressEdge?.invoke()
                            pressed = down
                            touchButtons = if (down) touchButtons or bit else touchButtons and bit.inv()
                        }
                    }
                }
                .padding(horizontal = 12.dp, vertical = 10.dp),
            contentAlignment = Alignment.Center
        ) {
            Text(label, color = Color(0xFFE6EDF3), fontSize = 11.sp, fontWeight = FontWeight.Bold)
        }
    }

    @Composable
    private fun VirtualStick(
        modifier: Modifier = Modifier,
        onChange: (Float, Float) -> Unit
    ) {
        var knob by remember { mutableStateOf(Offset.Zero) }
        val maxRadius = 92f
        Box(
            modifier = modifier
                .size(144.dp)
                .clip(CircleShape)
                .background(Color(0x441B222C))
                .border(1.5.dp, Color(0x66D4AF37), CircleShape)
                .pointerInput(Unit) {
                    detectDragGestures(
                        onDragEnd = {
                            knob = Offset.Zero
                            onChange(0f, 0f)
                        },
                        onDragCancel = {
                            knob = Offset.Zero
                            onChange(0f, 0f)
                        },
                        onDrag = { change, dragAmount ->
                            change.consume()
                            val next = knob + dragAmount
                            val dist = hypot(next.x, next.y)
                            knob = if (dist > maxRadius) next * (maxRadius / dist) else next
                            onChange((knob.x / maxRadius).coerceIn(-1f, 1f), (knob.y / maxRadius).coerceIn(-1f, 1f))
                        }
                    )
                },
            contentAlignment = Alignment.Center
        ) {
            Canvas(modifier = Modifier.fillMaxSize()) {
                drawCircle(
                    color = Color(0x44FFFFFF),
                    radius = size.minDimension * 0.25f,
                    style = Stroke(width = 1.dp.toPx())
                )
                drawCircle(
                    color = Color(0xCCD4AF37),
                    radius = 26.dp.toPx(),
                    center = center + knob
                )
            }
        }
    }

    @Composable
    private fun HarnessSheetContent() {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 12.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text(
                "Nightfire Android Harness",
                style = MaterialTheme.typography.titleMedium,
                color = Color(0xFFD4AF37),
                fontWeight = FontWeight.Bold
            )

            val levels = remember(dataReady) {
                if (dataReady) NativeBridge.nativeListLevels().split(",").filter { it.isNotBlank() }
                else emptyList()
            }
            Text("Jump to Carved Level (${levels.size})", style = MaterialTheme.typography.labelMedium, color = Color(0xFF9AA5B1))
            LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                items(levels) { arch ->
                    val label = when (arch.lowercase()) {
                        "07000048.bin" -> "Front End"
                        "07000001.bin" -> "01 Paris Prelude"
                        "07000002.bin" -> "02 Exchange"
                        "07000005.bin" -> "05 Alpine Escape"
                        "07000006.bin" -> "06 Enemies Vanq"
                        "07000007.bin" -> "07 Double Cross"
                        "07000008.bin" -> "08 Night Shift"
                        "07000009.bin" -> "09 Chain Reaction"
                        "0700000b.bin" -> "11 Phoenix Fire"
                        "0700000d.bin" -> "13 Countdown"
                        "07000011.bin" -> "17 Deep Descent"
                        "07000014.bin" -> "20 Island Infilt"
                        "07000016.bin" -> "22 Equinox"
                        "09000001.bin" -> "DRV Paris Prelude"
                        "09000005.bin" -> "DRV Alpine Escape"
                        "09000006.bin" -> "DRV Enemies Vanq"
                        "09000002.bin" -> "DRV Deep Descent"
                        "09000003.bin" -> "DRV Island A"
                        "09000004.bin" -> "DRV Island B"
                        "09000007.bin" -> "DRV Island C"
                        "09000008.bin" -> "DRV Bonus Race"
                        else -> arch.removeSuffix(".bin")
                    }
                    FilterChip(
                        selected = activeLevel.equals(arch, ignoreCase = true),
                        onClick = {
                            adsToggledOn = false
                            if (NativeBridge.nativeLoadLevel(arch)) {
                                activeLevel = arch
                                inMenu = NativeBridge.nativeIsInMenu()
                                statusText = NativeBridge.nativeGetStatus()
                            }
                            showSheet = false
                        },
                        label = { Text(label) }
                    )
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text("Touch Controls Overlay")
                Switch(checked = showTouchControls, onCheckedChange = { showTouchControls = it })
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text("Toggle ADS (Aim)")
                Switch(checked = toggleAds, onCheckedChange = { toggleAds = it; adsToggledOn = false })
            }

            Column {
                Text("Look Sensitivity: ${"%.1f".format(lookSensitivity)}x", style = MaterialTheme.typography.bodySmall)
                Slider(
                    value = lookSensitivity,
                    onValueChange = { lookSensitivity = it },
                    valueRange = 0.4f..3.0f
                )
            }

            Column {
                Text("Overlay Opacity: ${(overlayAlpha * 100).toInt()}%", style = MaterialTheme.typography.bodySmall)
                Slider(
                    value = overlayAlpha,
                    onValueChange = { overlayAlpha = it },
                    valueRange = 0.25f..1.0f
                )
            }

            Spacer(Modifier.height(12.dp))
        }
    }
}

@Composable
private fun NightfireTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = darkColorScheme(
            primary = Color(0xFFD4AF37),
            onPrimary = Color(0xFF0D1117),
            surface = Color(0xFF141923),
            background = Color(0xFF0D1117)
        ),
        content = content
    )
}
