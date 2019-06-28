use std::str;
use std::io;
use num_bigint::Sign;
use num_bigint::BigInt;
use byteorder::{ByteOrder, BigEndian};
extern crate hex;

use super::read_stream::ReadStream;
use super::elem_compose::ElemCompose;
use super::parse_result::{ParseResult, ParseError, ErrorCode};

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

struct SkipComposer;

impl ElemCompose for SkipComposer {
    fn open(&mut self, _name: &str) {}
    fn push_str<T: AsRef<str>>(&mut self, _elem: T) {}
    fn push_char(&mut self, _elem: char) {}
    fn close(&mut self) {}
}

struct RawStringComposer<'a> {
    result: &'a mut String
}

impl <'a>RawStringComposer<'a> {
    fn new(res_str: &'a mut String) -> RawStringComposer {
        RawStringComposer{
            result: res_str
        }
    }
}

impl<'a> ElemCompose for RawStringComposer<'a> {
    fn open(&mut self, _name: &str) {}
    fn push_str<T: AsRef<str>>(&mut self, elem: T) {
        self.result.push_str(elem.as_ref())
    }
    fn push_char(&mut self, elem: char) {
        self.result.push(elem);
    }
    fn close(&mut self) {}
}


pub fn parse<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    if s.read_u8()? == 131 {
        parse_any(s, c)?;
        Ok(())
    } else {
        Err(ParseError::new(ErrorCode::NotErlangBinary))
    }
}

fn parse_any<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    parse_term(s.read_u8()?, s, c)
}

fn parse_skip<S: ReadStream>(s: &mut S, n_skip: u32) -> ParseResult {
    let mut skip_comp = SkipComposer{};
    for _ in 0..n_skip {
        parse_any(s, &mut skip_comp)?
    }
    Ok(())
}

fn parse_atom_only<S: ReadStream>(s: &mut S, res: &mut String) -> ParseResult {
    let filter_atoms: [u8; 5] = [
        SMALL_ATOM_UTF8_EXT,
        ATOM_UTF8_EXT,
        ATOM_CACHE_REF,
        ATOM_EXT,
        SMALL_ATOM_EXT
    ];
    parse_filtered(&filter_atoms, s, res)
}

fn parse_filtered<S: ReadStream>(filter: &[u8], s: &mut S, res: &mut String) -> ParseResult {
    let term_type = s.read_u8()?;
    if filter.contains(&term_type) {
        let mut str_comp = RawStringComposer::new(res);
        parse_term(term_type, s, &mut str_comp)
    } else {
        Err(ParseError::new(ErrorCode::NotErlangBinary))
    }
}

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
    c.push_char('[');
    for i in 0..l {
        parse_any(s, c)?;
        if i + 1 < l {
            c.push_char(',');
        }
    };
    c.push_char(']');
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
    c.push_char('[');
    for i in 0..l {
        c.push_char(s.read_u8()? as char);
        if i + 1 < l {
            c.push_char(',');
        }
    };
    c.push_char(']');
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
    tuple(s.read_u32()?, s, c)
}

fn tuple<S: ReadStream, C: ElemCompose>(n: u32, s: &mut S, c: &mut C) -> ParseResult {
    c.open("tuple");
        c.push_char('[');
    for i in 0..n {
        parse_any(s, c)?;
        if i + 1 < n {
            c.push_char(',');
        }
    };
    c.push_char(']');
    c.close();
    Ok(()) 
}

fn float_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C)-> ParseResult {
    c.open("float");
    for _ in 0..31 {
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

fn atom_utf8<S: ReadStream, C: ElemCompose>(len: u16, s: &mut S, c: &mut C) -> ParseResult {
    c.open("atom");
    let mut v: Vec<u8> = vec![];
    for _ in 0..len {
        v.push(s.read_u8()?);
    };
    c.push_char('\"');
    c.push_str(str::from_utf8(&v)?);
    c.push_char('\"');
    c.close();
    Ok(())
}

fn reference_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("ref");
    let mut node: String = String::new();
    parse_atom_only(s, &mut node)?;
    c.push_str("{\"node\":");
    c.push_str(node);
    c.push_str(",\"id\":");
    c.push_str(s.read_u32()?.to_string());
    c.push_str(",\"creation\":");
    c.push_str(s.read_u8()?.to_string());
    c.push_char('}');
    c.close();
    Ok(())
}

fn port_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("port");
    let mut node: String = String::new();
    parse_atom_only(s, &mut node)?;
    c.push_str("{\"node\":");
    c.push_str(node);
    c.push_str(",\"id\":");
    c.push_str(s.read_u32()?.to_string());
    c.push_str(",\"creation\":");
    c.push_str(s.read_u8()?.to_string());
    c.push_char('}');
    c.close();
    Ok(())
}
fn pid_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("pid");
    let mut node: String = String::new();
    parse_atom_only(s, &mut node)?;
    c.push_str("{\"node\":");
    c.push_str(node);
    c.push_str(",\"id\":");
    c.push_str(s.read_u32()?.to_string());
    c.push_str(",\"serial\":");
    c.push_str(s.read_u32()?.to_string());
    c.push_str(",\"creation\":");
    c.push_str(s.read_u8()?.to_string());
    c.push_char('}');
    c.close();
    Ok(())
}

