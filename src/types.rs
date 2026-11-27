use grammers_client::{
    Client,
    client::{LoginToken, PasswordToken},
    sender::SenderPoolFatHandle,
    session::{storages::SqliteSession, types::PeerRef},
};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};
use tui_input::Input;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_id: i32,
    pub api_hash: String,
}

#[derive(Clone)]
pub struct Telegram {
    pub client: Client,
    pub session: Arc<SqliteSession>,
    pub handle: SenderPoolFatHandle,
    pub api_id: i32,
    pub api_hash: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ChatKind {
    User,
    Bot,
    Group,
    Supergroup,
    Channel,
    Community,
}

#[derive(Clone)]
pub struct Chat {
    pub peer: PeerRef,
    pub name: String,
    pub kind: ChatKind,
    pub count: usize,
}

pub enum LoginStep {
    Done,
    Password(PasswordToken),
    WrongPassword(PasswordToken),
}

pub enum AppEvent {
    Connected(Telegram, bool),
    CodeSent(LoginToken),
    Login(LoginStep),
    QrCode(String),
    ChatsTotal(usize),
    ChatScanned(Option<Chat>),
    ScanDone,
    ScanFailed(String),
    Collected(usize),
    Deleted(usize),
    EraseDone(usize),
    EraseFailed(String),
    Error(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    SetupChoice,
    ApiForm,
    Connecting,
    LoginChoice,
    Phone,
    Code,
    Password,
    Qr,
    Chats,
    Confirm,
    Erasing,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ApiField {
    Id,
    Hash,
}

#[derive(Default)]
pub struct Scan {
    pub running: bool,
    pub total: usize,
    pub scanned: usize,
}

#[derive(Default)]
pub struct Erase {
    pub chat_index: usize,
    pub collected: usize,
    pub deleted: usize,
}

pub struct AppState {
    pub tx: UnboundedSender<AppEvent>,
    pub screen: Screen,
    pub status: Option<String>,
    pub busy: bool,
    pub quit: bool,
    pub menu: ListState,
    pub api_id: Input,
    pub api_hash: Input,
    pub api_field: ApiField,
    pub phone: Input,
    pub code: Input,
    pub password: Input,
    pub telegram: Option<Telegram>,
    pub login_token: Option<Arc<LoginToken>>,
    pub password_token: Option<PasswordToken>,
    pub qr: Option<String>,
    pub qr_task: Option<JoinHandle<()>>,
    pub chats: Vec<Chat>,
    pub chat_list: ListState,
    pub scan: Scan,
    pub erase: Erase,
}
