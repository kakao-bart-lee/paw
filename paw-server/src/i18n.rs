use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::auth::AppState;
use crate::db::DbPool;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestLocale(pub String);

pub fn normalize_locale(input: &str) -> Option<String> {
    let normalized = input.trim().replace('_', "-");
    if normalized.is_empty() || normalized.len() > 35 {
        return None;
    }

    let parts: Vec<&str> = normalized.split('-').collect();
    if parts.is_empty() || parts.len() > 8 {
        return None;
    }

    let language = parts[0];
    if !(2..=3).contains(&language.len()) || !language.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }

    let mut canonical = Vec::with_capacity(parts.len());
    canonical.push(language.to_ascii_lowercase());

    for part in parts.iter().skip(1) {
        if part.is_empty() || part.len() > 8 || !part.chars().all(|ch| ch.is_ascii_alphanumeric()) {
            return None;
        }

        let value = if part.len() == 4 && part.chars().all(|ch| ch.is_ascii_alphabetic()) {
            let mut chars = part.chars();
            let first = chars.next()?.to_ascii_uppercase();
            let rest = chars.as_str().to_ascii_lowercase();
            format!("{first}{rest}")
        } else if (part.len() == 2 && part.chars().all(|ch| ch.is_ascii_alphabetic()))
            || (part.len() == 3 && part.chars().all(|ch| ch.is_ascii_digit()))
        {
            part.to_ascii_uppercase()
        } else {
            part.to_ascii_lowercase()
        };

        canonical.push(value);
    }

    Some(canonical.join("-"))
}

pub fn resolve_locale(accept_language: Option<&str>, default_locale: &str) -> String {
    let mut best: Option<(f32, usize, String)> = None;

    if let Some(header_value) = accept_language {
        for (index, item) in header_value.split(',').enumerate() {
            let raw = item.trim();
            if raw.is_empty() {
                continue;
            }

            let mut parts = raw.split(';');
            let Some(tag) = parts.next() else {
                continue;
            };

            let Some(locale) = normalize_locale(tag) else {
                continue;
            };

            let mut quality = 1.0_f32;
            let mut invalid_quality = false;

            for param in parts {
                let param = param.trim();
                if let Some(value) = param.strip_prefix("q=") {
                    match value.parse::<f32>() {
                        Ok(parsed) if (0.0..=1.0).contains(&parsed) => quality = parsed,
                        _ => {
                            invalid_quality = true;
                            break;
                        }
                    }
                }
            }

            if invalid_quality || quality <= 0.0 {
                continue;
            }

            match &best {
                Some((best_quality, best_index, _))
                    if *best_quality > quality
                        || (*best_quality == quality && *best_index < index) => {}
                _ => best = Some((quality, index, locale)),
            }
        }
    }

    best.map(|(_, _, locale)| locale)
        .unwrap_or_else(|| default_locale.to_string())
}

fn primary_language(locale: &str) -> &str {
    locale.split('-').next().unwrap_or(locale)
}

