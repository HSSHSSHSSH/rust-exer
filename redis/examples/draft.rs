use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}


fn main() {
    let leaf = Rc::new(Node {
        value: 3,
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    });
    // 初始化完成后 Rc::strong_count(&leaf) = 1 Rc::weak_count(&leaf) = 0


    println!(
        "leaf strong = {}, weak = {}",
        Rc::strong_count(&leaf), // 1
        Rc::weak_count(&leaf), // 0
    );

    {
        let branch = Rc::new(Node {
            value: 5,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![Rc::clone(&leaf)]),
        });
        // 初始化完成后 Rc::strong_count(&branch) = 1 Rc::weak_count(&branch) = 0 RC::strong_Count += 1

        *leaf.parent.borrow_mut() = Rc::downgrade(&branch);
        // 将 leaf 的 parent 指向 branch , Rc::weak_count(&branch) += 1

        println!(
            "branch strong = {}, weak = {}",
            Rc::strong_count(&branch), // 1
            Rc::weak_count(&branch), // 1
        );
        println!(
            "leaf strong = {}, weak = {}",
            Rc::strong_count(&leaf), // 2
            Rc::weak_count(&leaf), // 0
        );
        // branch droped 
    }

    println!("leaf parent = {:?}", leaf.parent.borrow().upgrade()); // None
    println!(
        "leaf strong = {}, weak = {}",
        Rc::strong_count(&leaf), // 1
        Rc::weak_count(&leaf), // 0
    );
}
