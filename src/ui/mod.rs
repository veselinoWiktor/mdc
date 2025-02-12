mod utils;
mod style;
mod ast_canvas_converter;
mod views;
mod loader;

use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{self, container, horizontal_space, pick_list, responsive, text, text_editor, toggler};
use iced::{event, highlighter, mouse, Color, Event, Font, Pixels, Point, Subscription, Theme};
use iced::{Center, Element, Fill, Size, Task};

use iced::alignment::{Horizontal};
use iced::mouse::{Cursor, ScrollDelta};
use iced::widget::canvas::{Frame, Geometry, Path, Program, Stroke, Style, Text};
use std::path::{PathBuf};
use std::sync::Arc;
use iced::widget::text_editor::{Action, Edit};
use crate::compiler::parser::parse_program;
use crate::compiler::tokenizer::tokenize;
use crate::ui::ast_canvas_converter::convert_into_ast_canvas;
use crate::ui::utils::{open_file, save_file, Error};
use crate::ui::views::{action, view_canvas, view_editor, view_loader};

pub(crate) fn run_ui() -> iced::Result {
    iced::application("AST Visualizer", ASTVisualizer::update, ASTVisualizer::view)
        .theme(ASTVisualizer::theme)
        .font(include_bytes!("fonts/icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .subscription(ASTVisualizer::subscription)
        .antialiasing(true)
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

    is_ast_valid: bool,
    ast: ASTCanvas,
}

#[derive(Debug, Clone)]
enum Message {
    // Pane Messages
    Resized(pane_grid::ResizeEvent),

    // Editor Messages
    ActionPerformed(Action),
    ThemeSelected(highlighter::Theme),
    WordWrapToggled(bool),
    NewFile,
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    SaveFile,
    FileSaved(Result<PathBuf, Error>),
    InsertTab,
    CompileCode,

    // AST Canvas
    ZoomCanvas(f32),
    GenerateAstCanvas,
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
                is_loading: false,
                is_dirty: false,
                is_ast_valid: false,
                ast: ASTCanvas {
                    root: example_ast(),
                    scale: 1.0
                },
            },
            Task::batch([Task::done(Message::NewFile), widget::focus_next()]),
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

                // here should be the update of the tree canvas
                // get the text
                Task::done(Message::GenerateAstCanvas)
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
                    self.is_ast_valid = false;
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
                    Task::done(Message::GenerateAstCanvas)
                } else {
                    Task::none()
                }
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
            },
            Message::InsertTab => {
                self.is_loading = true;

                // let mut text = self.content.text();
                // let (x, y) = self.content.cursor_position();
                // text.insert_str(y, "    ");
                // self.content = text_editor::Content::with_text(&text);
                self.content.perform(Action::Edit(Edit::Paste(Arc::new("    ".to_string()))));

                self.is_loading = false;

                Task::none()
            }
            Message::CompileCode => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;

                    //let mut text = self.content.text();
                    Task::none()
                }
            }
            Message::ZoomCanvas(zoom) => {
                self.ast.scale  = (self.ast.scale + zoom).clamp(0.5, 3.0);

                Task::none()
            }
            Message::GenerateAstCanvas => {
                let text = self.content.text();

                self.is_ast_valid = true;
                // tokenize it
                let mut tokens = match tokenize(&text) {
                    Ok(tokens) => tokens,
                    Err(_) => {
                        self.is_ast_valid = false;
                        return Task::none()
                    }
                };

                // generate ast
                let ast = match parse_program(&mut tokens) {
                    Ok(ast) => ast,
                    Err(_) => {
                        self.is_ast_valid = false;
                        return Task::none()
                    }
                };

                // generate ast canvas
                self.ast = convert_into_ast_canvas(&ast);

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
            action(
                style::compiler_icon(),
                "Compile code",
                self.is_dirty.then_some(Message::CompileCode)
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

        let pane_grid = PaneGrid::new(&self.panes, |id, pane, _| {
            let is_focused = focus == Some(id);

            pane_grid::Content::new(responsive(move |_| match pane.id {
                0 => view_editor(&self.content, self.word_wrap, &self.file, &self.theme),
                1 => {
                    if self.is_ast_valid {
                        view_canvas(&self.ast)
                    }
                    else {
                        view_loader()
                    }
                },
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

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(|event| {
            if let Event::Mouse(mouse::Event::WheelScrolled{ delta} ) = event {
                match delta {
                    ScrollDelta::Lines { y, .. } => Message::ZoomCanvas(y * 0.1),
                    ScrollDelta::Pixels { y, .. } => Message::ZoomCanvas(y * 0.001),
                }
            } else {
                Message::ZoomCanvas(0.0)
            }
        })
    }
}

#[derive(Clone, Debug)]
struct ASTNode {
    value: String,
    children: Vec<ASTNode>,
}

impl ASTNode {
    fn new(value: String) -> Self {
        Self {
            value,
            children: Vec::new(),
        }
    }
}



fn example_ast() -> ASTNode {
    ASTNode {
        value: "Program(function_definition)".to_string(),
        children: vec![ASTNode {
            value: "Function('main', body)".to_string(),
            children: vec![
                ASTNode {
                    value: "Return(exp)".to_string(),
                    children: vec![ASTNode {
                        value: "Constant(2)".to_string(),
                        children: vec![],
                    }],
                },
                ASTNode {
                    value: "Return(exp)".to_string(),
                    children: vec![ASTNode {
                        value: "Constant(2)".to_string(),
                        children: vec![],
                    }],
                },
            ],
        }],
    }
}

struct ASTCanvas {
    root: ASTNode,
    scale: f32
}

impl ASTCanvas {
    fn new(root: ASTNode) -> ASTCanvas {
        ASTCanvas {
            root,
            scale: 1.0
        }
    }

    // fn new empty
}

impl<Message> Program<Message> for ASTCanvas {
    type State = ();
    fn draw(
        &self,
        _state: &Self::State,
        _renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let frame_size = Size::new(3000.0, 2000.0);
        let mut frame = Frame::new(_renderer, frame_size);

        frame.scale(self.scale);

        let start_x = bounds.width / 2.0;
        let start_y = 50.0;
        let node_spacing = 100.0;

        self.draw_node(
            &mut frame,
            &self.root,
            start_x,
            start_y,
            bounds.width * 2.0,
            node_spacing,
        );

        vec![frame.into_geometry()]
    }
}

impl ASTCanvas {
    fn draw_node(
        &self,
        frame: &mut Frame,
        node: &ASTNode,
        x: f32,
        y: f32,
        x_offset: f32,
        y_offset: f32,
    ) {
        let rectangle_size = Size::new(10.0 + 7.0 * node.value.len() as f32, 30.0);
        let rectangle = Path::rectangle(
            Point::new(x - (rectangle_size.width / 2.0), y),
            rectangle_size,
        );

        frame.stroke(
            &rectangle,
            Stroke {
                width: 2.0,
                style: Style::Solid(Color::BLACK),
                ..Stroke::default()
            },
        );

        frame.fill_text(Text {
            content: node.value.to_string(),
            position: Point::new(x, y + 8.0),
            color: Color::BLACK,
            size: Pixels(14.0),
            horizontal_alignment: Horizontal::Center,
            ..Default::default()
        });

        // Draw children recursively
        let num_children = node.children.len();
        if num_children > 0 {
            let step = (2.0 * x_offset) / (num_children as f32);
            let mut child_x = x - x_offset + step / 2.0;

            for child in &node.children {
                // Draw line to child
                let line = Path::line(
                    Point::new(x, y + rectangle_size.height),
                    Point::new(child_x, y + y_offset),
                );
                frame.stroke(
                    &line,
                    Stroke {
                        width: 2.0,
                        style: Style::Solid(Color::BLACK),
                        ..Stroke::default()
                    },
                );

                // Draw child
                self.draw_node(
                    frame,
                    child,
                    child_x,
                    y + y_offset,
                    x_offset / 2.0,
                    y_offset,
                );
                child_x += step;
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Pane {
    id: usize,
}

impl Pane {
    fn new(id: usize) -> Pane {
        Self {
            id,
        }
    }
}
