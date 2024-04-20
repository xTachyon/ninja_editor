use bumpalo::Bump;
use ninja_editor::Ninja;

fn main() {
    let prefix = "p_";

    let ninja = Ninja::load("debug/build.ninja");
    let data = ninja.data();
    let mut changelist = ninja.change();
    let bump = Bump::new();
    // for (key, rule) in data.rules.iter() {
    //     if rule.name.elem == "phony" {
    //         continue;
    //     }
    //     changelist.rename_rule(
    //         key,
    //         bump.alloc_str(&format!("{}{}", prefix, rule.name.elem)),
    //     );
    // }

    for (k, v) in data.nodes.iter() {
        if k.starts_with("cmake_") {
            let text = bump.alloc_str(&format!("{}{}", prefix, k));
            for loc in v {
                changelist.change(*loc, text);
            }
            continue;
        }
        if k.ends_with("CMakeLists.txt")
            || k.ends_with(".cmake")
            || k.ends_with(".cmake.in")
            || k.ends_with("vcpkg.json")
            || k.starts_with("/usr/share/cmake")
        {
            let text = bump.alloc_str(&format!("{}{}", prefix, k));
            for loc in v {
                changelist.change(*loc, text);
            }
        }
    }

    changelist.commit();
}
