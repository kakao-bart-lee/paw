package dev.paw.android.presentation.auth

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AssistChip
import androidx.compose.material3.AssistChipDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import dev.paw.android.PawTestTags
import dev.paw.android.presentation.bootstrap.BootstrapUiState
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.components.AuthField
import dev.paw.android.presentation.components.EditorialPanel
import dev.paw.android.presentation.components.MetadataLine
import dev.paw.android.presentation.components.PawPrimaryButton
import dev.paw.android.presentation.components.PawSecondaryButton
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface3
import dev.paw.android.runtime.PawAndroidConfig
import uniffi.paw_core.AuthStepView

@Composable
fun AuthStepPanel(uiState: BootstrapUiState, viewModel: BootstrapViewModel) {
    val authVm = viewModel.authViewModel
    when (uiState.preview.auth.step) {
        AuthStepView.AUTH_METHOD_SELECT -> {
            AuthSectionIntro(
                title = "전화번호로 시작하기",
                description = "기존 사용자도 같은 OTP 흐름으로 바로 로그인할 수 있습니다. Android에서는 이 흐름을 먼저 안정화합니다.",
            )
            PawPrimaryButton(
                onClick = authVm::showPhoneOtp,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_CONTINUE_PHONE),
            ) {
                Text("전화번호로 계속")
            }
        }
        AuthStepView.PHONE_INPUT -> {
            AuthSectionIntro(
                title = "번호 확인",
                description = "국가 코드를 생략하면 한국 번호(+82)로 자동 보정합니다. 예: 01012341234",
            )
            AuthField(
                label = "전화번호",
                value = uiState.phoneInput,
                onValueChange = authVm::onPhoneChanged,
                keyboardType = KeyboardType.Phone,
                testTag = PawTestTags.AUTH_PHONE_INPUT,
            )
            PawPrimaryButton(
                onClick = authVm::requestOtp,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_REQUEST_OTP),
            ) {
                Text("OTP 요청")
            }
        }
        AuthStepView.OTP_VERIFY -> {
            AuthSectionIntro(
                title = "인증번호 입력",
                description = "개발 서버에서는 고정 OTP 137900을 사용할 수 있습니다.",
            )
            EditorialPanel(
                title = "Developer shortcut",
                subtitle = "fixed OTP for local bootstrap only",
                modifier = Modifier.padding(top = 12.dp),
            ) {
                AssistChip(
                    onClick = authVm::useDebugOtp,
                    label = { Text("개발용 OTP ${PawAndroidConfig.debugFixedOtp}") },
                    shape = RoundedCornerShape(6.dp),
                    colors = AssistChipDefaults.assistChipColors(
                        containerColor = PawSurface3,
                        labelColor = PawStrongText,
                    ),
                    border = AssistChipDefaults.assistChipBorder(
                        enabled = true,
                        borderColor = PawOutline,
                    ),
                )
            }
            AuthField(
                label = "OTP 코드",
                value = uiState.otpInput,
                onValueChange = authVm::onOtpChanged,
                keyboardType = KeyboardType.NumberPassword,
                testTag = PawTestTags.AUTH_OTP_INPUT,
            )
            FlowRow(
                modifier = Modifier.padding(top = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                PawPrimaryButton(
                    onClick = authVm::verifyOtp,
                    modifier = Modifier.testTag(PawTestTags.AUTH_VERIFY_OTP),
                ) {
                    Text("OTP 확인")
                }
            }
        }
        AuthStepView.DEVICE_NAME -> {
            AuthSectionIntro(
                title = "디바이스 등록",
                description = "이 기기 이름으로 세션을 등록하고 다음 단계로 진행합니다.",
            )
            EditorialPanel(
                title = "Session restore",
                subtitle = "기기 키와 이름을 함께 저장해 다음 실행에서 바로 복구합니다.",
                modifier = Modifier.padding(top = 12.dp),
            ) {
                MetadataLine("device keys", if (uiState.preview.deviceKeyReady) "ready" else "missing")
                MetadataLine("staged phone", uiState.preview.auth.phone.ifBlank { "(pending)" })
            }
            AuthField("디바이스 이름", uiState.deviceNameInput, authVm::onDeviceNameChanged, testTag = PawTestTags.AUTH_DEVICE_NAME_INPUT)
            PawPrimaryButton(
                onClick = authVm::registerDevice,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_REGISTER_DEVICE),
            ) {
                Text("디바이스 등록")
            }
        }
        AuthStepView.USERNAME_SETUP -> {
            AuthSectionIntro(
                title = "프로필 마무리",
                description = "username을 설정하면 검색/프로필 링크에 사용됩니다. 지금은 건너뛰고 나중에 설정할 수도 있습니다.",
            )
            AuthField("username", uiState.usernameInput, authVm::onUsernameChanged, testTag = PawTestTags.AUTH_USERNAME_INPUT)
            EditorialPanel(
                title = "Search visibility",
                subtitle = "전화번호 기반 검색 허용 여부를 여기서 정합니다.",
                modifier = Modifier.padding(top = 12.dp),
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("전화번호 검색 허용", color = PawMutedText)
                    Switch(checked = uiState.discoverableByPhone, onCheckedChange = authVm::onDiscoverableChanged)
                }
            }
            FlowRow(modifier = Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                PawPrimaryButton(onClick = authVm::completeUsernameSetup, modifier = Modifier.testTag(PawTestTags.AUTH_COMPLETE_USERNAME)) {
                    Text("완료")
                }
                PawSecondaryButton(onClick = authVm::skipUsernameSetup, modifier = Modifier.testTag(PawTestTags.AUTH_SKIP_USERNAME)) {
                    Text("건너뛰기")
                }
            }
        }
        AuthStepView.AUTHENTICATED -> {
            AuthSectionIntro(
                title = "로그인 완료",
                description = "이제 대화 목록과 채팅 런타임을 사용할 수 있습니다.",
            )
            MetadataLine("username", uiState.preview.auth.username.ifBlank { "(unset)" })
            MetadataLine("device", uiState.preview.auth.deviceName.ifBlank { uiState.deviceNameInput })
        }
    }
}

@Composable
fun AuthProgressSummary(step: AuthStepView) {
    val label = when (step) {
        AuthStepView.AUTH_METHOD_SELECT -> "1. 로그인 방식 선택"
        AuthStepView.PHONE_INPUT -> "2. 전화번호 입력"
        AuthStepView.OTP_VERIFY -> "3. OTP 확인"
        AuthStepView.DEVICE_NAME -> "4. 디바이스 등록"
        AuthStepView.USERNAME_SETUP -> "5. username 설정"
        AuthStepView.AUTHENTICATED -> "완료 · 채팅 진입 가능"
    }

    Text(
        text = label,
        modifier = Modifier.testTag(PawTestTags.AUTH_STEP_VALUE),
        style = MaterialTheme.typography.titleMedium,
        color = PawStrongText,
    )
}

@Composable
fun AuthStepChip(label: String, testTag: String, selected: Boolean, onClick: () -> Unit) {
    FilterChip(
        selected = selected,
        onClick = onClick,
        modifier = Modifier.testTag(testTag),
        label = { Text(label) },
        shape = RoundedCornerShape(6.dp),
    )
}

@Composable
private fun AuthSectionIntro(
    title: String,
    description: String,
) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(title, style = MaterialTheme.typography.titleLarge, color = PawStrongText)
        Text(description, style = MaterialTheme.typography.bodyMedium, color = PawMutedText)
    }
}
