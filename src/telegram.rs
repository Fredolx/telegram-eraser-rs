use crate::loc;
use crate::types::{Config, Telegram};
use crate::utils::{create_private_dir, get_project_dirs};
use anyhow::{Context, Result};
use grammers_client::{
    Client, SenderPool,
    sender::{AutoSleep, ConnectionParams},
    session::storages::SqliteSession,
};
use std::{num::NonZeroU32, path::PathBuf, sync::Arc, time::Duration};

const SESSION_NAME: &str = "session.sqlite";
const FLOOD_RETRIES: u32 = 5;
const FLOOD_MAX_WAIT: Duration = Duration::from_secs(600);
const IO_ERROR_WAIT: Duration = Duration::from_secs(1);

pub async fn connect(config: Config) -> Result<Telegram> {
    let session = SqliteSession::open(get_and_create_session_path()?)
        .await
        .context(loc::ERR_OPEN_SESSION)?;
    let session = Arc::new(session);
    let SenderPool { runner, handle, .. } = SenderPool::with_configuration(
        Arc::clone(&session),
        config.api_id,
        get_connection_params(),
    );
    let client = Client::new(handle.clone());
    tokio::spawn(runner.run());
    Ok(Telegram {
        client,
        session,
        handle,
        api_id: config.api_id,
        api_hash: config.api_hash,
    })
}

pub async fn is_authorized(telegram: &Telegram) -> Result<bool> {
    telegram
        .client
        .is_authorized()
        .await
        .context(loc::ERR_REACH_TELEGRAM)
}

fn get_connection_params() -> ConnectionParams {
    ConnectionParams {
        retry_policy: Box::new(AutoSleep {
            tries: NonZeroU32::new(FLOOD_RETRIES).unwrap(),
            threshold: FLOOD_MAX_WAIT,
            io_errors_as_flood_of: Some(IO_ERROR_WAIT),
        }),
        ..Default::default()
    }
}

fn get_and_create_session_path() -> Result<PathBuf> {
    let mut path = get_project_dirs()?.data_dir().to_owned();
    create_private_dir(&path)?;
    path.push(SESSION_NAME);
    Ok(path)
}
