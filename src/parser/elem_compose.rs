pub trait ElemCompose {
    fn open(&mut self, name: &str);
    fn push_str<T: AsRef<str>>(&mut self, elem: T);
    fn push_char(&mut self, elem: char);
    fn close(&mut self);
}
