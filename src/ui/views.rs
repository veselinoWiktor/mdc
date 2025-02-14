use std::ffi;
use std::path::{Path, PathBuf};
use std::time::Duration;
use iced::{highlighter, keyboard, Element, Fill};
use iced::keyboard::key::Named;
use iced::widget::{button, canvas, center, center_x, container, text, text_editor, tooltip};
use iced::widget::text_editor::Content;
use crate::ui::{ASTCanvas, Message};
use crate::ui::loader::{Circular, STANDARD};

pub(crate) fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(center_x(content).width(30));

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
            .style(container::rounded_box)
            .into()
    } else {
        action.style(button::secondary).into()
    }
}

pub(crate) fn view_editor<'a>(content: &'a Content, word_wrap: bool, file: &'a Option<PathBuf>, theme: &'a highlighter::Theme) -> Element<'a, Message> {
    text_editor(content)
        .height(Fill)
        .on_action(Message::ActionPerformed)
        .wrapping(if word_wrap {
            text::Wrapping::Word
        } else {
            text::Wrapping::None
        })
        .highlight(
            file
                .as_deref()
                .and_then(Path::extension)
                .and_then(ffi::OsStr::to_str)
                .unwrap_or("rs"),
            *theme,
        )
        .key_binding(|key_press| match key_press.key.as_ref() {
            keyboard::Key::Character("s") if key_press.modifiers.command() => {
                Some(text_editor::Binding::Custom(Message::SaveFile))
            },
            keyboard::Key::Named(Named::Tab) => {
                Some(text_editor::Binding::Custom(Message::InsertTab))
            },
            keyboard::Key::Character("c") if key_press.modifiers.command() && key_press.modifiers.shift() => {
                Some(text_editor::Binding::Custom(Message::CompileCode))
            }
            _ => text_editor::Binding::from_key_press(key_press),
        })
        .into()
}

pub(crate) fn view_loader<'a>() -> Element<'a, Message> {
    let easing =  &STANDARD;

    let loader:Circular<_> = Circular::new().easing(easing).cycle_duration(
        Duration::from_secs_f32(2.0));

    center(loader).into()
}

pub(crate) fn view_canvas(ast: &ASTCanvas) -> Element<Message> {
    canvas(ast).height(Fill).width(Fill).into()
}