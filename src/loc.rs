use crate::types::ChatKind;

pub const APP_TITLE: &str = "Telegram Eraser";
pub const PLEASE_WAIT: &str = "Please wait...";

pub const SETUP_TEXT: &str = "You must create a developer telegram app in order to use this tool";
pub const SETUP_OPTIONS: [&str; 2] = ["I already have an app", "Create a new app"];

pub const API_FORM_TEXT: &str = "On my.telegram.org, log in with your phone number and open \"API development tools\". To create an app, fill in any App title and Short name and pick Desktop as the platform. Then copy \"App api_id\" and \"App api_hash\" here.";
pub const API_ID_TITLE: &str = "api_id";
pub const API_HASH_TITLE: &str = "api_hash";

pub const CONNECTING: &str = "Connecting to Telegram...";

pub const LOGIN_TEXT: &str = "Log in to your Telegram account.";
pub const LOGIN_OPTIONS: [&str; 2] = ["Phone number", "QR code"];

pub const PHONE_TEXT: &str = "Enter your phone number in international format, e.g. +15551234567.";
pub const PHONE_TITLE: &str = "Phone number";

pub const CODE_TEXT: &str = "Enter the login code Telegram sent to your Telegram app.";
pub const CODE_TITLE: &str = "Login code";

pub const PASSWORD_TEXT: &str =
    "Your account has two-step verification. Enter your cloud password.";
pub const PASSWORD_TITLE: &str = "Password";
pub const WRONG_PASSWORD: &str = "Wrong password";

pub const QR_TEXT: &str =
    "On your phone, open Telegram > Settings > Devices > Link Desktop Device and scan this code.";
pub const QR_GENERATING: &str = "Generating QR code...";

pub const CHATS_HEADER_COUNT: &str = "Messages";
pub const CHATS_HEADER_KIND: &str = "Type";
pub const CHATS_HEADER_NAME: &str = "Name";
pub const SAVED_MESSAGES: &str = "Saved Messages";
pub const DELETED_ACCOUNT: &str = "Deleted Account";
pub const UNKNOWN_CHAT: &str = "Unknown";

pub const CONFIRM_WARNING: &str = "They will be deleted for everyone. This cannot be undone.";
pub const CONFIRM_KEYS: &str = "[y] Delete   [n] Cancel";

pub const HELP_MENU: &str = "↑/↓ select · Enter confirm · q quit";
pub const HELP_API_FORM: &str = "Tab switch field · Enter next/save · Esc back";
pub const HELP_CONNECTING: &str = "Esc change API credentials · q quit";
pub const HELP_LOGIN_CHOICE: &str =
    "↑/↓ select · Enter confirm · Esc change API credentials · q quit";
pub const HELP_INPUT: &str = "Enter submit · Esc back";
pub const HELP_QR: &str = "Esc back · q quit";
pub const HELP_CHATS: &str = "↑/↓ select · Enter erase · r rescan · q quit";
pub const HELP_CONFIRM: &str = "y delete · n cancel";
pub const HELP_ERASING: &str = "Ctrl+C quit";

pub const ERR_HOME_DIR: &str = "Failed to find home directory";
pub const ERR_LOGGER: &str = "Failed to initialize file logger";
pub const ERR_READ_CONFIG: &str = "Failed to read config file";
pub const ERR_INVALID_CONFIG: &str = "Invalid config file";
pub const ERR_WRITE_CONFIG: &str = "Failed to write config file";
pub const ERR_API_ID: &str = "api_id must be a number";
pub const ERR_OPEN_SESSION: &str = "Failed to open session file";
pub const ERR_REACH_TELEGRAM: &str = "Failed to reach Telegram";
pub const ERR_SEND_CODE: &str = "Failed to send login code";
pub const ERR_INVALID_CODE: &str = "Invalid login code";
pub const ERR_NO_ACCOUNT: &str = "This phone number has no Telegram account";
pub const ERR_SIGN_IN: &str = "Failed to sign in";
pub const ERR_CHECK_PASSWORD: &str = "Failed to check password";
pub const ERR_QR_TOKEN: &str = "Failed to get QR login token";
pub const ERR_QR_IMPORT: &str = "Failed to import QR login token";
pub const ERR_QR_UNEXPECTED: &str = "Unexpected QR login response from Telegram";
pub const ERR_QR_CREATE: &str = "Failed to create QR code";
pub const ERR_PASSWORD_SETTINGS: &str = "Failed to get 2FA password settings";
pub const ERR_VERIFY_DELETE: &str = "Failed to check which messages were deleted";
pub const ERR_COUNT_CHATS: &str = "Failed to count chats";
pub const ERR_LIST_CHATS: &str = "Failed to list chats";
pub const ERR_COUNT_MESSAGES: &str = "Failed to count your messages";
pub const ERR_SEARCH_MESSAGES: &str = "Failed to search your messages";
pub const ERR_DELETE_MESSAGES: &str = "Failed to delete messages";

pub fn kind_label(kind: ChatKind) -> &'static str {
    match kind {
        ChatKind::User => "User",
        ChatKind::Bot => "Bot",
        ChatKind::Group => "Group",
        ChatKind::Supergroup => "Supergroup",
        ChatKind::Channel => "Channel",
        ChatKind::Community => "Community",
    }
}

pub fn password_title_with_hint(hint: &str) -> String {
    format!("{PASSWORD_TITLE} (hint: {hint})")
}

pub fn scanning_chats(scanned: usize, total: usize) -> String {
    format!("Scanning chats {scanned}/{total}...")
}

pub fn chats_found(count: usize) -> String {
    format!("{count} chats contain messages you sent")
}

pub fn confirm_delete(count: usize, name: &str) -> String {
    format!("Delete all {count} of your messages in {name}?")
}

pub fn erasing(name: &str) -> String {
    format!("Erasing your messages in {name}")
}

pub fn messages_found(count: usize) -> String {
    format!("Found {count} messages")
}

pub fn delete_progress(deleted: usize, total: usize) -> String {
    format!("Deleted {deleted}/{total}")
}

pub fn erase_done(deleted: usize, name: &str) -> String {
    format!("Deleted {deleted} messages in {name}")
}

pub fn erase_partial(deleted: usize, name: &str, remaining: usize) -> String {
    format!("Deleted {deleted} messages in {name}, {remaining} could not be deleted")
}

pub fn err_api_hash(len: usize) -> String {
    format!("api_hash must be {len} hexadecimal characters")
}

pub fn err_open_browser(url: &str) -> String {
    format!("Failed to open a browser, go to {url}")
}

pub fn err_create_dir(path: &str) -> String {
    format!("Failed to create {path}")
}

pub fn err_permissions(path: &str) -> String {
    format!("Failed to set permissions on {path}")
}

pub fn err_scan_chat(name: &str) -> String {
    format!("Failed to scan {name}")
}
