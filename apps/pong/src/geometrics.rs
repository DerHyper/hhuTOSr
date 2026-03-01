extern crate alloc;

use core::ops::{Add, Div, Mul};

use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};

use crate::{frame::Frame, utils};

pub const EPS: f32 = 0.0001;

pub trait Renderable {
    fn draw(&self, frame: &mut Frame, symbol: char, color: usrlib::user_cga::Color);
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Point {
        Point {
            x,
            y
        }
    }

    /// Returns the distance between two points.
    /// Using the Pythagorean theorem
    pub fn distance(point1: &Point, point2: &Point) -> f32 {
        utils::sqrt(utils::square(point1.x - point2.x) + utils::square(point1.y - point2.y))
    }

    /// Returns the distance from (0,0)
    pub fn length(&self) -> f32 {
        let length = utils::sqrt(utils::square(self.x)+utils::square(self.y));
        length
    }

    /// Returns a Point with a distance from (0,0) of exactly one. 
    pub fn unit_vector(&self) -> Point {
        let length = self.length();
        let scaled_x = self.x/length;
        let scaled_y = self.y/length;
        let scaled_vector = Point::new(scaled_x, scaled_y);
        scaled_vector
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Mul<f32> for Point {
    type Output = Point;

    fn mul(self, rhs: f32) -> Self::Output {
        Point::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f32> for Point {
    type Output = Point;

    fn div(self, rhs: f32) -> Self::Output {
        Point::new(self.x / rhs, self.y / rhs)
    }
}

impl Renderable for Point {
    fn draw(&self, frame: &mut Frame, symbol: char, color: usrlib::user_cga::Color) {
        let y = utils::round(self.y);
        let x = utils::round(self.x);
        frame.draw_point(x, y, symbol, color);
    }
}

pub struct Line {
    pub start: Point,
    pub end: Point
}

impl Line {
    pub const fn new(start: Point, end: Point) -> Line {
        Line {
            start,
            end
        }
    }

    // TODO: Check if this can be optimized by using a different algorithm, e.g. by checking the orientation of the points.
    /// Returns the intersection point of two lines, if it exists and is within the line segments. Otherwise, returns None.
    pub fn lines_intersect(line1: &Line, line2: &Line) -> Option<Point> {
        // Represent lines in the form ax + by = c
        let a1 = line1.end.y - line1.start.y;
        let b1 = line1.start.x - line1.end.x;
        let c1 = a1 * line1.start.x + b1 * line1.start.y;

        let a2 = line2.end.y - line2.start.y;
        let b2 = line2.start.x - line2.end.x;
        let c2 = a2 * line2.start.x + b2 * line2.start.y;

        // Calculate intersection
        // Source: https://en.wikipedia.org/wiki/Intersection_(geometry)#Two_lines
        let determinant = a1 * b2 - a2 * b1;
        if determinant == 0.0 {
            return None; // Lines are parallel
        } 

        let x = (b2 * c1 - b1 * c2) / determinant;
        let y = (a1 * c2 - a2 * c1) / determinant;
        if (x < line1.start.x.min(line1.end.x) - EPS || x > line1.start.x.max(line1.end.x) + EPS) ||
           (y < line1.start.y.min(line1.end.y) - EPS || y > line1.start.y.max(line1.end.y) + EPS) ||
           (x < line2.start.x.min(line2.end.x) - EPS || x > line2.start.x.max(line2.end.x) + EPS) ||
           (y < line2.start.y.min(line2.end.y) - EPS || y > line2.start.y.max(line2.end.y) + EPS) {
            return None; // Intersection is outside the line segments
        }

        return Some(Point::new(x, y));
    }

    pub fn length(&self) -> f32 {
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;
        let length = utils::sqrt(utils::square(dx)+utils::square(dy));
        length
    }

    pub fn to_directional_vector(&self) -> Point {
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;
        Point::new(dx, dy)
    }
}

impl Renderable for Line {
    fn draw(&self, frame: &mut Frame, symbol: char, color: usrlib::user_cga::Color) {
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;
        let m = dx/dy;
        for x in (utils::round(self.start.x))..(utils::round(self.end.x)) {
            let y = (self.start.y + (x as f32 - self.start.x) / m) as usize;
            frame.draw_point(x, y, symbol, color);
        }
    }
}

pub struct Rect {
    pub pivot: Point,
    pub width: f32,
    pub height: f32
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            pivot: Point::new(x, y),
            width,
            height
        }
    }

    pub const fn new_from_point(point: Point, width: f32, height: f32) -> Rect {
        Rect {
            pivot: point,
            width,
            height
        }
    }

    pub fn contains(&self, point: &Point) -> bool {
        let left_bound = self.pivot.x - self.width/2.0;
        let right_bound = self.pivot.x + self.width/2.0;
        let upper_bound = self.pivot.y - self.height/2.0;
        let lower_bound = self.pivot.y + self.height/2.0;

        return point.x >= left_bound && point.x <= right_bound && point.y >= upper_bound && point.y <= lower_bound;
    }

    /// Checks if the line collides with the rectangle. 
    /// Returns a vector of all intersection points sorted by distance to the line's start point. 
    /// If the line starts or ends inside the rectangle, the respective endpoint is included in the result vector.
    pub fn collides_with_line(&self, line: &Line) -> [Option<Point>; 4] {// Vec<Point> {
        // Check if the line intersects with any of the rectangle's edges
        let p_up_left  = Point::new(self.pivot.x - self.width/2.0, self.pivot.y - self.height/2.0);
        let p_dn_left  = Point::new(self.pivot.x - self.width/2.0, self.pivot.y + self.height/2.0);
        let p_up_right = Point::new(self.pivot.x + self.width/2.0, self.pivot.y - self.height/2.0);
        let p_dn_right = Point::new(self.pivot.x + self.width/2.0, self.pivot.y + self.height/2.0);
        let rect_edges = [
            Line::new(p_up_left, p_up_right), // Top edge
            Line::new(p_up_right, p_dn_right), // Right edge
            Line::new(p_dn_left, p_dn_right), // Bottom edge
            Line::new(p_up_left, p_dn_left)  // Left edge
        ];

        let mut hitpoints: [Option<Point>; 4] = [None, None, None, None];
        for i in 0..rect_edges.len() {
            if let Some(intersection) = Line::lines_intersect(line, &rect_edges[i]) {
                hitpoints[i] = Some(intersection);
            }
        }

        hitpoints.sort_by( |a, b| {
            if a.is_none() && b.is_none() {
                return core::cmp::Ordering::Equal;
            } else if a.is_none() {
                return core::cmp::Ordering::Greater;
            } else if b.is_none() {
                return core::cmp::Ordering::Less;
            }

            let dist_a = Point::distance(&line.start, &a.unwrap());
            let dist_b = Point::distance(&line.start, &b.unwrap());
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        // return res;
        return hitpoints;
    }
}

impl Renderable for Rect {
    fn draw(&self, frame: &mut Frame, symbol: char, color: usrlib::user_cga::Color) {
        let left_bound = utils::round(self.pivot.x - self.width/2.0) as usize;
        let right_bound = utils::round(self.pivot.x + self.width/2.0) as usize;
        let upper_bound = utils::round(self.pivot.y - self.height/2.0) as usize;
        let lower_bound = utils::round(self.pivot.y + self.height/2.0) as usize;

        for y in upper_bound..lower_bound+1 {
            for x in left_bound..right_bound+1 {
                if x < 0 || x >= CGA_COLUMNS as usize || y < 0 || y >= CGA_ROWS as usize {
                    continue; // Skip out of bounds
                }
                frame.draw_point(x, y, symbol, color);
            }
        }
    }
}