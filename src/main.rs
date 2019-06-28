
use std::io::{BufRead, Read, BufReader};
use std::fmt;
use std::str;
use std::io;
use num_bigint::Sign;
use num_bigint::BigInt;
use byteorder::{ByteOrder, BigEndian};
extern crate hex;


type ParseResult = Result<(), ParseError>;

#[derive(Debug)]
enum ErrorCode {
    ReadError = 1,
    NotImplemented = 2,
    InvalidListTerm = 3,
    NotErlangBinary = 4,
    NotUtf8Atom = 5,
}

const ATOM_CACHE_REF: u8 = 82;
const SMALL_INTEGER_EXT: u8 = 97;
const INTEGER_EXT: u8 = 98;
const FLOAT_EXT: u8 = 99;
const REFERENCE_EXT: u8 = 101;
const PORT_EXT: u8 = 102;
const PID_EXT: u8 = 103;
const SMALL_TUPLE_EXT: u8 = 104;
const LARGE_TUPLE_EXT: u8 = 105;
const MAP_EXT: u8 = 116;
const NIL_EXT: u8 = 106;
const STRING_EXT: u8 = 107;
const LIST_EXT: u8 = 108;
const BINARY_EXT: u8 = 109;
const SMALL_BIG_EXT: u8 = 110;
const LARGE_BIG_EXT: u8 = 111;
const NEW_REFERENCE_EXT: u8 = 114;
const FUN_EXT: u8 = 117;
const NEW_FUN_EXT: u8 = 112;
const EXPORT_EXT: u8 = 113;
const BIT_BINARY_EXT: u8 = 77;
const NEW_FLOAT_EXT: u8 = 70;
const ATOM_UTF8_EXT: u8 = 118;
const SMALL_ATOM_UTF8_EXT: u8 = 119;
const ATOM_EXT: u8 = 100;
const SMALL_ATOM_EXT: u8 = 115;


trait ElemCompose {
    fn open(&mut self, name: &str);
    fn push_u8(&mut self, elem: &[u8]);
    fn push_str<T: AsRef<str>>(&mut self, elem: T);
    fn push_char(&mut self, elem: char);
    fn close(&mut self);
    fn get_result(&self) -> String;
}

trait ReadStream {
    fn read_u8(&mut self) -> Result<u8, ParseError>;
    fn read_u16(&mut self) -> Result<u16, ParseError>;
    fn read_u32(&mut self) -> Result<u32, ParseError>;
    fn read_i32(&mut self) -> Result<i32, ParseError>;
}



struct DefaultComposer {
    result: String
}

impl DefaultComposer {
    fn new() -> DefaultComposer {
        DefaultComposer{
            result: String::new()
        }
    }
}

impl ElemCompose for DefaultComposer {
    fn open(&mut self, name: &str) {
        self.result.push_str("{\"");
        self.result.push_str(name);
        self.result.push_str("\":");
    }
    fn push_u8(&mut self, elem: &[u8]) {

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

    fn get_result(&self) -> String {
        self.result.clone()
    }
 }


struct DefaultStreamer<'a> {
    buf_reader: &'a mut BufRead
}

impl <'a>DefaultStreamer<'a> {
    fn new<R: BufRead>(buff: &'a mut R) -> DefaultStreamer {
        DefaultStreamer{
            buf_reader: buff
        }
    }
}


impl<'a> ReadStream for DefaultStreamer<'a> {
    fn read_u8(&mut self) -> Result<u8, ParseError> {
        let mut b: [u8; 1] = [0];
        self.buf_reader.read_exact(&mut b)?;
        Ok(u8::from_be_bytes(b))
    }
    fn read_u16(&mut self) -> Result<u16, ParseError> {
        let mut b: [u8; 2] = [0; 2];
        self.buf_reader.read_exact(&mut b)?;
        Ok(u16::from_be_bytes(b))
    }
    fn read_u32(&mut self) -> Result<u32, ParseError> {
        let mut b: [u8; 4] = [0; 4];
        self.buf_reader.read_exact(&mut b)?;
        Ok(u32::from_be_bytes(b))
    }
    fn read_i32(&mut self) -> Result<i32, ParseError> {
        let mut b: [u8; 4] = [0; 4];
        self.buf_reader.read_exact(&mut b)?;
        Ok(i32::from_be_bytes(b))
    }
}




