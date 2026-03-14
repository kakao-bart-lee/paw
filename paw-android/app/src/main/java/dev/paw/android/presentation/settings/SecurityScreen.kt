package dev.paw.android.presentation.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Key
import androidx.compose.material.icons.filled.Laptop
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material.icons.filled.Smartphone
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import dev.paw.android.presentation.theme.PawAmber
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawDestructive
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawSecure
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import dev.paw.android.presentation.theme.PawSurface3

private data class DeviceSession(
    val id: String, val name: String, val type: String,
    val current: Boolean, val lastActive: String, val location: String,
)

private val devices = listOf(
    DeviceSession("1", "Android Emulator", "mobile", true, "현재 활성", "서울, 대한민국"),
    DeviceSession("2", "MacBook Pro", "desktop", false, "2시간 전", "서울, 대한민국"),
    DeviceSession("3", "iPad Air", "mobile", false, "3일 전", "부산, 대한민국"),
)

@Composable
fun SecurityScreen(navController: NavController) {
    Column(
        modifier = Modifier.fillMaxSize().background(PawBackground).statusBarsPadding(),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            IconButton(onClick = { navController.popBackStack() }) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "뒤로", tint = PawStrongText)
            }
            Text("보안", style = MaterialTheme.typography.titleLarge, color = PawStrongText)
        }

        Column(
            modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            // E2EE status
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(PawSecure.copy(alpha = 0.1f), RoundedCornerShape(16.dp))
                    .padding(16.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Box(
                    modifier = Modifier.size(40.dp).background(PawSecure.copy(alpha = 0.2f), RoundedCornerShape(12.dp)),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(Icons.Filled.Shield, null, tint = PawSecure, modifier = Modifier.size(20.dp))
                }
                Column(modifier = Modifier.weight(1f)) {
                    Text("종단간 암호화 활성", style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                    Text(
                        "모든 메시지와 통화가 암호화되어 보호됩니다. Paw를 포함한 어떤 제3자도 내용을 확인할 수 없습니다.",
                        style = MaterialTheme.typography.bodySmall,
                        color = PawMutedText,
                        modifier = Modifier.padding(top = 4.dp),
                    )
                }
            }

            // Security settings
            Text("보안 설정", style = MaterialTheme.typography.labelSmall, color = PawMutedText)
            Column(
                modifier = Modifier.background(PawSurface1, RoundedCornerShape(16.dp)),
            ) {
                SettingToggleRow(Icons.Filled.Lock, "앱 잠금", "Face ID / 비밀번호로 앱 보호", false)
                Box(modifier = Modifier.fillMaxWidth().padding(start = 68.dp).size(0.5.dp).background(PawOutline))
                SettingToggleRow(Icons.Filled.Key, "2단계 인증", "추가 보안 계층 활성화", true)
            }

            // Active sessions
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text("활성 세션", style = MaterialTheme.typography.labelSmall, color = PawMutedText)
                Text("${devices.size}개 기기", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
            }

            devices.forEach { device ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(PawSurface1, RoundedCornerShape(16.dp))
                        .padding(16.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Box(
                        modifier = Modifier
                            .size(40.dp)
                            .background(if (device.current) PawSecure.copy(alpha = 0.2f) else PawSurface3, RoundedCornerShape(12.dp)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            if (device.type == "mobile") Icons.Filled.Smartphone else Icons.Filled.Laptop,
                            null,
                            tint = if (device.current) PawSecure else PawStrongText,
                            modifier = Modifier.size(20.dp),
                        )
                    }
                    Column(modifier = Modifier.weight(1f)) {
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                            Text(device.name, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                            if (device.current) {
                                Box(modifier = Modifier.background(PawSecure.copy(alpha = 0.1f), RoundedCornerShape(999.dp)).padding(horizontal = 8.dp, vertical = 2.dp)) {
                                    Text("현재 기기", style = MaterialTheme.typography.labelSmall, color = PawSecure)
                                }
                            }
                        }
                        Text(device.location, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                        Text(device.lastActive, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                    }
                    if (!device.current) {
                        IconButton(onClick = {}) {
                            Icon(Icons.Filled.Delete, null, tint = PawDestructive, modifier = Modifier.size(18.dp))
                        }
                    }
                }
            }

            // Key verification
            Text("키 검증", style = MaterialTheme.typography.labelSmall, color = PawMutedText)
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(PawSurface1, RoundedCornerShape(16.dp))
                    .padding(16.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Box(modifier = Modifier.size(40.dp).background(PawAmber.copy(alpha = 0.1f), RoundedCornerShape(12.dp)), contentAlignment = Alignment.Center) {
                    Icon(Icons.Filled.Key, null, tint = PawAmber, modifier = Modifier.size(20.dp))
                }
                Column {
                    Text("보안 키 확인", style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                    Text("대화 상대의 암호화 키를 직접 확인", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                }
            }

            // Danger zone
            Text("위험 영역", style = MaterialTheme.typography.labelSmall, color = PawDestructive)
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(PawDestructive.copy(alpha = 0.05f), RoundedCornerShape(16.dp))
                    .padding(16.dp),
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Icon(Icons.Filled.Warning, null, tint = PawDestructive, modifier = Modifier.size(20.dp))
                    Column {
                        Text("모든 세션 종료", style = MaterialTheme.typography.titleMedium, color = PawStrongText)
                        Text("현재 기기를 제외한 모든 기기에서 로그아웃됩니다", style = MaterialTheme.typography.bodySmall, color = PawMutedText)
                    }
                }
                Spacer(Modifier.height(16.dp))
                OutlinedButton(
                    onClick = {},
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    colors = ButtonDefaults.outlinedButtonColors(contentColor = PawDestructive),
                ) {
                    Text("다른 모든 세션 종료")
                }
            }

            Spacer(Modifier.height(16.dp))
        }
    }
}

@Composable
private fun SettingToggleRow(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    title: String,
    subtitle: String,
    checked: Boolean,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(16.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(modifier = Modifier.size(40.dp).background(PawSurface3, RoundedCornerShape(12.dp)), contentAlignment = Alignment.Center) {
            Icon(icon, null, tint = PawStrongText, modifier = Modifier.size(20.dp))
        }
        Column(modifier = Modifier.weight(1f)) {
            Text(title, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
            Text(subtitle, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
        }
        Switch(checked = checked, onCheckedChange = {}, colors = SwitchDefaults.colors(checkedTrackColor = PawPrimary))
    }
}
