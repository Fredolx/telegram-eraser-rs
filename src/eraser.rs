use crate::loc;
use crate::types::{AppEvent, Telegram};
use anyhow::{Context, Result};
use grammers_client::{session::types::PeerRef, tl};
use tokio::sync::mpsc::UnboundedSender;

const DELETE_BATCH_SIZE: usize = 100;

pub async fn erase(telegram: Telegram, peer: PeerRef, tx: UnboundedSender<AppEvent>) -> Result<()> {
    let ids = collect_own_message_ids(&telegram, peer, &tx).await?;
    let mut deleted = 0;
    let mut remaining = 0;
    for batch in ids.chunks(DELETE_BATCH_SIZE) {
        telegram
            .client
            .delete_messages(peer, batch)
            .await
            .context(loc::ERR_DELETE_MESSAGES)?;
        let still_there = count_existing_messages(&telegram, peer, batch).await?;
        deleted += batch.len() - still_there;
        remaining += still_there;
        let _ = tx.send(AppEvent::Deleted(deleted));
    }
    let _ = tx.send(AppEvent::EraseDone(remaining));
    Ok(())
}

async fn count_existing_messages(telegram: &Telegram, peer: PeerRef, ids: &[i32]) -> Result<usize> {
    let messages = telegram
        .client
        .get_messages_by_id(peer, ids)
        .await
        .context(loc::ERR_VERIFY_DELETE)?;
    let existing = messages
        .iter()
        .flatten()
        .filter(|message| !matches!(message.raw, tl::enums::Message::Empty(_)))
        .count();
    Ok(existing)
}

async fn collect_own_message_ids(
    telegram: &Telegram,
    peer: PeerRef,
    tx: &UnboundedSender<AppEvent>,
) -> Result<Vec<i32>> {
    let mut messages = telegram.client.search_messages(peer).sent_by_self();
    let mut ids = Vec::new();
    while let Some(message) = messages.next().await.context(loc::ERR_SEARCH_MESSAGES)? {
        ids.push(message.id());
        let _ = tx.send(AppEvent::Collected(ids.len()));
    }
    Ok(ids)
}
