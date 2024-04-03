use crate::third::grandchild; 

pub fn child(x: u8) -> u8 {
    println!("{}", x);
    x + grandchild(x)
}