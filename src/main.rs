pub mod bsconfig;
pub mod build;
pub mod helpers;
pub mod package_tree;
pub mod structure_hashmap;
pub mod watcher;
use ahash::AHashMap;
use convert_case::{Case, Casing};
use helpers::*;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_abs_path(path: &str) -> String {
    let abs_path_buf = PathBuf::from(path);
    return fs::canonicalize(abs_path_buf)
        .expect("Could not canonicalize")
        .to_str()
        .expect("Could not canonicalize")
        .to_string();
}

fn get_basename(path: &str) -> String {
    let path_buf = PathBuf::from(path);
    return path_buf
        .file_stem()
        .expect("Could not get basename")
        .to_str()
        .expect("Could not get basename")
        .to_string();
}

//fn compile(package: &package_tree::Package, ast_path: &str, root_path: &str) {
//let abs_node_modules_path = get_node_modules_path(root_path);
//let namespace = get_namespace(package);
//let to_mjs_args = vec![
//match namespace {
//Some(namespace) => vec!["-bs-ns".to_string(), namespace.to_string()],
//None => vec![],
//},
//vec!["-I".to_string(), ".".to_string()],
//vec![
//"-bs-package-name".to_string(),
//package.bsconfig.name.to_owned(),
//"-bs-package-output".to_string(),
//// "src" here needs to be the relative folder name of the mjs file
//format!("es6:{}:.mjs", "src"),
//ast_path.to_string(),
//],
//]
//.concat();

//dbg!(&to_mjs_args);
//let build_path_abs = get_build_path(root_path, &package.name);

//let to_mjs = Command::new(
//abs_node_modules_path.to_string() + &"/rescript/darwinarm64/bsc.exe".to_string(),
//)
//.current_dir(build_path_abs.to_string())
//.args(to_mjs_args)
//.output()
//.expect("err");

//println!("STDOUT: {}", std::str::from_utf8(&to_mjs.stdout).expect(""));
//println!("STDERR: {}", std::str::from_utf8(&to_mjs.stderr).expect(""));
//}

fn compile_mlmap(package: &package_tree::Package, namespace: &str, root_path: &str) {
    let abs_node_modules_path = get_node_modules_path(root_path);
    let build_path_abs = get_build_path(root_path, &package.name);

    let mlmap_name = format!("{}.mlmap", namespace);
    let args = vec![vec![
        "-w",
        "-49",
        "-color",
        "always",
        "-no-alias-deps",
        &mlmap_name,
    ]]
    .concat();

    let _ = Command::new(
        abs_node_modules_path.to_string() + &"/rescript/darwinarm64/bsc.exe".to_string(),
    )
    .current_dir(build_path_abs.to_string())
    .args(args)
    .output()
    .expect("err");
}

fn main() {
    let folder = "walnut_monorepo";

    let packages = package_tree::make(&folder);
    let rescript_version = build::get_version(&folder);
    let source_files = build::get_dependencies(rescript_version, &folder, packages.to_owned());
    println!("FINISH CONVERSION TO AST");

    //let root = &packages["@teamwalnut/stdlib"];

    //let version_cmd = Command::new("node_modules/rescript/rescript")
    //.current_dir(project_root.to_string())
    //.args(["-v"])
    //.output()
    //.expect("failed to find version");

    //let version = std::str::from_utf8(&version_cmd.stdout)
    //.expect("Could not read version from rescript")
    //.replace("\n", "");

    //let ast_file = generate_ast(root, "/src/Bar.res", &project_root, &version);
    //compile(root, &ast_file, &project_root);

    //let ast_file = generate_ast(root, "/src/Foo.res", &project_root, &version);
    //compile(root, &ast_file, &project_root);

    println!("START COMPILING");
    // source_files
    //     .iter()
    //     .filter(|(_file, source)| source.ast_deps.len() == 0)
    //     .for_each(|(file, source)| {
    //         let pkg_path_abs = folder.to_owned() + "/node_modules/" + &source.package.bsconfig.name;
    //         let abs_node_modules_path =
    //             helpers::get_abs_path(&(folder.to_owned() + "/node_modules"));

    //         if source.is_ml_map {
    //             dbg!(file);
    //         }

    //         //if source.is_ml_map {
    //             //compile_mlmap(&source.package, &source.namespace, &folder)
    //         //} else {
    //             //build::compile_file(&pkg_path_abs, &abs_node_modules_path, source);
    //         //}
    //     });
}
