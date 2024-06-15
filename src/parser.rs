use minimo::*;

use crate::context::*;
use crate::reader::*;
use crate::token::*;
use crate::node::*;
use crate::expression::*;
use crate::utils::*;
use crate::objsys::*;
use crate::object::*;

pub fn parse(reader: &mut Reader,
             globals: &mut Vec<Node>,
             objsys: &mut ObjSys,
             ctx: &Ctx) -> Vec<String> {

    let imports = directives(reader, ctx);

    while reader.more() {
        decl(reader, objsys, globals, ctx);
    }
    if reader.pos() != reader.len() - 1 {
        showln!(red_bold, "error", white_bold, "Unexpected index at end of parse: ", gray_dim, reader.pos(), white_bold, " of ", gray_dim, reader.len());
    }

    imports
}

fn directives(reader: &mut Reader, ctx: &Ctx) -> Vec<String> {
    let mut imports: Vec<String> = Vec::new();

    while reader.more() {
        match reader.sym() {
            Some(Token::Import(_, _)) => {
                reader.next();
                if let Some(Token::Str(s, _, _, _)) = reader.sym() {
                    reader.next();
                    if let Err(e) = reader.skip(";", ctx) {
                        showln!(red_bold, "error", white_bold, "Error while skipping ';': ", yellow_bold, e);
                        break;
                    }
                    imports.push(s.clone());
                } else {
                    showln!(red_bold, "error", white_bold, "Expected string after 'import'.");
                    break;
                }
            }
            _ => break,
        }
    }
    imports
}

pub fn decl(reader: &mut Reader, objsys: &mut ObjSys, globals: &mut Vec<Node>, ctx: &Ctx) {
    dprint(format!("Parse: decl: {:?}", reader.sym()));

    match reader.sym() {
        Some(Token::Name(_, _, _)) => {
            match reader.next() {
                Some(Token::Name(fname, _, _)) => {
                    reader.next();
                    let mut node = Node::new(NodeType::FunDef(fname.to_string(), ctx.filepath.clone()));
                    let params = paramlist(reader, ctx);
                    node.children.push(params);
                    if let Err(e) = reader.skip("{", ctx) {
                        showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                        return;
                    }
                    let body = block(reader, ctx);
                    node.children.push(body);
                    globals.push(node);
                }
                _ => {
                    showln!(red_bold, "error", white_bold, "Expected function name.");
                }
            }
        }
        Some(Token::Class(_, _)) => {
            class(reader, objsys, globals, ctx);
        }
        Some(Token::Import(_, _)) => {
            dart_parseerror(
                "Directives must appear before any declarations.",
                ctx,
                &reader.tokens(),
                reader.pos()
            );
        }
        Some(x) => {
            showln!(red_bold, "error", white_bold, "Expected top level declaration. Got: ", yellow_bold, format!("{:?}", x));
            panic!();
        }
        None => {
            showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
        }
    }
}

fn paramlist(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: paramlist: {:?}", reader.sym()));

    if let Some(Token::Paren1(_, _)) = reader.sym() {
        let mut node = Node::new(NodeType::ParamList);
        let mut expect_comma = false;
        reader.next();
        while reader.more() {
            match reader.sym() {
                Some(Token::Paren2(_, _)) => {
                    reader.next();
                    return node;
                }
                Some(Token::Comma(_, _)) => {
                    if !expect_comma {
                        dart_parseerror(
                            "Unexpected separator in parameter list: ','.",
                            ctx,
                            reader.tokens(),
                            reader.pos()
                        );
                        break;
                    }
                    reader.next();
                    expect_comma = false;
                }
                Some(Token::Name(s, _, _)) => {
                    let paramnode = Node::new(NodeType::Name(s.to_string()));
                    node.children.push(paramnode);
                    expect_comma = true;
                    reader.next();
                }
                _ => {
                    dart_parseerror(
                        "Unexpected token when reading parameters.",
                        ctx,
                        reader.tokens(),
                        reader.pos()
                    );
                    break;
                }
            }
        }
    } else {
        dart_parseerror(
            "A function declaration needs an explicit list of parameters.",
            ctx,
            reader.tokens(),
            reader.pos() - 1
        )
    }
    Node::new(NodeType::ParamList)
}

