use crate::context::*;
use crate::reader::*;
use crate::token::*;
use crate::node::*;
use crate::parser::*;
use crate::utils::*;
use minimo::showln;
use queues::*;

pub fn expression(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: expression: {:?}", reader.sym()));
    disjunction(reader, ctx)
}

fn disjunction(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: disjunction: {:?}", reader.sym()));

    let left = conjunction(reader, ctx);

    if reader.pos() >= reader.len() {
        return left;
    }

    match reader.sym() {
        Some(Token::LogOr(_, _)) => {
            reader.next();
            let right = disjunction(reader, ctx);
            let mut disnode = Node::new(NodeType::LogOr);
            disnode.children.push(left);
            disnode.children.push(right);
            disnode
        }
        _ => left,
    }
}

fn conjunction(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: conjunction: {:?}", reader.sym()));

    let left = equality(reader, ctx);

    if reader.pos() >= reader.len() {
        return left;
    }

    match reader.sym() {
        Some(Token::LogAnd(_, _)) => {
            reader.next();
            let right = conjunction(reader, ctx);
            let mut connode = Node::new(NodeType::LogAnd);
            connode.children.push(left);
            connode.children.push(right);
            connode
        }
        _ => left,
    }
}

fn equality(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: equality: {:?}", reader.sym()));

    let left = comparison(reader, ctx);

    if reader.pos() >= reader.len() {
        return left;
    }

    match reader.sym() {
        Some(Token::Equal(_, _)) => {
            reader.next();
            let right = comparison(reader, ctx);
            let mut eqnode = Node::new(NodeType::Equal);
            eqnode.children.push(left);
            eqnode.children.push(right);
            eqnode
        }
        _ => left,
    }
}

fn comparison(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: comparison: {:?}", reader.sym()));

    let left = bit_or(reader, ctx);

    if reader.pos() >= reader.len() {
        return left;
    }

    match reader.sym() {
        Some(Token::LessThan(_, _)) => {
            reader.next();
            let right = bit_or(reader, ctx);
            let mut connode = Node::new(NodeType::LessThan);
            connode.children.push(left);
            connode.children.push(right);
            connode
        }
        Some(Token::GreaterThan(_, _)) => {
            reader.next();
            let right = bit_or(reader, ctx);
            let mut connode = Node::new(NodeType::GreaterThan);
            connode.children.push(left);
            connode.children.push(right);
            connode
        }
        Some(Token::LessOrEq(_, _)) => {
            reader.next();
            let right = bit_or(reader, ctx);
            let mut connode = Node::new(NodeType::LessOrEq);
            connode.children.push(left);
            connode.children.push(right);
            connode
        }
        Some(Token::GreaterOrEq(_, _)) => {
            reader.next();
            let right = bit_or(reader, ctx);
            let mut connode = Node::new(NodeType::GreaterOrEq);
            connode.children.push(left);
            connode.children.push(right);
            connode
        }
        _ => left,
    }
}

fn bit_or(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: bit_or: {:?}", reader.sym()));

    let left = bit_xor(reader, ctx);

    if reader.pos() >= reader.len() {
        return left;
    }

    match reader.sym() {
        Some(Token::BitOr(_, _)) => {
            let mut node = Node::new(NodeType::BitOr);
            node.children.push(left);
            reader.next();
            let right = bit_or(reader, ctx);
            node.children.push(right);
            node
        }
        _ => left,
    }
}

fn bit_xor(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: bit_xor: {:?}", reader.sym()));

    let left = bit_and(reader, ctx);

    if reader.pos() >= reader.len() {
        return left;
    }

    match reader.sym() {
        Some(Token::BitXor(_, _)) => {
            let mut node = Node::new(NodeType::BitXor);
            node.children.push(left);
            reader.next();
            let right = bit_xor(reader, ctx);
            node.children.push(right);
            node
        }
        _ => left,
    }
}

fn bit_and(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: bit_and: {:?}", reader.sym()));

    let left = sum(reader, ctx);

    if reader.pos() >= reader.len() {
        return left;
    }

    match reader.sym() {
        Some(Token::BitAnd(_, _)) => {
            let mut node = Node::new(NodeType::BitAnd);
            node.children.push(left);
            reader.next();
            let right = bit_and(reader, ctx);
            node.children.push(right);
            node
        }
        _ => left,
    }
}

fn sum(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: sum: {:?}", reader.sym()));
    sum_help(reader, &mut queue![], &mut queue![], ctx)
}

