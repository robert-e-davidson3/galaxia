use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct RectangularArea {
    pub width: f32,
    pub height: f32,
}

impl RectangularArea {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn new_square(size: f32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }

    pub fn left(&self) -> f32 {
        -self.width / 2.0
    }

    pub fn right(&self) -> f32 {
        self.width / 2.0
    }

    pub fn top(&self) -> f32 {
        self.height / 2.0
    }

    pub fn bottom(&self) -> f32 {
        -self.height / 2.0
    }

    pub fn dimensions(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    pub fn dimensions3(&self) -> Vec3 {
        Vec3::new(self.width, self.height, 0.0)
    }

    pub fn is_within(&self, position: Vec2, center: Vec2) -> bool {
        let min_x = center.x + self.left();
        let max_x = center.x + self.right();
        let min_y = center.y + self.bottom();
        let max_y = center.y + self.top();
        position.x >= min_x
            && position.x <= max_x
            && position.y >= min_y
            && position.y <= max_y
    }

    pub fn clamp(&self, position: Vec2, center: Vec2) -> Vec2 {
        let min_x = center.x + self.left();
        let max_x = center.x + self.right();
        let min_y = center.y + self.bottom();
        let max_y = center.y + self.top();

        Vec2::new(
            position.x.max(min_x).min(max_x),
            position.y.max(min_y).min(max_y),
        )
    }

    pub fn grow(&self, x: f32, y: f32) -> Self {
        Self {
            width: self.width + x,
            height: self.height + y,
        }
    }
}

impl From<RectangularArea> for Vec2 {
    fn from(area: RectangularArea) -> Self {
        area.dimensions()
    }
}

impl From<RectangularArea> for Collider {
    fn from(area: RectangularArea) -> Self {
        Collider::cuboid(area.width / 2.0, area.height / 2.0)
    }
}

#[derive(Debug, Default, Copy, Clone, Component)]
pub struct CircularArea {
    pub radius: f32,
}

impl CircularArea {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }

    pub fn dimensions(&self) -> Vec2 {
        Vec2::new(self.radius * 2.0, self.radius * 2.0)
    }

    pub fn dimensions3(&self) -> Vec3 {
        Vec3::new(self.radius * 2.0, self.radius * 2.0, 0.0)
    }

    pub fn is_within(&self, position: Vec2, center: Vec2) -> bool {
        let distance_squared = position.distance_squared(center);
        distance_squared <= self.radius * self.radius
    }

    pub fn grow(&self, radius: f32) -> Self {
        Self {
            radius: self.radius + radius,
        }
    }
}

impl From<CircularArea> for Collider {
    fn from(area: CircularArea) -> Self {
        Collider::ball(area.radius)
    }
}

impl From<CircularArea> for Circle {
    fn from(area: CircularArea) -> Self {
        Circle {
            radius: area.radius,
        }
    }
}

impl From<CircularArea> for RectangularArea {
    fn from(area: CircularArea) -> Self {
        RectangularArea::new_square(area.radius * 2.0)
    }
}
