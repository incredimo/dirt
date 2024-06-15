use  rustyline;
use  nuid;
use queues;

mod parser;
mod lexer;
mod evaluator;
mod builtin;
mod utils;
mod stack;
mod objsys;
mod expression;
mod token;
mod node;
mod object;
mod testlist;
mod context;
mod reader;

use std::{ fs::read_dir, io::prelude::* };
use std::env;
use std::fs::File;
use std::collections::HashMap;
use crate::context::*;
use minimo::{divider, showln};
use stack::Stack;
use objsys::ObjSys;
use node::{ Node, NodeType };

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Argument expected.");
    }

    let mut ctx = Ctx {
        filepath: String::from(""),
        debug: true,
    };

    let a1 = &args[1];

    match a1.as_str() {
        "lex" => {
            if args.len() < 3 {
                println!("Please specify file ...");
                return;
            }
            ctx.filepath = String::from(&args[2]);
            do_task("lex", args[2].clone(), &mut ctx);
        }
        "parse" => {
            if args.len() < 3 {
                println!("Please specify file...");
                return;
            }
            ctx.filepath = String::from(&args[2]);
            do_task("parse", args[2].clone(), &mut ctx);
        }
        "test" => {
            if args.len() < 3 {
                println!("Running all tests:");
                for s in testlist::TESTS {
      
                    if let Ok(path) = read_dir("./test") {
                        for entry in path {
                            if let Ok(entry) = entry {
                                let filename = entry.file_name().into_string().unwrap();
                                divider!(&filename,"-" );
                                do_task("eval", entry.path(), &mut ctx);
                                divider!();
                                println!(" ");
                            }
                        }
                    }
                }
            }

            let a2: &String = &args[2];
            let mut task = "eval";
            let nextarg: &String;

            match a2.as_str() {
                "lex" => {
                    task = "lex";
                    nextarg = &args[3];
                }
                "parse" => {
                    task = "parse";
                    nextarg = &args[3];
                }
                "eval" => {
                    nextarg = &args[3];
                }
                _ => {
                    nextarg = &args[2];
                }
            }

            let filepath = testlist::get_filepath(nextarg.clone());
            ctx.filepath = filepath.clone();
            do_task(task, filepath, &mut ctx);
        }
        "testfail" => {
            if args.len() < 3 {
                println!("Running all fail tests:");
                for s in testlist::FAILTESTS {
                    let path = format!("{}/{}", testlist::FAILTESTPATH, s);
                    ctx.filepath = String::from(path.as_str());
                    do_task("eval", String::from(path.as_str()), &mut ctx);
                }
                return;
            }

            let a2: &String = &args[2];
            let mut task = "eval";
            let nextarg: &String;

            match a2.as_str() {
                "lex" => {
                    task = "lex";
                    nextarg = &args[3];
                }
                "parse" => {
                    task = "parse";
                    nextarg = &args[3];
                }
                "eval" => {
                    nextarg = &args[3];
                }
                _ => {
                    nextarg = &args[2];
                }
            }

            let filepath = testlist::get_failfilepath(nextarg.clone());
            ctx.filepath = filepath.clone();
            do_task(task, filepath, &mut ctx);
        }
        _ => {
            println!("Illegal argument: {}", a1);
        }
    }
}

fn do_task(action: &str, path : impl AsRef<std::path::Path>, ctx: &mut Ctx) {
    match action {
        "lex" => {
            let input = read_file(path);
            let reader = lexer::lex(&input);
            for t in reader.tokens() {
                print!("{} ", t);
            }
            println!();
        }
        "parse" => {
            let input = read_file(path);
            let mut tokens = lexer::lex(&input);
            let mut globals: Vec<Node> = Vec::new();
            let mut objsys = ObjSys::new();
            parser::parse(&mut tokens, &mut globals, &mut objsys, ctx);

            for f in globals {
                println!("\n{}\n", f);
            }
        }
        "eval" => {
        evaluate(path, ctx);
        }
        x => {
            println!("Unknown action: {}", x);
        }
    }
}

