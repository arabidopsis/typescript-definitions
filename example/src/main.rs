#[allow(unused)]
use serde_json::Error;
mod interface;

fn main() -> Result<(), Error> {
    use self::interface::*;
    Ok(())
}
