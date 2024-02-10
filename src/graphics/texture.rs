
use std::fmt::Display;
use strum::IntoEnumIterator;


pub trait Textures: IntoEnumIterator + Display + Default + Into<u32> + Copy {
    fn bytes(&self) -> Vec<u8>;
}