fn filecurse(
    basepath: String,
    filepath: String,
    memo: &mut HashMap<String, (usize, usize)>,
    looktables: &mut HashMap<String, HashMap<String, usize>>,
    globals: &mut Vec<Node>,
    store: &mut Stack,
    objsys: &mut ObjSys,
    ctx: &mut Ctx
) {
    let mut fpath = basepath.clone();
    fpath.push_str("/");
    fpath.push_str(filepath.as_str());

    

    let input = read_file(fpath.as_str());
    let mut tokens = lexer::lex(&input);

    ctx.filepath = filepath.clone();

    let oldlen = globals.len();

    let imports = parser::parse(&mut tokens, globals, objsys, ctx);

    memo.insert(filepath.clone(), (oldlen, globals.len()));

    let mut looktable: HashMap<String, usize> = HashMap::new();

    for i in oldlen..globals.len() {
        let f = &globals[i];

        match &f.nodetype {
            NodeType::FunDef(funcname, _) => {
                looktable.insert(funcname.clone(), i);
            }
            NodeType::Constructor(name, _) => {
                looktable.insert(name.clone(), i);
            }
            _ => {
                panic!("Unexpected node type in globals");
            }
        }
    }

    for s in imports {
        if memo.contains_key(&s) {
            continue;
        }

        filecurse(basepath.clone(), s.clone(), memo, looktables, globals, store, objsys, ctx);

        // For every import, merge its functions into this files looktable.

        let (childstart, childend) = memo[&s];

        for i in childstart..childend {
            let f = &globals[i];
            match &f.nodetype {
                NodeType::FunDef(funcname, _) => {
                    looktable.insert(funcname.clone(), i);
                }
                NodeType::Constructor(name, _) => {
                    looktable.insert(name.clone(), i);
                }
                _ => {
                    panic!("Unexpected node type in globals");
                }
            }
        }
    }

    looktables.insert(filepath.clone(), looktable);
}

fn evaluate(filepath:  impl AsRef<std::path::Path>, ctx: &mut Ctx) {
    let mut globals: Vec<Node> = Vec::new();
    let mut memo: HashMap<String, (usize, usize)> = HashMap::new();
    let mut looktables: HashMap<String, HashMap<String, usize>> = HashMap::new();
    let mut store = Stack::new();
    let mut objsys = ObjSys::new();

    let basepath = String::from(filepath.as_ref().parent().unwrap().to_str().unwrap());
    let filename = filepath.as_ref().file_name().unwrap().to_str().unwrap();

    //handle if path is directory
    if filepath.as_ref().is_dir() {
        let path = filepath.as_ref();
        if let Ok(entries) = read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        let filename = path.file_name().unwrap().to_str().unwrap();
                        filecurse(
                            basepath.clone(),
                            String::from(filename),
                            &mut memo,
                            &mut looktables,
                            &mut globals,
                            &mut store,
                            &mut objsys,
                            ctx
                        );
                    }
                }
            }
        }
    }



   else {
    filecurse(
        basepath.clone(),
        String::from(filename),
        &mut memo,
        &mut looktables,
        &mut globals,
        &mut store,
        &mut objsys,
        ctx
    );

    let toptable = &looktables[filename];

    if !toptable.contains_key("main") {
        // As Dart.
        panic!("Error: No 'main' method found.");
    }

    let mainindex: &usize = toptable.get("main").unwrap();
    let mainfunc = &globals[*mainindex];
    ctx.filepath = filename.to_string();

    match &mainfunc.nodetype {
        NodeType::FunDef(_, _) => {
            utils::dprint(" ");
            utils::dprint("EVALUATE");
            utils::dprint(" ");

            let mainbody = &mainfunc.children[1];

            store.push_call();
            evaluator::eval(mainbody, &looktables, &globals, &mut store, &mut objsys, ctx);
            store.pop_call();
        }
        x => { panic!("Unexpected type of 'main': {:?}", x) }
    }
   }
}

fn read_file(filepath: impl AsRef<std::path::Path>) -> String {
    let mut input = String::new();
    let file_path = filepath.as_ref();

    if let Ok(mut file) = File::open(file_path.clone())
    {
        file.read_to_string(&mut input).unwrap();
    } else {
        showln!(red_bold,"error ", gray_dim, "could not open file: ",yellow_bold, file_path.display());
    }
    input
}
