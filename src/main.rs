mod lexer;
mod parser;

use lexer::Token;
use parser::parse;
use std::{fs, path::PathBuf};

struct Source {
    id: SourceId,
    text: String,
    // path: PathBuf,
}
impl Source {
    fn str(&self, token: &Token) -> &str {
        debug_assert_eq!(self.id, token.loc.source_id);
        let loc = &token.loc;
        &self.text[loc.start..loc.stop]
    }
}

#[derive(Default)]
struct SourceManager {
    sources: Vec<Source>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceId(u32);

impl SourceManager {
    fn load<I: Into<PathBuf>>(&mut self, path: I) -> &Source {
        fn inner<'x>(manager: &'x mut SourceManager, path: PathBuf) -> &'x Source {
            let id: u32 = manager.sources.len().try_into().unwrap();
            let id = SourceId(id);

            let mut text = fs::read_to_string(path).unwrap();
            if text.as_bytes().contains(&b'\0') {
                todo!("text can't contain zeros");
            }

            text.push('\0');
            manager.sources.push(Source { id, text });
            manager.sources.last().unwrap()
        }
        inner(self, path.into())
    }
}

fn main() {
    let path = "test.ninja";

    let mut source_manager = SourceManager::default();
    parse(&mut source_manager, path);
}
