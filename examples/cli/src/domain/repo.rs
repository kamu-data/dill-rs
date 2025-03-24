pub trait ValueRepo: Send + Sync {
    fn get(&self) -> std::io::Result<i32>;
    fn set(&self, value: i32) -> std::io::Result<()>;
}