struct ParseError {
    error_code: ErrorCode
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}", self.error_code)
    }
}
impl ParseError {
    fn new(err_code: ErrorCode) -> ParseError {
        ParseError{error_code: err_code}
    }
}

impl From<std::io::Error> for ParseError {
    fn from(_err: std::io::Error) -> Self {
        ParseError::new(ErrorCode::ReadError)
    }
}

impl From<std::str::Utf8Error> for ParseError {
    fn from(_err: std::str::Utf8Error) -> Self {
        ParseError::new(ErrorCode::NotUtf8Atom)
    }
}




trait MakeStr {
    fn make_str_term(&self, etype: &str, evalue: &String, result: &mut String);
}

struct DefaultMakeStr;
impl MakeStr for DefaultMakeStr {
    fn make_str_term(&self, etype: &str, evalue: &String, result: &mut String) {
        result.push_str("{\"");
        result.push_str(etype);
        result.push_str("\":");
        result.push_str(evalue);
        result.push_str("}");
    }
}

struct ReturnValueMakeStr;
impl MakeStr for ReturnValueMakeStr {
    fn make_str_term(&self, _etype: &str, evalue: &String, result: &mut String) {
        result.push_str(evalue);
    }
} 

struct SkipMakeStr;
impl MakeStr for SkipMakeStr {
    fn make_str_term(&self, _etype: &str, _evalue: &String, _result: &mut String) {}
}

fn read_u8(data_stream: &mut Read) -> Result<u8, ParseError> {
    let mut len: [u8; 1] = [0];
    data_stream.read_exact(&mut len)?;
    Ok(u8::from_be_bytes(len))
}

fn read_u16(data_stream: &mut Read) -> Result<u16, ParseError> {
    let mut len: [u8; 2] = [0; 2];
    data_stream.read_exact(&mut len)?;
    Ok(u16::from_be_bytes(len))
}

fn read_u32(data_stream: &mut Read) -> Result<u32, ParseError> {
    let mut len: [u8; 4] = [0; 4];
    data_stream.read_exact(&mut len)?;
    Ok(u32::from_be_bytes(len))
}

fn read_i32(data_stream: &mut Read) -> Result<i32, ParseError> {
    let mut val: [u8; 4] = [0; 4];
    data_stream.read_exact(&mut val)?;
    Ok(i32::from_be_bytes(val))
}

fn parse_skip(data_stream: &mut Read, n_skip: u32) -> ParseResult {
    let mstr: &MakeStr = &SkipMakeStr {};
    let mut empty_skip: String = String::new();
    for _ in 0..n_skip {
        parse_any(data_stream, mstr, &mut empty_skip)?
    }
    Ok(())
}

fn parse_atom_only(data_stream: &mut Read, result: &mut String) -> ParseResult {
    let mstr: &MakeStr = &ReturnValueMakeStr {};
    let filter_atoms: [u8; 5] = [
        SMALL_ATOM_UTF8_EXT,
        ATOM_UTF8_EXT,
        ATOM_CACHE_REF,
        ATOM_EXT,
        SMALL_ATOM_EXT
    ];
    parse_filtered(&filter_atoms, data_stream, mstr, result)
}

fn parse_filtered(filter: &[u8], data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let term_type = read_u8(data_stream)?;
    if filter.contains(&term_type) {
        parse_term(term_type, data_stream, make_str, result)
    } else {
        Err(ParseError::new(ErrorCode::NotErlangBinary))
    }
}

fn parse_any<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    parse_term(s.read_u8()?, s, c)
}

