use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use crate::pools::worker::Worker;
// use crate::pools::job::Job;
use crate::pools::message::Message;


// 优雅停机: 现在有个问题, 就在于主线程停止后, 哪怕其他线程还在处理任务, 也会被立即终止
// 做法就是为线程池实现Drop这个trait, 从而调用线程池中每个线程的join方法, 让子线程能够完成任务再停机
// 还需要某种操作防止线程继续接收新的请求, 为停机做好准备
pub struct ThreadPool {
    // * 用于存储线程
    // * 由于 thread::spawn 这个函数, 返回的类型为 JoinHandle<T>, 所以这里也使用这个类型来存储单个线程
    // * 我们的闭包只是用于做处理, 无返回值, 所以这里只给一个单元类型即可
    // threads: Vec<thread::JoinHandle<()>>,
    workers: Vec<Worker>,
    // * 发送者, 具体的类型就是Job
    // sender: mpsc::Sender<Job>,
    // 换成message, 让子线程能够正常停机
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Creates a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool.
    /// 
    /// # Panics
    /// The `new` function will panic if the size is zero or less then zero
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        // 通道可以有多个发送者, 但是只有一个接收者
        // * 创建一个 channel, 得到发送者和接收者
        // ? 我们希望的是所有线程贡献搞一个receiver, 从而在线程间分发任务, 此外从通道队列中取出来的任务, 也意味着这个receiver是可变的
        // ? 我们需要一个线程安全的方式来共享和修改这个receiver, 否则会触发数据竞态
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        // * 使用 with_capacity 函数来创建一个预分配好空间的 vector
        let mut workers = Vec::with_capacity(size);

        // * 创建线程并将其扔到线程池里面
        for id in 0..size {
            // 这里不能使用spawn, 一旦使用spawn 就会立即执行, 所以这里需要使用worker
            // 由于只能有一个接收者, 所以每一个循环体都入参一个接收者, 是肯定不对的
            // 这里也违反了借用规则, 前一次循环已经移动过了
            // 这里需要做修改, 修改使用Arc::clone
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        println!("线程池初始化完成, 开启线程数: {}", size);
        return ThreadPool {
            workers,
            sender
        };
    }

    pub fn execute<F>(&self, f: F) 
    where
        F: FnOnce() + Send + 'static,
    {   
        // * 创建一个job
        let job = Box::new(f);
        // * 通道的发送端, 发送job
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        // * 发送终止执行的消息, 让子线程优先开始进入终止等待
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all worker.");

        // * 
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);    
            // 这里有报错, 无法将worker的thread移出来
            // 必须要移出来, 因为这个join方法必须获得参数的所有权
            if let Some(thread) = worker.thread.take() {
                // 现在调用join并不会真正的关停线程, 因为他还在循环中持续等待任务
                // 此时使用Drop实现去丢弃ThreadPool, 主线程会永远阻塞, 去等待第一个子线程结束
                // 修复需要搞一下发送的信号
                thread.join().unwrap();
            }
        }

        // 不能将上面的操作放到一个遍历里, 因为发送完终止消息后, 不能保证一定是后面的worker能收到这个消息, 所以要提前一步让他们都收到终止信号
    }
}
