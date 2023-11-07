#![feature(fn_traits)]

use std::{sync::{Arc, Mutex}, cell::UnsafeCell, vec};

use shared::engine::job::job::{JobRunDataBuffer, Job};

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
/* 
trait JobRun {
    fn invoke(&mut self);
}

struct Job<T: Default> {
    val: T,
    func: fn (&mut T)
}

impl<T: Default> Job<T> {
    fn new(val: T, func: fn (&mut T)) -> Box<dyn JobRun>
    where T: 'static {
        return Box::new(
            Job {
                val,
                func
            }
        )

    }
}

impl<T: Default> JobRun for Job<T> { 
    fn invoke(&mut self) {
        let temp = (std::mem::replace(&mut self.val, T::default()), );
        std::ops::Fn::call(&self.func, temp);
    }
}*/

fn empty() { println!("hi")}


fn main() {
    //println!("Hello, world!");

    //let mut a = Example {num: 1};

    //println!("{}", std::mem::size_of::<JobFunc>());

    //let buf = JobRunDataBuffer::new::<String>(String::from("hello world!"));
    //let s = buf.get::<String>();
    //println!("did the thing? {}", s);

    //let mut vec = Arc::new(vec![0u32]);

    /*
    let job1 = unsafe { JobData::from_obj_mut::<Example>(&mut a, Example::display_mut) };
    let job2 = unsafe { JobData::from_obj::<Example>(&a, Example::display) };
    unsafe {
        a.num = 2;
        job1.invoke();
        a.num = 3;
        job2.invoke();
    } */
    
    //unsafe { job.invoke_member(JobRunDataBuffer::default()) };
    //unsafe { job2.invoke_member(JobRunDataBuffer::default()) };

    //let job = JobFunc::new( move || a.display() );
    //job.func.as_mut()();

    //let job2 = JobFunc::new( || a.display() );
    //job.func.as_mut()();


    //let bounded = || func(1, 2, 3);



    let mut v = vec![1, 2, 3, 4, 5];

    let job = Job::from_closure(move || v.push(1));


    /*
    let mut v = vec![];

    for _ in 0..10 {
        let job = Job::from_func(empty);
        v.push(job);
    }

    for job in v.iter() {
        job.invoke();
    }
    
    for job in v {
        job.invoke();
    } */
    
  

}
