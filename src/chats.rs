use crate::loc;
use crate::types::{AppEvent, Chat, ChatKind, Telegram};
use anyhow::{Context, Result};
use grammers_client::{
    peer::{Dialog, Peer},
    session::types::PeerRef,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

pub async fn scan(telegram: Telegram, tx: UnboundedSender<AppEvent>) -> Result<()> {
    let mut dialogs = telegram.client.iter_dialogs();
    let total = dialogs.total().await.context(loc::ERR_COUNT_CHATS)?;
    let _ = tx.send(AppEvent::ChatsTotal(total));
    while let Some(dialog) = dialogs.next().await.context(loc::ERR_LIST_CHATS)? {
        let chat = get_chat_with_own_messages(&telegram, dialog)
            .await
            .unwrap_or_else(|e| {
                error!("{:?}", e);
                None
            });
        let _ = tx.send(AppEvent::ChatScanned(chat));
    }
    let _ = tx.send(AppEvent::ScanDone);
    Ok(())
}

async fn count_own_messages(telegram: &Telegram, peer: PeerRef) -> Result<usize> {
    telegram
        .client
        .search_messages(peer)
        .sent_by_self()
        .total()
        .await
        .context(loc::ERR_COUNT_MESSAGES)
}

async fn get_chat_with_own_messages(telegram: &Telegram, dialog: Dialog) -> Result<Option<Chat>> {
    let peer = dialog.peer_ref();
    let name = get_name(dialog.peer());
    let count = count_own_messages(telegram, peer)
        .await
        .with_context(|| loc::err_scan_chat(&name))?;
    if count == 0 {
        return Ok(None);
    }
    Ok(Some(Chat {
        peer,
        name,
        kind: get_kind(dialog.peer()),
        count,
    }))
}

fn get_name(peer: &Peer) -> String {
    match peer {
        Peer::User(user) if user.is_self() => loc::SAVED_MESSAGES.to_string(),
        Peer::User(user) if user.deleted() => loc::DELETED_ACCOUNT.to_string(),
        Peer::User(user) => user.full_name(),
        _ => peer.name().unwrap_or(loc::UNKNOWN_CHAT).to_string(),
    }
}

fn get_kind(peer: &Peer) -> ChatKind {
    match peer {
        Peer::User(user) if user.is_bot() => ChatKind::Bot,
        Peer::User(_) => ChatKind::User,
        Peer::Group(group) if group.is_megagroup() => ChatKind::Supergroup,
        Peer::Group(_) => ChatKind::Group,
        Peer::Channel(_) => ChatKind::Channel,
        Peer::Community(_) => ChatKind::Community,
    }
}
