use std::error::Error;

pub mod app;


pub enum Data {
    Pid(u32),
    Key(String),
    Failure(String)
}