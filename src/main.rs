use std::io;
use std::io::{Read, Write};
use std::env;
extern crate iconv;
extern crate getopts;

#[derive(Copy, Clone, PartialEq)]
enum CharacterTable {
    CP866,
    CP437
}

#[derive(Copy, Clone, PartialEq)]
enum TypeFace {
    Draft,
    Roman,
    Serif
}

#[derive(Copy, Clone, PartialEq)]
enum FontSize {
    CPI10,
    CPI12,
    CPI15
}

#[derive(Copy, Clone)]
struct Chmapped {
    code: u8,
    chm: CharacterTable
}

fn select_charmap(t: CharacterTable) {
    let (d2, d3) = match t {
        CharacterTable::CP437 => (1 as u8, 0 as u8),
        CharacterTable::CP866 => (14 as u8, 0 as u8),
    };
    let mut epson_command: Vec<u8> = vec![0x1B, 0x28, 0x74, 0x03, 0x00, 0x01];
    epson_command.push(d2);
    epson_command.push(d3);
    let sz = io::stdout().write(epson_command.as_slice()).unwrap();
    assert_eq!(sz, epson_command.len());
}

fn transcode_character(ch: char) -> Vec<Chmapped> {
    let mut rsp = Vec::new();
    let encodings = vec!["437", "866"];

    let mchar = match ch {
        'І' => 'I',
        'і' => 'i',
        'Ґ' => 'Г',
        'ґ' => 'г',
        _ => ch
    };
    for en in encodings {
        let chstr = mchar.to_string();
        let r = iconv::encode(chstr.as_str(), en);
        if r.is_ok() {
            if en == "437" {
                rsp.push(Chmapped { code: r.unwrap()[0], chm: CharacterTable::CP437 });
            } else if en == "866" {
                rsp.push(Chmapped { code: r.unwrap()[0], chm: CharacterTable::CP866 });
            }
        }
    }
    if rsp.len() == 0 {
        rsp.push(Chmapped {code: 0x20, chm: CharacterTable::CP866});
        rsp.push(Chmapped {code: 0x20, chm: CharacterTable::CP437});
    }
    rsp
}

struct PrinterState {
    codepage: CharacterTable,
    columns: usize,
    typeface: TypeFace,
    fontsize: FontSize,
    condensed: bool,
    bold: bool,
    underline: bool
}

pub trait Process {
    fn process(&mut self, c: char);
}

impl Process for PrinterState {
    fn process(&mut self, c: char) {
        let chm = transcode_character(c);
        let mut out_buf: Vec<u8> = Vec::new();
        let mut need_switch_coding = true;
        let mut symbol: u8 = chm[0].code;
        for m in &chm {
            if m.chm == self.codepage {
                need_switch_coding = false;
                symbol = m.code;
            }
        }
        if need_switch_coding == true {
            self.codepage = chm[0].chm;
            select_charmap(self.codepage);
        }
        out_buf.push(symbol);
        let sz = io::stdout().write(out_buf.as_slice()).unwrap();
        assert_eq!(1, sz);
    }
}

fn set_number_of_columns(columns: usize) {
    assert!(columns <= 80);
    let mut epson_command: Vec<u8> = vec![0x1B, 0x51];
    epson_command.push(columns as u8);
    let sz = io::stdout().write(epson_command.as_slice()).unwrap();
    assert_eq!(epson_command.len(), sz);
}

fn set_font(f: TypeFace) {
    let epson_command: Vec<u8> = match f {
        TypeFace::Roman => vec![0x1B, 0x78, 0x01, 0x1B, 0x6B, 0x00],
        TypeFace::Serif => vec![0x1B, 0x78, 0x01, 0x1B, 0x6B, 0x01],
        TypeFace::Draft => vec![0x1B, 0x78, 0x00],
    };
    let sz = io::stdout().write(epson_command.as_slice()).unwrap();
    assert_eq!(epson_command.len(), sz);
}

