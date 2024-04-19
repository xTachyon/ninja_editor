mod lexer;
mod parser;

use lexer::{Location, Token};
use parser::parse;
use std::{borrow::Borrow, fs, path::PathBuf};

struct Source {
    id: SourceId,
    text: String,
    // path: PathBuf,
}
impl Source {
    fn str<A: Borrow<Token>>(&self, token: A) -> &str {
        self.str_loc(token.borrow().loc)
    }
    fn str_loc(&self, loc: Location) -> &str {
        debug_assert_eq!(self.id, loc.source_id);
        &self.text[loc.start..loc.stop]
    }
}

#[derive(Default)]
struct SourceManager {
    sources: Vec<&'static Source>, // TODO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceId(u32);

impl SourceManager {
    fn load<I: Into<PathBuf>>(&mut self, path: I) -> &'static Source {
        fn inner<'x>(manager: &'x mut SourceManager, path: PathBuf) -> &'static Source {
            let id: u32 = manager.sources.len().try_into().unwrap();
            let id = SourceId(id);

            let mut text = fs::read_to_string(path).unwrap();
            if text.as_bytes().contains(&b'\0') {
                todo!("text can't contain zeros");
            }

            text.push('\0');
            let source = Box::leak(Box::new(Source { id, text }));
            manager.sources.push(source);
            manager.sources.last().unwrap()
        }
        inner(self, path.into())
    }
}

fn main() {
    let path = "build.ninja";

    let mut source_manager = SourceManager::default();
    parse(&mut source_manager, path);
}
