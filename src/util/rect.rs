use bevy::{prelude::*, sprite::Rect};

pub fn partition_rect(rect: &Rect) -> [Rect; 4] {
    let start = rect.min;
    let diag = rect.max - rect.min;
    let width = diag.project_onto(Vec2::X);
    let height = diag.project_onto(Vec2::Y);
    let half_width = width / 2.;
    let half_height = height / 2.;
    let half_diag = diag / 2.;
    let center = start + half_diag;
    let end = rect.max;
    [
        Rect {
            min: start.clone(),
            max: center.clone(),
        },
        Rect {
            min: start + half_width,
            max: center + half_width,
        },
        Rect {
            min: start + half_height,
            max: center + half_height,
        },
        Rect {
            min: center.clone(),
            max: end.clone(),
        },
    ]
}

pub fn transform_to_rect(transform: &Transform) -> Rect {
    let min = transform.translation.truncate();
    let max = min + transform.scale.truncate();
    Rect { min, max }
}

pub fn magnify_rect(rect: &Rect, scale_factor: Vec2) -> Rect {
    let half_current_scale = rect.size() / 2.;
    let mid_point = rect.min + half_current_scale;
    let half_new_scale = Vec2::new(
        half_current_scale.x * scale_factor.x,
        half_current_scale.y * scale_factor.y,
    );
    let min = mid_point - half_new_scale;
    let max = mid_point + half_new_scale;
    Rect { min, max }
}

pub fn rect_contains_point(rect: &Rect, point: &Vec2) -> bool {
    rect.min.x < point.x && point.x < rect.max.x && rect.min.y < point.y && point.y < rect.max.y
}

pub fn rect_contains_rect(rect: &Rect, other: &Rect) -> bool {
    rect_contains_point(rect, &other.min) && rect_contains_point(rect, &other.max)
}