/*
fn parse_any(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let term_type = read_u8(data_stream)?;
    parse_term(term_type, data_stream, make_str, result)
}
*/
fn parse_term<S: ReadStream, C: ElemCompose>(ttype: u8, s: &mut S, c: &mut C) -> ParseResult {
    match ttype {
        LIST_EXT            => list_ext(s, c),
        STRING_EXT          => string_ext(s, c),
        INTEGER_EXT         => integer_ext(s, c),
        SMALL_INTEGER_EXT   => small_integer_ext(s, c),
        ATOM_EXT            => atom_ext(s, c),
        SMALL_ATOM_EXT      => small_atom_ext(s, c),
        SMALL_TUPLE_EXT     => small_tuple_ext(s, c),
        LARGE_TUPLE_EXT     => large_tuple_ext(s, c),
        BINARY_EXT          => binary_ext(s, c),
        FLOAT_EXT           => float_ext(s, c),
        SMALL_ATOM_UTF8_EXT => small_atom_utf8_ext(s, c),
        ATOM_UTF8_EXT       => atom_utf8_ext(s, c),
        REFERENCE_EXT       => reference_ext(s, c),
        PORT_EXT            => port_ext(s, c),
        ATOM_CACHE_REF      => atom_cache_ref(s, c),
        PID_EXT             => pid_ext(s, c),
        MAP_EXT             => map_ext(s, c),
        FUN_EXT             => fun_ext(s, c),
        SMALL_BIG_EXT       => small_big_ext(s, c),
        LARGE_BIG_EXT       => large_big_ext(s, c),
        NEW_REFERENCE_EXT   => new_reference_ext(s, c),
        EXPORT_EXT          => export_ext(s, c),
        BIT_BINARY_EXT      => bit_binary_ext(s, c),
        NEW_FLOAT_EXT       => new_float_ext(s, c),
        NEW_FUN_EXT         => new_fun_ext(s, c),
        NIL_EXT             => Ok(()),
        _ => Err(ParseError::new(ErrorCode::NotImplemented)),
    }
}



fn list_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("list");
    let l = s.read_u32()?;    
   // let mut f = || parse_any(s, c);
    create_list(l, c, &mut || parse_any(s, c))?;
    if s.read_u8()? == NIL_EXT {
        c.close();
        Ok(())
    } else {
        Err(ParseError::new(ErrorCode::InvalidListTerm))
    }
}

fn integer_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("int");
    c.push_str(s.read_i32()?.to_string());
    c.close();
    Ok(())
}

fn small_integer_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("int");
    c.push_str(s.read_u8()?.to_string());
    c.close();
    Ok(())
}

fn string_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("str");
    let l = s.read_u16()?;
    let mut f = || -> ParseResult {
        let ch = s.read_u8()? as char;
        c.push_char(ch);
        Ok(())
    };
    create_list(l as u32, c, &mut f)?;
    c.close();
    Ok(())
}
fn atom_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C)-> ParseResult {
    deprecated_atom(s.read_u16()?, s, c)
}
fn small_atom_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C)-> ParseResult {
    deprecated_atom(s.read_u8()? as u16, s, c)
}
fn deprecated_atom<S: ReadStream, C: ElemCompose>(n: u16, s: &mut S, c: &mut C) -> ParseResult {
    c.open("atom");
    c.push_char('\"');
    for _ in 0..n {
        c.push_char(s.read_u8()? as char);
    };
    c.push_char('\"');
    c.close();
    Ok(())
}

fn small_tuple_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C)-> ParseResult {
    tuple(s.read_u8()? as u32, s, c)
}

fn large_tuple_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C)-> ParseResult {
    tuple(s.read_u32()? as u32, s, c)
}

fn tuple<S: ReadStream, C: ElemCompose>(n: u32, s: &mut S, c: &mut C) -> ParseResult {
    c.open("tuple");
    create_list(n, c, &mut || parse_any(s, c))?;
    c.close();
    Ok(()) 
}

fn float_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C)-> ParseResult {
    c.open("float");
    for n in 0..31 {
        c.push_char(s.read_u8()? as char);
    };
    c.close();
    Ok(())
}

fn binary_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C)-> ParseResult {
    c.open("float");
    let len = s.read_u32()?;
    let mut v: Vec<u8> = Vec::with_capacity(len as usize);
    for _ in 0..len {
        v.push(s.read_u8()?);
    };
    c.push_char('\"');
    c.push_str(base64::encode(&v));
    c.push_char('\"');
    c.close();
    Ok(())
}
fn small_atom_utf8_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    atom_utf8(s.read_u8()? as u16, s, c)
}
fn atom_utf8_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    atom_utf8(s.read_u16()? as u16, s, c)
}