pub fn localized_message<'a>(code: &'a str, locale: &str, fallback: &'a str) -> &'a str {
    match (primary_language(locale), code) {
        ("ko", "missing_authorization") => "Authorization 헤더가 필요합니다",
        ("ko", "invalid_authorization") => "Authorization 헤더가 올바르지 않습니다",
        ("ko", "invalid_token") => "액세스 토큰이 올바르지 않습니다",
        ("ko", "invalid_phone") => "전화번호는 E.164 형식이어야 합니다",
        ("ko", "otp_store_failed") => "OTP를 생성하지 못했습니다",
        ("ko", "invalid_code_format") => "OTP는 6자리 숫자여야 합니다",
        ("ko", "transaction_start_failed") => "OTP 검증을 시작하지 못했습니다",
        ("ko", "otp_query_failed") => "OTP를 검증하지 못했습니다",
        ("ko", "invalid_otp") => "OTP가 올바르지 않거나 만료되었습니다",
        ("ko", "otp_already_used") => "이미 사용된 OTP입니다",
        ("ko", "user_upsert_failed") => "사용자 정보를 생성하지 못했습니다",
        ("ko", "transaction_commit_failed") => "OTP 검증을 완료하지 못했습니다",
        ("ko", "session_issue_failed") => "세션 토큰을 생성하지 못했습니다",
        ("ko", "invalid_device_name") => "기기 이름이 필요합니다",
        ("ko", "invalid_session_token") => "세션 토큰이 올바르지 않습니다",
        ("ko", "invalid_device_key") => "기기 키가 올바르지 않습니다",
        ("ko", "device_register_failed") => "기기를 등록하지 못했습니다",
        ("ko", "access_issue_failed") => "액세스 토큰을 발급하지 못했습니다",
        ("ko", "refresh_issue_failed") => "리프레시 토큰을 발급하지 못했습니다",
        ("ko", "invalid_refresh_token") => "리프레시 토큰이 올바르지 않습니다",
        ("ko", "session_revoke_failed") => "세션을 무효화하지 못했습니다",
        ("ko", "user_not_found") => "사용자를 찾을 수 없습니다",
        ("ko", "query_failed") => "조회에 실패했습니다",
        ("ko", "invalid_content") => "메시지 내용이 필요합니다",
        ("ko", "invalid_format") => "format은 markdown 또는 plain 이어야 합니다",
        ("ko", "spam_detected") => "금지된 내용이 포함된 메시지입니다",
        ("ko", "message_lookup_failed") => "메시지를 전송하지 못했습니다",
        ("ko", "message_send_failed") => "메시지를 전송하지 못했습니다",
        ("ko", "forbidden") => "이 작업을 수행할 권한이 없습니다",
        ("ko", "conversation_not_found") => "대화를 찾을 수 없습니다",
        ("ko", "message_not_found") => "메시지를 찾을 수 없습니다",
        ("ko", "membership_check_failed") => "대화 멤버십을 확인하지 못했습니다",
        ("ko", "message_history_failed") => "메시지 기록을 불러오지 못했습니다",
        ("ko", "conversation_list_failed") => "대화 목록을 불러오지 못했습니다",
        ("ko", "too_many_members") => "대화 최대 인원 수를 초과했습니다",
        ("ko", "conversation_create_failed") => "대화를 생성하지 못했습니다",
        ("ko", "already_member") => "이미 대화에 참여 중인 사용자입니다",
        ("ko", "member_not_found") => "대화 멤버를 찾을 수 없습니다",
        ("ko", "cannot_remove_last_owner") => "마지막 owner는 제거할 수 없습니다",
        ("ko", "invalid_group_name") => "그룹 이름이 필요합니다",
        ("ko", "invalid_channel_name") => "채널 이름이 필요합니다",
        ("ko", "channel_create_failed") => "채널을 생성하지 못했습니다",
        ("ko", "channel_list_failed") => "채널 목록을 불러오지 못했습니다",
        ("ko", "channel_not_found") => "채널을 찾을 수 없습니다",
        ("ko", "channel_permission_failed") => "채널 권한을 확인하지 못했습니다",
        ("ko", "channel_access_failed") => "채널 접근 권한을 확인하지 못했습니다",
        ("ko", "channel_message_history_failed") => "채널 메시지 기록을 불러오지 못했습니다",
        ("ko", "invalid_base64") => "하나 이상의 키가 올바른 base64 형식이 아닙니다",
        ("ko", "bundle_not_found") => "프리키 번들을 찾을 수 없습니다",
        ("ko", "keys_upload_failed") => "키 번들을 업로드하지 못했습니다",
        ("ko", "keys_fetch_failed") => "키 번들을 불러오지 못했습니다",
        ("ko", "backup_initiate_failed") => "백업을 시작하지 못했습니다",
        ("ko", "backup_list_failed") => "백업 목록을 불러오지 못했습니다",
        ("ko", "backup_not_found") => "백업을 찾을 수 없습니다",
        ("ko", "backup_restore_failed") => "백업을 복원하지 못했습니다",
        ("ko", "backup_delete_failed") => "백업을 삭제하지 못했습니다",
        ("ko", "backup_settings_failed") => "백업 설정을 불러오지 못했습니다",
        ("ko", "backup_settings_update_failed") => "백업 설정을 업데이트하지 못했습니다",
        ("ko", "missing_device_id") => "액세스 토큰에 device_id가 포함되어야 합니다",
        ("ko", "invalid_push_token") => "푸시 토큰은 비어 있을 수 없습니다",
        ("ko", "push_register_failed") => "푸시 토큰을 등록하지 못했습니다",
        ("ko", "push_unregister_failed") => "푸시 토큰을 해제하지 못했습니다",
        ("ko", "invalid_duration") => "duration_minutes는 0보다 커야 합니다",
        ("ko", "invalid_mute_request") => "duration_minutes 또는 forever 중 하나를 지정해야 합니다",
        ("ko", "mute_failed") => "대화를 음소거하지 못했습니다",
        ("ko", "unmute_failed") => "대화 음소거를 해제하지 못했습니다",
        ("ko", "invalid_frame") => "올바르지 않은 WebSocket 프레임입니다",
        ("ko", "message_too_large") => "WebSocket 메시지가 허용 크기를 초과했습니다",
        ("ko", "rate_limit_exceeded") => "요청이 너무 많습니다. 잠시 후 다시 시도해주세요.",
        ("ko", "too_many_connections") => "동시 WebSocket 연결 수를 초과했습니다",
        ("ko", "not_found") => "대상을 찾을 수 없습니다",
        ("ko", "unsupported_protocol_version") => "지원하지 않는 프로토콜 버전입니다",
        ("ko", "invalid_agent_token") => "에이전트 토큰이 올바르지 않습니다",
        ("ko", "agent_missing_token") => "에이전트 토큰이 필요합니다",
        ("ko", "nats_unavailable") => "에이전트 게이트웨이에 NATS가 필요합니다",
        ("ko", "subscribe_failed") => "에이전트 구독을 시작하지 못했습니다",
        ("ko", "invalid_target_type") => "target_type은 message, user, agent 중 하나여야 합니다",
        ("ko", "invalid_reason") => "신고 사유가 필요합니다",
        ("ko", "report_failed") => "신고를 접수하지 못했습니다",
        ("ko", "cannot_block_self") => "자기 자신을 차단할 수 없습니다",
        ("ko", "block_failed") => "사용자를 차단하지 못했습니다",
        ("ko", "unblock_failed") => "사용자 차단을 해제하지 못했습니다",
        ("ko", "block_list_failed") => "차단 사용자 목록을 불러오지 못했습니다",
        ("ko", "admin_check_failed") => "관리자 권한을 확인하지 못했습니다",
        ("ko", "suspend_failed") => "사용자를 정지하지 못했습니다",
        ("ko", "unsuspend_failed") => "사용자 정지를 해제하지 못했습니다",
        ("ko", "reports_failed") => "신고 목록을 불러오지 못했습니다",
        ("ko", "device_not_found") => "기기를 찾을 수 없습니다",
        ("ko", "delete_failed") => "삭제하지 못했습니다",
        ("ko", "registration_failed") => "에이전트를 등록하지 못했습니다",
        ("ko", "internal_error") => "내부 오류가 발생했습니다",
        ("ko", "agent_not_found") => "에이전트를 찾을 수 없습니다",
        ("ko", "agent_not_found_or_not_owner") => "에이전트를 찾을 수 없거나 소유자가 아닙니다",
        ("ko", "rotate_failed") => "에이전트 키를 교체하지 못했습니다",
        ("ko", "already_invited") => "에이전트가 이미 초대되어 있습니다",
        ("ko", "invite_failed") => "에이전트를 초대하지 못했습니다",
        ("ko", "remove_failed") => "에이전트를 제거하지 못했습니다",
        ("ko", "search_failed") => "마켓플레이스를 검색하지 못했습니다",
        ("ko", "already_installed") => "에이전트가 이미 설치되어 있습니다",
        ("ko", "install_failed") => "에이전트를 설치하지 못했습니다",
        ("ko", "not_installed") => "설치되지 않은 에이전트입니다",
        ("ko", "uninstall_failed") => "에이전트를 제거하지 못했습니다",
        ("ko", "invalid_manifest") => "매니페스트가 올바르지 않습니다",
        ("ko", "publish_failed") => "에이전트를 공개하지 못했습니다",
        ("ko", "invalid_preferred_locale") => {
            "선호 언어는 ko-KR 또는 en-US 같은 BCP-47 형식이어야 합니다"
        }
        ("ko", "invalid_username") => "사용자명은 3~20자의 소문자, 숫자, 밑줄만 사용할 수 있습니다",
        ("ko", "username_taken") => "이미 사용 중인 사용자명입니다",
        ("ko", "update_failed") => "프로필을 업데이트하지 못했습니다",
        ("ko", "missing_search_param") => "username 또는 phone 중 하나를 제공해야 합니다",
        _ => fallback,
    }
}

