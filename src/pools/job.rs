// * 表示要执行的工作
// struct Job;
// * 应该使用类型别名
pub type Job = Box<dyn FnOnce() + Send + 'static>;