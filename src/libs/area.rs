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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;

    // --- RectangularArea ---

    #[test]
    fn rect_edges_and_dimensions() {
        let rect = RectangularArea::new(100.0, 60.0);
        assert_eq!(rect.left(), -50.0);
        assert_eq!(rect.right(), 50.0);
        assert_eq!(rect.top(), 30.0);
        assert_eq!(rect.bottom(), -30.0);
        assert_eq!(rect.dimensions(), Vec2::new(100.0, 60.0));
    }

    #[test]
    fn rect_is_within() {
        let rect = RectangularArea::new(10.0, 10.0);
        let center = Vec2::new(100.0, 100.0);
        assert!(rect.is_within(Vec2::new(102.0, 97.0), center));
        // The boundary is inclusive.
        assert!(rect.is_within(Vec2::new(105.0, 100.0), center));
        assert!(!rect.is_within(Vec2::new(106.0, 100.0), center));
    }

    #[test]
    fn rect_overlaps() {
        let a = RectangularArea::new(10.0, 10.0);
        let b = RectangularArea::new(10.0, 10.0);
        // Offset is the other area's center relative to self's center.
        assert!(a.overlaps(&b, Vec2::new(8.0, 0.0)));
        assert!(!a.overlaps(&b, Vec2::new(12.0, 0.0)));
        // Exactly touching edges still counts as overlap.
        assert!(a.overlaps(&b, Vec2::new(10.0, 0.0)));
    }

    #[test]
    fn rect_grow() {
        let rect = RectangularArea::new(10.0, 20.0).grow(4.0, 2.0);
        assert_eq!(rect.width, 14.0);
        assert_eq!(rect.height, 22.0);
    }

    #[test]
    fn rect_clamp_inside_returns_point() {
        let rect = RectangularArea::new(10.0, 10.0);
        let p = Vec2::new(2.0, -3.0);
        assert_eq!(rect.clamp(p, Vec2::ZERO), p);
    }

    #[test]
    fn rect_nearest_edge_returns_a_corner_known_limitation() {
        // Documents CURRENT behavior: nearest_edge on a rectangle snaps to a
        // corner (and inverts y) instead of the true nearest edge point. See
        // the TODO on nearest_edge / the "Area nearest-point" board task;
        // update the expected value when that is fixed.
        let rect = RectangularArea::new(10.0, 10.0);
        let lower_right = Vec2::new(3.0, -3.0);
        let edge = rect.nearest_edge(lower_right, Vec2::ZERO);
        assert_eq!(edge, Vec2::new(5.0, 5.0));
    }

    // --- CircularArea ---

    #[test]
    fn circle_is_within() {
        let circle = CircularArea::new(5.0);
        let center = Vec2::new(2.0, 3.0);
        assert!(circle.is_within(Vec2::new(2.0, 6.0), center));
        // On the edge counts as within.
        assert!(circle.is_within(Vec2::new(7.0, 3.0), center));
        assert!(!circle.is_within(Vec2::new(8.0, 3.0), center));
    }

    #[test]
    fn circle_overlaps() {
        let a = CircularArea::new(5.0);
        let b = CircularArea::new(5.0);
        assert!(a.overlaps(&b, Vec2::new(9.0, 0.0)));
        // Touching (distance == sum of radii) counts as overlap.
        assert!(a.overlaps(&b, Vec2::new(10.0, 0.0)));
        assert!(!a.overlaps(&b, Vec2::new(10.5, 0.0)));
    }

    #[test]
    fn circle_nearest_edge_projects_onto_circle() {
        let circle = CircularArea::new(5.0);
        let edge = circle.nearest_edge(Vec2::new(10.0, 0.0), Vec2::ZERO);
        assert!(edge.abs_diff_eq(Vec2::new(5.0, 0.0), 1e-4));
        // A point at the exact center returns an arbitrary edge point.
        let from_center = circle.nearest_edge(Vec2::ZERO, Vec2::ZERO);
        assert!(from_center.abs_diff_eq(Vec2::new(5.0, 0.0), 1e-4));
    }

    #[test]
    fn circle_clamp() {
        let circle = CircularArea::new(5.0);
        let inside = Vec2::new(1.0, 1.0);
        assert_eq!(circle.clamp(inside, Vec2::ZERO), inside);
        let clamped = circle.clamp(Vec2::new(20.0, 0.0), Vec2::ZERO);
        assert!(clamped.abs_diff_eq(Vec2::new(5.0, 0.0), 1e-4));
    }

    #[test]
    fn circle_grow_and_dimensions() {
        let circle = CircularArea::new(5.0).grow(3.0);
        assert_eq!(circle.radius, 8.0);
        assert_eq!(circle.dimensions(), Vec2::new(16.0, 16.0));
    }

    // --- Conversions ---

    #[test]
    fn rectangular_from_circular_is_bounding_square() {
        let rect: RectangularArea = CircularArea::new(5.0).into();
        assert_eq!(rect.width, 10.0);
        assert_eq!(rect.height, 10.0);
    }

    #[test]
    fn circular_from_rectangular_uses_larger_half_dimension() {
        let circle: CircularArea = RectangularArea::new(100.0, 60.0).into();
        assert_eq!(circle.radius, 50.0);
    }

    #[test]
    fn area_overlaps_mixed_converts_to_rectangular() {
        let rect = Area::Rectangular(RectangularArea::new(10.0, 10.0));
        let circle = Area::Circular(CircularArea::new(5.0));
        // The circle becomes a 10x10 square, matching rect_overlaps.
        assert!(rect.overlaps(&circle, Vec2::new(8.0, 0.0)));
        assert!(!rect.overlaps(&circle, Vec2::new(12.0, 0.0)));
    }
}
