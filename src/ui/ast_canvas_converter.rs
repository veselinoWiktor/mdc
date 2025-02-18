use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use iced::{mouse, widget, Color, Event, Pixels, Point, Rectangle, Size, Theme, Vector};
use iced::alignment::{Horizontal, Vertical};
use iced::mouse::Cursor;
use iced::widget::canvas;
use iced::widget::canvas::{Frame, Geometry, Path, Program, Stroke, Style, Text};
use reingold_tilford::Dimensions;
use crate::storage::ast::{AstBinaryOp, AstBlockItem, AstDeclaration, AstExpression, AstFunctionDefinition, AstProgram, AstStatement, AstUnaryOp};
use crate::ui::ast_visualizer::Message;

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

extern crate reingold_tilford;

#[derive(Debug, Clone)]
pub struct ASTCanvas {
    pub root: Option<Node>,
    pub layout: Option<HashMap<usize, reingold_tilford::Coordinate>>,
    pub scaling: f32,
    pub translation: Vector,
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

pub(crate) fn convert_into_ast_canvas(ast: &AstProgram) -> Node {
    let mut root = Node::new("Program(function_definition)".to_string());

    match ast {
        AstProgram::Program(function) => {
            root.children.push(convert_ast_function(function))
        }
    }

    root
}

fn convert_ast_function(ast_function: &AstFunctionDefinition) -> Node {
    match ast_function {
        AstFunctionDefinition::Function(identifier, body) => {
            let mut function = Node::new(format!("Function('{}', body)", identifier));

            for block_item in body {
                match block_item {
                    AstBlockItem::Declaration(declaration) => {
                        function.children.push(convert_ast_declaration(declaration))
                    },
                    AstBlockItem::Statement(statement) => {
                        function.children.push(convert_ast_statement(statement))
                    }
                }
            }

            function
        }
    }
}

fn convert_ast_declaration(ast_declaration: &AstDeclaration) -> Node {
    match ast_declaration {
        AstDeclaration::Declaration(identifier, expression) => {
            match expression {
                Some(expr) => {
                    let mut declaration = Node::new(format!("{}=exp", identifier));
                    declaration.children.push(convert_ast_expression(expr));

                    declaration
                },
                None => {
                    let declaration = Node::new(format!("{}", identifier));
                    declaration
                }
            }
        }
    }
}

fn convert_ast_statement(ast_statement: &AstStatement) -> Node {
    match ast_statement {
        AstStatement::Return(expr) => {
            let mut statement = Node::new("Return(exp)".to_string());
            statement.children.push(convert_ast_expression(expr));

            statement
        },
        AstStatement::Expression(expr) => {
            let mut expression = Node::new("ExprStatement(exp)".to_string());
            expression.children.push(convert_ast_expression(expr));

            expression
        },
        AstStatement::Null => {
            Node::new("Null".to_string())
        }
    }
}

fn convert_ast_expression(ast_expression: &AstExpression) -> Node {
    match ast_expression {
        AstExpression::Constant(num) => {
            Node::new(format!("Constant({})", num))
        }
        AstExpression::Binary(operator, left, right) => {
            let mut binary_node = Node::new("Binary(operator, left, right)".to_string());
            binary_node.children.push(convert_ast_expression(left));

            let operator_node = match operator {
                AstBinaryOp::Add => Node::new("Add".into()),
                AstBinaryOp::And => Node::new("And".into()),
                AstBinaryOp::Divide => Node::new("Divide".into()),
                AstBinaryOp::Equal => Node::new("Equal".into()),
                AstBinaryOp::GreaterOrEqual => Node::new("GreaterOrEqual".into()),
                AstBinaryOp::GreaterThan => Node::new("GreaterThan".into()),
                AstBinaryOp::LessOrEqual => Node::new("LessOrEqual".into()),
                AstBinaryOp::LessThan => Node::new("LessThan".into()),
                AstBinaryOp::Multiply => Node::new("Multiply".into()),
                AstBinaryOp::NotEqual => Node::new("NotEqual".into()),
                AstBinaryOp::Or => Node::new("Or".into()),
                AstBinaryOp::Remainder => Node::new("Remainder".into()),
                AstBinaryOp::Subtract => Node::new("Subtract".into())
            };

            binary_node.children.push(operator_node);
            binary_node.children.push(convert_ast_expression(right));

            binary_node
        }
        AstExpression::Unary(operator, expr) => {
            let mut unary_node = Node::new("Unary(operator, expr)".into());

            let operator_node = match operator {
                AstUnaryOp::Not => Node::new("Not".into()),
                AstUnaryOp::Complement => Node::new("Complement".into()),
                AstUnaryOp::Negate => Node::new("Negate".into()),
            };

            unary_node.children.push(operator_node);
            unary_node.children.push(convert_ast_expression(expr));

            unary_node
        },
        AstExpression::Var(identifier) => {
            Node::new(format!("Var({})", identifier))
        },
        AstExpression::Assignment(identifier, expression) => {
            let mut assignment = Node::new(format!("Assignment(ident, expr)"));
            assignment.children.push(convert_ast_expression(&*identifier));
            assignment.children.push(convert_ast_expression(&*expression));

            assignment
        }
    }
}