fn class(reader: &mut Reader, objsys: &mut ObjSys, globals: &mut Vec<Node>, ctx: &Ctx) {
    match reader.next() {
        Some(Token::Name(classname, _, _)) => {
            let mut class = objsys.new_class(classname.clone());
            reader.next();
            if let Err(e) = reader.skip("{", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                return;
            }
            readmembers(&mut class, reader, globals, ctx);
            if let Err(e) = reader.skip("}", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping '}': ", yellow_bold, e);
                return;
            }
            objsys.register_class(class);
        }
        Some(x) => {
            showln!(red_bold, "error", white_bold, "Expected class name. Got {:?}", x);
        }
        None => {
            showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
        }
    }
}

fn readmembers(class: &mut Class, reader: &mut Reader, globals: &mut Vec<Node>, ctx: &Ctx) {
    let mut got_constructor = false;

    while reader.more() {
        match reader.sym() {
            Some(Token::Name(mtype, _, _)) => {
                if *mtype == class.name {
                    reader.next();
                    let mut constructor_node = Node::new(NodeType::Constructor(class.name.clone(), ctx.filepath.clone()));
                    let params = constructor_paramlist(reader, ctx);
                    constructor_node.children.push(params);
                    match reader.sym() {
                        Some(Token::Block1(_, _)) => {
                            reader.next();
                            let body = block(reader, ctx);
                            constructor_node.children.push(body);
                        }
                        Some(Token::EndSt(_, _)) => {
                            reader.next();
                            constructor_node.children.push(Node::new(NodeType::Null));
                        }
                        Some(x) => {
                            dart_parseerror(
                                format!("Expected constructor body, got: {:?}", x),
                                ctx,
                                reader.tokens(),
                                reader.pos()
                            );
                        }
                        None => {
                            showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                        }
                    }
                    got_constructor = true;
                    globals.push(constructor_node);
                    continue;
                }
                match reader.next() {
                    Some(Token::Name(fieldname, _, _)) => {
                        match reader.next() {
                            Some(Token::Paren1(_, _)) => {
                                let param_node = paramlist(reader, ctx);
                                if let Err(e) = reader.skip("{", ctx) {
                                    showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                                    return;
                                }
                                let body = block(reader, ctx);
                                let mut args: Vec<ParamObj> = Vec::new();
                                for i in 0..param_node.children.len() {
                                    let p = &param_node.children[i];
                                    match &p.nodetype {
                                        NodeType::Name(s) => {
                                            args.push(ParamObj { typ: String::from("var"), name: s.clone(), fieldinit: false });
                                        }
                                        x => {
                                            showln!(red_bold, "error", white_bold, "Invalid parameter: {:?}", x);
                                            return;
                                        }
                                    }
                                }
                                let methodobj = Object::Function(fieldname.to_string(), ctx.filepath.clone(), body, args);
                                class.add_method(fieldname.clone(), methodobj);
                            }
                            Some(Token::EndSt(_, _)) => {
                                reader.next();
                                class.add_field(mtype, fieldname, Node::new(NodeType::Null));
                            }
                            Some(Token::Assign(_, _)) => {
                                reader.next();
                                let val = expression(reader, ctx);
                                if let Err(e) = reader.skip(";", ctx) {
                                    showln!(red_bold, "error", white_bold, "Error while skipping ';': ", yellow_bold, e);
                                    return;
                                }
                                class.add_field(mtype, fieldname, val);
                            }
                            Some(Token::Block2(_, _)) => {
                                break;
                            }
                            Some(x) => {
                                showln!(red_bold, "error", white_bold, "Unexpected token when parsing class member: {:?}", x);
                            }
                            None => {
                                showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                            }
                        }
                    }
                    Some(Token::Block2(_, _)) => {
                        break;
                    }
                    Some(x) => {
                        showln!(red_bold, "error", white_bold, "expected class member name, got ", yellow_bold, x);
                    }
                    None => {
                        showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                    }
                }
            }
            Some(Token::Block2(_, _)) => {
                break;
            }
            Some(x) => {
                showln!(red_bold, "error", white_bold, "Unexpected first token when parsing class member ", yellow_bold, x);
                panic!();
            }
            None => {
                showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
            }
        }
    }

    if !got_constructor {
        let mut constructor_node = Node::new(NodeType::Constructor(class.name.clone(), ctx.filepath.clone()));
        constructor_node.children.push(Node::new(NodeType::ParamList));
        constructor_node.children.push(Node::new(NodeType::Null));
        globals.push(constructor_node);
    }
}

