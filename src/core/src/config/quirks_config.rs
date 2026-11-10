// The settings of the mod.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuirksConfig {
    // stops the idle screen and the low frame rate after 15 minutes with no input
    pub idle_enabled: bool,
}

impl Default for QuirksConfig {
    fn default() -> Self {
        Self { idle_enabled: true }
    }
}
