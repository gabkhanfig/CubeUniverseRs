use std::time::Duration;

use shared::engine::job::system::{job_system_run, job_system_wait};

use super::initialize_job_system_integration_test;

#[test]
fn add_many_jobs_with_delays() {
    initialize_job_system_integration_test();

    for _ in 0..100 {
        job_system_run(|| std::thread::sleep(Duration::from_millis(1)));
    }
    job_system_wait();
}

#[test]
fn add_many_jobs_with_delays_and_futures() {
    initialize_job_system_integration_test();

    let mut v = vec![];

    for i in 0..100 {
        v.push(job_system_run(move || {
            std::thread::sleep(Duration::from_millis(1));
            i
        }));
    }

    for i in 0..100 {
        assert_eq!(v[i].wait(), i);
    }
}
