/*
    TCP: TCP是一种底层的协议, 描述信息如何从一个服务器传输到另一个服务器的细节, 但他并不指定信息的具体内容
    HTTP: 建立在TCP之上, 定义了请求和响应的内容

    该项目主要是处理TCP的原始字节, 并且与HTTP的请求和响应打交道

    ? 使用线程池处理
*/

// * 这里将 std::io::prelude 引入作用域来获取读写流所需的特定 trait, 否则read方法报错
use std::io::prelude::*;
// * 监听TCP需要使用标注库的 TcpListene
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use web_server::pools::index::{ThreadPool};

fn main() {
    // * bind函数会监听传入的地址, 返回结果是一个 Result<T, E>, 所以调用unwrap()来处理一下失败
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // incoming 会产生一个流序列的迭代器, 也就是TCP Stream的流, 单个流就表示客户端和服务器之间打开了一个连接(三次握手完成)
    // 使用for循环就会依次处理每一个连接, 并生成一系列的流让我们来处理

    // * 初始化线程池, 放4个线程
    let pool = ThreadPool::new(4);
    // ! 这里并没有遍历连接, 而是遍历的 连接尝试, 因为incoming方法返回错误是可能的
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });

        // println!("Connection established!");
        // handle_connection(stream);
        // * 为每个连接都创建一个线程
        // ? 可以使用spawn为每一个请求都创建一个线程
        // ? 但是这样有一个缺点
        // ? 这个线程没有限制, 来一个请求就会创建一个线程, 如果被黑客使用dos攻击, 那么这个服务器马上就挂了
        // ? 所以必须限制最大线程数
        // thread::spawn(|| {
        //     handle_connection(stream);
        // });
    }
}

// * 处理连接
// * Tcp实例的内部记录了返回的数据, 他可能会读取多于请求的数据, 并将数据保留下来, 以供下一次请求使用
// ? 而TCP实例的内部状态可能会改变, 所以这里需要标记为 mut
fn handle_connection(mut stream: TcpStream) {
    // 这里搞了一个存放数据的缓存(buffer), 他是512字节的
    let mut buffer = [0; 1024];

    // 调用read方法从TCPStream读取数据, 并且将其放置到缓存区(buffer)
    stream.read(&mut buffer).unwrap();

    // * 将get请求放到get变量中, 前面有个b, 由于缓冲区接收的都是原始字节, 而这个b就代表的是字节字符串的语法, 他可以将get的文本转换为字节字符串, 然后进行比较
    let get = b"GET / HTTP/1.1\r\n";
    // ? 单线程服务器的弊端
    // ? 添加一个持续pending的请求
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // -------------------------------------------------------------------------- 改进代码
    let (status_line, file_name) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        // 阻塞当前线程5秒
        // ? 这样就出现了单线程服务器的缺点, 只要有一个请求在这里阻塞, 那么后面的所有请求都阻塞
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(file_name).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    // --------------------------------------------------------------------------

    // * 查看缓冲区是否以 get 开头
    /* if buffer.starts_with(get) {
        let contents = fs::read_to_string("hello.html").unwrap();

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else {
        let status_line = "HTTP/1.1 404 NOT FOUND";

        let contents = fs::read_to_string("404.html").unwrap();

        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            contents.len(),
            contents
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } */

    // 将缓存区中的字节转换为字符串打印出来, 使用 from_utf8_lossy, 接收一个 &[u8]类型, 并返回一个字符串
    // 这里获取的是请求头信息
    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    // 请求
    // Method Request-URI HTTP-Version CRLF
    // headers CRLF
    // message-body

    // 响应
    // HTTP-Version Status-Code Reason-Phrase CRLF
    // headers CRLF
    // message-body

    // * 将html文件转换为字符串
    // let contents = fs::read_to_string("hello.html").unwrap();
    // * 将contents放入消息体
    // 必须给Content-Length
    // let response = format!(
    //     "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
    //     contents.len(),
    //     contents
    // );

    // 利用 stream 的 write 方法将响应写回去
    // 但是这个write方法只接收 &[u8] 这个类型, 所以需要使用 as_bytes(), 将其转换为 &[u8]
    // stream.write(response.as_bytes()).unwrap();
    // 这里调用stream上的flush方法, 等待并阻止程序的运行, 直到所有的字节都被写入到连接中
    // stream.flush().unwrap();
}
