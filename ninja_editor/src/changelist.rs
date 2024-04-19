use crate::{lexer::Location, Ninja, RuleKey, SourceId};
use std::collections::HashMap;

#[derive(Debug)]
struct RuleChange {
    rule: RuleKey,
    new_name: String,
}

#[derive(Debug)]
enum Change {
    RuleRename(RuleChange),
}

struct ChangeRaw<'x> {
    loc: Location,
    new_text: &'x str,
}

struct ChangesRaw<'x> {
    files: HashMap<SourceId, Vec<ChangeRaw<'x>>>,
}
impl<'x> ChangesRaw<'x> {
    fn add_change(&mut self, loc: Location, new_text: &'x str) {
        self.files
            .entry(loc.source_id)
            .or_default()
            .push(ChangeRaw { loc: loc, new_text });
    }
}

pub struct ChangeList<'x> {
    ninja: &'x Ninja,
    changes: Vec<Change>,
}
impl<'x> ChangeList<'x> {
    pub(crate) fn new(ninja: &Ninja) -> ChangeList {
        ChangeList {
            ninja,
            changes: Vec::new(),
        }
    }
    pub fn rename_rule(&mut self, rule: RuleKey, new_name: String) {
        self.changes
            .push(Change::RuleRename(RuleChange { rule, new_name }));
    }

    pub fn commit(self) {
        let mut changes = ChangesRaw {
            files: HashMap::new(),
        };

        for i in self.changes.iter() {
            process_changes(self.ninja, &mut changes, i);
        }

        for (source, changes) in changes.files {
            let source = self.ninja.sm.get(source);
            let text = source.text();
            create_new_file(text, changes);
        }
    }
}

fn process_rule_change<'x>(ninja: &Ninja, changes: &mut ChangesRaw<'x>, c: &'x RuleChange) {
    let rule = &ninja.data.rules[c.rule];
    changes.add_change(rule.name_loc, &c.new_name);

    for i in ninja.data.edges.iter().filter(|x| c.rule == x.rule) {
        changes.add_change(i.rule_loc, &c.new_name);
    }
}

fn process_changes<'x>(ninja: &Ninja, changes: &mut ChangesRaw<'x>, c: &'x Change) {
    match c {
        Change::RuleRename(rule_rename) => process_rule_change(ninja, changes, rule_rename),
    }
}

fn create_new_file(original_text: &str, mut changes: Vec<ChangeRaw>) {
    changes.sort_by_key(|x| x.loc);

    let mut text = String::with_capacity(original_text.len());
    let mut original_text_offset = 0;

    for i in changes {
        text += &original_text[original_text_offset..i.loc.start];
        text += i.new_text;
        original_text_offset = i.loc.stop;
    }

    println!("{}", text);
}
