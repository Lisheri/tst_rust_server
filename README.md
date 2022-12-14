# 构建多线程web服务器

+ 在socket 上监听 TCP 连接
+ 解析少量的HTTP请求
+ 创建一个合适的HTTP响应
+ 使用线程池改进服务器的吞吐量
+ 优雅的停机和清理


> 注： 这并不是最佳实践, 主要是熟悉之前讲过的内容, 了解一下通用的技术和背后的思路


## 串行处理请求的缺点

只要有一个请求等待, 后面所有请求均等待

## 改进

+ 使用线程池

> 线程池是一种预分配出来的线程, 被用于等待并随时处理可能的任务, 当程序接收到一个新的任务时, 会给线程池中一个线程分配这个任务, 然后这个线程就会处理这个任务
> 
> 线程池里面的其他的线程, 在一个线程处理一个任务的同时, 还可以继续处理其他的任务
>
> 而当上面的线程处理完他的任务后, 就将其放回线程池, 此时他就可以处理新的任务了
> 
> 一个线程池允许并发的处理连接, 从而增加了服务器的吞吐量
