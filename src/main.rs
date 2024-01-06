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
        if import.len() == 2 {
            name = import[1].clone();
        } else if import.len() > 2 && import[2] == "as" {
            name = import[1].clone();
        } else {
            for fp in file_paths {
                if format!("{}/{}/__init__.py", directory, import[1]) == fp {
                    name = import[1].clone()
                }
            }
        }
    } else if import[0] == "from" {
        for fp in file_paths {
            if import[1].starts_with(".") {
                let import_paths_up = import[1].chars().collect::<Vec<char>>();
                let import_paths_down = import[1].split('.').collect::<Vec<&str>>();

                let mut file_dir = file_path.split('/').collect::<Vec<&str>>();

                for (_i, ip) in import_paths_up.iter().enumerate() {
                    if ip == &'.' {
                        file_dir.pop();
                    }
                }

                for ip in import_paths_down {
                    if ip != "" {
                        file_dir.push(ip);
                    }
                }

                let file_dir = file_dir.join("/");

                if format!("{}.py", file_dir) == fp {
                    name = format!("{}.py", file_dir.split("/").last().unwrap().to_string())
                } else if format!("{}/__init__.py", file_dir) == fp {
                    name = fp.replace("__init__.py", "")
                }
            } else {
                let mut import_dir = import[1].replace(".", "/");

                import_dir = format!("{}/{}", directory, import_dir);

                if format!("{}.py", import_dir) == fp {
                    name = format!("{}.py", import_dir.split("/").last().unwrap().to_string())
                } else if format!("{}/__init__.py", import_dir) == fp {
                    name = fp.replace("__init__.py", "")
                }
            }
        }
    }

    if name == "UNKNOWN" && import[0] == "from" && !import[1].starts_with(".") {
        println!("UNKNOWN: {:?} - {:?} ", import, file_path);
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
    let mut unknown_counter = 0;

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
        // if file_name != "build_ext.py" {
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

            if imported_from_file == "UNKNOWN" {
                unknown_counter += 1;
            }

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

    println!("Unknown imports: {}", unknown_counter);
}