pub fn error_response(
    status: StatusCode,
    code: &str,
    locale: &str,
    fallback: &str,
) -> (StatusCode, Json<Value>) {
    error_response_with_details(status, code, locale, None, fallback)
}

pub fn error_response_with_request_id(
    status: StatusCode,
    code: &str,
    locale: &str,
    request_id: Option<&str>,
    fallback: &str,
) -> (StatusCode, Json<Value>) {
    error_response_with_details_and_request_id(status, code, locale, None, request_id, fallback)
}

pub fn error_response_with_details(
    status: StatusCode,
    code: &str,
    locale: &str,
    details: Option<&str>,
    fallback: &str,
) -> (StatusCode, Json<Value>) {
    error_response_with_details_and_request_id(status, code, locale, details, None, fallback)
}

pub fn error_response_with_details_and_request_id(
    status: StatusCode,
    code: &str,
    locale: &str,
    details: Option<&str>,
    request_id: Option<&str>,
    fallback: &str,
) -> (StatusCode, Json<Value>) {
    let localized = localized_message(code, locale, fallback);
    let mut payload = Map::new();
    payload.insert("error".to_string(), Value::String(code.to_string()));
    payload.insert("message".to_string(), Value::String(localized.to_string()));

    if let Some(request_id) = request_id {
        payload.insert(
            "request_id".to_string(),
            Value::String(request_id.to_string()),
        );
    }

    if let Some(details) = details
        .map(str::trim)
        .filter(|details| !details.is_empty() && *details != localized)
    {
        payload.insert("details".to_string(), Value::String(details.to_string()));
    }

    (status, Json(Value::Object(payload)))
}

