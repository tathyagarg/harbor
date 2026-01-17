use crate::{css::colors::UsedColor, render::text::Vertex};

/// Gives vertices with 3D positions for a rectangle starting at (0,0) to be built with a triangle
/// list
pub fn rectangle_vertices(width: f32, height: f32, color: UsedColor) -> Vec<Vertex> {
    vec![
        Vertex {
            position: [0.0, 0.0, 0.0],
            color,
        },
        Vertex {
            position: [width, 0.0, 0.0],
            color,
        },
        Vertex {
            position: [width, height, 0.0],
            color,
        },
        Vertex {
            position: [0.0, 0.0, 0.0],
            color,
        },
        Vertex {
            position: [width, height, 0.0],
            color,
        },
        Vertex {
            position: [0.0, height, 0.0],
            color,
        },
    ]
}

/// positions are in ndc
pub fn rectangle_at(x: f32, y: f32, width: f32, height: f32, color: UsedColor) -> Vec<Vertex> {
    vec![
        Vertex {
            position: [x, y, 0.0],
            color,
        },
        Vertex {
            position: [x + width, y, 0.0],
            color,
        },
        Vertex {
            position: [x + width, y - height, 0.0],
            color,
        },
        Vertex {
            position: [x, y, 0.0],
            color,
        },
        Vertex {
            position: [x + width, y - height, 0.0],
            color,
        },
        Vertex {
            position: [x, y - height, 0.0],
            color,
        },
    ]
}
