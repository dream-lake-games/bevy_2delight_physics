use bevy::prelude::*;

pub type HBoxMarker = u32;

/// HBOX?????
#[derive(Clone, Debug, Reflect)]
pub struct HBox {
    offset: Vec2,
    size: UVec2,
    marker: HBoxMarker,
}
impl HBox {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            offset: default(),
            size: UVec2::new(w, h),
            marker: default(),
        }
    }
    pub fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset.x = x;
        self.offset.y = y;
        self
    }
    pub fn with_size(mut self, w: u32, h: u32) -> Self {
        self.size.x = w;
        self.size.y = h;
        self
    }
    pub fn with_marker(mut self, marker: HBoxMarker) -> Self {
        self.marker = marker;
        self
    }

    pub fn translated(&self, x: f32, y: f32) -> Self {
        Self {
            offset: self.offset + Vec2::new(x, y),
            size: self.size.clone(), // Not strictly needed but nice for clarity
            marker: self.marker,
        }
    }
    pub fn min_x(&self) -> f32 {
        self.offset.x - self.size.x as f32 / 2.0
    }
    pub fn max_x(&self) -> f32 {
        self.offset.x + self.size.x as f32 / 2.0
    }
    pub fn min_y(&self) -> f32 {
        self.offset.y - self.size.y as f32 / 2.0
    }
    pub fn max_y(&self) -> f32 {
        self.offset.y + self.size.y as f32 / 2.0
    }
    pub fn get_offset(&self) -> Vec2 {
        self.offset
    }
    pub fn get_size(&self) -> UVec2 {
        self.size
    }
    pub fn get_marker(&self) -> HBoxMarker {
        self.marker
    }
}

// I don't care that this is super verbose, and maybe inefficient. I want it to be correct.
// Can performance engineer later if needed.
impl HBox {
    /// Manhattan distance to another hitbox
    pub fn manhattan_distance(&self, rhs: &Self) -> f32 {
        let fsize = self.size.as_vec2();
        let my_x_min = self.offset.x - fsize.x / 2.0;
        let my_x_max = self.offset.x + fsize.x / 2.0;
        let my_y_min = self.offset.y - fsize.y / 2.0;
        let my_y_max = self.offset.y + fsize.y / 2.0;

        let ofsize = rhs.size.as_vec2();
        let o_x_min = rhs.offset.x - ofsize.x / 2.0;
        let o_x_max = rhs.offset.x + ofsize.x / 2.0;
        let o_y_min = rhs.offset.y - ofsize.y / 2.0;
        let o_y_max = rhs.offset.y + ofsize.y / 2.0;

        let x_dist = (my_x_min - o_x_max).abs().min((o_x_min - my_x_max).abs());
        let y_dist = (my_y_min - o_y_max).abs().min((o_y_min - my_y_max).abs());

        x_dist + y_dist
    }

    /// Manhattan distance to a point
    pub fn manhattan_distance_to_point(&self, point: Vec2) -> f32 {
        let fsize = self.size.as_vec2();
        let my_x_min = self.offset.x - fsize.x / 2.0;
        let my_x_max = self.offset.x + fsize.x / 2.0;
        let my_y_min = self.offset.y - fsize.y / 2.0;
        let my_y_max = self.offset.y + fsize.y / 2.0;

        let x_dist = if point.x >= my_x_min && point.x <= my_x_max {
            0.0
        } else {
            (my_x_min - point.x).abs().min((point.x - my_x_max).abs())
        };

        let y_dist = if point.y >= my_y_min && point.y <= my_y_max {
            0.0
        } else {
            (my_y_min - point.y).abs().min((point.y - my_y_max).abs())
        };

        x_dist + y_dist
    }

    /// Area overlapping with another hitbox
    /// NOTE: Assumes they are overlapping
    pub fn area_overlapping_assuming_overlap(&self, rhs: &Self) -> f32 {
        let fsize = self.size.as_vec2();
        let my_x_min = self.offset.x - fsize.x / 2.0;
        let my_x_max = self.offset.x + fsize.x / 2.0;
        let my_y_min = self.offset.y - fsize.y / 2.0;
        let my_y_max = self.offset.y + fsize.y / 2.0;

        let ofsize = rhs.size.as_vec2();
        let ox_min = rhs.offset.x - ofsize.x / 2.0;
        let ox_max = rhs.offset.x + ofsize.x / 2.0;
        let oy_min = rhs.offset.y - ofsize.y / 2.0;
        let oy_max = rhs.offset.y + ofsize.y / 2.0;

        let x_overlap = (my_x_min - ox_max).abs().min((ox_min - my_x_max).abs());
        let y_overlap = (my_y_min - oy_max).abs().min((oy_min - my_y_max).abs());

        x_overlap * y_overlap
    }

    /// Returns if the two hitboxes overlap
    pub fn overlaps_with(&self, rhs: &Self) -> bool {
        let fsize = self.size.as_vec2();
        let my_x_min = self.offset.x - fsize.x / 2.0;
        let my_x_max = self.offset.x + fsize.x / 2.0;
        let my_y_min = self.offset.y - fsize.y / 2.0;
        let my_y_max = self.offset.y + fsize.y / 2.0;

        let ofsize = rhs.size.as_vec2();
        let dont_overlap_x = (my_x_max <= rhs.offset.x - ofsize.x / 2.0)
            || (rhs.offset.x + ofsize.x / 2.0 <= my_x_min);
        let dont_overlap_y = (my_y_max <= rhs.offset.y - ofsize.y / 2.0)
            || (rhs.offset.y + ofsize.y / 2.0 <= my_y_min);

        let overlaps = !dont_overlap_x && !dont_overlap_y;
        overlaps
    }

    /// If the two hitboxes overlap, return the vec that you need to move self to get it out of rhs
    pub fn get_push_out(&self, rhs: &Self) -> Option<Vec2> {
        // Hear me out: this might not be that inefficient.
        // Almost everytime we call this it returns none. Better to use simpler logic to get quick no in usual case.
        if !self.overlaps_with(rhs) {
            return None;
        }

        let fsize = self.size.as_vec2();
        let my_x_min = self.offset.x - fsize.x / 2.0;
        let my_x_max = self.offset.x + fsize.x / 2.0;
        let my_y_min = self.offset.y - fsize.y / 2.0;
        let my_y_max = self.offset.y + fsize.y / 2.0;

        let ofsize = rhs.size.as_vec2();
        let ox_min = rhs.offset.x - ofsize.x / 2.0;
        let ox_max = rhs.offset.x + ofsize.x / 2.0;
        let oy_min = rhs.offset.y - ofsize.y / 2.0;
        let oy_max = rhs.offset.y + ofsize.y / 2.0;

        let needed_left_push = (ox_min - my_x_max).min(0.0);
        let needed_right_push = (ox_max - my_x_min).max(0.0);
        let needed_down_push = (oy_min - my_y_max).min(0.0);
        let needed_up_push = (oy_max - my_y_min).max(0.0);

        let needed_hor_push = if needed_left_push.abs() < needed_right_push.abs() {
            needed_left_push
        } else {
            needed_right_push
        };
        let needed_ver_push = if needed_down_push.abs() < needed_up_push.abs() {
            needed_down_push
        } else {
            needed_up_push
        };

        let push = if needed_hor_push.abs() < needed_ver_push.abs() {
            Vec2::new(needed_hor_push, 0.0)
        } else {
            Vec2::new(0.0, needed_ver_push)
        };

        Some(push)
    }
}
