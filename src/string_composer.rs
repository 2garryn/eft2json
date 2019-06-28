use crate::parser::ElemCompose;


impl ElemCompose for String {
    fn open(&mut self, name: &str) {
        self.push_str("{\"");
        self.push_str(name);
        self.push_str("\":");
    }
    fn push_str<T: AsRef<str>>(&mut self, elem: T) {
        self.push_str(elem.as_ref())
    }
    fn push_char(&mut self, elem: char) {
        self.push(elem);
    }
    fn close(&mut self) {
        self.push_str("}");
    }
}