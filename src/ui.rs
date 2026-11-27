use crate::loc;
use crate::types::{ApiField, AppState, Screen};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Gauge, HighlightSpacing, List, ListState, Padding, Paragraph, Wrap},
};
use tui_input::Input;

const ROW_SYMBOL: &str = "> ";

pub fn draw(frame: &mut Frame, state: &mut AppState) {
    let [title, body, status, help] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    frame.render_widget(
        Line::from(format!(" {} ", loc::APP_TITLE))
            .bold()
            .reversed(),
        title,
    );
    let block = Block::bordered().padding(Padding::horizontal(1));
    let inner = block.inner(body);
    frame.render_widget(block, body);
    let body = inner;
    match state.screen {
        Screen::SetupChoice => draw_menu(
            frame,
            body,
            loc::SETUP_TEXT,
            &loc::SETUP_OPTIONS,
            &mut state.menu,
        ),
        Screen::ApiForm => draw_api_form(frame, body, state),
        Screen::Connecting => draw_text(frame, body, loc::CONNECTING),
        Screen::LoginChoice => draw_menu(
            frame,
            body,
            loc::LOGIN_TEXT,
            &loc::LOGIN_OPTIONS,
            &mut state.menu,
        ),
        Screen::Phone => draw_input_screen(
            frame,
            body,
            loc::PHONE_TEXT,
            loc::PHONE_TITLE,
            &state.phone,
            false,
        ),
        Screen::Code => draw_input_screen(
            frame,
            body,
            loc::CODE_TEXT,
            loc::CODE_TITLE,
            &state.code,
            false,
        ),
        Screen::Password => draw_password(frame, body, state),
        Screen::Qr => draw_qr(frame, body, state),
        Screen::Chats => draw_chats(frame, body, state),
        Screen::Confirm => draw_confirm(frame, body, state),
        Screen::Erasing => draw_erasing(frame, body, state),
    }
    draw_status(frame, status, state);
    frame.render_widget(
        Line::from(format!(" {}", get_help(state.screen))).dark_gray(),
        help,
    );
}

fn draw_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = match &state.status {
        Some(status) => status.as_str(),
        None if state.busy => loc::PLEASE_WAIT,
        None => "",
    };
    let paragraph = Paragraph::new(format!(" {text}"))
        .yellow()
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn get_help(screen: Screen) -> &'static str {
    match screen {
        Screen::SetupChoice => loc::HELP_MENU,
        Screen::ApiForm => loc::HELP_API_FORM,
        Screen::Connecting => loc::HELP_CONNECTING,
        Screen::LoginChoice => loc::HELP_LOGIN_CHOICE,
        Screen::Phone | Screen::Code | Screen::Password => loc::HELP_INPUT,
        Screen::Qr => loc::HELP_QR,
        Screen::Chats => loc::HELP_CHATS,
        Screen::Confirm => loc::HELP_CONFIRM,
        Screen::Erasing => loc::HELP_ERASING,
    }
}

fn draw_text(frame: &mut Frame, area: Rect, text: &str) {
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), area);
}

fn draw_menu(frame: &mut Frame, area: Rect, text: &str, options: &[&str], menu: &mut ListState) {
    let [text_area, list_area] =
        Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).areas(area);
    draw_text(frame, text_area, text);
    let list = List::new(options.iter().copied())
        .highlight_symbol(ROW_SYMBOL)
        .highlight_style(Style::new().bold().yellow());
    frame.render_stateful_widget(list, list_area, menu);
}

fn draw_api_form(frame: &mut Frame, area: Rect, state: &AppState) {
    let [text_area, id_area, hash_area, _] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .areas(area);
    draw_text(frame, text_area, loc::API_FORM_TEXT);
    let id_focused = state.api_field == ApiField::Id;
    draw_input(
        frame,
        id_area,
        loc::API_ID_TITLE,
        &state.api_id,
        false,
        id_focused,
    );
    draw_input(
        frame,
        hash_area,
        loc::API_HASH_TITLE,
        &state.api_hash,
        false,
        !id_focused,
    );
}

fn draw_input_screen(
    frame: &mut Frame,
    area: Rect,
    text: &str,
    title: &str,
    input: &Input,
    masked: bool,
) {
    let [text_area, input_area, _] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .areas(area);
    draw_text(frame, text_area, text);
    draw_input(frame, input_area, title, input, masked, true);
}

