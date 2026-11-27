use crate::types::{ApiField, AppEvent, AppState, Config, Erase, LoginStep, Scan, Screen};
use crate::{auth, chats, config, eraser, loc, telegram, ui};
use anyhow::{Error, Result};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures_util::StreamExt;
use ratatui::{DefaultTerminal, widgets::ListState};
use std::sync::Arc;
use tokio::{
    sync::mpsc::{self, UnboundedSender},
    task::JoinHandle,
};
use tracing::error;
use tui_input::{Input, backend::crossterm::EventHandler};

const DEVELOPER_URL: &str = "https://my.telegram.org/apps";
const NEW_APP: usize = 1;
const QR_LOGIN: usize = 1;

pub async fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut state = new_state(tx);
    start(&mut state);
    let mut events = EventStream::new();
    while !state.quit {
        terminal.draw(|frame| ui::draw(frame, &mut state))?;
        tokio::select! {
            Some(event) = events.next() => handle_terminal_event(&mut state, event?),
            Some(event) = rx.recv() => handle_app_event(&mut state, event),
            else => break,
        }
    }
    Ok(())
}

fn new_state(tx: UnboundedSender<AppEvent>) -> AppState {
    AppState {
        tx,
        screen: Screen::SetupChoice,
        status: None,
        busy: false,
        quit: false,
        menu: ListState::default(),
        api_id: Input::default(),
        api_hash: Input::default(),
        api_field: ApiField::Id,
        phone: Input::default(),
        code: Input::default(),
        password: Input::default(),
        telegram: None,
        login_token: None,
        password_token: None,
        qr: None,
        qr_task: None,
        chats: Vec::new(),
        chat_list: ListState::default(),
        scan: Scan::default(),
        erase: Erase::default(),
    }
}

fn start(state: &mut AppState) {
    match config::load() {
        Ok(Some(config)) => connect(state, config),
        Ok(None) => show_menu(state, Screen::SetupChoice),
        Err(e) => {
            show_menu(state, Screen::SetupChoice);
            set_error(state, e);
        }
    }
}

fn spawn<F>(
    tx: &UnboundedSender<AppEvent>,
    on_error: fn(String) -> AppEvent,
    task: F,
) -> JoinHandle<()>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    let tx = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = task.await {
            error!("{:?}", e);
            let _ = tx.send(on_error(format!("{:#}", e)));
        }
    })
}

fn go_to(state: &mut AppState, screen: Screen) {
    state.screen = screen;
    state.status = None;
    state.busy = false;
}

fn show_menu(state: &mut AppState, screen: Screen) {
    state.menu.select(Some(0));
    go_to(state, screen);
}

fn set_error(state: &mut AppState, e: Error) {
    error!("{:?}", e);
    state.status = Some(format!("{:#}", e));
    state.busy = false;
}

fn handle_terminal_event(state: &mut AppState, event: Event) {
    let Event::Key(key) = event else {
        return;
    };
    if key.kind != KeyEventKind::Press {
        return;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        state.quit = true;
        return;
    }
    match state.screen {
        Screen::SetupChoice => handle_setup_choice_key(state, key),
        Screen::ApiForm => handle_api_form_key(state, key, &event),
        Screen::Connecting => handle_connecting_key(state, key),
        Screen::LoginChoice => handle_login_choice_key(state, key),
        Screen::Phone => handle_phone_key(state, key, &event),
        Screen::Code => handle_code_key(state, key, &event),
        Screen::Password => handle_password_key(state, key, &event),
        Screen::Qr => handle_qr_key(state, key),
        Screen::Chats => handle_chats_key(state, key),
        Screen::Confirm => handle_confirm_key(state, key),
        Screen::Erasing => {}
    }
}

fn handle_setup_choice_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => state.menu.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => state.menu.select_next(),
        KeyCode::Enter => {
            let open_browser = state.menu.selected() == Some(NEW_APP);
            state.api_field = ApiField::Id;
            go_to(state, Screen::ApiForm);
            if open_browser {
                open_developer_page(state);
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => state.quit = true,
        _ => {}
    }
}

fn open_developer_page(state: &mut AppState) {
    if let Err(e) = open::that(DEVELOPER_URL) {
        set_error(
            state,
            Error::new(e).context(loc::err_open_browser(DEVELOPER_URL)),
        );
    }
}

