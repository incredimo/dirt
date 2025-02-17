use std::process;
use crate::{token::*, Ctx};



pub fn dprint<S: Into<String>>(s: S) {
    let debug = false;

    if debug {
        println!("{}", s.into());
    }
}


pub fn dart_parseerror<S: Into<String>>(msg: S, ctx: &Ctx, tokens: &Vec<Token>, index: usize) -> ! {

    let (linenum, symnum) = &tokens[index].find_token_position();

    if ctx.debug {
        panic!("{}:{}:{}: Error: {}", ctx.filepath, linenum, symnum, msg.into());
    }
    else {
        println!("{}:{}:{}: Error: {}", ctx.filepath, linenum, symnum, msg.into());
        process::exit(1);
    }
}


pub fn dart_evalerror<S: Into<String>>(msg: S, ctx: &Ctx) -> ! {

    if ctx.debug {
        panic!("{}: Error: {}", ctx.filepath, msg.into());
    }
    else {
        println!("{}: Error: {}", ctx.filepath, msg.into());
        process::exit(1);
    }
}
