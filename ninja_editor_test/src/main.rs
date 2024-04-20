use ninja_editor::Ninja;

fn main() {
    let ninja = Ninja::load_folder(".");
    let data = ninja.data();
    let mut changelist = ninja.change();
    for (key, rule) in data.rules.iter() {
        if rule.name.elem == "phony" {
            continue;
        }
        changelist.rename_rule(key, format!("p_{}", rule.name.elem));
    }

    changelist.commit();
}