fn set_font_size(f: FontSize) {
    let epson_command: Vec<u8> = match f {
        FontSize::CPI10 => vec![0x1B, 0x50],
        FontSize::CPI12 => vec![0x1B, 0x4D],
        FontSize::CPI15 => vec![0x1B, 0x67],
    };
    let sz = io::stdout().write(epson_command.as_slice()).unwrap();
    assert_eq!(epson_command.len(), sz);
}

fn set_condensed(c: bool) {
    let epson_command: Vec<u8> = if c {
        vec![0x1B, 0x0F]
    } else {
        vec![0x12]
    };
    let sz = io::stdout().write(epson_command.as_slice()).unwrap();
    assert_eq!(epson_command.len(), sz);
}

fn set_bold(b: bool) {
    let epson_command: Vec<u8> = if b {
        vec![0x1B, 0x45]
    } else {
        vec![0x1B, 0x46]
    };
    let sz = io::stdout().write(epson_command.as_slice()).unwrap();
    assert_eq!(epson_command.len(), sz);
}

fn set_underline(u: bool) {
    let epson_command: Vec<u8> = if u {
        vec![0x1B, 0x2D, 0x01]
    } else {
        vec![0x1B, 0x2D, 0x00]
    };
    let sz = io::stdout().write(epson_command.as_slice()).unwrap();
    assert_eq!(epson_command.len(), sz);
}

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief).as_str());
}

fn main() {
    let mut inbuf: [u8; 4096] = [0; 4096];
    let mut state: PrinterState = PrinterState {
        codepage: CharacterTable::CP437,
        columns: 75,
        typeface: TypeFace::Draft,
        fontsize: FontSize::CPI10,
        condensed: false,
        bold: false,
        underline: false
    };

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = getopts::Options::new();
    opts.optopt("c", "cols", "set number of columns", "COLS");
    opts.optopt("s", "size", "font size", "<10 | 12 | 15>");
    opts.optopt("f", "font", "font", "<draft | roman | serif>");
    opts.optflag("", "condensed", "condensed");
    opts.optflag("b", "bold", "bold typefaces");
    opts.optflag("u", "under", "underline typeface");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!("{}", f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.opt_present("u") {
        state.underline = true;
    } else {
        state.underline = false;
    }

    if matches.opt_present("b") {
        state.bold = true;
    } else {
        state.bold = false;
    }

    if matches.opt_present("condensed") {
        state.condensed = true;
    } else {
        state.condensed = false;
    }

    if matches.opt_present("font") {
        let font = matches.opt_str("font").unwrap();
        state.typeface = match font.as_str() {
            "draft" => TypeFace::Draft,
            "serif" => TypeFace::Serif,
            "roman" => TypeFace::Roman,
            _ => panic!("Unknown typeface name provided")
        };
    }

    if matches.opt_present("size") {
        let sz_txt = matches.opt_str("size").unwrap();
        state.fontsize = match sz_txt.as_str() {
            "10" => FontSize::CPI10,
            "12" => FontSize::CPI12,
            "15" => FontSize::CPI15,
            _ => panic!("Unsupported font size")
        };
    }

    if matches.opt_present("c") {
        let cols_txt = matches.opt_str("c").unwrap();
        let cols = cols_txt.parse::<usize>().unwrap();
        assert!(cols <= 80);
        state.columns = cols;
    }

    set_condensed(false);
    set_bold(false);
    set_underline(false);
    set_font(TypeFace::Draft);
    set_font_size(FontSize::CPI10);

    set_number_of_columns(state.columns);
    set_font(state.typeface);
    set_font_size(state.fontsize);
    set_condensed(state.condensed);
    set_bold(state.bold);
    set_underline(state.underline);
    loop {
        let isz = io::stdin().read(&mut inbuf).unwrap();
        let line = String::from_utf8(Vec::from(&inbuf[0..isz])).unwrap();
        if isz > 0 {
            for chr in line.chars(){
                state.process(chr);
            }
            assert!(io::stdout().flush().is_ok());
        } else {
            break;
        }
    }
}
