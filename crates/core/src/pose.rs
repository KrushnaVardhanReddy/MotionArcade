use serde::{Deserialize, Serialize};

/// Standard COCO 17-keypoint landmark indices.
pub const COCO_NOSE: usize = 0;
pub const COCO_LEFT_EYE: usize = 1;
pub const COCO_RIGHT_EYE: usize = 2;
pub const COCO_LEFT_EAR: usize = 3;
pub const COCO_RIGHT_EAR: usize = 4;
pub const COCO_LEFT_SHOULDER: usize = 5;
pub const COCO_RIGHT_SHOULDER: usize = 6;
pub const COCO_LEFT_ELBOW: usize = 7;
pub const COCO_RIGHT_ELBOW: usize = 8;
pub const COCO_LEFT_WRIST: usize = 9;
pub const COCO_RIGHT_WRIST: usize = 10;
pub const COCO_LEFT_HIP: usize = 11;
pub const COCO_RIGHT_HIP: usize = 12;
pub const COCO_LEFT_KNEE: usize = 13;
pub const COCO_RIGHT_KNEE: usize = 14;
pub const COCO_LEFT_ANKLE: usize = 15;
pub const COCO_RIGHT_ANKLE: usize = 16;
pub const KEYPOINT_COUNT: usize = 17;

/// Single 2D body landmark in normalized coordinate space [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Keypoint {
    /// Horizontal position: 0.0 (left edge of camera) to 1.0 (right edge).
    pub x: f32,
    /// Vertical position: 0.0 (top edge of camera) to 1.0 (bottom edge).
    pub y: f32,
    /// Confidence score from neural model: 0.0 (unseen) to 1.0 (certain).
    pub confidence: f32,
}

