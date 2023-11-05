use std::sync::{Arc, Mutex};

use shared::engine::job::job_data::{JobFunc, JobRunDataBuffer, JobData};

fn func(x: u32, y: u32, z: u32) {

}

struct Example {
    num: u32
}

impl Example {
    fn display_mut(&mut self) {
        println!("{}", self.num);
    }
    fn display(&self) {
        println!("{}", self.num);
    }
}

fn main() {
    println!("Hello, world!");

    let mut a = Example {num: 1};

    println!("{}", std::mem::size_of::<JobFunc>());

    //let job = unsafe { JobFunc::new_from_obj_mut::<Balls>(&mut a as *mut Balls, Balls::display) };
    //let job2 = unsafe { JobFunc::new_from_obj_mut::<Balls>(&mut a as *mut Balls, Balls::display) };

    let job1 = unsafe { JobData::from_obj_mut::<Example>(&mut a, Example::display_mut) };
    let job2 = unsafe { JobData::from_obj::<Example>(&a, Example::display) };
    unsafe {
        a.num = 2;
        job1.invoke();
        a.num = 3;
        job2.invoke();
    }

    //unsafe { job.invoke_member(JobRunDataBuffer::default()) };
    //unsafe { job2.invoke_member(JobRunDataBuffer::default()) };

    //let job = JobFunc::new( move || a.display() );
    //job.func.as_mut()();

    //let job2 = JobFunc::new( || a.display() );
    //job.func.as_mut()();


    //let bounded = || func(1, 2, 3);


}
