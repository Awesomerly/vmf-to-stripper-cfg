use std::collections::HashMap;
use std::path::PathBuf;
use std::{collections::HashSet, fs::File};

use clap::Parser;
// use vbsp;
use vmf_forge::{Entity, VmfError, VmfFile};

const ABOUT: &str = "Diffs VMFs and outputs a stripper config";

#[derive(Parser)]
#[command(version, about = ABOUT)]
struct Cli {
    #[arg(value_name = "Unmodified VMF")]
    origin_path: PathBuf,

    #[arg(value_name = "Modified VMF")]
    mod_path: PathBuf,
}

struct MyVmfWrapper {
    vmf: VmfFile,
    id_set: HashSet<u64>,
    id_map: HashMap<u64, usize>,
}

impl MyVmfWrapper {
    fn new(path: PathBuf) -> Self {
        let mut f = File::open(path).unwrap();
        let vmf = VmfFile::parse_file(&mut f).unwrap();
        let id_set = vmf.entities.iter().map(|e| e.id()).collect();
        let id_map = vmf
            .entities
            .vec
            .iter()
            .enumerate()
            .map(|(idx, e)| (e.id(), idx))
            .collect();
        Self {
            vmf,
            id_set,
            id_map,
        }
    }

    fn find_entity_by_id(&self, id: &u64) -> Option<&Entity> {
        if self.id_map.contains_key(id) {
            let idx = self.id_map.get(id).unwrap();
            return Some(&self.vmf.entities.vec[*idx]);
        } else {
            None
        }
    }
}

// inspired by
// https://github.com/IaVashik/vmf-forge/blob/49be533bfa1c2bcaaf00059bf4f30d784d3c5567/src/vmf/entities.rs#L284
// TODO: Optionally include hammer id in these functions
fn create_add_string(entity: &Entity) -> String {
    let mut output = String::with_capacity(256);
    output.push_str(&format!("add:\n{{\n"));

    // Adds key_values of the main block
    for (key, value) in &entity.key_values {
        output.push_str(&format!("\t\"{}\" \"{}\"\n", key, value));
    }

    // TODO: test this out
    for i in entity.connections.iter() {
        let keystr = &i[0].0;
        let valstr = i[0].1.split('\u{1b}').collect::<Vec<_>>().join(",");
        output.push_str(&format!("\t\"{}\" \"{}\"\n", keystr, valstr));
    }

    output.push_str("}\n");
    output
}

fn create_filter_string(entity: &Entity) -> String {
    let mut output = String::with_capacity(256);
    output.push_str(&format!("filter:\n{{\n"));
    output.push_str(&format!(
        "\t\"{}\" \"{}\"\n",
        "classname",
        &entity.key_values.get("classname").unwrap()
    ));

    if entity.key_values.get("origin").is_none() {
        println!("{:#?}", entity)
    }
    output.push_str(&format!(
        "\t\"{}\" \"{}\"\n",
        "origin",
        &entity.key_values.get("origin").unwrap()
    ));

    output.push_str("}\n");
    output
}

fn create_modify_string(
    entity: &Entity,
    added: Vec<(&str, &str)>,
    removed: Vec<&str>,
    modified: Vec<(&str, &str)>,
) -> String {
    let mut output = String::with_capacity(256);
    output.push_str("modify:\n{\n");

    // MATCH
    output.push_str("\tmatch:\n\t{\n");

    output.push_str(&format!(
        "\t\t\"{}\" \"{}\"\n",
        "classname",
        &entity.key_values.get("classname").unwrap()
    ));

    output.push_str(&format!(
        "\t\t\"{}\" \"{}\"\n",
        "origin",
        &entity.key_values.get("origin").unwrap()
    ));

    output.push_str("\t}\n");

    // REPLACE
    if !modified.is_empty() {
        output.push_str("\treplace:\n\t{\n");
        for m in modified.iter() {
            output.push_str(&format!("\t\t\"{}\" \"{}\"\n\t}}\n", m.0, m.1));
        }
    }

    // DELETE
    if !removed.is_empty() {
        output.push_str("\tdelete:\n\t{\n");
        for r in removed.iter() {
            output.push_str(&format!(
                "\t\t\"{}\" \"{}\"\n\t}}\n",
                r,
                entity.get(&r).unwrap()
            ));
        }
    }

    if !added.is_empty() {
        output.push_str("\tinsert:\n\t{\n");
        for a in added.iter() {
            output.push_str(&format!("\t\t\"{}\" \"{}\"\n\t}}\n", a.0, a.1,));
        }
    }

    output.push_str("}");

    output
}

fn main() -> Result<(), VmfError> {
    let cli = Cli::parse();

    let old_vmf = MyVmfWrapper::new(cli.origin_path);
    let new_vmf = MyVmfWrapper::new(cli.mod_path);

    println!("{:#?}", new_vmf.vmf.visgroups);

    // for e in old_vmf.vmf.entities.iter() {
    //     if e.connections != None {
    //         println!("{:#?}", e);
    //         let blah = e.connections.as_ref().unwrap().iter();
    //         println!("{:?}", blah);
    //     }
    // }

    let added_entities = new_vmf
        .id_set
        .difference(&old_vmf.id_set)
        .map(|x| new_vmf.find_entity_by_id(x));

    let removed_entities = old_vmf
        .id_set
        .difference(&new_vmf.id_set)
        .map(|x| old_vmf.find_entity_by_id(x));

    for a in added_entities {
        println!("{}", create_add_string(a.unwrap()));
    }

    for a in removed_entities {
        println!("{}", create_filter_string(a.unwrap()));
    }

    let ent_intersect = old_vmf.id_set.intersection(&new_vmf.id_set);

    // TODO: add support for connections in here
    for e in ent_intersect {
        let old = old_vmf.find_entity_by_id(e).unwrap();
        let new = new_vmf.find_entity_by_id(e).unwrap();

        if old.key_values != new.key_values {
            let mut added_list: Vec<(&str, &str)> = vec![];
            let mut modified_list: Vec<(&str, &str)> = vec![];
            let mut removed_list: Vec<&str> = vec![];

            // Finding diffs in entity string
            for (old_key, old_value) in old.key_values.iter() {
                if new.key_values.contains_key(old_key) {
                    let new_value = new.key_values.get(old_key).unwrap();
                    if !new_value.eq(old_value) {
                        modified_list.push((old_key, new_value));
                    }
                } else {
                    removed_list.push(old_key);
                }
            }

            for (new_key, new_value) in new.key_values.iter() {
                if !old.key_values.contains_key(new_key) {
                    added_list.push((new_key, new_value));
                }
            }

            println!(
                "{}",
                create_modify_string(&old, added_list, removed_list, modified_list)
            )
        }
    }

    Ok(())
}
