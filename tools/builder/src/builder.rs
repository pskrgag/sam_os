use crate::runner::*;
use crate::toml::*;

fn check_wrong_impl_and_deps(c: &Component) -> Result<(), String> {
    if let Some(ref deps) = c.depends  && let Some(ref impls) = c.implements {
        let inter = impls
            .iter()
            .filter(|&num| deps.contains(num))
            .cloned()
            .collect::<Vec<_>>();

        if inter.len() != 0 {
            Err(format!(
                "implements and dependencies array contain common {:?}",
                inter
            ))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn find_deps(b: &BuildScript) -> Result<(), String> {
    // Stupid O(n^2), since input won't be very big (I hope)
    // Here we know that each component does not depend on itself, so it's possible to search
    // through all

    for i in b.component.iter() {
        if let Some(ref d) = i.depends {
            let mut found = false;

            for deps in d {
                for j in b.component.iter() {
                    if let Some(ref impls) = j.implements {
                        if impls.iter().find(|x| *x == deps).is_some() {
                            found = true;
                            break;
                        }
                    }
                }

                if !found {
                    return Err(format!(
                        "Dependency '{}' for component '{}' is not satisfied",
                        deps, i.name
                    ));
                }
            }
        }
    }

    Ok(())
}

fn check_init(b: &BuildScript) -> Result<(), ()> {
    for i in b.component.iter() {
        if let Some(ref impls) = i.implements {
            if impls.iter().find(|x| x.as_str() == "init").is_some() {
                return Ok(());
            }
        }
    }

    Err(())
}

fn check_names(b: &BuildScript) -> Result<(), String> {
    let names = b.component.iter().map(|x| &x.name).collect::<Vec<_>>();
    let mut idx = 0;

    if (1..names.len()).any(|i| {
        idx = i;
        names[i..].contains(&names[i - 1])
    }) == true
    {
        return Err(format!(
            "Component name '{}' is not unique",
            b.component[idx].name
        ));
    }

    Ok(())
}

fn sanitize_script(b: &BuildScript) -> Result<(), String> {
    check_names(b)?;

    for c in b.component.iter() {
        check_wrong_impl_and_deps(c)?;
    }

    find_deps(b)?;
    check_init(b).map_err(|_| String::from("None of components implement init"))?;

    Ok(())
}

pub fn build(b: BuildScript) -> Result<(), ()> {
    if let Err(e) = sanitize_script(&b) {
        error!("Build script is wrong: {}", e);
        Err(())
    } else {
        let mut init = None;

        if let Err(e) = prepare_idls() {
            error!("Failed to prepare idls: {}", e);
            return Err(());
        }

        for i in b.component.iter() {
            if let Some(ref impls) = i.implements {
                if impls.iter().find(|x| x.as_str() == "init").is_some() {
                    init = Some(i);
                    continue;
                }
            }

            if let Err(e) = build_component(&i) {
                error!("Failed to build component '{}': {}", i.name, e);
                return Err(());
            }
        }

        if let Err(e) = prepare_cpio(&b.component, "/tmp/archive.cpio") {
            error!("Failed to prepare cpio: {}", e);
            return Err(());
        }

        if let Err(e) = build_component(init.unwrap()) {
            error!("Failed to build component '{}': {}", init.unwrap().name, e);
            return Err(());
        }

        if let Err(e) = build_kernel() {
            error!("Failed to build kernel: {}", e);
            return Err(());
        }

        Ok(())
    }
}