pub async fn lookup_user_preferred_locale(
    db: &DbPool,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let locale =
        sqlx::query_scalar::<_, String>("SELECT preferred_locale FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(db.as_ref())
            .await?;

    Ok(locale.and_then(|value| normalize_locale(&value)))
}

pub async fn locale_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let resolved = resolve_locale(
        request
            .headers()
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|value| value.to_str().ok()),
        &state.default_locale,
    );

    request
        .extensions_mut()
        .insert(RequestLocale(resolved.clone()));
    tracing::debug!(locale = %resolved, path = %request.uri().path(), "resolved request locale");

    let mut response = next.run(request).await;
    let final_locale = response
        .extensions()
        .get::<RequestLocale>()
        .map(|locale| locale.0.clone())
        .unwrap_or(resolved);

    if let Ok(header_value) = HeaderValue::from_str(&final_locale) {
        response
            .headers_mut()
            .insert(header::CONTENT_LANGUAGE, header_value);
    }
    response
        .headers_mut()
        .append(header::VARY, HeaderValue::from_static("accept-language"));

    response
}

#[cfg(test)]
mod tests {
    use super::{
        error_response, error_response_with_details, localized_message, normalize_locale,
        resolve_locale,
    };
    use axum::{http::StatusCode, Json};
    use serde_json::json;

    #[test]
    fn normalize_locale_canonicalizes_common_tags() {
        assert_eq!(normalize_locale(" ko_kr "), Some("ko-KR".to_string()));
        assert_eq!(normalize_locale("en-us"), Some("en-US".to_string()));
        assert_eq!(
            normalize_locale("zh-hant-tw"),
            Some("zh-Hant-TW".to_string())
        );
    }

    #[test]
    fn normalize_locale_rejects_invalid_tags() {
        assert_eq!(normalize_locale(""), None);
        assert_eq!(normalize_locale("k"), None);
        assert_eq!(normalize_locale("ko-한글"), None);
        assert_eq!(normalize_locale("ko--KR"), None);
    }

    #[test]
    fn resolve_locale_prefers_first_valid_header_entry() {
        let resolved = resolve_locale(Some("fr-CA,ko-KR;q=0.8,en;q=0.5"), "ko-KR");
        assert_eq!(resolved, "fr-CA");
    }

    #[test]
    fn resolve_locale_respects_quality_values() {
        let resolved = resolve_locale(Some("en;q=0.5, ko-KR;q=1.0"), "fr-FR");
        assert_eq!(resolved, "ko-KR");
    }

    #[test]
    fn resolve_locale_ignores_zero_quality_and_invalid_q_values() {
        let resolved = resolve_locale(Some("en;q=0, ko;q=bogus, ja;q=0.8"), "fr-FR");
        assert_eq!(resolved, "ja");
    }

    #[test]
    fn resolve_locale_falls_back_to_default() {
        assert_eq!(resolve_locale(None, "ko-KR"), "ko-KR");
        assert_eq!(resolve_locale(Some("***"), "ko-KR"), "ko-KR");
    }

    #[test]
    fn localized_message_returns_korean_when_available() {
        assert_eq!(
            localized_message("user_not_found", "ko-KR", "User not found"),
            "사용자를 찾을 수 없습니다"
        );
        assert_eq!(
            localized_message("user_not_found", "en-US", "User not found"),
            "User not found"
        );
    }

    #[test]
    fn error_response_uses_localized_message() {
        let (_, Json(payload)) = error_response(
            StatusCode::BAD_REQUEST,
            "invalid_phone",
            "ko-KR",
            "Phone number must be E.164 format",
        );
        assert_eq!(payload["error"], json!("invalid_phone"));
        assert_eq!(
            payload["message"],
            json!("전화번호는 E.164 형식이어야 합니다")
        );
    }

    #[test]
    fn error_response_preserves_dynamic_details() {
        let (_, Json(payload)) = error_response_with_details(
            StatusCode::BAD_REQUEST,
            "invalid_manifest",
            "ko-KR",
            Some("manifest.version must be >= 1"),
            "manifest.version must be >= 1",
        );
        assert_eq!(payload["message"], json!("매니페스트가 올바르지 않습니다"));
        assert_eq!(payload["details"], json!("manifest.version must be >= 1"));
    }
}