fn sum_help(reader: &mut Reader, righties: &mut Queue<Node>, ops: &mut Queue<Node>, ctx: &Ctx) -> Node {
    let n = product(reader, ctx);
    righties.add(n).ok();

    if reader.pos() >= reader.len() {
        return righties.remove().unwrap();
    }

    match reader.sym() {
        Some(Token::Add(_, _)) => {
            ops.add(Node::new(NodeType::Add)).ok();
            reader.next();
            let deeper = sum_help(reader, righties, ops, ctx);
            let mut node = ops.remove().unwrap();
            node.children.push(deeper);
            node.children.push(righties.remove().unwrap());
            node
        }
        Some(Token::Sub(_, _)) => {
            ops.add(Node::new(NodeType::Sub)).ok();
            reader.next();
            let deeper = sum_help(reader, righties, ops, ctx);
            let mut node = ops.remove().unwrap();
            node.children.push(deeper);
            node.children.push(righties.remove().unwrap());
            node
        }
        _ => righties.remove().unwrap(),
    }
}

fn product(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: product: {:?}", reader.sym()));
    product_help(reader, &mut queue![], &mut queue![], ctx)
}

fn product_help(reader: &mut Reader, righties: &mut Queue<Node>, ops: &mut Queue<Node>, ctx: &Ctx) -> Node {
    let n = access(reader, ctx);
    righties.add(n).ok();

    if reader.pos() >= reader.len() {
        return righties.remove().unwrap();
    }

    match reader.sym() {
        Some(Token::Mul(_, _)) => {
            ops.add(Node::new(NodeType::Mul)).ok();
            reader.next();
            let deeper = product_help(reader, righties, ops, ctx);
            let mut node = ops.remove().unwrap();
            node.children.push(deeper);
            node.children.push(righties.remove().unwrap());
            node
        }
        Some(Token::Div(_, _)) => {
            ops.add(Node::new(NodeType::Div)).ok();
            reader.next();
            let deeper = product_help(reader, righties, ops, ctx);
            let mut node = ops.remove().unwrap();
            node.children.push(deeper);
            node.children.push(righties.remove().unwrap());
            node
        }
        _ => righties.remove().unwrap(),
    }
}

fn access(reader: &mut Reader, ctx: &Ctx) -> Node {
    let n = term(reader, ctx);

    match reader.sym() {
        Some(Token::Access(_, _)) => access_help(reader, n, ctx),
        _ => n,
    }
}

fn access_help(reader: &mut Reader, owner: Node, ctx: &Ctx) -> Node {
    match reader.sym() {
        Some(Token::Access(_, _)) => match reader.next() {
            Some(Token::Name(name, _, _)) => match reader.next() {
                Some(Token::Paren1(_, _)) => {
                    let args_node = arglist(reader, ctx);
                    let mut funcall_node = Node::new(NodeType::MethodCall(name.to_string(), Box::new(owner), ctx.filepath.clone()));
                    funcall_node.children.push(args_node);
                    access_help(reader, funcall_node, ctx)
                }
                Some(Token::Decrement(_, _)) => {
                    let mut decnode = Node::new(NodeType::PostDecrement);
                    let node = Node::new(NodeType::Name(name.clone()));
                    decnode.children.push(node);
                    decnode
                }
                Some(Token::Increment(_, _)) => {
                    let mut incnode = Node::new(NodeType::PostIncrement);
                    let node = Node::new(NodeType::Name(name.clone()));
                    incnode.children.push(node);
                    incnode
                }
                _ => {
                    let mut node = Node::new(NodeType::Name(name.clone()));
                    node.children.push(owner);
                    access_help(reader, node, ctx)
                }
            },
            Some(x) => {
                showln!(red_bold, "error", white_bold, "Expected name after accessor, got: ", yellow_bold,  x);
                owner
            }
            None => {
                showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                owner
            }
        },
        _ => owner,
    }
}

