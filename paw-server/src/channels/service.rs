use crate::channels::models::{ChannelRecord, ChannelSummary};
use crate::db::DbPool;
use anyhow::{anyhow, Context};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelAccess {
    Owner,
    Subscriber,
    Forbidden,
    NotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscribeError {
    NotFound,
    Forbidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsubscribeError {
    NotFound,
    CannotUnsubscribeOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendPermission {
    Owner,
    NotOwner,
    NotFound,
}

pub async fn create_channel(
    pool: &DbPool,
    owner_id: Uuid,
    name: &str,
    is_public: bool,
) -> anyhow::Result<ChannelRecord> {
    let mut tx = pool
        .begin()
        .await
        .context("begin channel create transaction")?;
    let channel_id = Uuid::new_v4();

    insert_channel_conversation(&mut tx, channel_id, owner_id, name).await?;
    let channel = insert_channel(&mut tx, channel_id, owner_id, name, is_public).await?;
    insert_owner_membership(&mut tx, channel_id, owner_id).await?;

    tx.commit()
        .await
        .context("commit channel create transaction")?;
    Ok(channel)
}

pub async fn list_public_channels(
    pool: &DbPool,
    user_id: Uuid,
    query: Option<&str>,
) -> anyhow::Result<Vec<ChannelSummary>> {
    let normalized = query
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .map(ToOwned::to_owned);

    sqlx::query_as::<_, ChannelSummary>(
        "SELECT
            c.id,
            c.name,
            c.owner_id,
            c.is_public,
            c.created_at,
            EXISTS(
                SELECT 1
                FROM channel_subscriptions cs
                WHERE cs.channel_id = c.id AND cs.user_id = $1
            ) AS subscribed
         FROM channels c
         WHERE c.is_public = TRUE
           AND ($2::TEXT IS NULL OR c.name ILIKE '%' || $2 || '%')
         ORDER BY c.created_at DESC",
    )
    .bind(user_id)
    .bind(normalized)
    .fetch_all(pool.as_ref())
    .await
    .context("list public channels")
}

pub async fn subscribe(
    pool: &DbPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<bool, SubscribeError> {
    let channel =
        sqlx::query_as::<_, (Uuid, bool)>("SELECT owner_id, is_public FROM channels WHERE id = $1")
            .bind(channel_id)
            .fetch_optional(pool.as_ref())
            .await
            .map_err(|_| SubscribeError::NotFound)?;

    let Some((owner_id, is_public)) = channel else {
        return Err(SubscribeError::NotFound);
    };

    if owner_id == user_id {
        return Ok(true);
    }

    if !is_public {
        return Err(SubscribeError::Forbidden);
    }

    let mut tx = pool.begin().await.map_err(|_| SubscribeError::NotFound)?;

    sqlx::query(
        "INSERT INTO channel_subscriptions (channel_id, user_id)
         VALUES ($1, $2)
         ON CONFLICT (channel_id, user_id) DO NOTHING",
    )
    .bind(channel_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| SubscribeError::NotFound)?;

    sqlx::query(
        "INSERT INTO conversation_members (conversation_id, user_id, role)
         VALUES ($1, $2, 'member')
         ON CONFLICT (conversation_id, user_id) DO NOTHING",
    )
    .bind(channel_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| SubscribeError::NotFound)?;

    tx.commit().await.map_err(|_| SubscribeError::NotFound)?;

    Ok(true)
}

pub async fn unsubscribe(
    pool: &DbPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> Result<bool, UnsubscribeError> {
    let owner = sqlx::query_scalar::<_, Uuid>("SELECT owner_id FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|_| UnsubscribeError::NotFound)?;

    let Some(owner_id) = owner else {
        return Err(UnsubscribeError::NotFound);
    };

    if owner_id == user_id {
        return Err(UnsubscribeError::CannotUnsubscribeOwner);
    }

    let mut tx = pool.begin().await.map_err(|_| UnsubscribeError::NotFound)?;

    let deleted = sqlx::query(
        "DELETE FROM channel_subscriptions
         WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| UnsubscribeError::NotFound)?
    .rows_affected();

    sqlx::query(
        "DELETE FROM conversation_members
         WHERE conversation_id = $1 AND user_id = $2 AND role <> 'admin'",
    )
    .bind(channel_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| UnsubscribeError::NotFound)?;

    tx.commit().await.map_err(|_| UnsubscribeError::NotFound)?;

    Ok(deleted > 0)
}

pub async fn access_for_user(
    pool: &DbPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<ChannelAccess> {
    let channel =
        sqlx::query_as::<_, (Uuid, bool)>("SELECT owner_id, is_public FROM channels WHERE id = $1")
            .bind(channel_id)
            .fetch_optional(pool.as_ref())
            .await
            .context("load channel for access check")?;

    let Some((owner_id, _)) = channel else {
        return Ok(ChannelAccess::NotFound);
    };

    if owner_id == user_id {
        return Ok(ChannelAccess::Owner);
    }

    let subscribed = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1
            FROM channel_subscriptions
            WHERE channel_id = $1 AND user_id = $2
        )",
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_one(pool.as_ref())
    .await
    .context("check channel subscription")?;

    if subscribed {
        Ok(ChannelAccess::Subscriber)
    } else {
        Ok(ChannelAccess::Forbidden)
    }
}

pub async fn send_permission(
    pool: &DbPool,
    channel_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<SendPermission> {
    let owner = sqlx::query_scalar::<_, Uuid>("SELECT owner_id FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_optional(pool.as_ref())
        .await
        .context("load channel owner")?;

    match owner {
        None => Ok(SendPermission::NotFound),
        Some(owner_id) if owner_id == user_id => Ok(SendPermission::Owner),
        Some(_) => Ok(SendPermission::NotOwner),
    }
}

async fn insert_channel_conversation(
    tx: &mut Transaction<'_, Postgres>,
    channel_id: Uuid,
    owner_id: Uuid,
    name: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO conversations (id, type, title, created_by)
         VALUES ($1, 'channel', $2, $3)",
    )
    .bind(channel_id)
    .bind(name)
    .bind(owner_id)
    .execute(&mut **tx)
    .await
    .context("insert channel conversation")?;

    Ok(())
}

async fn insert_channel(
    tx: &mut Transaction<'_, Postgres>,
    channel_id: Uuid,
    owner_id: Uuid,
    name: &str,
    is_public: bool,
) -> anyhow::Result<ChannelRecord> {
    sqlx::query_as::<_, ChannelRecord>(
        "INSERT INTO channels (id, name, owner_id, is_public)
         VALUES ($1, $2, $3, $4)
         RETURNING id, name, owner_id, is_public, created_at",
    )
    .bind(channel_id)
    .bind(name)
    .bind(owner_id)
    .bind(is_public)
    .fetch_one(&mut **tx)
    .await
    .context("insert channel")
}

async fn insert_owner_membership(
    tx: &mut Transaction<'_, Postgres>,
    channel_id: Uuid,
    owner_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO conversation_members (conversation_id, user_id, role)
         VALUES ($1, $2, 'admin')
         ON CONFLICT (conversation_id, user_id) DO NOTHING",
    )
    .bind(channel_id)
    .bind(owner_id)
    .execute(&mut **tx)
    .await
    .map_err(|err| anyhow!(err))?;

    Ok(())
}
