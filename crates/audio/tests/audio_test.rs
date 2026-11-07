#[test]
fn test_play_raw() {
    if let Ok((_stream, handle)) = rodio::OutputStream::try_default() {
        let samples: Vec<f32> = vec![0.0; 100];
        let buf = rodio::buffer::SamplesBuffer::new(2, 44100, samples);
        let res = handle.play_raw(buf);
        assert!(res.is_ok());
    }
}