impl Keypoint {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        confidence: 0.0,
    };

    pub const fn new(x: f32, y: f32, confidence: f32) -> Self {
        Self { x, y, confidence }
    }

    #[inline]
    pub fn is_confident(&self, threshold: f32) -> bool {
        self.confidence >= threshold
    }

    /// Euclidean distance to another keypoint in normalized coordinate space.
    #[inline]
    pub fn distance_to(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Keypoint {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Standard 17-point COCO pose skeleton.
/// Index Map:
/// 0: Nose, 1: Left Eye, 2: Right Eye, 3: Left Ear, 4: Right Ear,
/// 5: Left Shoulder, 6: Right Shoulder, 7: Left Elbow, 8: Right Elbow,
/// 9: Left Wrist, 10: Right Wrist, 11: Left Hip, 12: Right Hip,
/// 13: Left Knee, 14: Right Knee, 15: Left Ankle, 16: Right Ankle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skeleton {
    /// Persistent player tracking identifier (1 to 4).
    pub id: u8,
    /// Normalized body scale: estimated distance between shoulder midpoint and hip midpoint.
    pub body_scale: f32,
    /// 17 COCO keypoints.
    pub keypoints: [Keypoint; KEYPOINT_COUNT],
}

impl Skeleton {
    /// Constructs a new skeleton and automatically calculates the body_scale from torso length.
    pub fn new(id: u8, keypoints: [Keypoint; KEYPOINT_COUNT]) -> Self {
        let mut skeleton = Self {
            id,
            body_scale: 0.0,
            keypoints,
        };
        skeleton.body_scale = skeleton.torso_length();
        skeleton
    }

    /// Constructs a new skeleton with an explicit body scale.
    pub fn with_body_scale(id: u8, body_scale: f32, keypoints: [Keypoint; KEYPOINT_COUNT]) -> Self {
        Self {
            id,
            body_scale,
            keypoints,
        }
    }

    #[inline]
    pub fn nose(&self) -> Keypoint {
        self.keypoints[COCO_NOSE]
    }

    #[inline]
    pub fn left_eye(&self) -> Keypoint {
        self.keypoints[COCO_LEFT_EYE]
    }

    #[inline]
    pub fn right_eye(&self) -> Keypoint {
        self.keypoints[COCO_RIGHT_EYE]
    }

    #[inline]
    pub fn left_ear(&self) -> Keypoint {
        self.keypoints[COCO_LEFT_EAR]
    }

    #[inline]
    pub fn right_ear(&self) -> Keypoint {
        self.keypoints[COCO_RIGHT_EAR]
    }

    #[inline]
    pub fn left_shoulder(&self) -> Keypoint {
        self.keypoints[COCO_LEFT_SHOULDER]
    }

    #[inline]
    pub fn right_shoulder(&self) -> Keypoint {
        self.keypoints[COCO_RIGHT_SHOULDER]
    }

    #[inline]
    pub fn left_elbow(&self) -> Keypoint {
        self.keypoints[COCO_LEFT_ELBOW]
    }

    #[inline]
    pub fn right_elbow(&self) -> Keypoint {
        self.keypoints[COCO_RIGHT_ELBOW]
    }

    #[inline]
    pub fn left_wrist(&self) -> Keypoint {
        self.keypoints[COCO_LEFT_WRIST]
    }

    #[inline]
    pub fn right_wrist(&self) -> Keypoint {
        self.keypoints[COCO_RIGHT_WRIST]
    }

    #[inline]
    pub fn left_hip(&self) -> Keypoint {
        self.keypoints[COCO_LEFT_HIP]
    }

    #[inline]
    pub fn right_hip(&self) -> Keypoint {
        self.keypoints[COCO_RIGHT_HIP]
    }

    #[inline]
    pub fn left_knee(&self) -> Keypoint {
        self.keypoints[COCO_LEFT_KNEE]
    }

    #[inline]
    pub fn right_knee(&self) -> Keypoint {
        self.keypoints[COCO_RIGHT_KNEE]
    }

    #[inline]
    pub fn left_ankle(&self) -> Keypoint {
        self.keypoints[COCO_LEFT_ANKLE]
    }

    #[inline]
    pub fn right_ankle(&self) -> Keypoint {
        self.keypoints[COCO_RIGHT_ANKLE]
    }

    /// Normalized shoulder-to-hip torso length for body-scale invariant hitboxes.
    /// Clamped to a minimum of 0.1 to prevent division by zero in downstream scaling.
    pub fn torso_length(&self) -> f32 {
        let shoulder_y = (self.left_shoulder().y + self.right_shoulder().y) * 0.5;
        let hip_y = (self.left_hip().y + self.right_hip().y) * 0.5;
        (hip_y - shoulder_y).abs().max(0.1)
    }

    /// Check if hand is raised above nose level with sufficient confidence.
    pub fn is_hand_raised(&self, is_right: bool) -> bool {
        let hand = if is_right {
            self.right_wrist()
        } else {
            self.left_wrist()
        };
        hand.is_confident(0.3) && hand.y < self.nose().y
    }

    /// Estimate root position (ground/pelvis reference point) in normalized space.
    /// Prefers hip midpoint, falling back to shoulders or nose if hips lack confidence.
    pub fn root_position(&self) -> (f32, f32) {
        let lh = self.left_hip();
        let rh = self.right_hip();
        let lh_ok = lh.is_confident(0.1);
        let rh_ok = rh.is_confident(0.1);

        if lh_ok && rh_ok {
            ((lh.x + rh.x) * 0.5, (lh.y + rh.y) * 0.5)
        } else if lh_ok {
            (lh.x, lh.y)
        } else if rh_ok {
            (rh.x, rh.y)
        } else {
            let ls = self.left_shoulder();
            let rs = self.right_shoulder();
            let ls_ok = ls.is_confident(0.1);
            let rs_ok = rs.is_confident(0.1);

            if ls_ok && rs_ok {
                ((ls.x + rs.x) * 0.5, (ls.y + rs.y) * 0.5)
            } else if ls_ok {
                (ls.x, ls.y)
            } else if rs_ok {
                (rs.x, rs.y)
            } else {
                (self.nose().x, self.nose().y)
            }
        }
    }
}

/// Complete multi-player frame payload delivered to the game loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Active filtered players inside the play boundary (0 to 4 players).
    pub players: Vec<Skeleton>,
    /// Monotonic timestamp in seconds since application start.
    pub timestamp: f64,
}

