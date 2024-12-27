use std::io::Cursor;
use std::io::{Read, Seek, SeekFrom, Write};
use bytes::Buf;
use mini_redis::{
    Result,
    frame::Frame
};

fn main() {
    let _ = example_parse();
}

fn base_use()  -> std::io::Result<()> {
    // 1. 基本用法 - 从 Vec<u8> 创建
    let mut buf = Cursor::new(vec![1, 2, 3, 4, 5]);

    // 读取单个字节
    let mut byte = [0; 1];
    println!("position: {}", buf.position());
    println!("buf: {:?}", buf);
    buf.read(&mut byte)?;
    assert_eq!(byte[0], 1);

    // 2. 使用 seek 移动游标
    buf.seek(SeekFrom::Start(2))?; // 移动到位置 2
    let mut byte = [0; 1];
    buf.read(&mut byte)?;
    assert_eq!(byte[0], 3);

    // 3. 写入数据
    let mut writer = Cursor::new(Vec::new());
    writer.write_all(&[1, 2, 3])?;
    assert_eq!(writer.into_inner(), vec![1, 2, 3]);

    // 4. 在字符串上使用
    let mut cursor = Cursor::new("Hello, World!");
    let mut buffer = String::new();
    cursor.read_to_string(&mut buffer)?;
    assert_eq!(buffer, "Hello, World!");

    Ok(())
}


// 解析 GET 命令: "*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n"
fn example_parse() -> Result<()> {
    let data = b"*2\r\n$3\r\nGET\r\n$3\r\nkey\r\n";
    let mut buf = Cursor::new(&data[..]);
    let start = buf.get_u8();
    println!("start: {}", start);
    // 将 u8 转换为对应的 ASCII 字符
    let char = start as char;
    println!("对应的 ASCII 字符: {}", char); // 应该输出 '*'
    buf.set_position(0);
    let frame = Frame::parse(&mut buf)?;
    println!("frame: {:?}", frame);
    
    // frame 将是:
    // Frame::Array([
    //     Frame::Bulk(b"GET"),
    //     Frame::Bulk(b"key")
    // ])
    
    Ok(())
}