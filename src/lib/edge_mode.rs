#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
/// Declares an edge handling mode
pub enum EdgeMode {
    /// If kernel goes out of bounds it will be clipped to an edge and edge pixel replicated across filter
    Clamp = 0,
    #[default]
    /// If kernel goes out of bound it will be clipped, this is a slightly faster than clamp, however have different visual effects at the edge
    KernelClip = 1,
}

impl From<usize> for EdgeMode {
    fn from(value: usize) -> Self {
        return match value {
            0 => EdgeMode::Clamp,
            1 => EdgeMode::KernelClip,
            _ => {
                panic!("Unknown edge mode for value: {}", value);
            }
        }
    }
}
