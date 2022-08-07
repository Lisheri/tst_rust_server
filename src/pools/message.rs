use crate::pools::job::Job;

pub enum Message {
    // 存放job
    NewJob(Job),
    // 表示终止执行
    Terminate
}