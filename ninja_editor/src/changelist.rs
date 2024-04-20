use filetime::{set_file_mtime, FileTime};

use crate::{lexer::Location, Ninja, RuleKey, Source, SourceId};
use std::{collections::HashMap, fs};


struct ChangeRaw<'x> {
    loc: Location,
    new_text: &'x str,
}

#[derive(Default)]
struct ChangesRaw<'x> {
    files: HashMap<SourceId, Vec<ChangeRaw<'x>>>,
}
impl<'x> ChangesRaw<'x> {
    fn add_change(&mut self, loc: Location, new_text: &'x str) {
        self.files
            .entry(loc.source_id)
            .or_default()
            .push(ChangeRaw { loc, new_text });
    }
}

pub struct ChangeList<'x> {
    ninja: &'x Ninja,
    changes: ChangesRaw<'x>,
}
impl<'x> ChangeList<'x> {
    pub(crate) fn new(ninja: &Ninja) -> ChangeList {
        ChangeList {
            ninja,
            changes: ChangesRaw::default(),
        }
    }

    pub fn rename_rule(&mut self, rule_key: RuleKey, new_name: &'x str) {
        let rule = &self.ninja.data.rules[rule_key];
        self.changes.add_change(rule.name.loc, new_name);
    
        for i in self.ninja.data.edges.values().filter(|x| rule_key == x.rule) {
            self.changes.add_change(i.rule_loc, new_name);
        }
    }

    pub fn change(&mut self, loc: Location, new_text: &'x str) {
        self.changes.add_change(loc, new_text);
    }

    pub fn commit(self) {
        for (source, changes) in self.changes.files {
            let source = self.ninja.sm.get(source);
            create_new_file(source, changes);
        }
    }
}

fn create_new_file(source: &Source, changes: Vec<ChangeRaw>) {
    let mtime = FileTime::from_last_modification_time(&source.path.metadata().unwrap());

    let text = generate_new_file(source.text(), changes);
    fs::write(&source.path, text).unwrap();

    set_file_mtime(&source.path, mtime).unwrap();
}

fn generate_new_file(original_text: &str, mut changes: Vec<ChangeRaw>) -> String {
    changes.sort_by_key(|x| x.loc);

    let mut text = String::with_capacity(original_text.len());
    let mut original_text_offset = 0;

    for i in changes {
        text += &original_text[original_text_offset..i.loc.start];
        text += i.new_text;
        original_text_offset = i.loc.stop;
    }

    text += &original_text[original_text_offset..];

    text
}
