mod ast_canvas_converter;
mod loader;
mod style;
mod utils;
mod views;

use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{
    self, canvas, container, horizontal_space, pick_list, responsive, text, text_editor, toggler,
};
use iced::{highlighter, mouse, Color, Event, Font, Pixels, Point, Rectangle, Theme,
    Vector,
};
use iced::{Center, Element, Fill, Size, Task};
use std::collections::HashMap;

use crate::compiler::parser::parse_program;
use crate::compiler::tokenizer::tokenize;
use crate::ui::ast_canvas_converter::convert_into_ast_canvas;
use crate::ui::utils::{open_file, save_file, Error};
use crate::ui::views::{action, view_canvas, view_editor, view_loader};
use iced::alignment::{Horizontal, Vertical};
use iced::mouse::{Cursor};
use iced::widget::canvas::{Frame, Geometry, Path, Program, Stroke, Style, Text};
use iced::widget::text_editor::{Action, Edit};
use reingold_tilford::Dimensions;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crate::compiler::assembly::codegen::gen;
use crate::compiler::assembly::instruction_fixup::fixup_program;
use crate::compiler::assembly::replace_pseudos::replace_pseudos;
use crate::compiler::emit::emit_assembly;
use crate::compiler::tackygen::emit_tacky;

pub(crate) fn run_ui() -> iced::Result {
    iced::application("AST Visualizer", ASTVisualizer::update, ASTVisualizer::view)
        .theme(ASTVisualizer::theme)
        .font(include_bytes!("fonts/icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
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
    Translated(Vector),
    Scaled(f32, Option<Vector>),
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
                    layout: None,
                    root: None,
                    scaling: 1.0,
                    translation: Vector::default(),
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
            }
            Message::InsertTab => {
                self.is_loading = true;

                self.content
                    .perform(Action::Edit(Edit::Paste(Arc::new("    ".to_string()))));

                self.is_loading = false;

                Task::none()
            }
            Message::CompileCode => {
                if self.is_loading {
                    return Task::none()
                }

                if self.is_ast_valid {
                    let text = self.content.text();

                    let mut tokens = tokenize(&text).unwrap();
                    let ast_program = parse_program(&mut tokens).unwrap();
                    let tacky_ast = emit_tacky(ast_program);
                    let codegen_ast = gen(tacky_ast);
                    let replace_pseudos_ast = replace_pseudos(codegen_ast);
                    let fixup_ast = fixup_program(replace_pseudos_ast.1, replace_pseudos_ast.0);
                    let mut assembly_source_code = emit_assembly(fixup_ast);

                    if let Some(ending) = self.content.line_ending() {
                        if !assembly_source_code.ends_with(ending.as_str()) {
                            assembly_source_code.push_str(ending.as_str());
                        }
                    }

                    Task::perform(save_file(None, assembly_source_code), Message::FileSaved)
                } else {
                    Task::none()
                }
            },
            Message::Translated(translation) => {
                self.ast.translation = translation;

                Task::none()
            }
            Message::Scaled(scaling, translation) => {
                self.ast.scaling = scaling;

                if let Some(translation) = translation {
                    self.ast.translation = translation;
                }

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
                        return Task::none();
                    }
                };

                // generate ast
                let ast = match parse_program(&mut tokens) {
                    Ok(ast) => ast,
                    Err(_) => {
                        self.is_ast_valid = false;
                        return Task::none();
                    }
                };

                // generate ast canvas
                let root = convert_into_ast_canvas(&ast);

                let layout = Some(reingold_tilford::layout(&Tree, &root));

                self.ast = ASTCanvas {
                    root: Some(root),
                    layout,
                    scaling: 1.0,
                    translation: Vector::default(),
                };

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
                self.is_ast_valid.then_some(Message::CompileCode)
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
                    } else {
                        view_loader()
                    }
                }
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
}

pub enum Interaction {
    None,
    Panning { translation: Vector, start: Point },
}

impl Default for Interaction {
    fn default() -> Self {
        Self::None
    }
}

impl Program<Message> for ASTCanvas {
    type State = Interaction;

