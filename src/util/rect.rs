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
    let new_min = Vec2::new(transform.translation.x, transform.translation.y);
    let new_max = new_min + Vec2::new(transform.scale.x, transform.scale.y);
    Rect {
        min: new_min,
        max: new_max,
    }
}

pub fn rect_contains_point(rect: &Rect, point: &Vec2) -> bool {
    rect.min.x < point.x && point.x < rect.max.x && rect.min.y < point.y && point.y < rect.max.y
}

pub fn rect_contains_rect(rect: &Rect, other: &Rect) -> bool {
    let start = other.min;
    let diag = other.max - other.min;
    let width = diag.project_onto(Vec2::X);
    let height = diag.project_onto(Vec2::Y);
    let end = other.max;
    // check all four corners
    rect_contains_point(rect, &start)
        && rect_contains_point(rect, &(start + width))
        && rect_contains_point(rect, &(start + height))
        && rect_contains_point(rect, &end)
}
