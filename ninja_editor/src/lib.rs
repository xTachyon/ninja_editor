mod changelist;
mod lexer;
mod parser;

use std::collections::HashMap;
use std::path::Path;

use changelist::ChangeList;
use slotmap::{new_key_type, SlotMap};

use crate::lexer::{Location, Token};
use crate::parser::parse;
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
    fn text_parser(&self) -> &str {
        &self.text
    }
    fn text(&self) -> &str {
        &self.text[0..self.text.len() - 1]
    }
}

#[derive(Default)]
struct SourceManager {
    sources: Vec<&'static Source>, // TODO
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct SourceId(u32);

impl SourceManager {
    fn load<I: Into<PathBuf>>(&mut self, path: I) -> &'static Source {
        fn inner(manager: &mut SourceManager, path: PathBuf) -> &'static Source {
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
    fn get(&self, id: SourceId) -> &'static Source {
        self.sources[id.0 as usize]
    }
}

#[derive(Default, Debug)]
pub struct Rule<'x> {
    pub name: &'x str,
    name_loc: Location,
    command: String,
    depfile: Option<String>,
    deps: Option<String>,
    description: Option<String>,
    restat: Option<String>,
    generator: Option<String>,
}

new_key_type! {
    pub struct RuleKey;
}

struct Edge {
    rule: RuleKey,
    rule_loc: Location,
    // ins: Vec<String>,
    // outs: Vec<String>,
}

#[derive(Default)]
pub struct Data<'x> {
    pub rules: SlotMap<RuleKey, Rule<'x>>,
    rules_by_name: HashMap<&'x str, RuleKey>,
    edges: Vec<Edge>,
    vars: HashMap<String, String>,
    default: Option<String>,
}
impl<'x> Data<'x> {
    fn new() -> Data<'x> {
        let mut rules = SlotMap::with_key();
        let phony = rules.insert(Rule {
            name: "phony",
            ..Rule::default()
        });

        let rules_by_name = HashMap::from([("phony", phony)]);

        Data {
            rules,
            rules_by_name,
            edges: Vec::new(),
            vars: HashMap::new(),
            default: None,
        }
    }
}

pub struct Ninja {
    sm: SourceManager,
    data: Data<'static>,
}

impl Ninja {
    fn load_impl(path: &Path) -> Ninja {
        let mut sm = SourceManager::default();
        let mut data = Data::new();

        parse(&mut sm, &mut data, path);

        Ninja { sm, data }
    }
    //  fn load<P: AsRef<Path>>(path: P) {
    //     Self::load_impl(path.as_ref())
    // }
    pub fn load_folder<P: AsRef<Path>>(path: P) -> Ninja {
        Self::load_impl(path.as_ref().join("build.ninja").as_path())
    }
    pub fn data(&self) -> &Data {
        &self.data
    }
    pub fn change(&self) -> ChangeList {
        ChangeList::new(self)
    }
}
