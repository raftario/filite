pub trait DefaultExt {
    fn is_default(&self) -> bool;
}
impl<T: Default + PartialEq> DefaultExt for T {
    fn is_default(&self) -> bool {
        self.eq(&Default::default())
    }
}
