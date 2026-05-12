//! Main crate for workspace fixture.

pub use helper_crate::{helper_add, helper_mul, helper_neg, get_feature_flags};

/// Result of a workspace operation.
#[repr(C)]
pub struct WorkspaceResult {
    pub value: i32,
    pub from_helper: bool,
    pub build_ran: bool,
}

impl WorkspaceResult {
    pub fn new(value: i32, from_helper: bool, build_ran: bool) -> Self {
        WorkspaceResult {
            value,
            from_helper,
            build_ran,
        }
    }
}

/// Call helper add and wrap result.
#[no_mangle]
pub extern "C" fn workspace_add(a: i32, b: i32) -> WorkspaceResult {
    let result = helper_add(a, b);
    let build_ran = std::env::var("BUILD_SCRIPT_RAN").is_ok();
    WorkspaceResult::new(result, true, build_ran)
}

/// Call helper multiply and wrap result.
#[no_mangle]
pub extern "C" fn workspace_mul(a: i32, b: i32) -> WorkspaceResult {
    let result = helper_mul(a, b);
    let build_ran = std::env::var("BUILD_SCRIPT_RAN").is_ok();
    WorkspaceResult::new(result, true, build_ran)
}

/// Compute a pipeline: negate, then add.
#[no_mangle]
pub extern "C" fn workspace_pipeline(a: i32, b: i32) -> WorkspaceResult {
    let neg = helper_neg(a);
    let result = helper_add(neg, b);
    let build_ran = std::env::var("BUILD_SCRIPT_RAN").is_ok();
    WorkspaceResult::new(result, true, build_ran)
}

/// Get combined feature flags from helper.
#[no_mangle]
pub extern "C" fn workspace_features() -> u32 {
    get_feature_flags()
}

/// Wrapper that does nothing but call through.
#[no_mangle]
pub extern "C" fn workspace_identity(a: i32) -> WorkspaceResult {
    let build_ran = std::env::var("BUILD_SCRIPT_RAN").is_ok();
    WorkspaceResult::new(a, false, build_ran)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_add() {
        let result = workspace_add(10, 20);
        assert_eq!(result.value, 30);
        assert!(result.from_helper);
    }

    #[test]
    fn test_workspace_mul() {
        let result = workspace_mul(5, 6);
        assert_eq!(result.value, 30);
        assert!(result.from_helper);
    }

    #[test]
    fn test_workspace_pipeline() {
        let result = workspace_pipeline(5, 3);
        // negate(5) + 3 = -5 + 3 = -2
        assert_eq!(result.value, -2);
    }

    #[test]
    fn test_workspace_identity() {
        let result = workspace_identity(42);
        assert_eq!(result.value, 42);
        assert!(!result.from_helper);
    }

    #[test]
    fn test_workspace_result_builder() {
        let result = WorkspaceResult::new(99, true, true);
        assert_eq!(result.value, 99);
        assert!(result.from_helper);
        assert!(result.build_ran);
    }
}