fn draw_password(frame: &mut Frame, area: Rect, state: &AppState) {
    let hint = state.password_token.as_ref().and_then(|token| token.hint());
    let title = match hint {
        Some(hint) => loc::password_title_with_hint(hint),
        None => loc::PASSWORD_TITLE.to_string(),
    };
    draw_input_screen(
        frame,
        area,
        loc::PASSWORD_TEXT,
        &title,
        &state.password,
        true,
    );
}

fn draw_input(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    input: &Input,
    masked: bool,
    focused: bool,
) {
    let width = area.width.saturating_sub(3) as usize;
    let scroll = input.visual_scroll(width);
    let text = if masked {
        "*".repeat(input.value().chars().count())
    } else {
        input.value().to_string()
    };
    let style = if focused {
        Style::new().yellow()
    } else {
        Style::new()
    };
    let paragraph = Paragraph::new(text)
        .scroll((0, scroll as u16))
        .block(Block::bordered().title(title).border_style(style));
    frame.render_widget(paragraph, area);
    if focused {
        let x = input.visual_cursor().saturating_sub(scroll) as u16;
        frame.set_cursor_position((area.x + 1 + x, area.y + 1));
    }
}

fn draw_qr(frame: &mut Frame, area: Rect, state: &AppState) {
    let [text_area, qr_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);
    draw_text(frame, text_area, loc::QR_TEXT);
    let Some(qr) = &state.qr else {
        draw_text(frame, qr_area, loc::QR_GENERATING);
        return;
    };
    let width = qr
        .lines()
        .next()
        .map(|line| line.chars().count())
        .unwrap_or(0) as u16;
    let height = qr.lines().count() as u16;
    let qr_area = Rect {
        width: width.min(qr_area.width),
        height: height.min(qr_area.height),
        ..qr_area
    };
    let paragraph =
        Paragraph::new(qr.as_str()).style(Style::new().fg(Color::White).bg(Color::Black));
    frame.render_widget(paragraph, qr_area);
}

fn draw_chats(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let [summary_area, header_area, list_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(area);
    let summary = if state.scan.running {
        loc::scanning_chats(state.scan.scanned, state.scan.total)
    } else {
        loc::chats_found(state.chats.len())
    };
    draw_text(frame, summary_area, &summary);
    let header = format!(
        "{}{}",
        " ".repeat(ROW_SYMBOL.len()),
        format_row(
            loc::CHATS_HEADER_COUNT,
            loc::CHATS_HEADER_KIND,
            loc::CHATS_HEADER_NAME,
        )
    );
    frame.render_widget(Line::from(header).bold(), header_area);
    let rows = state.chats.iter().map(|chat| {
        format_row(
            &chat.count.to_string(),
            loc::kind_label(chat.kind),
            &chat.name,
        )
    });
    let list = List::new(rows)
        .highlight_symbol(ROW_SYMBOL)
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_style(Style::new().bold().yellow());
    frame.render_stateful_widget(list, list_area, &mut state.chat_list);
}

fn format_row(count: &str, kind: &str, name: &str) -> String {
    format!("{count:>8}  {kind:<10}  {name}")
}

fn draw_confirm(frame: &mut Frame, area: Rect, state: &AppState) {
    let chat = &state.chats[state.erase.chat_index];
    let lines = vec![
        Line::from(loc::confirm_delete(chat.count, &chat.name)).bold(),
        Line::from(""),
        Line::from(loc::CONFIRM_WARNING),
        Line::from(""),
        Line::from(loc::CONFIRM_KEYS).yellow(),
    ];
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}

fn draw_erasing(frame: &mut Frame, area: Rect, state: &AppState) {
    let [text_area, gauge_area, _] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(area);
    let chat = &state.chats[state.erase.chat_index];
    draw_text(frame, text_area, &loc::erasing(&chat.name));
    let erase = &state.erase;
    let (ratio, label) = if erase.deleted == 0 {
        (0.0, loc::messages_found(erase.collected))
    } else {
        let ratio = erase.deleted as f64 / erase.collected.max(1) as f64;
        (
            ratio.min(1.0),
            loc::delete_progress(erase.deleted, erase.collected),
        )
    };
    let gauge = Gauge::default()
        .ratio(ratio)
        .label(label)
        .gauge_style(Style::new().yellow());
    frame.render_widget(gauge, gauge_area);
}
