
use std::io::Read;
use std::fmt;
use std::str;
use std::io;
use num_bigint::Sign;
use num_bigint::BigInt;



type ParseResult = Result<(), ParseError>;

#[derive(Debug)]
enum ErrorCode {
    ReadError = 1,
    NotImplemented = 2,
    InvalidListTerm = 3,
    NotErlangBinary = 4,
    NotUtf8Atom = 5,
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
        result.push_str("{\"t\":\"");
        result.push_str(&etype);
        result.push_str("\",\"v\":");
        result.push_str(evalue);
        result.push_str("}");
    }
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

fn parse(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut el_type: [u8; 1] = [0];
    data_stream.read_exact(&mut el_type)?;
    match el_type[0] {
        108 => list_ext(data_stream, make_str, result),
        107 => string_ext(data_stream, make_str, result),
        98  => integer_ext(data_stream, make_str, result),
        97  => small_integer_ext(data_stream, make_str, result),
        100 => atom_ext(data_stream, make_str, result),
        115 => small_atom_ext(data_stream, make_str, result),
        104 => small_tuple_ext(data_stream, make_str, result),
        105 => large_tuple_ext(data_stream, make_str, result),
        109 => binary_ext(data_stream, make_str, result),
        99  => float_ext(data_stream, make_str, result),
        119 => small_atom_utf8_ext(data_stream, make_str, result),
        118 => atom_utf8_ext(data_stream, make_str, result),
        101 => reference_ext(data_stream, make_str, result),
        102 => port_ext(data_stream, make_str, result),
        82  => atom_cache_ref(data_stream, make_str, result),
        103 => pid_ext(data_stream, make_str, result),
        116 => map_ext(data_stream, make_str, result),
        117 => fun_ext(data_stream, make_str, result),
        110 => small_big_ext(data_stream, make_str, result),
        111 => large_big_ext(data_stream, make_str, result),
        114 => new_reference_ext(data_stream, make_str, result),
        113 => export_ext(data_stream, make_str, result),
        77  => bit_binary_ext(data_stream, make_str, result),
        _ => Err(ParseError::new(ErrorCode::NotImplemented)),
    }
}

fn list_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let l = read_u32(data_stream)?;    
    let mut list_str: String = String::new();
    let mut f = |x: &mut String| parse(data_stream, make_str, x);
    create_list(l, &mut list_str, &mut f)?;
    if read_u8(data_stream)? == 106 {
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
    let mut f = |x: &mut String| parse(data_stream, make_str, x);
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
    parse(data_stream, make_str, &mut node)?;
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
    parse(data_stream, make_str, &mut node)?;
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
    parse(data_stream, make_str, &mut node)?;
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
        parse(data_stream, make_str, x)?;
        x.push_str(",\"val\":");
        parse(data_stream, make_str, x)?;
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
    parse(data_stream, make_str, &mut pid)?;
    let mut module = String::new();
    parse(data_stream, make_str, &mut module)?;
    let mut index = String::new();
    parse(data_stream, make_str, &mut index)?;
    let mut uniq = String::new();
    parse(data_stream, make_str, &mut uniq)?;
    let mut free_vars = String::new();
    let mut f = |x: &mut String| parse(data_stream, make_str, x);
    create_list(num_free as u32, &mut free_vars, &mut f)?;
    let res: String = format!(
        "{{\"pid\":{},\"mod\":{},\"index\":{},\"uniq\":{},\"vars\":{}}}", 
        pid, module,  index, uniq, free_vars
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
    let _len = read_u16(data_stream)?;
    let mut node: String = String::new();
    parse(data_stream, make_str, &mut node)?;
    let creation = read_u8(data_stream)?;
    let res: String = format!(
        "{{\"node\":{},\"creation\":{}}}", 
        node, 
        creation
    );
    make_str.make_str_term("nref", &res, result);
    Ok(())
}

fn export_ext(_data_stream: &mut Read, _make_str: &MakeStr, _result: &mut String) -> ParseResult {
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
    let mut is_erl: [u8; 1] = [0];
    let mstr: &MakeStr = &DefaultMakeStr {};
    let mut res_str: String = String::new();
    f.read_exact(&mut is_erl)?;
    if is_erl[0] == 131 {
        parse(f, mstr, &mut res_str)?;
        Ok(res_str)
    } else {
        Err(ParseError::new(ErrorCode::NotErlangBinary))
    }
}

