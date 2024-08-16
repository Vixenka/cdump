use std::fmt::Debug;

pub fn eval_debug<T>(obj: &T)
where
    T: Debug,
{
    println!("{:?}", obj);
}
