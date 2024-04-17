use crate::{
    lexer::{Lexer, Location, TokenKind},
    Source, SourceManager,
};

type K = TokenKind;

#[derive(Debug)]
struct Rule<'x> {
    name: &'x str,
    command: &'x str,
}

#[derive(Debug)]
enum ItemKind<'x> {
    Rule(Rule<'x>),
}
#[derive(Debug)]
struct Item<'x> {
    kind: ItemKind<'x>,
    loc: Location,
}

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

fn parse_let<'x>(parser: &mut Parser<'x>, rule: &mut Rule<'x>, has_command: &mut bool) {
    let key = expect!(parser, Ident);
    let key = parser.source.str(&key);
    expect!(parser, Eq);
    let line = parser.lexer.until_eol();
    let line = parser.source.str(&line);

    match key {
        "command" => {
            rule.command = line;
            *has_command = true;
        }
        _ => todo!("unknown key `{key}`"),
    }
}

fn parse_rule<'x>(parser: &mut Parser<'x>) -> Item<'x> {
    let name_token = expect!(parser, Ident);
    let name = parser.source.str(&name_token);
    expect!(parser, Newline);

    let mut rule = Rule { name, command: "" };
    let mut has_command = false;

    while let K::Indent = parser.lexer.peek().kind {
        parser.lexer.next();
        parse_let(parser, &mut rule, &mut has_command);
    }

    assert!(has_command);

    Item {
        kind: ItemKind::Rule(rule),
        loc: parser.lexer.loc_extend_to_last(name_token.loc),
    }
}

fn parse_build<'x>(parser: &mut Parser<'x>) -> Item<'x> {
    todo!()
}

fn parse_item<'x>(parser: &mut Parser<'x>, items: &mut Vec<Item<'x>>) {
    loop {
        let first = parser.lexer.next();
        let item = match first.kind {
            K::Eof => break,
            K::Newline => continue,
            K::Rule => parse_rule(parser),
            K::Build => parse_build(parser),
            _ => todo!("{:?}", first.kind),
        };
        items.push(item);
    }
}

pub fn parse(source_manager: &mut SourceManager, path: &str) {
    let source = source_manager.load(path);
    let mut items = Vec::with_capacity(16);
    let lexer = Lexer::new(&source.text, source.id);
    let mut parser = Parser { lexer, source };
    parse_item(&mut parser, &mut items);

    for i in items {
        println!("{:?}", i);
    }
}
