use crate::pose::{PlayerInput, Skeleton};

/// Cylindrical / circular boundary filter on the 2D normalized camera plane.
///
/// Used to discard background bystanders, phantom noise detections, or
/// tracking artifacts that fall outside the calibrated physical play mat or floor circle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloorBoundary {
    /// Normalized center X coordinate [0.0, 1.0].
    pub center_x: f32,
    /// Normalized center Y coordinate [0.0, 1.0].
    pub center_y: f32,
    /// Normalized boundary radius [0.0, 1.0].
    pub radius: f32,
}

/// Alias for FloorBoundary matching contract terminology.
pub type FloorCylinderBoundary = FloorBoundary;

impl Default for FloorBoundary {
    fn default() -> Self {
        Self {
            center_x: 0.5,
            center_y: 0.5,
            radius: 0.4,
        }
    }
}

impl FloorBoundary {
    /// Creates a new floor boundary with specified center coordinates and radius.
    pub fn new(center_x: f32, center_y: f32, radius: f32) -> Self {
        Self {
            center_x,
            center_y,
            radius: radius.max(0.0),
        }
    }

    /// Checks if a 2D point (x, y) falls inside the circular boundary.
    #[inline]
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        (dx * dx + dy * dy) <= (self.radius * self.radius)
    }

    /// Checks if a skeleton's estimated root position falls inside the boundary.
    #[inline]
    pub fn contains_skeleton(&self, skeleton: &Skeleton) -> bool {
        let (rx, ry) = skeleton.root_position();
        self.contains_point(rx, ry)
    }

    /// Filters a list of skeletons, keeping only those whose root position is within the boundary.
    pub fn filter_skeletons(&self, skeletons: &[Skeleton]) -> Vec<Skeleton> {
        skeletons
            .iter()
            .filter(|s| self.contains_skeleton(s))
            .cloned()
            .collect()
    }

    /// Filters a `PlayerInput` frame in place, removing players outside the boundary.
    pub fn filter_player_input(&self, mut input: PlayerInput) -> PlayerInput {
        input.players.retain(|s| self.contains_skeleton(s));
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pose::{Keypoint, COCO_LEFT_HIP, COCO_RIGHT_HIP, KEYPOINT_COUNT};

    fn make_skeleton_at(id: u8, x: f32, y: f32) -> Skeleton {
        let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
        keypoints[COCO_LEFT_HIP] = Keypoint::new(x, y, 0.9);
        keypoints[COCO_RIGHT_HIP] = Keypoint::new(x, y, 0.9);
        Skeleton::new(id, keypoints)
    }

    #[test]
    fn test_floor_boundary_contains_point() {
        let boundary = FloorBoundary::new(0.5, 0.5, 0.3);

        assert!(boundary.contains_point(0.5, 0.5)); // Center
        assert!(boundary.contains_point(0.7, 0.5)); // Inside (distance 0.2 < 0.3)
        assert!(boundary.contains_point(0.5, 0.8)); // On boundary (distance 0.3)
        assert!(!boundary.contains_point(0.9, 0.9)); // Outside
    }

    #[test]
    fn test_floor_boundary_filters_skeletons() {
        let boundary = FloorBoundary::new(0.5, 0.5, 0.3);

        let inside_player = make_skeleton_at(1, 0.5, 0.5);
        let outside_phantom = make_skeleton_at(2, 0.95, 0.95);

        assert!(boundary.contains_skeleton(&inside_player));
        assert!(!boundary.contains_skeleton(&outside_phantom));

        let skeletons = vec![inside_player.clone(), outside_phantom.clone()];
        let filtered = boundary.filter_skeletons(&skeletons);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_filter_player_input() {
        let boundary = FloorBoundary::new(0.5, 0.5, 0.3);

        let inside_player = make_skeleton_at(1, 0.5, 0.5);
        let outside_player = make_skeleton_at(2, 0.1, 0.1);

        let input = PlayerInput::new(vec![inside_player, outside_player], 1.0);
        let filtered_input = boundary.filter_player_input(input);

        assert_eq!(filtered_input.players.len(), 1);
        assert_eq!(filtered_input.players[0].id, 1);
        assert_eq!(filtered_input.timestamp, 1.0);
    }
}
