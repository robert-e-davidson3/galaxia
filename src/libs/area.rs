use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Copy, Clone, Component)]
pub struct PositionedArea {
    pub position: Vec2, // Center point
    pub area: Area,
}

impl PositionedArea {
    pub fn new(position: Vec2, area: Area) -> Self {
        Self { position, area }
    }

    pub fn dimensions(&self) -> Vec2 {
        self.area.dimensions()
    }

    pub fn dimensions3(&self) -> Vec3 {
        self.area.dimensions3()
    }

    pub fn grow(&self, amount: f32) -> Self {
        Self {
            position: self.position,
            area: self.area.grow(amount),
        }
    }

    pub fn overlaps(&self, other: &PositionedArea) -> bool {
        let offset = other.position - self.position;
        self.area.overlaps(&other.area, offset)
    }

    pub fn is_within(&self, point: Vec2) -> bool {
        self.area.is_within(point, self.position)
    }

    pub fn nearest_edge(&self, point: Vec2) -> Vec2 {
        self.area.nearest_edge(point, self.position)
    }

    pub fn clamp(&self, point: Vec2) -> Vec2 {
        self.area.clamp(point, self.position)
    }
}

#[derive(Debug, Copy, Clone, Component)]
pub enum Area {
    Rectangular(RectangularArea),
    Circular(CircularArea),
}

impl Area {
    // Returns the size in two dimensions.
    pub fn dimensions(&self) -> Vec2 {
        match self {
            Area::Rectangular(rect) => rect.dimensions(),
            Area::Circular(circle) => circle.dimensions(),
        }
    }

    // Returns the size in three dimensions
    // For 2D areas, z is always 0.0.
    pub fn dimensions3(&self) -> Vec3 {
        match self {
            Area::Rectangular(rect) => rect.dimensions3(),
            Area::Circular(circle) => circle.dimensions3(),
        }
    }

    // Changes size of area by the amount.
    // To shrink, use negative amount.
    pub fn grow(&self, amount: f32) -> Self {
        match self {
            Area::Rectangular(rect) => {
                Area::Rectangular(rect.grow(amount, amount))
            }
            Area::Circular(circle) => Area::Circular(circle.grow(amount)),
        }
    }

    // TODO center before position

    // Returns true if the position is within the area.
    pub fn is_within(&self, position: Vec2, center: Vec2) -> bool {
        match self {
            Area::Rectangular(rect) => rect.is_within(position, center),
            Area::Circular(circle) => circle.is_within(position, center),
        }
    }

    // Returns the nearest point on the edge of the area.
    // Differs from clamp in that it always returns a point on the edge.
    // If position is virtually dead center, returns arbitrary point on the edge.
    pub fn nearest_edge(&self, position: Vec2, center: Vec2) -> Vec2 {
        match self {
            Area::Rectangular(rect) => rect.nearest_edge(position, center),
            Area::Circular(circle) => circle.nearest_edge(position, center),
        }
    }

    // Returns true if the two areas overlap.
    // Mixed types are converted to rectangular for the check.
    pub fn overlaps(&self, other: &Area, offset: Vec2) -> bool {
        match (self, other) {
            (Area::Rectangular(a), Area::Rectangular(b)) => {
                a.overlaps(b, offset)
            }
            (Area::Circular(a), Area::Circular(b)) => a.overlaps(b, offset),
            // In mixed case, convert to rectangular
            _ => {
                let rect_a: RectangularArea = self.into();
                let rect_b: RectangularArea = other.into();
                rect_a.overlaps(&rect_b, offset)
            }
        }
    }

    // Returns position if within the area, else the nearest point on the edge.
    pub fn clamp(&self, position: Vec2, center: Vec2) -> Vec2 {
        match self {
            Area::Rectangular(rect) => rect.clamp(position, center),
            Area::Circular(circle) => circle.clamp(position, center),
        }
    }
}

impl From<&Area> for RectangularArea {
    fn from(area: &Area) -> Self {
        match area {
            Area::Rectangular(rect) => *rect,
            Area::Circular(circle) => (*circle).into(),
        }
    }
}

impl From<RectangularArea> for Area {
    fn from(area: RectangularArea) -> Self {
        Area::Rectangular(area)
    }
}

impl From<CircularArea> for Area {
    fn from(area: CircularArea) -> Self {
        Area::Circular(area)
    }
}

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

    pub fn grow(&self, x: f32, y: f32) -> Self {
        Self {
            width: self.width + x,
            height: self.height + y,
        }
    }

    pub fn overlaps(&self, other: &RectangularArea, offset: Vec2) -> bool {
        !(self.left() > other.right() + offset.x
            || self.right() < other.left() + offset.x
            || self.top() < other.bottom() + offset.y
            || self.bottom() > other.top() + offset.y)
    }

    pub fn is_within(&self, point: Vec2, center: Vec2) -> bool {
        let min_x = center.x + self.left();
        let max_x = center.x + self.right();
        let min_y = center.y + self.bottom();
        let max_y = center.y + self.top();
        point.x >= min_x
            && point.x <= max_x
            && point.y >= min_y
            && point.y <= max_y
    }

    // TODO this needs to actually be nearest, not just the cardinal positions
    pub fn nearest_edge(&self, point: Vec2, center: Vec2) -> Vec2 {
        let x = if point.x < center.x {
            center.x + self.left()
        } else {
            center.x + self.right()
        };
        let y = if point.y < center.y {
            center.y + self.top()
        } else {
            center.y + self.bottom()
        };
        Vec2::new(x, y)
    }

    pub fn clamp(&self, point: Vec2, center: Vec2) -> Vec2 {
        if self.is_within(point, center) {
            point
        } else {
            self.nearest_edge(point, center)
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

impl From<RectangularArea> for CircularArea {
    fn from(area: RectangularArea) -> Self {
        CircularArea {
            radius: (area.width.max(area.height)) / 2.0,
        }
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

    pub fn grow(&self, radius: f32) -> Self {
        Self {
            radius: self.radius + radius,
        }
    }

    pub fn overlaps(&self, other: &CircularArea, offset: Vec2) -> bool {
        let distance_squared = offset.length_squared();
        let combined_radius = self.radius + other.radius;
        distance_squared <= combined_radius * combined_radius
    }

    pub fn is_within(&self, position: Vec2, center: Vec2) -> bool {
        let distance_squared = position.distance_squared(center);
        distance_squared <= self.radius * self.radius
    }

    pub fn nearest_edge(&self, position: Vec2, center: Vec2) -> Vec2 {
        let direction = position - center;
        if direction.length() <= 0.001 {
            return Vec2::new(center.x + self.radius, center.y);
        }

        let distance = direction.length();
        let scale = self.radius / distance;
        center + direction * scale
    }

    pub fn clamp(&self, position: Vec2, center: Vec2) -> Vec2 {
        if self.is_within(position, center) {
            position
        } else {
            self.nearest_edge(position, center)
        }
    }
}

impl From<CircularArea> for Vec2 {
    fn from(area: CircularArea) -> Self {
        area.dimensions()
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
