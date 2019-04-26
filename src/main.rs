
use std::io::Read;
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
        result.push_str("{\"t\":\"");
        result.push_str(&etype);
        result.push_str("\",\"v\":");
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
    let mstr: &MakeStr = &ReturnValueMakeStr {};
    let mut empty_skip: String = String::new();
    for _ in 0..n_skip {
        parse_any(data_stream, mstr, &mut empty_skip)?
    }
    Ok(())
}

fn parse_filtered(filter: &[u8], data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let term_type = read_u8(data_stream)?;
    if filter.contains(&term_type) {
        parse_term(term_type, data_stream, make_str, result)
    } else {
        Err(ParseError::new(ErrorCode::NotErlangBinary))
    }
}



fn parse_any(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let term_type = read_u8(data_stream)?;
    parse_term(term_type, data_stream, make_str, result)
}

fn parse_term(term_type: u8, data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    match term_type {
        LIST_EXT            => list_ext(data_stream, make_str, result),
        STRING_EXT          => string_ext(data_stream, make_str, result),
        INTEGER_EXT         => integer_ext(data_stream, make_str, result),
        SMALL_INTEGER_EXT   => small_integer_ext(data_stream, make_str, result),
        ATOM_EXT            => atom_ext(data_stream, make_str, result),
        SMALL_ATOM_EXT      => small_atom_ext(data_stream, make_str, result),
        SMALL_TUPLE_EXT     => small_tuple_ext(data_stream, make_str, result),
        LARGE_TUPLE_EXT     => large_tuple_ext(data_stream, make_str, result),
        BINARY_EXT          => binary_ext(data_stream, make_str, result),
        FLOAT_EXT           => float_ext(data_stream, make_str, result),
        SMALL_ATOM_UTF8_EXT => small_atom_utf8_ext(data_stream, make_str, result),
        ATOM_UTF8_EXT       => atom_utf8_ext(data_stream, make_str, result),
        REFERENCE_EXT       => reference_ext(data_stream, make_str, result),
        PORT_EXT            => port_ext(data_stream, make_str, result),
        ATOM_CACHE_REF      => atom_cache_ref(data_stream, make_str, result),
        PID_EXT             => pid_ext(data_stream, make_str, result),
        MAP_EXT             => map_ext(data_stream, make_str, result),
        FUN_EXT             => fun_ext(data_stream, make_str, result),
        SMALL_BIG_EXT       => small_big_ext(data_stream, make_str, result),
        LARGE_BIG_EXT       => large_big_ext(data_stream, make_str, result),
        NEW_REFERENCE_EXT   => new_reference_ext(data_stream, make_str, result),
        EXPORT_EXT          => export_ext(data_stream, make_str, result),
        BIT_BINARY_EXT      => bit_binary_ext(data_stream, make_str, result),
        NEW_FLOAT_EXT       => new_float_ext(data_stream, make_str, result),
        NEW_FUN_EXT         => new_fun_ext(data_stream, make_str, result),
        NIL_EXT             => Ok(()),
        _ => Err(ParseError::new(ErrorCode::NotImplemented)),
    }
}



fn list_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let l = read_u32(data_stream)?;    
    let mut list_str: String = String::new();
    let mut f = |x: &mut String| parse_any(data_stream, make_str, x);
    create_list(l, &mut list_str, &mut f)?;
    if read_u8(data_stream)? == NIL_EXT {
        make_str.make_str_term("l", &list_str, result);
        Ok(())
    } else {
        Err(ParseError::new(ErrorCode::InvalidListTerm))
    }
}

fn integer_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let s = read_i32(data_stream)?.to_string();
    make_str.make_str_term("i32", &s, result);
    Ok(())
}

fn small_integer_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let s = read_u8(data_stream)?.to_string();
    make_str.make_str_term("i32", &s, result);
    Ok(())
}

fn string_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let l = read_u16(data_stream)?;
    let mut str_term: String = String::new();
    let mut f = |x: &mut String| -> ParseResult {
        x.push_str(&(read_u8(data_stream)?).to_string());
        Ok(())
    };
    create_list(l as u32, &mut str_term, &mut f)?;
    make_str.make_str_term("str", &str_term, result);
    Ok(())
}
fn atom_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let n = read_u16(data_stream)?;
    deprecated_atom(n, data_stream, make_str, result)
}
fn small_atom_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let n = read_u8(data_stream)?;
    deprecated_atom(n as u16, data_stream, make_str, result)
}

fn deprecated_atom(n: u16, data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut atom_term: String = String::new();
    atom_term.push_str("\"");
    for _ in 0..n {
        atom_term.push(read_u8(data_stream)? as char);
    };
    atom_term.push_str("\"");
    make_str.make_str_term("a", &atom_term, result);
    Ok(())
}

fn small_tuple_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let n = read_u8(data_stream)?;
    tuple(n as u32, data_stream, make_str, result)
}

fn large_tuple_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let n = read_u32(data_stream)?;
    tuple(n, data_stream, make_str, result)
}

fn tuple(n: u32, data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut tuple_str: String = String::new();
    let mut f = |x: &mut String| parse_any(data_stream, make_str, x);
    create_list(n, &mut tuple_str, &mut f)?;
    make_str.make_str_term("t", &tuple_str, result);
    Ok(()) 
}

fn float_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut float_arr: [u8; 31] = [0; 31];
    data_stream.read_exact(&mut float_arr)?;
    let mut res = String::new();
    for n in 0..31 {
        res.push(float_arr[n] as char);
    }
    make_str.make_str_term("f", &res, result);
    Ok(())
}

