use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{
    self, button, center_x, center_y, container, horizontal_space, pick_list, responsive,
    scrollable, text, text_editor, toggler, tooltip, Button,
};
use iced::{highlighter, keyboard, Font, Theme};
use iced::{Center, Element, Fill, Size, Task};
use std::{ffi, io};

use iced::keyboard::on_key_press;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use iced::keyboard::key::NativeCode::Windows;

pub(crate) fn run_ui() -> iced::Result {
    iced::application("AST Visualizer", ASTVisualizer::update, ASTVisualizer::view)
        .theme(ASTVisualizer::theme)
        .font(include_bytes!("fonts/icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .run_with(ASTVisualizer::new)
}

struct ASTVisualizer {
    panes: pane_grid::State<Pane>,
    focus: Option<pane_grid::Pane>,

    file: Option<PathBuf>,
    content: text_editor::Content,
    theme: highlighter::Theme,
    word_wrap: bool,
    is_loading: bool,
    is_dirty: bool,
}

#[derive(Debug, Clone)]
enum Message {
    // Pane Messages
    Resized(pane_grid::ResizeEvent),

    // Editor Messages
    ActionPerformed(text_editor::Action),
    ThemeSelected(highlighter::Theme),
    WordWrapToggled(bool),
    NewFile,
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    SaveFile,
    FileSaved(Result<PathBuf, Error>),
}

impl ASTVisualizer {
    fn new() -> (Self, Task<Message>) {
        let pane = Pane::new(0);
        let (mut panes, pane) = pane_grid::State::new(pane);
        panes.split(pane_grid::Axis::Vertical, pane, Pane::new(1));
        (
            Self {
                panes,
                focus: None,
                file: None,
                content: text_editor::Content::new(),
                theme: highlighter::Theme::SolarizedDark,
                word_wrap: true,
                is_loading: true,
                is_dirty: false,
            },
            Task::batch([
                Task::perform(
                    load_file(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR"))),
                    Message::FileOpened,
                ),
                widget::focus_next(),
            ]),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Resized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);

                Task::none()
            }
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();

                self.content.perform(action);

                Task::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Task::none()
            }
            Message::WordWrapToggled(word_wrap) => {
                self.word_wrap = word_wrap;

                Task::none()
            }
            Message::NewFile => {
                if !self.is_loading {
                    self.file = None;
                    self.content = text_editor::Content::new();
                }

                Task::none()
            }
            Message::OpenFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;

                    Task::perform(open_file(), Message::FileOpened)
                }
            }
            Message::FileOpened(result) => {
                self.is_loading = false;
                self.is_dirty = false;

                if let Ok((path, contents)) = result {
                    self.file = Some(path);
                    self.content = text_editor::Content::with_text(&contents);
                }

                Task::none()
            }
            Message::SaveFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;

                    let mut text = self.content.text();

                    if let Some(ending) = self.content.line_ending() {
                        if !text.ends_with(ending.as_str()) {
                            text.push_str(ending.as_str());
                        }
                    }

                    Task::perform(save_file(self.file.clone(), text), Message::FileSaved)
                }
            }
            Message::FileSaved(result) => {
                self.is_loading = false;
                if let Ok(path) = result {
                    self.file = Some(path);
                    self.is_dirty = false;
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let controls = iced::widget::row![
            action(style::new_icon(), "New file", Some(Message::NewFile)),
            action(
                style::open_icon(),
                "Open file",
                (!self.is_loading).then_some(Message::OpenFile)
            ),
            action(
                style::save_icon(),
                "Save file",
                self.is_dirty.then_some(Message::SaveFile)
            ),
            horizontal_space(),
            toggler(self.word_wrap)
                .label("Word Wrap")
                .on_toggle(Message::WordWrapToggled),
            pick_list(
                highlighter::Theme::ALL,
                Some(self.theme),
                Message::ThemeSelected,
            )
            .text_size(14)
            .padding([5, 10])
        ]
        .spacing(10)
        .align_y(Center);

        let focus = self.focus;
        let total_panes = self.panes.len();

        let pane_grid = PaneGrid::new(&self.panes, |id, pane, is_maximized| {
            let is_focused = focus == Some(id);

            pane_grid::Content::new(responsive(move |size| match pane.id {
                0 => self.view_editor(),
                1 => view_content(id, total_panes, pane.is_pinned, size),
                _ => todo!(),
            }))
            .style(if is_focused {
                style::pane_focused
            } else {
                style::pane_active
            })
        })
        .width(Fill)
        .height(Fill)
        .spacing(10)
        .on_resize(10, Message::Resized);

        let status = iced::widget::row![
            text(if let Some(path) = &self.file {
                let path = path.display().to_string();

                if path.len() > 60 {
                    format!("...{}", &path[path.len() - 40..])
                } else {
                    path
                }
            } else {
                String::from("New file")
            }),
            horizontal_space(),
            text({
                let (line, column) = self.content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })
        ]
        .spacing(10);

        iced::widget::column![controls, container(pane_grid), status]
            .spacing(10)
            .padding(10)
            .into()
    }

    fn view_editor(&self) -> Element<Message> {
        text_editor(&self.content)
            .height(Fill)
            .on_action(Message::ActionPerformed)
            .wrapping(if self.word_wrap {
                text::Wrapping::Word
            } else {
                text::Wrapping::None
            })
            .highlight(
                self.file
                    .as_deref()
                    .and_then(Path::extension)
                    .and_then(ffi::OsStr::to_str)
                    .unwrap_or("rs"),
                self.theme,
            )
            .key_binding(|key_press| match key_press.key.as_ref() {
                keyboard::Key::Character("s") if key_press.modifiers.command() => {
                    Some(text_editor::Binding::Custom(Message::SaveFile))
                }
                _ => text_editor::Binding::from_key_press(key_press),
            })
            .into()
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

#[derive(Clone, Copy)]
struct Pane {
    id: usize,
    is_pinned: bool,
}

impl Pane {
    fn new(id: usize) -> Pane {
        Self {
            id,
            is_pinned: false,
        }
    }
}

fn view_content<'a>(
    pane: pane_grid::Pane,
    total_panes: usize,
    is_pinned: bool,
    size: Size,
) -> Element<'a, Message> {
    let button: fn(&str, Message) -> Button<'_, Message> = |label, message| {
        button(text(label).width(Fill).align_x(Center).size(16))
            .width(Fill)
            .padding(8)
            .on_press(message)
    };

    let content = iced::widget::column![text!("{}x{}", size.width, size.height).size(24)]
        .spacing(10)
        .align_x(Center);

    center_y(scrollable(content)).padding(5).into()
}

fn action<'a, Message: Clone + 'a>(
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

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

async fn open_file() -> Result<(PathBuf, Arc<String>), Error> {
    let picked_file = rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(picked_file).await
}

async fn load_file(path: impl Into<PathBuf>) -> Result<(PathBuf, Arc<String>), Error> {
    let path = path.into();

    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, contents: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .save_file()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(Path::to_owned)
            .ok_or(Error::DialogClosed)?
    };

    tokio::fs::write(&path, contents)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;

    Ok(path)
}

mod style {
    use iced::font::Style;
    use iced::widget::{container, text};
    use iced::{Border, Element, Font, Theme};

    pub fn pane_active(theme: &Theme) -> container::Style {
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

    pub fn pane_focused(theme: &Theme) -> container::Style {
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

    pub(crate) fn open_icon<'a, Message>() -> Element<'a, Message> {
        icon('\u{0f115}')
    }

    fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
        const ICON_FONT: Font = Font::with_name("editor-icons");

        text(codepoint).font(ICON_FONT).into()
    }
}
