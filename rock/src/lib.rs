mod bindings;

use bindings::exports::seungjin::rock::calc;

struct Component;

impl calc::Guest for Component {
    fn add(x: i32, y: i32) -> i32 {
        x + y
    }

    fn sub(x: i32, y: i32) -> i32 {
        x - y
    }
}

bindings::export!(Component with_types_in bindings);
