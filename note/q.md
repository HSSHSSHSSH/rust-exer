## 模块化

## 特征

## cell 内存图
```rs
fn memory_explanation() {
    // "asdf" 在只读数据段，地址假设为 0x1000
    // "qwer" 在只读数据段，地址假设为 0x2000
    
    let c = Cell::new("asdf");  // c 内部存储指向 0x1000 的指针
    let one = c.get();          // one 获得指向 0x1000 的指针的拷贝
    c.set("qwer");             // c 内部的指针更新为指向 0x2000
    let two = c.get();         // two 获得指向 0x2000 的指针的拷贝
}
```
## RefCell 借用判断
```rs
use std::cell::RefCell;

fn explain_borrow_counts() {
    let data = RefCell::new(vec![1, 2, 3]);
    
    // 1. 检查当前借用状态
    println!("初始状态:");
    println!("可变借用是否可用: {}", data.try_borrow_mut().is_ok());   // true
    println!("不可变借用是否可用: {}", data.try_borrow().is_ok());     // true
    
    // 2. 创建不可变借用
    let ref1 = data.borrow();
    let ref2 = data.borrow();
    println!("\n两个不可变借用后:");
    println!("可变借用是否可用: {}", data.try_borrow_mut().is_ok());   // false，因为有不可变借用
    println!("不可变借用是否可用: {}", data.try_borrow().is_ok());     // true，可以继续不可变借用
    
    // 释放不可变借用
    drop(ref1);
    drop(ref2);
    
    // 3. 创建可变借用
    {
        let mut mut_ref = data.borrow_mut();
        println!("\n可变借用存在时:");
        println!("可变借用是否可用: {}", data.try_borrow_mut().is_ok());   // false，已经有一个可变借用
        println!("不可变借用是否可用: {}", data.try_borrow().is_ok());     // false，有可变借用时不能不可变借用
        
        // 修改数据
        mut_ref.push(4);
    } // mut_ref 在这里被 drop
    
    // 4. 所有借用都释放后
    println!("\n所有借用释放后:");
    println!("可变借用是否可用: {}", data.try_borrow_mut().is_ok());   // true
    println!("不可变借用是否可用: {}", data.try_borrow().is_ok());     // true
    println!("最终数据: {:?}", data.borrow());
}

fn main() {
    explain_borrow_counts();
}


```

## NLL 中的"最后一次使用"是针对借用（引用）的，而不是变量本身