impl PlayerInput {
    pub fn new(players: Vec<Skeleton>, timestamp: f64) -> Self {
        Self { players, timestamp }
    }

    pub fn empty(timestamp: f64) -> Self {
        Self {
            players: Vec::new(),
            timestamp,
        }
    }

    pub fn player_by_id(&self, id: u8) -> Option<&Skeleton> {
        self.players.iter().find(|p| p.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_skeleton() -> Skeleton {
        let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
        // Nose at (0.5, 0.2)
        keypoints[COCO_NOSE] = Keypoint::new(0.5, 0.2, 0.9);
        // Left & right shoulders at y = 0.3
        keypoints[COCO_LEFT_SHOULDER] = Keypoint::new(0.4, 0.3, 0.85);
        keypoints[COCO_RIGHT_SHOULDER] = Keypoint::new(0.6, 0.3, 0.85);
        // Left & right hips at y = 0.6
        keypoints[COCO_LEFT_HIP] = Keypoint::new(0.45, 0.6, 0.8);
        keypoints[COCO_RIGHT_HIP] = Keypoint::new(0.55, 0.6, 0.8);
        // Left wrist below nose (y = 0.4)
        keypoints[COCO_LEFT_WRIST] = Keypoint::new(0.3, 0.4, 0.9);
        // Right wrist raised above nose (y = 0.1)
        keypoints[COCO_RIGHT_WRIST] = Keypoint::new(0.7, 0.1, 0.9);

        Skeleton::new(1, keypoints)
    }

    #[test]
    fn test_keypoint_helpers() {
        let kp1 = Keypoint::new(0.0, 0.0, 0.8);
        let kp2 = Keypoint::new(0.3, 0.4, 0.2);

        assert!(kp1.is_confident(0.5));
        assert!(!kp2.is_confident(0.5));
        assert!((kp1.distance_to(&kp2) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_skeleton_helpers() {
        let skel = dummy_skeleton();

        assert_eq!(skel.id, 1);
        assert_eq!(skel.nose().x, 0.5);
        assert_eq!(skel.left_shoulder().x, 0.4);
        assert_eq!(skel.right_shoulder().x, 0.6);

        // Torso length: shoulder_y is 0.3, hip_y is 0.6 -> diff is 0.3
        let torso = skel.torso_length();
        assert!((torso - 0.3).abs() < 1e-5);
        assert!((skel.body_scale - 0.3).abs() < 1e-5);

        // Right wrist is at y=0.1, nose at y=0.2 -> raised!
        assert!(skel.is_hand_raised(true));
        // Left wrist is at y=0.4, nose at y=0.2 -> not raised
        assert!(!skel.is_hand_raised(false));

        // Root position should be hip midpoint: ((0.45 + 0.55)/2, (0.6 + 0.6)/2) = (0.5, 0.6)
        let (rx, ry) = skel.root_position();
        assert!((rx - 0.5).abs() < 1e-5);
        assert!((ry - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_hand_raised_low_confidence() {
        let mut skel = dummy_skeleton();
        // Set right wrist confidence below 0.3 threshold
        skel.keypoints[COCO_RIGHT_WRIST] = Keypoint::new(0.7, 0.1, 0.2);
        assert!(!skel.is_hand_raised(true));
    }

    #[test]
    fn test_player_input_serialization() {
        let skel = dummy_skeleton();
        let input = PlayerInput::new(vec![skel], 12.34);

        let json = serde_json::to_string(&input).expect("serialization failed");
        let decoded: PlayerInput = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(input, decoded);
        assert_eq!(decoded.player_by_id(1).map(|p| p.id), Some(1));
        assert_eq!(decoded.player_by_id(99), None);
    }
}
