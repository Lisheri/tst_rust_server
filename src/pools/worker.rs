use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use crate::pools::job::Job;
use crate::pools::message::Message;

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        /* 
            在 worker 中，传递给 thread::spawn 的闭包仍然还只是 引用 了信道的接收端。
            相反我们需要闭包一直循环，向信道的接收端请求任务，并在得到任务时执行他们。
            首先在 receiver 上调用了 lock 来获取互斥器，接着 unwrap 在出现任何错误时 panic。
            如果互斥器处于一种叫做 被污染（poisoned）的状态时获取锁可能会失败，这可能发生于其他线程在持有锁时 panic 了且没有释放锁。
            在这种情况下，调用 unwrap 使其 panic 是正确的行为。请随意将 unwrap 改为包含有意义错误信息的 expect。

            如果锁定了互斥器，接着调用 recv 从信道中接收 Job。
            最后的 unwrap 也绕过了一些错误，这可能发生于持有信道发送端的线程停止的情况，类似于如果接收端关闭时 send 方法如何返回 Err 一样。

            调用 recv 会阻塞当前线程，所以如果还没有任务，其会等待直到有可用的任务。Mutex<T> 确保一次只有一个 Worker 线程尝试请求任务。

            注意: 注意如果同时在多个浏览器窗口打开 /sleep，它们可能会彼此间隔地加载 5 秒，因为一些浏览器处于缓存的原因会顺序执行相同请求的多个实例。这些限制并不是由于我们的 web server 造成的。
        */
        let thread = thread::spawn(move|| loop {
            // while let Ok<job> = receiver.lock().unwrap().recv() {job();}
            /* 
                无法使用 while let, 上面的代码虽然会编译和执行, 但是并不会产生所期望的线程行为: 一个慢请求仍然会导致其他请求等待执行。原因有些微妙: 
                Mutex 结构体没有公有 unlock 方法，因为锁的所有权依赖 lock 方法返回的 LockResult<MutexGuard<T>> 中 MutexGuard<T> 的生命周期。
                这允许借用检查器在编译时确保绝不会在没有持有锁的情况下访问由 Mutex 守护的资源，不过如果没有认真的思考 MutexGuard<T> 的生命周期的话，也可能会导致比预期更久的持有锁。

                let job = receiver.lock().unwrap().recv().unwrap(); 之所以可以工作是因为对于 let 来说
                当 let 语句结束时任何表达式中等号右侧使用的临时值都会立即被丢弃。
                然而 while let（if let 和 match）直到相关的代码块结束都不会丢弃临时值。
                job() 调用期间锁一直持续，这也意味着其他的 worker 无法接受任务。
            */
            // * 接收job
            // * 1.锁定receiver, 2. 通过recv 接收任务
            // let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} go a job; executing.", id);
            // * 3. 执行任务
            // job();
            // ! 停机改造
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} go a job executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });
        return Worker {
            id,
            thread: Some(thread),
        };
    }
}