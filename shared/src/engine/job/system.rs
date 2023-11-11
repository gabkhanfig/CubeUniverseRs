use std::{sync::{Mutex, Arc, RwLock}, thread, cell::{UnsafeCell, OnceCell}, mem::MaybeUninit};
use super::{thread::JobThread, future::JobFuture};

pub(crate) const QUEUE_CAPACITY: usize = 8192;

struct Inner {
    threads: Box<[Box<JobThread>]>,
    thread_count: usize,
    current_optimal_thread: usize
}

pub struct JobSystem {
    inner: Arc<Mutex<Inner>>
}

unsafe impl Send for JobSystem {}
unsafe impl Sync for JobSystem {}

impl JobSystem {
    ///
    pub fn new(thread_count: usize) -> JobSystem {
        debug_assert_ne!(thread_count, 0, "Cannot create a job system using 0 threads");
        let mut v: Vec<Box<JobThread>> = Vec::with_capacity(QUEUE_CAPACITY);
        for _ in 0..thread_count {
            v.push(JobThread::new());
        }
        return JobSystem { 
            inner: Arc::new(Mutex::new(Inner {
                threads: v.into_boxed_slice(), 
                thread_count: thread_count,
                current_optimal_thread: 0
            }))        
        }
    }

    ///
    pub fn run_job<T, F>(&self, func: F) -> JobFuture<T>
    where T: 'static, F: FnMut() -> T + 'static {
        let job_thread = {
            let mut lock = self.inner.lock().unwrap();
            let optimal_thread_index = (*lock).get_optimal_thread_for_execution();
            &mut (*lock).threads[optimal_thread_index] as *mut Box<JobThread>
        };
        unsafe {
            let future = (*job_thread).queue_job(func);
            (*job_thread).execute();
            return future;
        }
    }

    /// 
    pub fn wait(&self) {
        thread::yield_now();
        let lock = self.inner.lock().unwrap();
        for job_thread in (*lock).threads.iter() {
            job_thread.wait();
        }
    }
}

impl Inner {
    fn get_optimal_thread_for_execution(&mut self) -> usize {
        let mut minimum_queue_load = usize::MAX;
        let mut is_optimal_executing = true;
        let mut current_optimal = self.current_optimal_thread;

        for i in 0..self.thread_count {
            let check_index = (self.current_optimal_thread + i) % self.thread_count;
            let is_not_executing = !self.threads[check_index].is_executing();
            let queue_load = self.threads[check_index].queued_count();
            if is_not_executing && queue_load == 0 {
                self.current_optimal_thread = (check_index + 1) % self.thread_count;
                return check_index;
            }

            if is_not_executing {
                if minimum_queue_load > queue_load {
                    current_optimal = check_index;
                    minimum_queue_load = queue_load;
                    is_optimal_executing = false;
                    continue;
                }
            }

            if minimum_queue_load > queue_load && is_optimal_executing {
                current_optimal = check_index;
                minimum_queue_load = queue_load;
            }
        }

        return current_optimal;
    }
}

struct JobSystemHandle(*const JobSystem);

unsafe impl Send for JobSystemHandle {}
unsafe impl Sync for JobSystemHandle {}

static mut JOB_SYSTEM: RwLock<Option<JobSystem>> = RwLock::new(None);
static mut JOB_SYSTEM_PTR: JobSystemHandle = JobSystemHandle(std::ptr::null_mut());

/// Get the maximum number of job threads allowed on the system.
/// Will always be non-zero.
/// ```
/// # use shared::engine::job::system::max_available_job_threads;
/// assert!(max_available_job_threads() > 0);
/// ```
pub fn max_available_job_threads() -> usize {
    return std::thread::available_parallelism().unwrap().get() - 1;
}

/// Initializes the job system given a specified thread count.
/// max_available_job_threads() is a sensible default.
/// ```
/// # use shared::engine::job::system::{job_system_init, max_available_job_threads};
/// // Initializes the global job system with N threads.
/// job_system_init(max_available_job_threads());
/// ```
pub fn job_system_init(thread_count: usize) {
    println!("Initializing global job system with {} threads", thread_count);
    unsafe { 
        JOB_SYSTEM = RwLock::new(Some(JobSystem::new(thread_count))); 
        let ptr = JOB_SYSTEM.read().unwrap().as_ref().unwrap() as *const JobSystem;
        JOB_SYSTEM_PTR = JobSystemHandle(ptr);
    }
}

/// Run a job on the global job system, returning a future for the job.
/// ```
/// # use shared::engine::job::system::{job_system_init, job_system_run, max_available_job_threads};
/// job_system_init(max_available_job_threads());
/// let future = job_system_run(|| 123);
/// assert_eq!(future.wait(), 123);
/// ```
/// Will panic in debug mode if job_system_init() wasn't called sometime prior.
/// ``` should_panic
/// # use shared::engine::job::system::{job_system_init, job_system_run, max_available_job_threads};
/// // Don't initialize
/// //job_system_init(max_available_job_threads());
/// // Will panic
/// let future = job_system_run(|| 123);
/// ```
pub fn job_system_run<T, F>(func: F) -> JobFuture<T>
where T: 'static, F: FnMut() -> T + 'static {
    return unsafe { 
        debug_assert!(!JOB_SYSTEM_PTR.0.is_null(), "Cannot run a job on the global job system because it hasn't been intiailized");
        (*JOB_SYSTEM_PTR.0).run_job(func) 
    }; 
}

/// Waits for the global job system to finish execution of the current jobs.
/// After wait is called, it can be assumed that there are no active jobs running.
/// 
/// Note: It is technically possible for there to be jobs executing, 
/// if the jobs created more jobs that happened to be on earlier threads.
/// ```
/// # use shared::engine::job::system::{job_system_init, job_system_run, job_system_wait, max_available_job_threads};
/// job_system_init(max_available_job_threads());
/// job_system_run(|| std::thread::sleep(std::time::Duration::from_millis(10)));
/// job_system_wait();
/// // Jobs are completed here
/// ```
/// Will panic in debug mode if job_system_init() wasn't called sometime prior.
/// ``` should_panic
/// # use shared::engine::job::system::{job_system_init, job_system_run, job_system_wait, max_available_job_threads};
/// // Don't initialize
/// //job_system_init(max_available_job_threads());
/// // Will panic
/// job_system_wait();
/// ```
pub fn job_system_wait() {
    unsafe { 
        debug_assert!(!JOB_SYSTEM_PTR.0.is_null(), "Cannot run a job on the global job system because it hasn't been intiailized");
        (*JOB_SYSTEM_PTR.0).wait(); 
    }
}