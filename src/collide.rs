use bevy::math::{Vec2, Vec3, Quat};

pub struct Box {
    pub centre: Vec2,
    pub size : Vec2,
    pub rotation : Quat
}

pub struct Line {
    start : Vec2,
    end : Vec2
}

pub struct Circle {
    pub centre : Vec2,
    pub radius : f32
}

impl Box {
    pub fn collide(self : &Self, circle : Circle) -> bool {
        self.lines().iter().all(
            |line| line.collide(&circle)
        )
    }

    fn lines(self : &Self) -> [Line; 4] {
        let local_x3 = self.rotation * Vec3::new(self.size.x, 0.0, 0.0);
        let local_y3 = self.rotation * Vec3::new(self.size.x, 0.0, 0.0);
        let local_x = Vec2::new(local_x3.x, local_x3.y);
        let local_y = Vec2::new(local_y3.x, local_y3.y);
        let points : [Vec2; 4] = [
            self.centre + local_x + local_y,
            self.centre + local_x - local_y,
            self.centre - local_x - local_y,
            self.centre - local_x + local_y
        ];
        [
            Line {start : points[0], end : points[1]},
            Line {start : points[1], end : points[2]},
            Line {start : points[2], end : points[3]},
            Line {start : points[3], end : points[0]}
        ]
    }
}

impl Line {
    pub fn collide(self : &Self, circle : &Circle) -> bool {
        let radius_sq = circle.radius * circle.radius;
        self.nearest_point(&circle.centre).distance_squared(circle.centre) < radius_sq
    }

    fn nearest_point(self : &Self, pt : &Vec2) -> Vec2 {
        if self.start.distance_squared(self.end) < 1e-6 {
            self.start.lerp(self.end, 0.5);
        }
        let diff = self.end - self.start;
        let delta = diff.dot(*pt - self.start) / diff.length_squared();
        self.start.lerp(self.end, delta.clamp(0.0, 1.0))
    }
}