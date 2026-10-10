#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowLifecycle {
    #[allow(dead_code)]
    Initializing,
    Hidden,
    SpotlightVisible,
    FullWindowVisible,
    ShuttingDown,
}

impl WindowLifecycle {
    #[allow(dead_code)]
    pub fn is_visible(&self) -> bool {
        matches!(self, Self::SpotlightVisible | Self::FullWindowVisible)
    }

    pub fn is_spotlight(&self) -> bool {
        matches!(self, Self::SpotlightVisible)
    }

    #[allow(dead_code)]
    pub fn is_full(&self) -> bool {
        matches!(self, Self::FullWindowVisible)
    }
}