fn constructor_paramlist(reader: &mut Reader, ctx: &Ctx) -> Node {
    if let Some(Token::Paren1(_, _)) = reader.sym() {
        let mut node = Node::new(NodeType::ParamList);
        let mut expect_comma = false;
        reader.next();
        while reader.more() {
            match reader.sym() {
                Some(Token::Paren2(_, _)) => {
                    reader.next();
                    return node;
                }
                Some(Token::This(_, _)) => {
                    reader.next();
                    if let Err(e) = reader.skip(".", ctx) {
                        showln!(red_bold, "error", white_bold, "Error while skipping '.': ", yellow_bold, e);
                        break;
                    }
                    match reader.sym() {
                        Some(Token::Name(s, _, _)) => {
                            let this_fieldinit = Node::new(NodeType::ThisFieldInit(s));
                            node.children.push(this_fieldinit);
                            expect_comma = true;
                            reader.next();
                        }
                        Some(x) => {
                            dart_parseerror(
                                format!("Expected identifier. Got {:?}", x),
                                ctx,
                                reader.tokens(),
                                reader.pos()
                            );
                            break;
                        }
                        None => {
                            showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                            break;
                        }
                    }
                }
                Some(Token::Comma(_, _)) => {
                    if !expect_comma {
                        dart_parseerror(
                            "Expected an identifier, but got ','.",
                            ctx,
                            reader.tokens(),
                            reader.pos()
                        );
                        break;
                    }
                    reader.next();
                    expect_comma = false;
                }
                Some(Token::Name(s, _, _)) => {
                    let paramnode = Node::new(NodeType::Name(s.to_string()));
                    node.children.push(paramnode);
                    expect_comma = true;
                    reader.next();
                }
                _ => {
                    dart_parseerror(
                        "Unexpected token when reading parameters.",
                        ctx,
                        reader.tokens(),
                        reader.pos()
                    );
                    break;
                }
            }
        }
    } else {
        dart_parseerror(
            "A function declaration needs an explicit list of parameters.",
            ctx,
            reader.tokens(),
            reader.pos() - 1
        )
    }
    Node::new(NodeType::ParamList)
}

pub fn arglist(reader: &mut Reader, ctx: &Ctx) -> Node {
    if let Some(Token::Paren1(_, _)) = reader.sym() {
        let mut node = Node::new(NodeType::ArgList);
        let mut expect_comma = false;
        reader.next();
        while reader.more() {
            match reader.sym() {
                Some(Token::Paren2(_, _)) => {
                    reader.next();
                    return node;
                }
                Some(Token::Comma(_, _)) => {
                    if !expect_comma {
                        dart_parseerror(
                            "Unexpected separator in arg list: ','.",
                            ctx,
                            reader.tokens(),
                            reader.pos()
                        );
                        break;
                    }
                    reader.next();
                    expect_comma = false;
                }
                Some(_) => {
                    if expect_comma {
                        dart_parseerror(
                            "Expected separator in arg list.",
                            ctx,
                            reader.tokens(),
                            reader.pos()
                        );
                        break;
                    }
                    let arg = expression(reader, ctx);
                    node.children.push(arg);
                    expect_comma = true;
                }
                None => {
                    dart_parseerror(
                        "Unexpected end of tokens in arg list.",
                        ctx,
                        reader.tokens(),
                        reader.pos()
                    );
                    break;
                }
            }
        }
    } else {
        dart_parseerror(
            "Expected start of arglist: '('.",
            ctx,
            reader.tokens(),
            reader.pos()
        )
    }
    Node::new(NodeType::ArgList)
}

fn block(reader: &mut Reader, ctx: &Ctx) -> Node {
    let mut node = Node::new(NodeType::Block);

    while reader.more() {
        match reader.sym() {
            Some(Token::Block2(_, _)) => {
                reader.next();
                break;
            }
            Some(Token::End) => {
                break;
            }
            Some(Token::EndSt(_, _)) => {
                reader.next();
                continue;
            }
            Some(_) => {
                let snode = statement(reader, ctx);
                node.children.push(snode);

                match reader.sym() {
                    Some(Token::Block2(_, _)) => {
                        reader.next();
                        continue;
                    }
                    Some(Token::EndSt(_, _)) => {
                        reader.next();
                        continue;
                    }
                    _ => continue,
                }
            }
            None => {
                showln!(red_bold, "error", white_bold, "Unexpected end of tokens in block.");
                break;
            }
        }
    }
    node
}

