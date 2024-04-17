use crate::SourceId;

#[derive(Debug)]
pub struct Location {
    pub start: usize,
    pub stop: usize,
    pub source_id: SourceId,
}
// impl Location {
//     pub fn merge(&self, last: Location) -> Location  {
//         debug_assert_eq!(self.source_id, last.source_id);
//         debug_assert!(self.start < last.start);

//         Location {
//             start: self.start,
//             stop: last.stop,
//             source_id: self.source_id,
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Eof,

    Ident,
    Eq,
    Colon,
    Newline,

    Rule,
    Build,
    Default,
    Pool,
    Include,
    Subninja,

    Comment,
    Indent,
    Dollar,
    DollarDollar,
    LBrace,
    RBrace,
}
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub loc: Location,
}
impl Token {
    // fn debug<'x>(&'x self, text: &'x str) -> TokenDisplay<'x> {
    //     TokenDisplay { token: self, text }
    // }
}

struct TokenDisplay<'x> {
    token: &'x Token,
    text: &'x str,
}
impl<'x> std::fmt::Debug for TokenDisplay<'x> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} -> `{}`",
            self.token.kind,
            &self.text[self.token.loc.start..self.token.loc.stop]
        )
    }
}

type K = TokenKind;

pub struct Lexer<'x> {
    text: &'x [u8],
    text_str: &'x str,
    offset: usize,
    source_id: SourceId,
}

const ZERO: u8 = b'\0';

impl<'x> Lexer<'x> {
    pub fn new(text: &str, source_id: SourceId) -> Lexer {
        Lexer {
            text: text.as_bytes(),
            text_str: text,
            offset: 0,
            source_id,
        }
    }
    fn next_impl(&mut self) -> Token {
        let Some(&first) = self.text.get(self.offset) else {
            panic!("already done");
        };
        let start_offset = self.offset;
        self.offset += 1;

        let mut eat_whitespace = false;
        let kind = match first {
            0 => K::Eof,
            b'=' => K::Eq,
            b':' => K::Colon,
            b'{' => K::LBrace,
            b'}' => K::RBrace,
            b'\n' => K::Newline,
            b'\r' => {
                if self.text[self.offset] == b'\n' {
                    self.offset += 1;
                }
                K::Newline
            }
            b'$' => {
                if self.text[self.offset] == b'$' {
                    self.offset += 1;
                    K::DollarDollar
                } else {
                    K::Dollar
                }
            }
            b'#' => {
                loop {
                    let current = self.text[self.offset];
                    if current == b'\n' || current == ZERO {
                        break;
                    }
                    self.offset += 1;
                }
                K::Comment
            }
            b' ' => {
                loop {
                    let current = self.text[self.offset];
                    if current != b' ' || current == ZERO {
                        break;
                    }
                    self.offset += 1;
                }
                K::Indent
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'/' | b'.' | b'_' | b'&' | b'-' | b'\\' => {
                loop {
                    let current = self.text[self.offset];
                    if !matches!(current, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'/' | b'.' | b'_' | b'&' | b'-' | b'\\')
                    {
                        break;
                    }
                    self.offset += 1;
                }
                eat_whitespace = true;
                if self.text[start_offset - 1] == b'\n' {
                    let s = &self.text[start_offset..self.offset];
                    match s {
                        b"rule" => K::Rule,
                        b"build" => K::Build,
                        b"default" => K::Default,
                        b"ident" => K::Ident,
                        b"pool" => K::Pool,
                        b"include" => K::Include,
                        b"subninja" => K::Subninja,
                        _ => todo!(
                            "unknown keyword `{}`",
                            &self.text_str[start_offset..self.offset]
                        ),
                    }
                } else {
                    K::Ident
                }
            }
            _ => todo!("unknown char `{}`({})", first as char, first),
        };
        let loc = Location {
            start: start_offset,
            stop: self.offset,
            source_id: self.source_id,
        };

        if eat_whitespace {
            while let K::Indent = self.peek().kind {
                self.next();
            }
        }

        Token { kind, loc }
    }
    pub fn next(&mut self) -> Token {
        loop {
            let next = self.next_impl();
            // if let K::Whitespace = next.kind {
            //     continue;
            // }
            break next;
        }
    }
    pub fn peek(&mut self) -> Token {
        let last_offset = self.offset;
        let r = loop {
            let next = self.next_impl();
            // if next.kind == K::Whitespace {
            //     continue;
            // }
            break next;
        };
        self.offset = last_offset;
        r
    }
    // pub fn read_eval_string(&mut self, _path: bool) {
    //     let start_offset = self.offset;
    //     loop {
    //         let token = self.peek();
    //         match token.kind {
    //             K::Whitespace | K::Dollar | K::DollarDollar  => {}
    //             _ => todo!(),
    //         }
    //     }
    //     // let start_offset = self.offset;
    //     // let first = self.text[self.offset];
    //     // self.offset += 1;

    //     // match first {
    //     //     0 => todo!(),

    //     //     _ => todo!(),
    //     // }
    // }
    pub fn until_eol(&mut self) -> Token {
        while self.text[self.offset] == b' ' {
            self.offset += 1;
        }
        let start_offset = self.offset;
        loop {
            let current = self.text[self.offset];
            if current == b'\n' || current == ZERO {
                break;
            }
            self.offset += 1;
        }
        let loc = Location {
            start: start_offset,
            stop: self.offset,
            source_id: self.source_id,
        };
        Token {
            loc,
            kind: K::Ident,
        }
    }
    pub fn loc_extend_to_last(&self, first: Location) -> Location {
        debug_assert_eq!(self.source_id, first.source_id);

        Location {
            start: first.start,
            stop: self.offset,
            source_id: self.source_id,
        }
    }
}

// pub fn lex(text: &mut String) -> Vec<Token> {
//     if text.as_bytes().contains(&b'\0') {
//         todo!("text can't contain zeros");
//     }
//     text.push('\0');

//     let mut tokens = Vec::new();
//     let mut lexer = Lexer::new(text);
//     // while let Some(tok) = lexer.next() {
//     //     if matches!(tok.kind, K::Comment | K::Whitespace) {
//     //         continue;
//     //     }
//     //     dbg!(tok.debug(text));
//     //     tokens.push(tok);
//     // }

//     tokens
// }
