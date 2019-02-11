// extern crate serde;
// extern crate serde_json;
// extern crate typescript_definitions;
// use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Error;
mod interface;

#[cfg(any(debug_assertions, feature="export-typescript"))]
fn main() -> Result<(), Error> {

    use self::interface::*;
    // need the trait
    use typescript_definitions::TypeScriptifyTrait;

    let p = Point {
        x: 23,
        y: 24,
        z: 33,
    };
    let j = serde_json::to_string(&p)?;
    let f = FrontendMessage::Render {
        html: "stuff".into(),
        time: 33,
        other: Err(32),
    };
    let f2 = FrontendMessage::ButtonState {
        selected: vec!["a".into(), "b".into()],
        time: 33,
        other: None,
    };

    let b = MyBytes {
        buffer: vec![5u8, 6, 7, 8, 9, 186, 233],
    };
    println!("Point {:?}", p);
    println!("{}", j);
    println!("{}", serde_json::to_string(&f)?);
    println!("{}", serde_json::to_string(&f2)?);
    println!("{}", serde_json::to_string(&b)?);

    println!("{}", Point::type_script_ify());

    println!("{}", Enum::type_script_ify());
    println!("{}", FrontendMessage::type_script_ify());
    println!("{}", Value::<i32>::type_script_ify());
    println!("{}", MyBytes::type_script_ify());

    Ok(())
}

#[cfg(not(any(debug_assertions, feature="export-typescript")))]
fn main() {}