    fn update(
        &self,
        interaction: &mut Interaction,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<widget::Action<Message>> {
        if let Event::Mouse(mouse::Event::ButtonReleased(_)) = event {
            *interaction = Interaction::None;
        }

        let cursor_position = cursor.position_in(bounds)?;

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => {
                    let message = match button {
                        mouse::Button::Right => {
                            *interaction = Interaction::Panning {
                                translation: self.translation,
                                start: cursor_position,
                            };

                            None
                        }
                        _ => None,
                    };

                    Some(
                        message
                            .map(canvas::Action::publish)
                            .unwrap_or(canvas::Action::request_redraw())
                            .and_capture(),
                    )
                }
                mouse::Event::CursorMoved { .. } => {
                    let message = match *interaction {
                        Interaction::None => None,
                        Interaction::Panning { translation, start } => {
                            Some(Message::Translated(
                                translation + (cursor_position - start) * (1.0 / self.scaling),
                            ))
                        }
                    };

                    let action = message
                        .map(canvas::Action::publish)
                        .unwrap_or(canvas::Action::request_redraw());

                    Some(match interaction {
                        Interaction::None => action,
                        _ => action.and_capture(),
                    })
                }
                mouse::Event::WheelScrolled { delta } => match *delta {
                    mouse::ScrollDelta::Lines { y, .. }
                    | mouse::ScrollDelta::Pixels { y, .. } => {
                        if y < 0.0 && self.scaling > Self::MIN_SCALING
                            || y > 0.0 && self.scaling < Self::MAX_SCALING
                        {
                            let old_scaling = self.scaling;

                            let scaling = (self.scaling * (1.0 + y / 30.0))
                                .clamp(
                                    Self::MIN_SCALING,
                                    Self::MAX_SCALING,
                                );

                            let translation =
                                if let Some(cursor_to_center) =
                                    cursor.position_from(bounds.center())
                                {
                                    let factor = scaling - old_scaling;

                                    Some(
                                        self.translation
                                            - Vector::new(
                                            cursor_to_center.x * factor
                                                / (old_scaling
                                                * old_scaling),
                                            cursor_to_center.y * factor
                                                / (old_scaling
                                                * old_scaling),
                                        ),
                                    )
                                } else {
                                    None
                                };

                            Some(
                                canvas::Action::publish(Message::Scaled(
                                    scaling,
                                    translation,
                                ))
                                    .and_capture(),
                            )
                        } else {
                            Some(canvas::Action::capture())
                        }
                    }
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        _renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        if let Some(root) = &self.root {
            if let Some(layout) = &self.layout {
                let mut frame = Frame::new(_renderer, bounds.size());

                frame.scale(self.scaling);
                frame.translate(self.translation);

                let start_x = bounds.width / 2.0 - root.width / 2.0;
                let start_y = 50.0;
                let node_spacing = 100.0;

                self.draw_node(
                    &mut frame,
                    root,
                    layout,
                    start_x,
                    start_y,
                    bounds.width * 2.0,
                    node_spacing,
                );

                vec![frame.into_geometry()]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}

impl ASTCanvas {
    const MIN_SCALING: f32 = 0.5;
    const MAX_SCALING: f32 = 3.0;

    fn draw_node(
        &self,
        frame: &mut Frame,
        node: &Node,
        layout: &HashMap<usize, reingold_tilford::Coordinate>,
        x: f32,
        y: f32,
        x_offset: f32,
        y_offset: f32,
    ) {
        let coord = layout.get(&node.id).unwrap();
        let rectangle_size = Size::new(node.width, 30.0);
        let rectangle = Path::rectangle(
            Point::new(
                coord.x as f32 - node.width / 2.0 + x,
                (coord.y as f32 - 15.0) + y,
            ),
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
            position: Point::new(coord.x as f32 + x, coord.y as f32 + y),
            color: Color::BLACK,
            size: Pixels(14.0),
            horizontal_alignment: Horizontal::Center,
            vertical_alignment: Vertical::Center,
            ..Default::default()
        });

        // Draw children recursively
        let num_children = node.children.len();
        if num_children > 0 {
            let step = (2.0 * x_offset) / (num_children as f32);
            let mut child_x = x - x_offset + step / 2.0;

            for child in &node.children {
                let child_coord = layout.get(&child.id).unwrap();

                // Draw line to child
                let line = Path::line(
                    Point::new(coord.x as f32 + x, coord.y as f32 + y + 15.0),
                    Point::new(
                        child_x + child_coord.x as f32,
                        child_coord.y as f32 + y + y_offset - 15.0,
                    ),
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
                    layout,
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
        Self { id }
    }
}

extern crate reingold_tilford;

#[derive(Debug, Clone)]
pub struct ASTCanvas {
    root: Option<Node>,
    layout: Option<HashMap<usize, reingold_tilford::Coordinate>>,
    scaling: f32,
    translation: Vector,
}

#[derive(Debug, Clone)]
pub struct Tree;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub children: Vec<Node>,
    value: String,
    width: f32,
}

impl Node {
    fn new(value: String) -> Self {
        let width = 10.0 + 7.0 * value.len() as f32;
        static VAR_COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self {
            id: VAR_COUNTER.fetch_add(1, Ordering::Relaxed),
            value,
            children: vec![],
            width,
        }
    }
}

impl<'n> reingold_tilford::NodeInfo<&'n Node> for Tree {
    type Key = usize;

    fn key(&self, node: &'n Node) -> Self::Key {
        node.id
    }

    fn children(&self, node: &'n Node) -> reingold_tilford::SmallVec<&'n Node> {
        node.children.iter().collect()
    }

    fn dimensions(&self, node: &'n Node) -> reingold_tilford::Dimensions {
        Dimensions {
            top: 15.0,
            right: (node.width / 2.0) as f64,
            bottom: 15.0,
            left: (node.width / 2.0) as f64,
        }
    }
}
