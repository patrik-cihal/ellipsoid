
use std::fmt::Display;
use strum::IntoEnumIterator;


pub trait Textures: IntoEnumIterator + Default + Into<u32> + Clone {
    fn bytes(&self) -> Vec<u8>;
}