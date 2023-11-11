pub mod integration_tests;

use std::sync::Once;

use shared::engine::job::system::{job_system_init, max_available_job_threads};

static INIT: Once = Once::new();

pub(crate) fn initialize_job_system_integration_test() {
    INIT.call_once(|| {
        job_system_init(max_available_job_threads());
    })
}