fn handle_api_form_key(state: &mut AppState, key: KeyEvent, event: &Event) {
    match key.code {
        KeyCode::Esc => show_menu(state, Screen::SetupChoice),
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down => toggle_api_field(state),
        KeyCode::Enter if state.api_field == ApiField::Id => state.api_field = ApiField::Hash,
        KeyCode::Enter => submit_api_form(state),
        _ => {
            let input = match state.api_field {
                ApiField::Id => &mut state.api_id,
                ApiField::Hash => &mut state.api_hash,
            };
            input.handle_event(event);
        }
    }
}

fn toggle_api_field(state: &mut AppState) {
    state.api_field = match state.api_field {
        ApiField::Id => ApiField::Hash,
        ApiField::Hash => ApiField::Id,
    };
}

fn submit_api_form(state: &mut AppState) {
    let result = config::parse(state.api_id.value(), state.api_hash.value())
        .and_then(|config| config::save(&config).map(|_| config));
    match result {
        Ok(config) => connect(state, config),
        Err(e) => set_error(state, e),
    }
}

fn connect(state: &mut AppState, config: Config) {
    go_to(state, Screen::Connecting);
    let tx = state.tx.clone();
    spawn(&state.tx, AppEvent::Error, async move {
        let telegram = telegram::connect(config).await?;
        let authorized = telegram::is_authorized(&telegram).await?;
        let _ = tx.send(AppEvent::Connected(telegram, authorized));
        Ok(())
    });
}

fn disconnect(state: &mut AppState) {
    if let Some(telegram) = state.telegram.take() {
        telegram.client.disconnect();
    }
}

fn handle_connecting_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => show_menu(state, Screen::SetupChoice),
        KeyCode::Char('q') => state.quit = true,
        _ => {}
    }
}

fn handle_login_choice_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => state.menu.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => state.menu.select_next(),
        KeyCode::Enter if state.menu.selected() == Some(QR_LOGIN) => start_qr_login(state),
        KeyCode::Enter => go_to(state, Screen::Phone),
        KeyCode::Esc => {
            disconnect(state);
            show_menu(state, Screen::SetupChoice);
        }
        KeyCode::Char('q') => state.quit = true,
        _ => {}
    }
}

fn handle_phone_key(state: &mut AppState, key: KeyEvent, event: &Event) {
    match key.code {
        KeyCode::Esc => show_menu(state, Screen::LoginChoice),
        KeyCode::Enter if !state.busy => request_code(state),
        _ => {
            state.phone.handle_event(event);
        }
    }
}

fn request_code(state: &mut AppState) {
    let Some(telegram) = state.telegram.clone() else {
        return;
    };
    let phone = state.phone.value().to_string();
    let tx = state.tx.clone();
    state.busy = true;
    spawn(&state.tx, AppEvent::Error, async move {
        let token = auth::request_code(&telegram, &phone).await?;
        let _ = tx.send(AppEvent::CodeSent(token));
        Ok(())
    });
}

fn handle_code_key(state: &mut AppState, key: KeyEvent, event: &Event) {
    match key.code {
        KeyCode::Esc => go_to(state, Screen::Phone),
        KeyCode::Enter if !state.busy => sign_in(state),
        _ => {
            state.code.handle_event(event);
        }
    }
}

fn sign_in(state: &mut AppState) {
    let (Some(telegram), Some(token)) = (state.telegram.clone(), state.login_token.clone()) else {
        return;
    };
    let code = state.code.value().to_string();
    let tx = state.tx.clone();
    state.busy = true;
    spawn(&state.tx, AppEvent::Error, async move {
        let step = auth::sign_in(&telegram, &token, &code).await?;
        let _ = tx.send(AppEvent::Login(step));
        Ok(())
    });
}

fn handle_password_key(state: &mut AppState, key: KeyEvent, event: &Event) {
    match key.code {
        KeyCode::Esc => show_menu(state, Screen::LoginChoice),
        KeyCode::Enter if !state.busy => check_password(state),
        _ => {
            state.password.handle_event(event);
        }
    }
}

fn check_password(state: &mut AppState) {
    let (Some(telegram), Some(token)) = (state.telegram.clone(), state.password_token.take())
    else {
        return;
    };
    let password = state.password.value_and_reset();
    let tx = state.tx.clone();
    state.busy = true;
    spawn(&state.tx, AppEvent::Error, async move {
        let step = auth::check_password(&telegram, token, &password).await?;
        let _ = tx.send(AppEvent::Login(step));
        Ok(())
    });
}

fn start_qr_login(state: &mut AppState) {
    let Some(telegram) = state.telegram.clone() else {
        return;
    };
    state.qr = None;
    go_to(state, Screen::Qr);
    let tx = state.tx.clone();
    let task = spawn(&state.tx, AppEvent::Error, async move {
        let step = auth::qr_login(&telegram, &tx).await?;
        let _ = tx.send(AppEvent::Login(step));
        Ok(())
    });
    state.qr_task = Some(task);
}

