
mod interface;

use self::interface::{TypeScriptifyTrait};


fn main() {
    println!("Hello, world2! {:?}", interface::Point {x:23, y:24, z: 33});
    println!("{}", interface::Point::type_script_ify());
    println!("{}", interface::Borrow::type_script_ify());
}
