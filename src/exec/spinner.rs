pub const FRAMES: [&str; 4] = ["|", "/", "-", "\\"];

pub fn frame(index: usize) -> &'static str {
    FRAMES[index % FRAMES.len()]
}

pub fn next_index(index: usize) -> usize {
    (index + 1) % FRAMES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_frames_cycle() {
        assert_eq!(frame(0), "|");
        assert_eq!(frame(1), "/");
        assert_eq!(frame(2), "-");
        assert_eq!(frame(3), "\\");
        assert_eq!(frame(4), "|");
        assert_eq!(next_index(3), 0);
    }
}