fn atom_cache_ref<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("acr");
    c.push_str(s.read_u8()?.to_string());
    c.close();
    Ok(())
}

fn map_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("map");
    let n = s.read_u32()?;
    c.push_char('[');
    for i in 0..n {
        c.push_str("{\"key\":");
        parse_any(s, c)?;
        c.push_str(",\"val\":");
        parse_any(s, c)?;
        c.push_str("}");
        if i + 1 < n {
            c.push_char(',');
        }
    };
    c.push_char(']');
    c.close();
    Ok(())
}


fn fun_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    let num_free = s.read_u32()?;
    let mut pid = String::new();
    parse_any(s, &mut pid)?;
    let mut module = String::new();
    parse_atom_only(s, &mut module)?;
    let mut index = String::new();
    parse_any(s, &mut index)?;
    let mut uniq = String::new();
    parse_any(s, &mut uniq)?;
    parse_skip(s, num_free)?;
    c.push_str(format!(
        "{{\"pid\":{},\"m\":{},\"index\":{},\"uniq\":{}}}", 
        pid, module,  index, uniq
    ));
    c.close();
    Ok(())
}

fn small_big_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    big(s.read_u8()? as usize, s, c)
}

fn large_big_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    big(s.read_u32()? as usize, s, c)
}
// TODO: deside best type
fn big<S: ReadStream, C: ElemCompose>(n: usize, s: &mut S, c: &mut C) -> ParseResult {
    c.open("bigint");
    let sign: Sign = if s.read_u8()? > 0 { Sign::Minus } else { Sign::Plus};
    let mut digits: Vec<u8> = Vec::with_capacity(n);
    for _ in 0..n {
        digits.push(s.read_u8()?);
    };
    let r = BigInt::from_bytes_le(sign, &digits);
    c.push_str(r.to_string());
    Ok(())
}

fn new_reference_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    let len = s.read_u16()?;
    let mut node: String = String::new();
    parse_atom_only(s, &mut node)?;
    let creation = s.read_u8()?;
    for _ in 0..len {
        s.read_u32()?;
    };
    c.push_str(format!(
        "{{\"node\":{},\"creation\":{}}}", 
        node, 
        creation
    ));
    c.close();
    Ok(())
}


fn bit_binary_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("bitstr");
    let len = s.read_u32()?;
    let bits = s.read_u8()?;
    let mut v: Vec<u8> = vec![];
    for _ in 0..len {
        v.push(s.read_u8()?);
    };
    c.push_str("{\"bits\":");
    c.push_str(bits.to_string());
    c.push_str(",\"data\":\"");
    c.push_str(base64::encode(&v));
    c.push_str("\"}");
    c.close();
    Ok(())
}

fn export_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("expfun");
    let mut module = String::new();
    parse_atom_only(s, &mut module)?;
    let mut func = String::new();
    parse_atom_only(s, &mut func)?;
    let mut arity = String::new();
    parse_filtered(&[SMALL_INTEGER_EXT], s, &mut arity)?;
    c.push_str("{\"m\":"); c.push_str(module);
    c.push_str(",\"f\":"); c.push_str(func);
    c.push_str(",\"a\":"); c.push_str(arity);
    c.push_char('}');
    c.close();
    Ok(())
}
fn new_float_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("float");
    let mut ieee_float: [u8; 8] = [0; 8];
    s.read_exact(&mut ieee_float)?;
    let fl = BigEndian::read_f64(&ieee_float).to_string();
    c.push_str(fl);
    c.close();
    Ok(())
}


fn new_fun_ext<S: ReadStream, C: ElemCompose>(s: &mut S, c: &mut C) -> ParseResult {
    c.open("newfun");
    let _size = s.read_u32()?;
    let arity = s.read_u8()?;
    let mut uniq: [u8; 16] = [0; 16];
    s.read_exact(&mut uniq)?;
    let uniq_hex = hex::encode(uniq.to_vec());
    let index = s.read_u32()?;
    let num_free = s.read_u32()?;

    let mut module = String::new();
    parse_atom_only(s, &mut module)?;
    let mut old_index = String::new();
    parse_filtered(&[INTEGER_EXT, SMALL_INTEGER_EXT], s, &mut old_index)?;
    let mut old_uniq = String::new();
    parse_filtered(&[INTEGER_EXT, SMALL_INTEGER_EXT], s, &mut old_uniq)?; 
    let mut pid = String::new();
    parse_filtered(&[PID_EXT], s, &mut pid)?;
    parse_skip(s, num_free)?;
    c.push_str(format!(
        "{{\"m\":{},\"a\":{},\"uniq\":\"{}\",\"index\":{},\"old_uniq\":{},\"old_index\":{},\"pid\":{}}}", 
        module, 
        arity,
        uniq_hex,
        index, 
        old_uniq,
        old_index,
        pid
    ));
    c.close();
    Ok(())
}
