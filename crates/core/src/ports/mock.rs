use crate::ports::input::{PoseStreamPort, TrackingError};
use crate::pose::PlayerInput;
use std::io::BufRead;
use std::path::Path;

/// Mock inbound adapter that streams pre-recorded JSON pose sequences line-by-line.
/// Enables headless CI testing and deterministic game verification without real camera hardware.
#[derive(Debug, Clone)]
pub struct MockFilePoseStreamAdapter {
    lines: Vec<String>,
    cursor: usize,
    looping: bool,
}

impl MockFilePoseStreamAdapter {
    /// Creates a new mock adapter from pre-loaded JSON lines.
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            cursor: 0,
            looping: false,
        }
    }

    /// Loads JSON lines from a recorded file path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, TrackingError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::from_reader(reader)
    }

    /// Reads JSON lines from any reader implementing `BufRead`.
    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self, TrackingError> {
        let mut lines = Vec::new();
        for line_result in reader.lines() {
            let line = line_result?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                lines.push(trimmed.to_string());
            }
        }
        Ok(Self {
            lines,
            cursor: 0,
            looping: false,
        })
    }

    /// Parses JSON lines from a multi-line string.
    pub fn from_json_lines(content: &str) -> Result<Self, TrackingError> {
        Self::from_reader(content.as_bytes())
    }

    /// Configures whether the adapter loops back to the start when reaching EOF.
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Builder method to enable looping.
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Resets stream cursor to the first frame.
    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    /// Total number of frames in the recording.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// True if there are no frames loaded.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Number of remaining unpolled frames.
    pub fn remaining(&self) -> usize {
        self.lines.len().saturating_sub(self.cursor)
    }
}

impl PoseStreamPort for MockFilePoseStreamAdapter {
    fn poll_input(&mut self) -> Result<Option<PlayerInput>, TrackingError> {
        if self.lines.is_empty() {
            return Err(TrackingError::StreamEnded);
        }

        if self.cursor >= self.lines.len() {
            if self.looping {
                self.cursor = 0;
            } else {
                return Err(TrackingError::StreamEnded);
            }
        }

        let line = match self.lines.get(self.cursor) {
            Some(l) => l,
            None => return Err(TrackingError::StreamEnded),
        };
        self.cursor += 1;

        let input: PlayerInput = serde_json::from_str(line)?;
        Ok(Some(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pose::{Keypoint, Skeleton, KEYPOINT_COUNT};

    fn make_sample_input(timestamp: f64) -> PlayerInput {
        let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
        keypoints[0] = Keypoint::new(0.5, 0.2, 0.95);
        let skel = Skeleton::new(1, keypoints);
        PlayerInput::new(vec![skel], timestamp)
    }

    #[test]
    fn test_mock_adapter_streaming_and_eof() {
        let frame1 = make_sample_input(1.0);
        let frame2 = make_sample_input(2.0);

        let json1 = serde_json::to_string(&frame1).expect("serialize frame1");
        let json2 = serde_json::to_string(&frame2).expect("serialize frame2");
        let multi_line = format!("{}\n{}\n", json1, json2);

        let mut adapter =
            MockFilePoseStreamAdapter::from_json_lines(&multi_line).expect("parse json lines");

        assert_eq!(adapter.len(), 2);
        assert_eq!(adapter.remaining(), 2);

        // Frame 1
        let polled1 = adapter.poll_input().expect("poll frame 1");
        assert_eq!(polled1, Some(frame1));
        assert_eq!(adapter.remaining(), 1);

        // Frame 2
        let polled2 = adapter.poll_input().expect("poll frame 2");
        assert_eq!(polled2, Some(frame2));
        assert_eq!(adapter.remaining(), 0);

        // EOF
        let err = adapter.poll_input();
        match err {
            Err(TrackingError::StreamEnded) => {}
            other => panic!("Expected StreamEnded, got {:?}", other),
        }
    }

    #[test]
    fn test_mock_adapter_looping() {
        let frame1 = make_sample_input(1.0);
        let json1 = serde_json::to_string(&frame1).expect("serialize frame1");

        let mut adapter = MockFilePoseStreamAdapter::from_json_lines(&json1)
            .expect("parse json")
            .with_looping(true);

        for _ in 0..5 {
            let polled = adapter.poll_input().expect("poll frame");
            assert_eq!(polled, Some(frame1.clone()));
        }
    }

    #[test]
    fn test_mock_adapter_invalid_json() {
        let invalid = "not a valid json line";
        let mut adapter = MockFilePoseStreamAdapter::from_json_lines(invalid).expect("parse mock");

        let res = adapter.poll_input();
        assert!(matches!(res, Err(TrackingError::SerializationError(_))));
    }
}
