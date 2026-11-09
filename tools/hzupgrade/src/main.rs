#![allow(dead_code)]
mod upgrade;
fn main(){ let a:Vec<String>=std::env::args().collect();
 let r=upgrade::upgrade_into(std::path::Path::new(&a[1]),std::path::Path::new(&a[2])).unwrap();
 for l in r.lines(){println!("{l}");} }
