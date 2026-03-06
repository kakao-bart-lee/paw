use crate::db::DbPool;
use crate::moderation::models::*;
use anyhow::Context;
use uuid::Uuid;

pub fn content_matches_keywords(content: &str, keywords: &[String]) -> bool {
    let lower = content.to_ascii_lowercase();
    keywords
        .iter()
        .any(|kw| lower.contains(&kw.to_ascii_lowercase()))
}

pub async fn check_spam(content: &str, db: &DbPool) -> bool {
    let keywords: Vec<String> = match sqlx::query_scalar::<_, String>(
        "SELECT keyword FROM spam_keywords",
    )
    .fetch_all(db.as_ref())
    .await
    {
        Ok(kws) => kws,
        Err(_) => return false,
    };

    content_matches_keywords(content, &keywords)
}

pub async fn create_report(
    db: &DbPool,
    reporter_id: Uuid,
    target_type: &str,
    target_id: Uuid,
    reason: &str,
) -> anyhow::Result<Report> {
    sqlx::query_as::<_, Report>(
        "INSERT INTO reports (reporter_id, target_type, target_id, reason)
         VALUES ($1, $2, $3, $4)
         RETURNING id, reporter_id, target_type, target_id, reason, status, created_at",
    )
    .bind(reporter_id)
    .bind(target_type)
    .bind(target_id)
    .bind(reason)
    .fetch_one(db.as_ref())
    .await
    .context("create report")
}

pub async fn block_user(
    db: &DbPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "INSERT INTO user_blocks (blocker_id, blocked_id)
         VALUES ($1, $2)
         ON CONFLICT (blocker_id, blocked_id) DO NOTHING",
    )
    .bind(blocker_id)
    .bind(blocked_id)
    .execute(db.as_ref())
    .await
    .context("block user")?;

    Ok(result.rows_affected() > 0)
}

pub async fn unblock_user(
    db: &DbPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "DELETE FROM user_blocks WHERE blocker_id = $1 AND blocked_id = $2",
    )
    .bind(blocker_id)
    .bind(blocked_id)
    .execute(db.as_ref())
    .await
    .context("unblock user")?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_blocked_users(
    db: &DbPool,
    blocker_id: Uuid,
) -> anyhow::Result<Vec<BlockedUserItem>> {
    sqlx::query_as::<_, BlockedUserItem>(
        "SELECT blocked_id, created_at FROM user_blocks WHERE blocker_id = $1 ORDER BY created_at DESC",
    )
    .bind(blocker_id)
    .fetch_all(db.as_ref())
    .await
    .context("list blocked users")
}

pub async fn suspend_user(
    db: &DbPool,
    user_id: Uuid,
    suspended_until: chrono::DateTime<chrono::Utc>,
    reason: Option<&str>,
    suspended_by: Uuid,
) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "INSERT INTO user_suspensions (user_id, suspended_until, reason, suspended_by)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id) DO UPDATE SET suspended_until = $2, reason = $3, suspended_by = $4",
    )
    .bind(user_id)
    .bind(suspended_until)
    .bind(reason)
    .bind(suspended_by)
    .execute(db.as_ref())
    .await
    .context("suspend user")?;

    Ok(result.rows_affected() > 0)
}

pub async fn unsuspend_user(db: &DbPool, user_id: Uuid) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM user_suspensions WHERE user_id = $1")
        .bind(user_id)
        .execute(db.as_ref())
        .await
        .context("unsuspend user")?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_pending_reports(db: &DbPool) -> anyhow::Result<Vec<Report>> {
    sqlx::query_as::<_, Report>(
        "SELECT id, reporter_id, target_type, target_id, reason, status, created_at
         FROM reports WHERE status = 'pending'
         ORDER BY created_at ASC",
    )
    .fetch_all(db.as_ref())
    .await
    .context("list pending reports")
}

pub async fn is_admin(db: &DbPool, user_id: Uuid) -> anyhow::Result<bool> {
    sqlx::query_scalar::<_, bool>(
        "SELECT COALESCE(is_admin, false) FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(db.as_ref())
    .await
    .context("check admin status")
    .map(|opt| opt.unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spam_check_matches_keyword() {
        let keywords = vec!["viagra".into(), "casino".into()];
        assert!(content_matches_keywords("Buy VIAGRA now!", &keywords));
        assert!(content_matches_keywords("Visit our casino", &keywords));
    }

    #[test]
    fn spam_check_case_insensitive() {
        let keywords = vec!["SPAM".into()];
        assert!(content_matches_keywords("this is spam", &keywords));
        assert!(content_matches_keywords("this is Spam", &keywords));
        assert!(content_matches_keywords("this is SPAM", &keywords));
    }

    #[test]
    fn spam_check_no_match() {
        let keywords = vec!["viagra".into(), "casino".into()];
        assert!(!content_matches_keywords("Hello, how are you?", &keywords));
    }

    #[test]
    fn spam_check_empty_keywords() {
        let keywords: Vec<String> = vec![];
        assert!(!content_matches_keywords("anything at all", &keywords));
    }

    #[test]
    fn spam_check_empty_content() {
        let keywords = vec!["spam".into()];
        assert!(!content_matches_keywords("", &keywords));
    }
}