fn atom_utf8(len: u16, data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut v: Vec<u8> = vec![];
    for _ in 0..len {
        v.push(read_u8(data_stream)?);
    };
    let s = [
        "\"".to_string(),
        str::from_utf8(&v)?.to_string(),
        "\"".to_string(),
    ].concat();
    make_str.make_str_term("atom", &s, result);
    Ok(())
}

fn reference_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut node: String = String::new();
    parse_atom_only(data_stream, &mut node)?;
    let res: String = format!(
        "{{\"node\":{},\"id\":{},\"creation\":{}}}", 
        node, 
        read_u32(data_stream)?, 
        read_u8(data_stream)?
    );
    make_str.make_str_term("ref", &res, result);
    Ok(())
}

fn port_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut node: String = String::new();
    parse_atom_only(data_stream, &mut node)?;
    let res: String = format!(
        "{{\"node\":{},\"id\":{},\"creation\":{}}}", 
        node, 
        read_u32(data_stream)?, 
        read_u8(data_stream)?
    );
    make_str.make_str_term("port", &res, result);
    Ok(())
}
fn pid_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut node: String = String::new();
    parse_atom_only(data_stream, &mut node)?;
    let res: String = format!(
        "{{\"node\":{},\"id\":{},\"serial\":{},\"creation\":{}}}", 
        node, 
        read_u32(data_stream)?, 
        read_u32(data_stream)?,
        read_u8(data_stream)?
    );
    make_str.make_str_term("pid", &res, result);
    Ok(())
}

fn atom_cache_ref(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let acr = read_u8(data_stream)?;
    make_str.make_str_term("acr", &acr.to_string(), result);
    Ok(())
}

fn map_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let arity = read_u32(data_stream)?;
    let mut map_str: String = String::new();
    let mut f = |x: &mut String| -> ParseResult {
        x.push_str("{\"key\":");
        parse_any(data_stream, make_str, x)?;
        x.push_str(",\"val\":");
        parse_any(data_stream, make_str, x)?;
        x.push_str("}");
        Ok(())
    };
    create_list(arity as u32, &mut map_str, &mut f)?;
    make_str.make_str_term("map", &map_str.to_string(), result);
    Ok(())
}


fn fun_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let num_free = read_u32(data_stream)?;
    let mut pid = String::new();
    parse_any(data_stream, make_str, &mut pid)?;
    let mut module = String::new();
    parse_atom_only(data_stream, &mut module)?;
    let mut index = String::new();
    parse_any(data_stream, make_str, &mut index)?;
    let mut uniq = String::new();
    parse_any(data_stream, make_str, &mut uniq)?;
    parse_skip(data_stream, num_free)?;
    let res: String = format!(
        "{{\"pid\":{},\"m\":{},\"index\":{},\"uniq\":{}}}", 
        pid, module,  index, uniq
    );
    make_str.make_str_term("fun", &res.to_string(), result);
    Ok(())
}

fn small_big_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let n = read_u8(data_stream)?;
    big(n as u32, data_stream, make_str, result)
}

fn large_big_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let n = read_u32(data_stream)?;
    big(n, data_stream, make_str, result)
}

fn big(n: u32, data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let sign: Sign = if read_u8(data_stream)? > 0 { Sign::Minus } else { Sign::Plus};
    let mut digits: Vec<u8> = vec![];
    for _ in 0..n {
        digits.push(read_u8(data_stream)?);
    };
    let r = BigInt::from_bytes_le(sign, &digits);
    make_str.make_str_term("bigint", &r.to_string(), result);
    Ok(())
}

fn new_reference_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let len = read_u16(data_stream)?;
    let mut node: String = String::new();
    parse_atom_only(data_stream, &mut node)?;
    let creation = read_u8(data_stream)?;
    for _ in 0..len {
        read_u32(data_stream)?;
    };
    let res: String = format!(
        "{{\"node\":{},\"creation\":{}}}", 
        node, 
        creation
    );
    make_str.make_str_term("newref", &res, result);
    Ok(())
}


