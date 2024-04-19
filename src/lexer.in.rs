use crate::SourceId;

#[derive(Debug, Copy, Clone)]
pub struct Location {
    pub start: usize,
    pub stop: usize,
    pub source_id: SourceId,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    Eof,

    Ident,
    Equals,
    Colon,
    Newline,

    Rule,
    Build,
    Default,
    Pool,
    Include,
    Subninja,

    Indent,
    Pipe,
    Pipe2,
    PipeAt,
    Comment,
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

// struct TokenDisplay<'x> {
//     token: &'x Token,
//     text: &'x str,
// }
// impl<'x> std::fmt::Debug for TokenDisplay<'x> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{:?} -> `{}`",
//             self.token.kind,
//             &self.text[self.token.loc.start..self.token.loc.stop]
//         )
//     }
// }

type K = TokenKind;

pub struct Lexer<'x> {
    text: &'x [u8],
    text_str: &'x str,
    offset: usize,
    source_id: SourceId,
}

impl<'x> Lexer<'x> {
    pub fn new(text: &str, source_id: SourceId) -> Lexer {
        Lexer {
            text: text.as_bytes(),
            text_str: text,
            offset: 0,
            source_id,
        }
    }
    pub fn next_impl(&mut self) -> Token {
        let start_offset = self.offset;
        let mut offset = self.offset;
        let token;
        let mut marker = 0;
        
        #[allow(unused_unsafe)]
        /*!re2c
        re2c:define:YYCTYPE = "u8";
        re2c:define:YYPEEK = "self.text[offset]";
        re2c:define:YYSKIP = "offset += 1;";
        re2c:define:YYBACKUP = "marker = offset;";
        re2c:define:YYRESTORE = "offset = marker;";
        re2c:yyfill:enable = 0;

        nul = "\000";
        simple_varname = [a-zA-Z0-9_-]+;
        varname = [a-zA-Z0-9_.-]+;

        [ ]*"#"[^\000\n]*"\n" { token = K::Comment; break; }
        [ ]*"\r\n" { token = K::Newline;  break; }
        [ ]*"\n"   { token = K::Newline;  break; }
        [ ]+       { token = K::Indent;   break; }
        "build"    { token = K::Build;    break; }
        "pool"     { token = K::Pool;     break; }
        "rule"     { token = K::Rule;     break; }
        "default"  { token = K::Default;  break; }
        "="        { token = K::Equals;   break; }
        ":"        { token = K::Colon;    break; }
        "|@"       { token = K::PipeAt;   break; }
        "||"       { token = K::Pipe2;    break; }
        "|"        { token = K::Pipe;     break; }
        "include"  { token = K::Include;  break; }
        "subninja" { token = K::Subninja; break; }
        varname    { token = K::Ident;    break; }
        nul        { token = K::Eof;      break; }
        [^]        { panic!("error");            }
        */

        self.offset = offset;

        let loc = Location {
            start: start_offset,
            stop: self.offset,
            source_id: self.source_id,
        };

        if token != K::Newline && token != K::Eof {
            self.eat_whitespace();
        }

        Token { kind: token, loc }
    }
    pub fn next(&mut self) -> Token {
        loop {
            let next = self.next_impl();
            if next.kind == K::Comment {
                continue;
            }
            return next;
        }
    }
    pub fn maybe_peek(&mut self, kind: K) -> bool {
        let last_offset = self.offset;
        let r = self.next();
        if r.kind == kind {
            return true;
        }
        self.offset = last_offset;
        false
    }
    pub fn peek(&mut self) -> Token {
        let last_offset = self.offset;
        let r = self.next();
        self.offset = last_offset;
        r
    }
    fn eat_whitespace(&mut self) {
        let mut marker = 0;
        let mut offset = self.offset;
        #[allow(unused_unsafe)]
        'lex: loop {
            self.offset = offset;
            /*!re2c
            [ ]+    { continue 'lex; }
            "$\r\n" { continue 'lex; }
            "$\n"   { continue 'lex; }
            nul     { return; }
            [^]     { return; }
            */
        }
    }
    pub fn read_eval_string(&mut self, s: &mut String, path: bool) {
        let mut marker = 0;
        let mut offset = self.offset;
        let mut start;
        #[allow(unused_unsafe)]
        'lex: loop {
            start = offset;
            // https://github.com/ninja-build/ninja/blob/master/src/lexer.in.cc
            /*!re2c
            [^$ :\r\n|\000]+ {
              *s += &self.text_str[start..offset];
              continue 'lex;
            }
            "\r\n" {
              if path {
                offset = start;
              }
              break 'lex;
            }
            [ :|\n] {
              if path {
                offset = start;
                break 'lex;
              } else {
                if self.text[start] == b'\n' {
                    break 'lex;
                }
                *s += &self.text_str[start..start + 1];
                continue 'lex;
              }
            }
            "$$" {
              s.push('$');
              continue 'lex;
            }
            "$ " {
              s.push(' ');
              continue 'lex;
            }
            "$\r\n"[ ]* {
              continue 'lex;
            }
            "$\n"[ ]* {
              continue 'lex;
            }
            "${"varname"}" {
              *s += &self.text_str[start + 2..offset];
              continue 'lex;
            }
            "$"simple_varname {
              *s += &self.text_str[start + 1..offset];
              continue 'lex;
            }
            "$:" {
              s.push(':');
              continue 'lex;
            }
            "$". {
              // last_token_ = start;
              panic!("bad $-escape (literal $ must be written as $$)");
            }
            nul {
              // last_token_ = start;
              panic!("unexpected EOF");
            }
            [^] {
              // last_token_ = start;
              // return Error(DescribeLastError(), err);
              panic!();
            }
            */
        }
		self.offset = offset;
        if path {
            self.eat_whitespace();
        }
    }
    pub fn read_path(&mut self, s: &mut String) {
        self.read_eval_string(s, true)
    }
    pub fn read_var_value(&mut self, s: &mut String) {
        self.read_eval_string(s, false)
    }
    pub fn read_ident(&mut self) -> Location {
        let next = self.next();
        match next.kind {
            K::Ident | K::Pool => next.loc,
            _ => panic!("expected ident"),
        }
    }
}
