use iced::widget::{container, text};
use iced::{Border, Element, Font, Theme};

pub(crate) fn pane_active(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            width: 2.0,
            color: palette.background.strong.color,
            ..Border::default()
        },
        ..Default::default()
    }
}

pub(crate) fn pane_focused(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            width: 2.0,
            color: palette.primary.strong.color,
            ..Border::default()
        },
        ..Default::default()
    }
}

pub(crate) fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

pub(crate) fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

pub(crate) fn compiler_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e802}')
}

pub(crate) fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}
