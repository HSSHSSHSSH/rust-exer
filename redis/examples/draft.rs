use std::rc::Rc;

fn main() {
    let s = String::from("hello, world");
    // s在这里被转移给a
    let a = Box::new(s);    // 同上

}
