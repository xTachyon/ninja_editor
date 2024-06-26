use crate::{
    lexer::{Lexer, TokenKind},
    Data, Edge, Rule, Source, SourceManager, L,
};
use std::path::Path;

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

fn parse_let(parser: &mut Parser) -> (String, L<String>) {
    let key_token = parser.lexer.read_ident();
    let key = parser.source.str_loc(key_token).to_string();
    expect!(parser, Equals);
    let value = parser.lexer.read_var_value();

    (key, value)
}

fn parse_rule<'x>(parser: &mut Parser<'x>, data: &mut Data<'x>) {
    let name_token = expect!(parser, Ident);
    let name = parser.source.str(&name_token);
    let name = L {
        loc: name_token.loc,
        elem: name,
    };
    expect!(parser, Newline);

    let rule = Rule {
        name,
        ..Default::default()
    };
    let mut has_command = false;

    while let K::Indent = parser.lexer.peek().kind {
        parser.lexer.next();

        let (key, _value) = parse_let(parser);

        match key.as_str() {
            "command" => {
                // rule.command = value;
                has_command = true;
            }
            "depfile" => {
                // rule.depfile = Some(value)
            }
            "deps" => {
                // rule.deps = Some(value),
            }
            "description" => {
                // rule.description = Some(value),
            }
            "restat" => {
                // rule.restat = Some(value),
            }
            "generator" => {
                // rule.generator = Some(value),
            }
            _ => todo!("unknown key `{key}`"),
        }
    }

    assert!(has_command);

    assert!(!data.rules_by_name.contains_key(name.elem));
    let rule = data.rules.insert(rule);
    data.rules_by_name.insert(name.elem, rule);
}

fn parse_build(parser: &mut Parser<'_>, data: &mut Data) {
    let mut ins = Vec::new();
    let mut outs = Vec::new();

    loop {
        let tmp = parser.lexer.read_path();
        if tmp.elem.is_empty() {
            break;
        }

        outs.push(tmp);
    }

    if parser.lexer.maybe_peek(K::Pipe) {
        loop {
            let tmp = parser.lexer.read_path();
            if tmp.elem.is_empty() {
                break;
            }
            // TODO: ignore for now
        }
    }

    expect!(parser, Colon);

    let rule_name_token = expect!(parser, Ident);
    let rule_name = parser.source.str(&rule_name_token);

    let Some(&rule) = data.rules_by_name.get(rule_name) else {
        panic!("unknown rule `{}`", rule_name);
    };

    loop {
        let tmp = parser.lexer.read_path();
        if tmp.elem.is_empty() {
            break;
        }
    }

    if parser.lexer.maybe_peek(K::Pipe) {
        // Add all implicit deps
        loop {
            let tmp = parser.lexer.read_path();
            if tmp.elem.is_empty() {
                break;
            }

            ins.push(tmp);
        }
    }

    if parser.lexer.maybe_peek(K::Pipe2) {
        // Add all order-only deps
        loop {
            let tmp = parser.lexer.read_path();
            if tmp.elem.is_empty() {
                break;
            }

            ins.push(tmp);
        }
    }

    expect!(parser, Newline);

    while parser.lexer.peek().kind == K::Indent {
        parser.lexer.next();

        // args
        let _ = parse_let(parser);
    }

    for i in outs.iter().chain(ins.iter()) {
        data.nodes.entry(i.elem.clone()).or_default().push(i.loc);
    }
    let edge = Edge {
        rule,
        rule_loc: rule_name_token.loc,
    };

    data.edges.insert(edge);
}

fn parse_var(parser: &mut Parser<'_>, data: &mut Data) {
    use std::collections::hash_map::Entry;

    let (key, value) = parse_let(parser);
    match data.vars.entry(key) {
        Entry::Occupied(_) => panic!("var already used"),
        Entry::Vacant(x) => x.insert(value),
    };
}

fn parse_default(parser: &mut Parser<'_>, data: &mut Data) {
    let s = parser.lexer.read_path();

    match data.default {
        Some(_) => panic!("default edge already defined"),
        None => data.default = Some(s),
    }
}

fn parse_include(parser: &mut Parser<'_>, data: &mut Data, sm: &mut SourceManager) {
    let path = parser.lexer.read_path();

    let source = sm.load(path.elem);
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
            K::Include | K::Subninja => parse_include(parser, data, sm),
            _ => todo!("{:?}", first.kind),
        };
    }
}

pub fn parse(sm: &mut SourceManager, data: &mut Data, path: &Path) {
    let source = sm.load(path);
    let lexer = Lexer::new(source.text_parser(), source.id);
    let mut parser = Parser { lexer, source };

    parse_item(&mut parser, data, sm);
}