fn binary_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let len = read_u32(data_stream)?;
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
    make_str.make_str_term("b", &s, result);
    Ok(())
}
fn small_atom_utf8_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let len = read_u8(data_stream)?;
    atom_utf8(len as u16, data_stream, make_str, result)
}
fn atom_utf8_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let len = read_u16(data_stream)?;
    atom_utf8(len, data_stream, make_str, result)
}

fn atom_utf8(len: u16, data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut v: Vec<u8> = vec![];
    for _ in 0..len {
        let mut one_b: [u8; 1] = [0];
        data_stream.read_exact(&mut one_b)?;
        v.push(one_b[0]);
    };
    let s = [
        "\"".to_string(),
        str::from_utf8(&v)?.to_string(),
        "\"".to_string(),
    ].concat();
    make_str.make_str_term("a", &s, result);
    Ok(())
}

fn reference_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut node: String = String::new();
    parse_any(data_stream, make_str, &mut node)?;
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
    parse_any(data_stream, make_str, &mut node)?;
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
    parse_any(data_stream, make_str, &mut node)?;
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
    make_str.make_str_term("m", &map_str.to_string(), result);
    Ok(())
}


fn fun_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let num_free = read_u32(data_stream)?;
    let mut pid = String::new();
    parse_any(data_stream, make_str, &mut pid)?;
    let mut module = String::new();
    parse_any(data_stream, make_str, &mut module)?;
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
    make_str.make_str_term("bi", &r.to_string(), result);
    Ok(())
}

fn new_reference_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mstr: &MakeStr = &ReturnValueMakeStr {};
    let filter_atoms: [u8; 5] = [
        SMALL_ATOM_UTF8_EXT,
        ATOM_UTF8_EXT,
        ATOM_CACHE_REF,
        ATOM_EXT,
        SMALL_ATOM_EXT
    ];
    let len = read_u16(data_stream)?;
    let mut node: String = String::new();
    parse_filtered(&filter_atoms, data_stream, mstr, &mut node)?;
    let creation = read_u8(data_stream)?;
    for _ in 0..len {
        read_u32(data_stream)?;
    };
    let res: String = format!(
        "{{\"node\":{},\"creation\":{}}}", 
        node, 
        creation
    );
    make_str.make_str_term("nref", &res, result);
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
    make_str.make_str_term("bs", &res, result);
    Ok(())
}

fn export_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mstr: &MakeStr = &ReturnValueMakeStr {};
    let filter_atoms: [u8; 5] = [
        SMALL_ATOM_UTF8_EXT,
        ATOM_UTF8_EXT,
        ATOM_CACHE_REF,
        ATOM_EXT,
        SMALL_ATOM_EXT
    ];
    let mut module = String::new();
    parse_filtered(&filter_atoms, data_stream, mstr, &mut module)?;
    let mut func = String::new();
    parse_filtered(&filter_atoms, data_stream, mstr, &mut func)?;
    let mut arity = String::new();
    parse_filtered(&[SMALL_INTEGER_EXT], data_stream, mstr, &mut arity)?;
    let res: String = format!(
        "{{\"m\":{},\"f\":{},\"a\":{}}}", 
        module, func, arity
    );
    make_str.make_str_term("efun", &res, result);
    Ok(())
}
fn new_float_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut ieee_float: [u8; 8] = [0; 8];
    data_stream.read_exact(&mut ieee_float)?;
    let fl = BigEndian::read_f64(&ieee_float).to_string();
    make_str.make_str_term("f", &fl, result);
    Ok(())
}


fn new_fun_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mstr: &MakeStr = &ReturnValueMakeStr {};
    let filter_atoms: [u8; 5] = [
        SMALL_ATOM_UTF8_EXT,
        ATOM_UTF8_EXT,
        ATOM_CACHE_REF,
        ATOM_EXT,
        SMALL_ATOM_EXT
    ];
    let _size = read_u32(data_stream)?;
    let arity = read_u8(data_stream)?;
    let mut uniq: [u8; 16] = [0; 16];
    data_stream.read_exact(&mut uniq)?;
    let uniq_hex = hex::encode(uniq.to_vec());
    let index = read_u32(data_stream)?;
    let num_free = read_u32(data_stream)?;

    let mut module = String::new();
    parse_filtered(&filter_atoms, data_stream, mstr, &mut module)?;
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
    make_str.make_str_term("nfun", &res, result);
    Ok(())
}


fn create_list<F>(n: u32, result: &mut String, lf: &mut F) -> ParseResult 
    where F: FnMut(&mut String) -> ParseResult {
    result.push_str("[");
    for i in 0..n {
        lf(result)?;
        if i + 1 < n {
            result.push_str(",");
        }
    };
    result.push_str("]");
    Ok(())
}


fn main() {
    let mut f = io::stdin();
    match start_parsing(&mut f) {
        Ok(json) => println!("{}", json),
        Err(error) => println!("Error: {}", error),
    }
}

fn start_parsing(f: &mut Read) -> Result<String, ParseError> {
   // let mut is_erl: [u8; 1] = [0];
    let mstr: &MakeStr = &DefaultMakeStr {};
    let mut res_str: String = String::new();
  //  f.read_exact(&mut is_erl)?;
    if read_u8(f)? == 131 {
        parse_any(f, mstr, &mut res_str)?;
        Ok(res_str)
    } else {
        Err(ParseError::new(ErrorCode::NotErlangBinary))
    }
}

