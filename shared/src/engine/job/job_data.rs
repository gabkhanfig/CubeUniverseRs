use std::cell::UnsafeCell;
use std::mem::size_of;

pub enum JobFunc {
    FreeFunction(fn ()),
    FreeFunctionBuffer(fn (&mut JobRunDataBuffer)),
    Member(UnsafeCell<Box<dyn FnMut()>>),
    MemberBuffer(UnsafeCell<Box<dyn FnMut(&mut JobRunDataBuffer)>>),
    Closure(UnsafeCell<Box<dyn FnMut()>>),
    ClosureBuffer(UnsafeCell<Box<dyn FnMut(&mut JobRunDataBuffer)>>),
}

impl JobFunc {
    fn new_from_func(func: fn()) -> Self {
        return JobFunc::FreeFunction(func);
    }

    fn new_from_func_buffer(func: fn (&mut JobRunDataBuffer)) -> Self {
        return JobFunc::FreeFunctionBuffer(func);
    }

    unsafe fn new_from_obj<T>(object: *const T, func: fn (&T)) -> Self 
    where T: 'static {
        assert!(!object.is_null());

        let closure = move || func(&*object);
        return JobFunc::Member(UnsafeCell::new(Box::new(closure)));
    }

    unsafe fn new_from_obj_mut<T>(object: *mut T, func: fn (&mut T)) -> Self 
    where T: 'static {
        assert!(!object.is_null());

        let closure = move || func(&mut *object);
        return JobFunc::Member(UnsafeCell::new(Box::new(closure)));
    }

    unsafe fn new_from_obj_buffer<T>(object: *const T, func: fn (&T, &mut JobRunDataBuffer)) -> Self 
    where T: 'static {
        assert!(!object.is_null());

        let closure = move |job_data_buffer: &mut JobRunDataBuffer| func(&*object, job_data_buffer);
        return JobFunc::MemberBuffer(UnsafeCell::new(Box::new(closure)));
    }

    unsafe fn new_from_obj_buffer_mut<T>(object: *mut T, func: fn (&mut T, &mut JobRunDataBuffer)) -> Self 
    where T: 'static {
        assert!(!object.is_null());

        let closure = move |job_data_buffer: &mut JobRunDataBuffer| func(&mut *object, job_data_buffer);
        return JobFunc::MemberBuffer(UnsafeCell::new(Box::new(closure)));
    }

    fn new_from_closure<F>(func: F) -> Self
    where F: FnMut() + 'static {
        return JobFunc::Closure(UnsafeCell::new(Box::new(func)));
    }

    fn new_from_closure_buffer<F>(func: F) -> Self 
    where F: FnMut(&mut JobRunDataBuffer) + 'static {
        return JobFunc::ClosureBuffer(UnsafeCell::new(Box::new(func)));
    }
}

pub struct JobRunDataBuffer {
    buffer: [usize; 5]
}

impl JobRunDataBuffer {
    pub fn new<T: Default>(mut data: T) -> Self {
        assert!(size_of::<T>() <= size_of::<JobRunDataBuffer>());

        unsafe {
            let mut buffer = JobRunDataBuffer::default();
            std::ptr::copy_nonoverlapping(&mut data as *mut T as *mut usize, buffer.buffer.as_mut_ptr(), size_of::<T>());
            std::mem::forget(data);
            return buffer;
        }
    }

    pub fn get<T: Default>(&mut self) -> T {
        assert!(size_of::<T>() <= size_of::<JobRunDataBuffer>());
  
        let mut out = T::default();
        unsafe {
            std::ptr::copy_nonoverlapping(self.buffer.as_mut_ptr() as *mut T, &mut out as *mut T, size_of::<T>());
        }
        self.fill_with_zero();
        return out;
    }

    pub fn is_zeroed(&self) -> bool {
        return self.buffer[0] == 0
        && self.buffer[1] == 0
        && self.buffer[2] == 0
        && self.buffer[3] == 0
        && self.buffer[4] == 0;
    }

    fn fill_with_zero(&mut self) {
        unsafe { std::ptr::write_bytes::<usize>(self.buffer.as_mut_ptr(), 0, size_of::<JobRunDataBuffer>()) }         
    }
}

