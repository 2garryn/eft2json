pub struct DefaultComposer<'a> {
    result: &'a mut String
}

pub impl <'a>DefaultComposer<'a> {
    fn new(buf: &'a mut String) -> DefaultComposer {
        DefaultComposer{
            result: buf
        }
    }
}

pub impl <'a>ElemCompose for DefaultComposer<'a> {
    fn open(&mut self, name: &str) {
        self.result.push_str("{\"");
        self.result.push_str(name);
        self.result.push_str("\":");
    }
    fn push_str<T: AsRef<str>>(&mut self, elem: T) {
        self.result.push_str(elem.as_ref())
    }
    fn push_char(&mut self, elem: char) {
        self.result.push(elem);
    }
    fn close(&mut self) {
        self.result.push_str("}");
    }
}