fn stop_qr_login(state: &mut AppState) {
    if let Some(task) = state.qr_task.take() {
        task.abort();
    }
    state.qr = None;
}

fn handle_qr_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            stop_qr_login(state);
            show_menu(state, Screen::LoginChoice);
        }
        KeyCode::Char('q') => state.quit = true,
        _ => {}
    }
}

fn handle_login_step(state: &mut AppState, step: LoginStep) {
    match step {
        LoginStep::Done => {
            stop_qr_login(state);
            state.login_token = None;
            start_scan(state);
        }
        LoginStep::Password(token) => {
            stop_qr_login(state);
            state.password_token = Some(token);
            state.password.reset();
            go_to(state, Screen::Password);
        }
        LoginStep::WrongPassword(token) => {
            state.password_token = Some(token);
            state.busy = false;
            state.status = Some(loc::WRONG_PASSWORD.to_string());
        }
    }
}

fn start_scan(state: &mut AppState) {
    let Some(telegram) = state.telegram.clone() else {
        return;
    };
    state.chats.clear();
    state.chat_list.select(None);
    state.scan = Scan {
        running: true,
        ..Default::default()
    };
    go_to(state, Screen::Chats);
    spawn(
        &state.tx,
        AppEvent::ScanFailed,
        chats::scan(telegram, state.tx.clone()),
    );
}

fn handle_chats_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => state.chat_list.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => state.chat_list.select_next(),
        KeyCode::Enter => {
            if let Some(index) = state
                .chat_list
                .selected()
                .filter(|i| *i < state.chats.len())
            {
                state.erase.chat_index = index;
                go_to(state, Screen::Confirm);
            }
        }
        KeyCode::Char('r') if !state.scan.running => start_scan(state),
        KeyCode::Esc | KeyCode::Char('q') => state.quit = true,
        _ => {}
    }
}

fn handle_confirm_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => start_erase(state),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => go_to(state, Screen::Chats),
        _ => {}
    }
}

fn start_erase(state: &mut AppState) {
    let Some(telegram) = state.telegram.clone() else {
        return;
    };
    let peer = state.chats[state.erase.chat_index].peer;
    state.erase.collected = 0;
    state.erase.deleted = 0;
    go_to(state, Screen::Erasing);
    spawn(
        &state.tx,
        AppEvent::EraseFailed,
        eraser::erase(telegram, peer, state.tx.clone()),
    );
}

fn finish_erase(state: &mut AppState, remaining: usize) {
    let index = state.erase.chat_index;
    let name = state.chats[index].name.clone();
    let deleted = state.erase.deleted;
    if remaining == 0 {
        state.chats.remove(index);
    } else {
        state.chats[index].count = remaining;
    }
    go_to(state, Screen::Chats);
    state.status = Some(if remaining == 0 {
        loc::erase_done(deleted, &name)
    } else {
        loc::erase_partial(deleted, &name, remaining)
    });
}

fn handle_app_event(state: &mut AppState, event: AppEvent) {
    match event {
        AppEvent::Connected(telegram, authorized) => {
            if state.screen != Screen::Connecting {
                return;
            }
            state.telegram = Some(telegram);
            if authorized {
                start_scan(state);
            } else {
                show_menu(state, Screen::LoginChoice);
            }
        }
        AppEvent::CodeSent(token) => {
            state.login_token = Some(Arc::new(token));
            state.code.reset();
            go_to(state, Screen::Code);
        }
        AppEvent::Login(step) => handle_login_step(state, step),
        AppEvent::QrCode(qr) => state.qr = Some(qr),
        AppEvent::ChatsTotal(total) => state.scan.total = total,
        AppEvent::ChatScanned(chat) => {
            state.scan.scanned += 1;
            if let Some(chat) = chat {
                state.chats.push(chat);
                if state.chat_list.selected().is_none() {
                    state.chat_list.select(Some(0));
                }
            }
        }
        AppEvent::ScanDone => state.scan.running = false,
        AppEvent::ScanFailed(message) => {
            state.scan.running = false;
            state.status = Some(message);
        }
        AppEvent::Collected(count) => state.erase.collected = count,
        AppEvent::Deleted(count) => state.erase.deleted = count,
        AppEvent::EraseDone(remaining) => finish_erase(state, remaining),
        AppEvent::EraseFailed(message) => {
            go_to(state, Screen::Chats);
            state.status = Some(message);
        }
        AppEvent::Error(message) => {
            state.busy = false;
            state.status = Some(message);
        }
    }
}
