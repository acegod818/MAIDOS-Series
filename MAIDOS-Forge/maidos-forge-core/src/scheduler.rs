//! Scheduler module

/// Schedule result
#[derive(Debug)]
pub struct ScheduleResult {
    /// Whether scheduling succeeded
    pub success: bool,
    /// Error message (if any)
    pub error: Option<String>,
    /// Schedule plan
    pub schedule: Option<Schedule>,
}

/// Schedule plan
#[derive(Debug)]
pub struct Schedule {
    /// List of layers
    pub layers: Vec<ScheduleLayer>,
}

/// Schedule layer
#[derive(Debug)]
pub struct ScheduleLayer {
    /// List of modules
    pub modules: Vec<String>,
}

impl ScheduleResult {
    /// Create a successful schedule result.
    pub fn success(schedule: Schedule) -> Self {
        Self {
            success: true,
            error: None,
            schedule: Some(schedule),
        }
    }

    /// Create a failed schedule result.
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            error: Some(error),
            schedule: None,
        }
    }
}