fn term(reader: &mut Reader, ctx: &Ctx) -> Node {
    dprint(format!("Parse: term: {:?}", reader.sym()));

    match reader.sym() {
        Some(Token::Int(val, _, _)) => {
            reader.next();
            Node::new(NodeType::Int(val))
        }
        Some(Token::Double(val, _, _)) => {
            reader.next();
            Node::new(NodeType::Double(val))
        }
        Some(Token::Add(_, _)) => {
            dart_parseerror(
                "'+' is not a prefix operator.",
                ctx,
                reader.tokens(),
                reader.pos()
            );
            Node::new(NodeType::Null)
        }
        Some(Token::Sub(_, _)) => {
            reader.next();
            let mut unary = Node::new(NodeType::Sub);
            let next = term(reader, ctx);
            unary.children.push(next);
            unary
        }
        Some(Token::Not(_, _)) => {
            reader.next();
            let mut notnode = Node::new(NodeType::Not);
            let next = term(reader, ctx);
            notnode.children.push(next);
            notnode
        }
        Some(Token::Str(ref s, interpols, _, _)) => {
            if interpols.is_empty() {
                reader.next();
                Node::new(NodeType::Str(s.clone()))
            } else {
                let mut node = Node::new(NodeType::Str(s.clone()));
                for itp in interpols {
                    let mut r = Reader::new(itp);
                    let itpn = expression(&mut r, ctx);
                    node.children.push(itpn);
                }
                if reader.len() > reader.pos() + 1 {
                    reader.next();
                }
                node
            }
        }
        Some(Token::Bool(v, _, _)) => {
            reader.next();
            Node::new(NodeType::Bool(v))
        }
        Some(Token::Name(ref s, _, _)) => {
            if reader.len() > reader.pos() + 1 {
                reader.next();
                if let Some(Token::Increment(_, _)) = reader.sym() {
                    let mut incnode = Node::new(NodeType::PostIncrement);
                    let node = Node::new(NodeType::Name(s.clone()));
                    incnode.children.push(node);
                    reader.next();
                    return incnode;
                }
                if let Some(Token::Decrement(_, _)) = reader.sym() {
                    let mut decnode = Node::new(NodeType::PostDecrement);
                    let node = Node::new(NodeType::Name(s.clone()));
                    decnode.children.push(node);
                    reader.next();
                    return decnode;
                }
                if let Some(Token::Paren1(_, _)) = reader.sym() {
                    let args_node = arglist(reader, ctx);
                    let mut funcall_node = Node::new(NodeType::FunCall(s.to_string()));
                    funcall_node.nodetype = NodeType::FunCall(s.to_string());
                    funcall_node.children.push(args_node);
                    return funcall_node;
                }
            }
            Node::new(NodeType::Name(s.clone()))
        }
        Some(Token::Increment(_, _)) => {
            match reader.next() {
                Some(Token::Name(name, _, _)) => {
                    reader.next();
                    let mut node = Node::new(NodeType::PreIncrement);
                    node.children.push(Node::new(NodeType::Name(name)));
                    node
                }
                Some(x) => {
                    showln!(red_bold, "error", white_bold, "Invalid operand for increment: ", yellow_bold, format!("{:?}", x));
                    Node::new(NodeType::Null)
                }
                None => {
                    showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                    Node::new(NodeType::Null)
                }
            }
        }
        Some(Token::Decrement(_, _)) => {
            match reader.next() {
                Some(Token::Name(name, _, _)) => {
                    reader.next();
                    let mut node = Node::new(NodeType::PreDecrement);
                    node.children.push(Node::new(NodeType::Name(name)));
                    node
                }
                Some(x) => {
                    showln!(red_bold, "error", white_bold, "Invalid operand for decrement: ", yellow_bold, format!("{:?}", x));
                    Node::new(NodeType::Null)
                }
                None => {
                    showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                    Node::new(NodeType::Null)
                }
            }
        }
        Some(Token::Paren1(_, _)) => {
            reader.next();
            let wnode = expression(reader, ctx);
            if let Err(e) = reader.skip(")", ctx) {
                showln!(red_bold, "error", white_bold, "Error while skipping ')': ", yellow_bold, e);
            }
            wnode
        }
        Some(Token::Brack1(_, _)) => {
            reader.next();
            let mut list_node = Node::new(NodeType::List);
            let mut expect_sep = false;

            match reader.sym() {
                Some(Token::Brack2(_, _)) => {
                    reader.next();
                    list_node
                }
                _ => {
                    while reader.pos() < reader.len() {
                        if expect_sep {
                            match reader.sym() {
                                Some(Token::Comma(_, _)) => {
                                    if !expect_sep {
                                        showln!(red_bold, "error", white_bold, "Expected an identifier, but got ','.");
                                    }
                                    reader.next();
                                    expect_sep = false;
                                    continue;
                                }
                                Some(Token::Brack2(_, _)) => {
                                    reader.next();
                                    break;
                                }
                                Some(x) => {
                                    showln!(red_bold, "error", white_bold, "Unexpected token when parsing list: ", yellow_bold, format!("{:?}", x));
                                    break;
                                }
                                None => {
                                    showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
                                    break;
                                }
                            }
                        }
                        expect_sep = true;
                        let entry = expression(reader, ctx);
                        list_node.children.push(entry);
                    }
                    list_node
                }
            }
        }
        Some(x) => {
            showln!(red_bold, "error", white_bold, "Unexpected token: ", yellow_bold, format!("{:?}", x));
            Node::new(NodeType::Null)
        }
        None => {
            showln!(red_bold, "error", white_bold, "Unexpected end of tokens.");
            Node::new(NodeType::Null)
        }
    }
}
