use crate::loc;
use crate::types::{AppEvent, LoginStep, Telegram};
use anyhow::{Context, Result, bail};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use grammers_client::{
    SignInError,
    client::{LoginToken, PasswordToken},
    session::Session,
    tl,
};
use qrcode::{QrCode, render::unicode::Dense1x2};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

const QR_POLL_INTERVAL: Duration = Duration::from_secs(2);
const PASSWORD_NEEDED: &str = "SESSION_PASSWORD_NEEDED";

pub async fn request_code(telegram: &Telegram, phone: &str) -> Result<LoginToken> {
    telegram
        .client
        .request_login_code(phone.trim(), &telegram.api_hash)
        .await
        .context(loc::ERR_SEND_CODE)
}

pub async fn sign_in(telegram: &Telegram, token: &LoginToken, code: &str) -> Result<LoginStep> {
    match telegram.client.sign_in(token, code.trim()).await {
        Ok(_) => Ok(LoginStep::Done),
        Err(SignInError::PasswordRequired(token)) => Ok(LoginStep::Password(token)),
        Err(SignInError::InvalidCode) => bail!(loc::ERR_INVALID_CODE),
        Err(SignInError::SignUpRequired) => bail!(loc::ERR_NO_ACCOUNT),
        Err(e) => Err(e).context(loc::ERR_SIGN_IN),
    }
}

pub async fn check_password(
    telegram: &Telegram,
    token: PasswordToken,
    password: &str,
) -> Result<LoginStep> {
    match telegram.client.check_password(token, password).await {
        Ok(_) => Ok(LoginStep::Done),
        Err(SignInError::InvalidPassword(token)) => Ok(LoginStep::WrongPassword(token)),
        Err(e) => Err(e).context(loc::ERR_CHECK_PASSWORD),
    }
}

pub async fn qr_login(telegram: &Telegram, tx: &UnboundedSender<AppEvent>) -> Result<LoginStep> {
    let mut last_token = Vec::new();
    loop {
        let request = tl::functions::auth::ExportLoginToken {
            api_id: telegram.api_id,
            api_hash: telegram.api_hash.clone(),
            except_ids: Vec::new(),
        };
        let response = match telegram.client.invoke(&request).await {
            Err(e) if e.is(PASSWORD_NEEDED) => return get_password_step(telegram).await,
            response => response.context(loc::ERR_QR_TOKEN)?,
        };
        match response {
            tl::enums::auth::LoginToken::Token(token) => {
                if token.token != last_token {
                    let _ = tx.send(AppEvent::QrCode(render_qr(&token.token)?));
                    last_token = token.token;
                }
                tokio::time::sleep(QR_POLL_INTERVAL).await;
            }
            tl::enums::auth::LoginToken::MigrateTo(migrate) => {
                return import_qr_token(telegram, migrate).await;
            }
            tl::enums::auth::LoginToken::Success(_) => return Ok(LoginStep::Done),
        }
    }
}

async fn import_qr_token(
    telegram: &Telegram,
    migrate: tl::types::auth::LoginTokenMigrateTo,
) -> Result<LoginStep> {
    let old_dc_id = telegram.session.home_dc_id()?;
    telegram.handle.disconnect_from_dc(old_dc_id);
    telegram.session.set_home_dc_id(migrate.dc_id).await?;
    let request = tl::functions::auth::ImportLoginToken {
        token: migrate.token,
    };
    match telegram.client.invoke(&request).await {
        Err(e) if e.is(PASSWORD_NEEDED) => get_password_step(telegram).await,
        Ok(tl::enums::auth::LoginToken::Success(_)) => Ok(LoginStep::Done),
        Ok(_) => bail!(loc::ERR_QR_UNEXPECTED),
        Err(e) => Err(e).context(loc::ERR_QR_IMPORT),
    }
}

async fn get_password_step(telegram: &Telegram) -> Result<LoginStep> {
    let password: tl::types::account::Password = telegram
        .client
        .invoke(&tl::functions::account::GetPassword {})
        .await
        .context(loc::ERR_PASSWORD_SETTINGS)?
        .into();
    Ok(LoginStep::Password(PasswordToken::new(password)))
}

fn render_qr(token: &[u8]) -> Result<String> {
    let url = format!("tg://login?token={}", BASE64_URL_SAFE_NO_PAD.encode(token));
    let code = QrCode::new(url).context(loc::ERR_QR_CREATE)?;
    Ok(code
        .render::<Dense1x2>()
        .dark_color(Dense1x2::Light)
        .light_color(Dense1x2::Dark)
        .build())
}