impl Default for JobRunDataBuffer {
    fn default() -> Self {
        JobRunDataBuffer { buffer: [0; 5] }
    }
}

impl Drop for JobRunDataBuffer {
    fn drop(&mut self) {
        assert!(self.is_zeroed(), "Job run data buffer was not properly consumed in a job function that expects consumption");
    }
}

/// Holds data to run jobs.
/// ```
/// # use std::mem::{size_of, align_of};
/// use shared::engine::job::job_data::JobData;
/// assert_eq!(size_of::<JobData>(), 64);
/// assert_eq!(align_of::<JobData>(), 64);
/// ```
#[repr(align(64))]
pub struct JobData {
    pub func: JobFunc,
    pub buffer: UnsafeCell<JobRunDataBuffer>
}

impl JobData {
    pub fn from_func(func: fn()) -> Self {
        return JobData { 
            func: JobFunc::new_from_func(func), 
            buffer: UnsafeCell::new(JobRunDataBuffer::default()) 
        }
    }

    pub fn from_func_buffer(func: fn (&mut JobRunDataBuffer), buffer: JobRunDataBuffer) -> Self {
        return JobData { 
            func: JobFunc::new_from_func_buffer(func), 
            buffer: UnsafeCell::new(buffer) 
        }
    }

    pub unsafe fn from_obj<T>(object: &T, func: fn (&T)) -> Self 
    where T: 'static  {
        return JobData { 
            func: JobFunc::new_from_obj(object as *const T, func), 
            buffer: UnsafeCell::new(JobRunDataBuffer::default()) 
        }
    }

    pub unsafe fn from_obj_mut<T>(object: &mut T, func: fn (&mut T)) -> Self 
    where T: 'static  {
        return JobData { 
            func: JobFunc::new_from_obj_mut(object as *mut T, func), 
            buffer: UnsafeCell::new(JobRunDataBuffer::default()) 
        }
    }

    pub unsafe fn from_obj_buffer<T>(object: &T, func: fn (&T, &mut JobRunDataBuffer), buffer: JobRunDataBuffer) -> Self 
    where T: 'static  {
        return JobData { 
            func: JobFunc::new_from_obj_buffer(object as *const T, func), 
            buffer: UnsafeCell::new(buffer) 
        }
    }

    pub unsafe fn from_obj_buffer_mut<T>(object: &mut T, func: fn (&mut T, &mut JobRunDataBuffer), buffer: JobRunDataBuffer) -> Self 
    where T: 'static {
        return JobData { 
            func: JobFunc::new_from_obj_buffer_mut(object as *mut T, func), 
            buffer: UnsafeCell::new(buffer) 
        }
    }

    pub fn from_closure<F>(func: F) -> Self
    where F: FnMut() + 'static {
        return JobData { 
            func: JobFunc::new_from_closure(func), 
            buffer: UnsafeCell::new(JobRunDataBuffer::default()) 
        }
    }

    pub fn from_closure_buffer<F>(func: F, buffer: JobRunDataBuffer) -> Self 
    where F: FnMut(&mut JobRunDataBuffer) + 'static {
        return JobData { 
            func: JobFunc::new_from_closure_buffer(func), 
            buffer: UnsafeCell::new(buffer) 
        }
    }

    pub unsafe fn invoke(&self) {
        let fetched_buffer = unsafe { &mut *self.buffer.get() };
        match &self.func {
            JobFunc::FreeFunction(execute) => execute(),
            JobFunc::FreeFunctionBuffer(execute) => execute(fetched_buffer),
            JobFunc::Member(execute) => (unsafe { &mut *execute.get() }).as_mut()(),
            JobFunc::MemberBuffer(execute) => (unsafe { &mut *execute.get() }).as_mut()(fetched_buffer),
            JobFunc::Closure(execute) => (unsafe { &mut *execute.get() }).as_mut()(),
            JobFunc::ClosureBuffer(execute) => (unsafe { &mut *execute.get() }).as_mut()(fetched_buffer),
        }
        debug_assert!(fetched_buffer.is_zeroed(), "Job Run Data Buffer must be consumed by the executing function");
    }

}