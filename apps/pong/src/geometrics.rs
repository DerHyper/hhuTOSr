extern crate alloc;

use alloc::vec::Vec;
use usrlib::consts::{CGA_COLUMNS, CGA_ROWS};

use crate::{frame::Frame, utils};

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
        if (x < line1.start.x.min(line1.end.x) || x > line1.start.x.max(line1.end.x)) ||
           (y < line1.start.y.min(line1.end.y) || y > line1.start.y.max(line1.end.y)) ||
           (x < line2.start.x.min(line2.end.x) || x > line2.start.x.max(line2.end.x)) ||
           (y < line2.start.y.min(line2.end.y) || y > line2.start.y.max(line2.end.y)) {
            return None; // Intersection is outside the line segments
        }

        return Some(Point::new(x, y));
    }
}

impl Renderable for Line {
    fn draw(&self, frame: &mut Frame, symbol: char, color: usrlib::user_cga::Color) {
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;
        let m = dx/dy;
        for x in (self.start.x as usize)..(self.end.x as usize) {
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
    pub fn collides_with_line(&self, line: &Line) -> Vec<Point> {
        let mut res = Vec::new();

        // Check if either of the line's endpoints are inside the rectangle
        if self.contains(&line.start) {
            res.push(Point::new(line.start.x, line.start.y));
            return res;
        }
        if self.contains(&line.end) {
            res.push(Point::new(line.end.x, line.end.y));
            return res;
        }

        // Check if the line intersects with any of the rectangle's edges
        let rect_edges = [
            Line::new(Point::new(self.pivot.x - self.width/2.0, self.pivot.y - self.height/2.0), Point::new(self.pivot.x + self.width/2.0, self.pivot.y - self.height/2.0)), // Top edge
            Line::new(Point::new(self.pivot.x + self.width/2.0, self.pivot.y - self.height/2.0), Point::new(self.pivot.x + self.width/2.0, self.pivot.y + self.height/2.0)), // Right edge
            Line::new(Point::new(self.pivot.x + self.width/2.0, self.pivot.y + self.height/2.0), Point::new(self.pivot.x - self.width/2.0, self.pivot.y + self.height/2.0)), // Bottom edge
            Line::new(Point::new(self.pivot.x - self.width/2.0, self.pivot.y + self.height/2.0), Point::new(self.pivot.x - self.width/2.0, self.pivot.y - self.height/2.0))  // Left edge
        ];

        for edge in &rect_edges {
            if let Some(intersection) = Line::lines_intersect(line, edge) {
                res.push(intersection);
            }
        }

        res.sort_by( |a, b| {
            let dist_a = Point::distance(&line.start, a);
            let dist_b = Point::distance(&line.start, b);
            dist_a.partial_cmp(&dist_b).unwrap()
        });

        return res;
    }
}

impl Renderable for Rect {
    fn draw(&self, frame: &mut Frame, symbol: char, color: usrlib::user_cga::Color) {
        let left_bound = (self.pivot.x - self.width/2.0) as usize;
        let right_bound = (self.pivot.x + self.width/2.0) as usize;
        let upper_bound = (self.pivot.y - self.height/2.0) as usize;
        let lower_bound = (self.pivot.y + self.height/2.0) as usize;

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