fn statement(reader: &mut Reader, ctx: &Ctx) -> Node {
    match reader.sym() {
        Some(Token::Name(s, _, _)) => {
            let t2 = reader.peek();
            match t2 {
                Some(Token::Name(name, _, _)) => {
                    let typed_var = Node::new(NodeType::TypedVar(s.to_string(), name.to_string()));
                    reader.next();
                    reader.next();
                    match reader.sym() {
                        Some(Token::Assign(_, _)) => {
                            reader.next();
                            let right_node = expression(reader, ctx);
                            let mut ass_node = Node::new(NodeType::Assign);
                            ass_node.children.push(typed_var);
                            ass_node.children.push(right_node);
                            ass_node
                        }
                        Some(Token::Paren1(_, _)) => {
                            let params = paramlist(reader, ctx);
                            if let Err(e) = reader.skip("{", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            let body = block(reader, ctx);
                            let mut funcnode = Node::new(NodeType::FunDef(name.clone(), ctx.filepath.clone()));
                            funcnode.children.push(params);
                            funcnode.children.push(body);
                            funcnode
                        }
                        Some(x) => {
                            showln!(red_bold, "error", white_bold, "Unexpected token: {:?}", x);
                            Node::new(NodeType::Null)
                        }
                        None => {
                            showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                            Node::new(NodeType::Null)
                        }
                    }
                }
                Some(Token::Assign(_, _)) => {
                    reader.next();
                    reader.next();
                    let right_node = expression(reader, ctx);
                    let var = Node::new(NodeType::Name(s.to_string()));
                    let mut ass_node = Node::new(NodeType::Assign);
                    ass_node.children.push(var);
                    ass_node.children.push(right_node);
                    ass_node
                }
                _ => expression(reader, ctx),
            }
        }
        Some(Token::If(_, _)) => {
            let mut condnode = Node::new(NodeType::Conditional);
            let condpart = conditional(reader, ctx);
            condnode.children.push(condpart);
            loop {
                match reader.sym() {
                    Some(Token::Else(_, _)) => {
                        let lastcond = conditional(reader, ctx);
                        condnode.children.push(lastcond);
                    }
                    _ => break,
                }
            }
            condnode
        }
        Some(Token::While(_, _)) => {
            reader.next();
            if let Err(e) = reader.skip("(", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping '(': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            let boolexpr = expression(reader, ctx);
            if let Err(e) = reader.skip(")", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping ')': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            if let Err(e) = reader.skip("{", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            let blocknode = block(reader, ctx);
            let mut node = Node::new(NodeType::While);
            node.children.push(boolexpr);
            node.children.push(blocknode);
            node
        }
        Some(Token::Do(_, _)) => {
            reader.next();
            if let Err(e) = reader.skip("{", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            let blocknode = block(reader, ctx);
            if let Err(e) = reader.skip("while", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping 'while': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            if let Err(e) = reader.skip("(", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping '(': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            let boolexpr = expression(reader, ctx);
            if let Err(e) = reader.skip(")", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping ')': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            let mut node = Node::new(NodeType::DoWhile);
            node.children.push(blocknode);
            node.children.push(boolexpr);
            node
        }
        Some(Token::For(_, _)) => {
            reader.next();
            if let Err(e) = reader.skip("(", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping '(': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            match reader.sym() {
                Some(Token::Name(n1, _, _)) => {
                    reader.next();
                    match reader.sym() {
                        Some(Token::Name(n2, _, _)) => {
                            reader.next();
                            let typvar = Node::new(NodeType::TypedVar(n1.clone(), n2.clone()));
                            if let Err(e) = reader.skip("=", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping '=': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            let initexpr = expression(reader, ctx);
                            let mut assign = Node::new(NodeType::Assign);
                            assign.children.push(typvar);
                            assign.children.push(initexpr);
                            if let Err(e) = reader.skip(";", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping ';': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            let condexpr = expression(reader, ctx);
                            if let Err(e) = reader.skip(";", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping ';': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            let mutexpr = expression(reader, ctx);
                            if let Err(e) = reader.skip(")", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping ')': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            if let Err(e) = reader.skip("{", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            let body = block(reader, ctx);
                            let mut forloop = Node::new(NodeType::For);
                            forloop.children.extend([assign, condexpr, mutexpr, body]);
                            forloop
                        }
                        Some(Token::Assign(_, _)) => {
                            reader.next();
                            let initexpr = expression(reader, ctx);
                            let mut assign = Node::new(NodeType::Assign);
                            let namenode = Node::new(NodeType::Name(n1.clone()));
                            assign.children.push(namenode);
                            assign.children.push(initexpr);
                            if let Err(e) = reader.skip(";", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping ';': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            let condexpr = expression(reader, ctx);
                            if let Err(e) = reader.skip(";", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping ';': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            let mutexpr = expression(reader, ctx);
                            if let Err(e) = reader.skip(")", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping ')': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            if let Err(e) = reader.skip("{", ctx) {
                                showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                                return Node::new(NodeType::Null);
                            }
                            let body = block(reader, ctx);
                            let mut forloop = Node::new(NodeType::For);
                            forloop.children.extend([assign, condexpr, mutexpr, body]);
                            forloop
                        }
                        Some(x) => {
                            dart_parseerror(
                                format!("Expected identifier or assignment. Got: {:?}", x),
                                ctx,
                                &reader.tokens(),
                                reader.pos()
                            );
                            Node::new(NodeType::Null)
                        }
                        None => {
                            showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                            Node::new(NodeType::Null)
                        }
                    }
                }
                _ => {
                    dart_parseerror(
                        "Expected identifier.",
                        ctx,
                        &reader.tokens(),
                        reader.pos()
                    );
                    Node::new(NodeType::Null)
                }
            }
        }
        Some(Token::Return(_, _)) => {
            reader.next();
            let val = expression(reader, ctx);
            let mut ret = Node::new(NodeType::Return);
            ret.children.push(val);
            ret
        }
        _ => expression(reader, ctx),
    }
}

fn conditional(reader: &mut Reader, ctx: &Ctx) -> Node {
    match reader.sym() {
        Some(Token::If(_, _)) => {
            reader.next();
            if let Err(e) = reader.skip("(", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping '(': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            let boolnode = expression(reader, ctx);
            if let Err(e) = reader.skip(")", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping ')': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            if let Err(e) = reader.skip("{", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                return Node::new(NodeType::Null);
            }
            let bodynode = block(reader, ctx);
            let mut ifnode = Node::new(NodeType::If);
            ifnode.children.push(boolnode);
            ifnode.children.push(bodynode);
            ifnode
        }
        Some(Token::Else(_, _)) => {
            reader.next();
            match reader.sym() {
                Some(Token::If(_, _)) => {
                    reader.next();
                    if let Err(e) = reader.skip("(", ctx) {
                        showln!(red_bold, "error", white_bold, "Error while skipping '(': ", yellow_bold, e);
                        return Node::new(NodeType::Null);
                    }
                    let boolnode = expression(reader, ctx);
                    if let Err(e) = reader.skip(")", ctx) {
                        showln!(red_bold, "error", white_bold, "Error while skipping ')': ", yellow_bold, e);
                        return Node::new(NodeType::Null);
                    }
                    if let Err(e) = reader.skip("{", ctx) {
                        showln!(red_bold, "error", white_bold, "Error while skipping '{': ", yellow_bold, e);
                        return Node::new(NodeType::Null);
                    }
                    let bodynode = block(reader, ctx);
                    let mut elseifnode = Node::new(NodeType::ElseIf);
                    elseifnode.children.push(boolnode);
                    elseifnode.children.push(bodynode);
                    elseifnode
                }
                Some(Token::Block1(_, _)) => {
                    reader.next();
                    let bodynode = block(reader, ctx);
                    let mut elsenode = Node::new(NodeType::Else);
                    elsenode.children.push(bodynode);
                    elsenode
                }
                Some(x) => {
                    showln!(red_bold, "error", white_bold, "Unexpected token after 'else': {:?}", x);
                    Node::new(NodeType::Null)
                }
                None => {
                    showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                    Node::new(NodeType::Null)
                }
            }
        }
        _ => {
            showln!(red_bold, "error", white_bold, "Expected conditional.");
            Node::new(NodeType::Null)
        }
    }
}