fn bit_binary_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let len = read_u32(data_stream)?;
    let bits = read_u8(data_stream)?;
    let mut v: Vec<u8> = vec![];
    for _ in 0..len {
        let mut one_b: [u8; 1] = [0];
        data_stream.read_exact(&mut one_b)?;
        v.push(one_b[0]);
    };
    let s = [
        "\"".to_string(),
        base64::encode(&v),
        "\"".to_string(),
    ].concat();
    let res: String = format!(
        "{{\"bits\":{},\"data\":{}}}", 
        bits, 
        s
    );
    make_str.make_str_term("bitstr", &res, result);
    Ok(())
}

fn export_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mstr: &MakeStr = &ReturnValueMakeStr {};
    let mut module = String::new();
    parse_atom_only(data_stream, &mut module)?;
    let mut func = String::new();
    parse_atom_only(data_stream, &mut func)?;
    let mut arity = String::new();
    parse_filtered(&[SMALL_INTEGER_EXT], data_stream, mstr, &mut arity)?;
    let res: String = format!(
        "{{\"m\":{},\"f\":{},\"a\":{}}}", 
        module, func, arity
    );
    make_str.make_str_term("expfun", &res, result);
    Ok(())
}
fn new_float_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut ieee_float: [u8; 8] = [0; 8];
    data_stream.read_exact(&mut ieee_float)?;
    let fl = BigEndian::read_f64(&ieee_float).to_string();
    make_str.make_str_term("float", &fl, result);
    Ok(())
}


fn new_fun_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mstr: &MakeStr = &ReturnValueMakeStr {};
    let _size = read_u32(data_stream)?;
    let arity = read_u8(data_stream)?;
    let mut uniq: [u8; 16] = [0; 16];
    data_stream.read_exact(&mut uniq)?;
    let uniq_hex = hex::encode(uniq.to_vec());
    let index = read_u32(data_stream)?;
    let num_free = read_u32(data_stream)?;

    let mut module = String::new();
    parse_atom_only(data_stream, &mut module)?;
    let mut old_index = String::new();
    parse_filtered(&[INTEGER_EXT, SMALL_INTEGER_EXT], data_stream, mstr, &mut old_index)?;
    let mut old_uniq = String::new();
    parse_filtered(&[INTEGER_EXT, SMALL_INTEGER_EXT], data_stream, mstr, &mut old_uniq)?; 
    let mut pid = String::new();
    parse_filtered(&[PID_EXT], data_stream, mstr, &mut pid)?;
    parse_skip(data_stream, num_free)?;
    let res: String = format!(
        "{{\"m\":{},\"a\":{},\"uniq\":\"{}\",\"index\":{},\"old_uniq\":{},\"old_index\":{},\"pid\":{}}}", 
        module, 
        arity,
        uniq_hex,
        index, 
        old_uniq,
        old_index,
        pid
    );
    make_str.make_str_term("newfun", &res, result);
    Ok(())
}


fn create_list<F, C>(n: u32, c: &mut C, lf: &mut F) -> ParseResult 
    where F: FnMut() -> ParseResult, C: ElemCompose {
    c.push_str("[");
    for i in 0..n {
        lf()?;
        if i + 1 < n {
            c.push_str(",");
        }
    };
    c.push_str("]");
    Ok(())
}


fn main() {
    let mut f = io::stdin();
    let mut bf = BufReader::new(f);
    match start_parsing(&mut bf) {
        Ok(json) => println!("{}", json),
        Err(error) => println!("Error: {}", error),
    }
}

fn start_parsing<BF: BufRead>(f: &mut BF) -> Result<String, ParseError> {
    let mut stream = DefaultStreamer::new(f);
    let mut composer = DefaultComposer::new();
    if stream.read_u8()? == 131 {
        parse_any(&stream, &composer)
        Ok(composer.get_result())
    } else {
        Err(ParseError::new(ErrorCode::NotErlangBinary))
    }
    /*
    if read_u8(f)? == 131 {
        parse_any(f, mstr, &mut res_str)?;
        Ok(res_str)
    } else {
        Err(ParseError::new(ErrorCode::NotErlangBinary))
    }
    */
}

