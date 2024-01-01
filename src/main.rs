use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

mod py_std_libs;

#[derive(Serialize, Deserialize, Debug)]
struct Element {
    data: HashMap<String, String>,
}

#[derive(Parser)]
pub struct Args {
    pub directory: String,
}

fn list_python_files(directory: &str) -> Vec<String> {
    let mut python_files = Vec::new();
    for entry in WalkDir::new(directory) {
        let entry = entry.unwrap();
        if Path::new(entry.path()).extension().unwrap_or_default() == "py" {
            python_files.push(entry.path().display().to_string());
        }
    }
    python_files
}

fn extract_imports(file_path: &str) -> Vec<String> {
    let content = fs::read_to_string(file_path).expect("Should have been able to read the file");
    let lines = content.lines();

    let mut imports = Vec::new();
    for line in lines {
        if line.starts_with("import") || line.starts_with("from") {
            imports.push(line.to_string());
        }
    }
    imports
}

fn filter_imports(imports_list: Vec<String>) -> Vec<Vec<String>> {
    let mut filtered = Vec::new();
    for imp in imports_list {
        let parts: Vec<&str> = imp.split_whitespace().collect();
        if py_std_libs::LIBS.contains(&parts[1]) {
            continue;
        }
        filtered.push(parts.iter().map(|s| s.to_string()).collect());
    }
    filtered
}

fn get_file_or_module_name_from_import(
    directory: &str,
    import: Vec<String>,
    file_name: &str,
    file_path: &str,
    file_paths: Vec<String>,
) -> String {
    let mut name = "UNKNOWN".to_string();

    if import[0] == "import" {
        for fp in file_paths {
            if format!("{}/{}/__init__.py", directory, import[1]) == fp {
                name = import[1].clone()
            }
        }
    } else if import[0] == "from" {
        for fp in file_paths {
            if format!("{}/{}/__init__.py", directory, import[1]) == fp {
                name = import[1].clone()
            } else if import[1].starts_with(".") {
                let file_dir = file_path.replace(file_name, "");
                if format!("{}{}.py", file_dir, import[1].replace(".", "")) == fp {
                    println!("{} <- {:?}", file_path, import);
                    name = fp
                }
            }
        }
    }

    name
}

fn main() {
    let ignore_files = vec!["__init__.py"];

    let args = Args::parse();

    let directory = args.directory.as_str();
    let python_files = list_python_files(directory);

    let mut elements = Vec::new();
    let mut all_nodes = HashSet::new();

    for file_path in python_files.clone() {
        let file_name = Path::new(&file_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        if ignore_files.contains(&file_name.as_str()) {
            continue;
        }

        // TODO: This is a hack to ignore the cfuncs.py and auxfuncs.py files
        // if file_name != "cfuncs.py" && file_name != "auxfuncs.py" {
        //     continue;
        // }

        let imports = extract_imports(&file_path);
        // let normalized_imports = normalize_imports(imports);
        let filtered_imports = filter_imports(imports);

        let directory_with_slash = format!("/{}", directory);
        let file_rel_path = file_path.replace(&directory_with_slash, "");

        all_nodes.insert(file_rel_path.clone());
        for imp in filtered_imports.iter() {
            let imported_from_file = get_file_or_module_name_from_import(
                directory,
                imp.clone(),
                &file_name,
                &file_path,
                python_files.clone(),
            );

            all_nodes.insert(imported_from_file.clone());
            elements.push(Element {
                data: [
                    ("source".to_string(), file_rel_path.clone()),
                    ("target".to_string(), imported_from_file.clone()),
                ]
                .iter()
                .cloned()
                .collect(),
            });
        }
    }

    for node in all_nodes {
        elements.push(Element {
            data: [
                ("id".to_string(), node.clone()),
                ("label".to_string(), node.clone()),
            ]
            .iter()
            .cloned()
            .collect(),
        });
    }

    let json = serde_json::to_string(&elements).expect("Could not serialize to JSON");
    let mut file = File::create("graph_data.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
