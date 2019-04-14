use std::fs::File;
use std::io::Read;

type ParseResult = Result<(), String>;

trait MakeStr {
    fn make_str_term(&self, etype: String, evalue: &String, result: &mut String);
}

struct DefaultMakeStr;

impl MakeStr for DefaultMakeStr {
    fn make_str_term(&self, etype: String, evalue: &String, result: &mut String) {
        //    println!("Blah {} {} {}", etype, evalue, result);
        result.push_str("{\"t\":\"");
        result.push_str(&etype);
        result.push_str("\",\"v\":");
        result.push_str(evalue);
        result.push_str("}");
    }
}

fn parse(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut el_type: [u8; 1] = [0];
    return match data_stream.read_exact(&mut el_type) {
        Ok(_) => {
            return match el_type[0] {
                108 => elist_parse(data_stream, make_str, result),
                107 => estrext_parse(data_stream, make_str, result),
                98 => einteger_parse(data_stream, make_str, result),
                97 => esmall_integer_parse(data_stream, make_str, result),
                100 => eatom_ext_parse(data_stream, make_str, result),
                104 => small_tuple_ext(data_stream, make_str, result),
                _ => Err("Not implemented".to_string()),
            };
        }
        Err(_) => Err("cant read".to_string()),
    };
}

fn elist_parse(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut l_len: [u8; 4] = [0, 0, 0, 0];
    let mut list_str: String = String::new();
    return match data_stream.read_exact(&mut l_len) {
        Ok(_) => {
            let l = u32::from_be_bytes(l_len);
            list_str.push_str("[");
            for n in 0..l {
                // println!("loop: {}", n);
                match parse(data_stream, make_str, &mut list_str) {
                    Ok(_) => {
                        if n + 1 < l {
                            list_str.push_str(",");
                        }
                    }
                    Err(myerr) => return Err(myerr),
                }
            }
            list_str.push_str("]");
            let mut nil_ext: [u8; 1] = [0];
            match data_stream.read_exact(&mut nil_ext) {
                Ok(_) => {
                    if nil_ext[0] == 106 {
                        make_str.make_str_term("l".to_string(), &list_str, result);
                        Ok(())
                    } else {
                        Err("wrong list".to_string())
                    }
                }
                Err(_) => Err("Cant read".to_string()),
            }
        }
        Err(_) => Err("cant read".to_string()),
    };
}



fn einteger_parse(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut v_int_arr: [u8; 4] = [0, 0, 0, 0];
    return match data_stream.read_exact(&mut v_int_arr) {
        Ok(_) => {
            let s = [
                "\"".to_string(),
                i32::from_be_bytes(v_int_arr).to_string(),
                "\"".to_string(),
            ]
            .concat();
            make_str.make_str_term("i".to_string(), &s, result);
            Ok(())
        }
        Err(_) => Err("cant read".to_string()),
    };
}

fn esmall_integer_parse(
    data_stream: &mut Read,
    make_str: &MakeStr,
    result: &mut String,
) -> ParseResult {
    let mut v_int_arr: [u8; 1] = [0];
    return match data_stream.read_exact(&mut v_int_arr) {
        Ok(_) => {
            let s = ["\"".to_string(), v_int_arr[0].to_string(), "\"".to_string()].concat();
            make_str.make_str_term("i".to_string(), &s, result);
            Ok(())
        }
        Err(_) => Err("cant read".to_string()),
    };
}

fn estrext_parse(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut l_len: [u8; 2] = [0, 0];
    return match data_stream.read_exact(&mut l_len) {
        Ok(_) => {
            let l = u16::from_be_bytes(l_len);
            let mut str_term: String = String::new();
            str_term.push_str("[");
            for n in 0..l {
                let mut ch: [u8; 1] = [0];
                match data_stream.read_exact(&mut ch) {
                    Ok(_) => str_term.push_str(&ch[0].to_string()),
                    Err(e) => return Err(e.to_string()),
                }
                if n + 1 < l {
                    str_term.push_str(",");
                }
            }
            str_term.push_str("]");
            make_str.make_str_term("str".to_string(), &str_term, result);
            Ok(())
        }
        Err(_) => Err("cant read".to_string()),
    };
}
fn eatom_ext_parse(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut len: [u8; 2] = [0, 0];
    return match data_stream.read_exact(&mut len) {
        Ok(_) => {
            let l = u16::from_be_bytes(len);
            let mut atom_term: String = String::new();
            atom_term.push_str("\"");
            for _ in 0..l {
                let mut ch: [u8; 1] = [0];
                match data_stream.read_exact(&mut ch) {
                    Ok(_) => atom_term.push(ch[0] as char),
                    Err(e) => return Err(e.to_string()),
                }
            }
            atom_term.push_str("\"");
            make_str.make_str_term("a".to_string(), &atom_term, result);
            Ok(())
        }
        Err(_) => Err("cant read".to_string()),
    };
}

fn small_tuple_ext(data_stream: &mut Read, make_str: &MakeStr, result: &mut String) -> ParseResult {
    let mut arity: [u8; 1] = [0];
    return match data_stream.read_exact(&mut arity) {
        Ok(_) => {
            let l = u8::from_be_bytes(arity);
            let mut tuple_str: String = String::new();
            tuple_str.push_str("{");
            for n in 0..l {
                tuple_str.push_str("\"");
                tuple_str.push_str(&u8::from_be_bytes([n + 1]).to_string());
                tuple_str.push_str("\":");
                match parse(data_stream, make_str, &mut tuple_str) {
                    Ok(_) => {
                        if n + 1 < l {
                            tuple_str.push_str(",");
                        }
                    }
                    Err(myerr) => return Err(myerr),
                }
            }
            tuple_str.push_str("}");
            make_str.make_str_term("t".to_string(), &tuple_str, result);
            Ok(()) 
        },
         Err(_) => Err("cant read".to_string()),
    };
}

fn main() {
    let mut f = open_file(&"kk.bin".to_string());
    match start_parsing(&mut f) {
        Ok(json) => println!("{}", json),
        Err(error) => println!("Error: {}", error),
    }
}

fn start_parsing(f: &mut Read) -> Result<String, String> {
    let mut is_erl: [u8; 1] = [0];

    let mstr: &MakeStr = &DefaultMakeStr {};
    let mut res_str: String = String::new();
    return match f.read_exact(&mut is_erl) {
        Ok(_) => {
            if is_erl[0] == 131 {
                let res = parse(f, mstr, &mut res_str);
                match res {
                    Ok(_) => Ok(res_str.to_string()),
                    Err(er) => Err(er),
                }
            } else {
                Err("Wrong data type".to_string())
            }
        }
        Err(_) => Err("Can read stread".to_string()),
    };
}

fn open_file(name: &String) -> File {
    match File::open(name) {
        Err(_) => panic!("couldn't open"),
        Ok(file) => file,
    }
}
