use std::path::PathBuf;
use std::sync::Arc;
use iced::{highlighter, widget, Center, Element, Fill, Font, Task, Theme, Vector};
use iced::widget::{container, horizontal_space, pane_grid, pick_list, responsive, text, text_editor, toggler, PaneGrid};
use iced::widget::text_editor::{Action, Edit};
use crate::compiler::assembly::codegen::gen;
use crate::compiler::assembly::instruction_fixup::fixup_program;
use crate::compiler::assembly::replace_pseudos::replace_pseudos;
use crate::compiler::emit::emit_assembly;
use crate::compiler::parser::parse_program;
use crate::compiler::semantics::variable_resolution::resolve_program;
use crate::compiler::tackygen::emit_tacky;
use crate::compiler::tokenizer::tokenize;
use crate::ui::{style};
use crate::ui::ast_canvas_converter::{convert_into_ast_canvas, ASTCanvas, Tree};
use crate::ui::utils::{open_file, save_file, Error};
use crate::ui::views::{action, view_canvas, view_editor, view_loader};

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
pub enum Message {
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

                // resolve ast semantics
                let resolved_ast = match resolve_program(ast) {
                    Ok(resolved_ast) => resolved_ast,
                    Err(_) => {
                        self.is_ast_valid = false;
                        return Task::none();
                    }
                };

                // generate ast canvas
                let root = convert_into_ast_canvas(&resolved_ast);

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
                _ => unreachable!(),
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

#[derive(Clone, Copy)]
struct Pane {
    id: usize,
}

impl Pane {
    fn new(id: usize) -> Pane {
        Self { id }
    }
}