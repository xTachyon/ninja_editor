use crate::{
    lexer::{Lexer, TokenKind},
    Data, Edge, Rule, Source, SourceManager,
};
use slotmap::{new_key_type, SlotMap};
use std::{collections::HashMap, path::Path};

type K = TokenKind;

struct Parser<'x> {
    lexer: Lexer<'x>,
    source: &'x Source,
}

macro_rules! expect {
    ($obj:expr, $kind:ident) => {{
        let next = $obj.lexer.next();
        if next.kind != K::$kind {
            panic!("Expected {}, got {:?}", stringify!($kind), next.kind);
        }
        next
    }};
}

fn parse_let<'x>(parser: &mut Parser<'x>) -> (String, String) {
    let key_token = parser.lexer.read_ident();
    let key = parser.source.str_loc(key_token).to_string();
    expect!(parser, Equals);
    let mut value = String::new();
    parser.lexer.read_var_value(&mut value);

    (key, value)
}

fn parse_rule<'x>(parser: &mut Parser<'x>, data: &mut Data<'x>) {
    let name_token = expect!(parser, Ident);
    let name = parser.source.str(&name_token);
    expect!(parser, Newline);

    let mut rule = Rule {
        name,
        ..Default::default()
    };
    let mut has_command = false;

    while let K::Indent = parser.lexer.peek().kind {
        parser.lexer.next();

        let (key, value) = parse_let(parser);

        match key.as_str() {
            "command" => {
                rule.command = value;
                has_command = true;
            }
            "depfile" => rule.depfile = Some(value),
            "deps" => rule.deps = Some(value),
            "description" => rule.description = Some(value),
            "restat" => rule.restat = Some(value),
            "generator" => rule.generator = Some(value),
            _ => todo!("unknown key `{key}`"),
        }
    }

    assert!(has_command);

    assert!(!data.rules_by_name.contains_key(name));
    let rule = data.rules.insert(rule);
    data.rules_by_name.insert(name, rule);
}

fn parse_build<'x>(parser: &mut Parser<'x>, data: &mut Data) {
    let mut outs = Vec::new();

    let mut tmp = String::new();
    loop {
        tmp.clear();

        parser.lexer.read_eval_string(&mut tmp, true);
        outs.push(tmp.clone());

        if tmp.is_empty() {
            break;
        }
    }

    if parser.lexer.maybe_peek(K::Pipe) {
        loop {
            tmp.clear();
            parser.lexer.read_eval_string(&mut tmp, true);
            if tmp.is_empty() {
                break;
            }
            // TODO: ignore for now
        }
    }

    expect!(parser, Colon);

    let rule_name = expect!(parser, Ident);
    let rule_name = parser.source.str(rule_name);

    let Some(&rule) = data.rules_by_name.get(rule_name) else {
        panic!("unknown rule `{}`", rule_name);
    };

    loop {
        tmp.clear();

        parser.lexer.read_eval_string(&mut tmp, true);
        if tmp.is_empty() {
            break;
        }
    }

    let mut ins = Vec::new();

    if parser.lexer.maybe_peek(K::Pipe) {
        // Add all implicit deps
        loop {
            tmp.clear();

            parser.lexer.read_eval_string(&mut tmp, true);
            if tmp.is_empty() {
                break;
            }

            ins.push(tmp.clone());
        }
    }

    if parser.lexer.maybe_peek(K::Pipe2) {
        // Add all order-only deps
        loop {
            tmp.clear();

            parser.lexer.read_eval_string(&mut tmp, true);
            if tmp.is_empty() {
                break;
            }

            ins.push(tmp.clone());
        }
    }

    expect!(parser, Newline);

    while parser.lexer.peek().kind == K::Indent {
        parser.lexer.next();

        // args
        let _ = parse_let(parser);
    }

    let edge = Edge { rule, ins, outs };

    data.edges.push(edge);
}

fn parse_var<'x>(parser: &mut Parser<'x>, data: &mut Data) {
    use std::collections::hash_map::Entry;

    let (key, value) = parse_let(parser);
    match data.vars.entry(key) {
        Entry::Occupied(_) => panic!("var already used"),
        Entry::Vacant(x) => x.insert(value),
    };
}

fn parse_default<'x>(parser: &mut Parser<'x>, data: &mut Data) {
    let mut s = String::new();
    parser.lexer.read_path(&mut s);

    match data.default {
        Some(_) => panic!("default edge already defined"),
        None => data.default = Some(s),
    }
}

fn parse_include<'x>(parser: &mut Parser<'x>, data: &mut Data, sm: &mut SourceManager) {
    let mut path = String::new();
    parser.lexer.read_path(&mut path);

    let source = sm.load(path);
    let lexer = Lexer::new(&source.text, source.id);
    let mut parser = Parser { lexer, source };

    parse_item(&mut parser, data, sm);
}

fn parse_item<'x>(parser: &mut Parser<'x>, data: &mut Data<'x>, sm: &mut SourceManager) {
    loop {
        let first = parser.lexer.peek();
        if first.kind != K::Ident {
            parser.lexer.next();
        }
        match first.kind {
            K::Eof => break,
            K::Newline => continue,
            K::Rule => parse_rule(parser, data),
            K::Build => parse_build(parser, data),
            K::Default => parse_default(parser, data),
            K::Ident => parse_var(parser, data),
            K::Include => parse_include(parser, data, sm),
            _ => todo!("{:?}", first.kind),
        };
    }
}

pub fn parse(sm: &mut SourceManager, data: &mut Data, path: &Path) {
    let source = sm.load(path);
    let lexer = Lexer::new(&source.text, source.id);
    let mut parser = Parser { lexer, source };

    parse_item(&mut parser, data, sm);
}
