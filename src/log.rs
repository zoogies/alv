macro_rules! alv_log {
    ($($args:tt)*) => {
        print!("[INFO] "); 
        println!($($args)*);
    };
}

macro_rules! alv_error {
    ($($args:tt)*) => {
        print!("[ERROR] "); 
        println!($($args)*);
    };
}

pub(crate) use alv_log;
pub(crate) use alv_error;