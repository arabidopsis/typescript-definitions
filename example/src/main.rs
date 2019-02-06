
extern crate serde_json;
extern crate serde;
// use serde::{Deserialize, Serialize};
use serde_json::{Error};
mod interface;

use self::interface::*;

// use self::interface::{TypeScriptifyTrait};


fn main() -> Result<(),Error> {
    let p = Point {x:23, y:24, z: 33};
    let j = serde_json::to_string(&p)?;
    let f = FrontendMessage::Render { html : "stuff".into(), time: 33 , other: Err(32)};
    let f2 = FrontendMessage::ButtonState { selected : vec!["a".into(), "b".into()], time: 33 , other:None };
    println!("Point {:?}", p);
    println!("{}", j);
    println!("{}", serde_json::to_string(&f)?);
    println!("{}", serde_json::to_string(&f2)?);

    println!("{}",Point::type_script_ify());
    
    println!("{}", Enum::type_script_ify());
    println!("{}", FrontendMessage::type_script_ify());
    println!("{}", Value::<i32>::type_script_ify());

    Ok(())
}

