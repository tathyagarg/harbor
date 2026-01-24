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

pub fn circle_at(
    center_x: f32,
    center_y: f32,
    radius: f32,
    segments: u32,
    color: UsedColor,
    screen_width: f32,
    screen_height: f32,
) -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity((segments * 3) as usize);

    let to_clip = |x: f32, y: f32| -> [f32; 3] {
        [
            (x / screen_width) * 2.0 - 1.0,
            1.0 - (y / screen_height) * 2.0,
            0.0, // z-coordinate
        ]
    };

    let angle_increment = 2.0 * std::f32::consts::PI / segments as f32;
    let center = to_clip(center_x, center_y);

    for i in 0..segments {
        let theta1 = i as f32 * angle_increment;
        let theta2 = (i + 1) as f32 * angle_increment;

        let x1 = center_x + radius * theta1.cos();
        let y1 = center_y + radius * theta1.sin();
        let x2 = center_x + radius * theta2.cos();
        let y2 = center_y + radius * theta2.sin();

        // Triangle vertices
        vertices.push(Vertex {
            position: center,
            color,
        });
        vertices.push(Vertex {
            position: to_clip(x1, y1),
            color,
        });
        vertices.push(Vertex {
            position: to_clip(x2, y2),
            color,
        });
    }

    vertices
}
