use crate::build;
use crate::build_types::*;
use crate::helpers;
use crate::package_tree::Package;
use ahash::{AHashMap, AHashSet};
use rayon::prelude::*;
use std::fs;
use std::time::SystemTime;

fn remove_asts(source_file: &str, package_name: &str, namespace: &Option<String>, root_path: &str) {
    let _ = std::fs::remove_file(helpers::get_compiler_asset(
        source_file,
        package_name,
        namespace,
        root_path,
        "ast",
    ));
    let _ = std::fs::remove_file(helpers::get_compiler_asset(
        source_file,
        package_name,
        namespace,
        root_path,
        "iast",
    ));
}

fn remove_compile_assets(
    source_file: &str,
    package_name: &str,
    namespace: &Option<String>,
    root_path: &str,
) {
    let _ = std::fs::remove_file(helpers::change_extension(source_file, "mjs"));
    // optimization
    // only issue cmti if htere is an interfacce file
    for extension in &["cmj", "cmi", "cmt", "cmti"] {
        let _ = std::fs::remove_file(helpers::get_compiler_asset(
            source_file,
            package_name,
            namespace,
            root_path,
            extension,
        ));
        let _ = std::fs::remove_file(helpers::get_bs_compiler_asset(
            source_file,
            package_name,
            namespace,
            root_path,
            extension,
        ));
    }
}

fn failed_to_compile(module: &Module) -> bool {
    match &module.source_type {
        SourceType::SourceFile(SourceFile {
            implementation:
                Implementation {
                    compile_state: CompileState::Error | CompileState::Warning,
                    ..
                },
            ..
        }) => true,
        SourceType::SourceFile(SourceFile {
            interface:
                Some(Interface {
                    compile_state: CompileState::Error | CompileState::Warning,
                    ..
                }),
            ..
        }) => true,
        SourceType::SourceFile(SourceFile {
            implementation:
                Implementation {
                    parse_state: ParseState::ParseError | ParseState::Warning,
                    ..
                },
            ..
        }) => true,
        SourceType::SourceFile(SourceFile {
            interface:
                Some(Interface {
                    parse_state: ParseState::ParseError | ParseState::Warning,
                    ..
                }),
            ..
        }) => true,
        _ => false,
    }
}

pub fn cleanup_previous_build(
    packages: &AHashMap<String, Package>,
    all_modules: &mut AHashMap<String, Module>,
    root_path: &str,
) -> (usize, usize, AHashSet<String>) {
    let mut ast_modules: AHashMap<String, (String, String, Option<String>, SystemTime, String)> =
        AHashMap::new();
    let mut ast_rescript_file_locations = AHashSet::new();

    let mut rescript_file_locations = all_modules
        .values()
        .filter_map(|module| match &module.source_type {
            SourceType::SourceFile(source_file) => {
                Some(source_file.implementation.path.to_string())
            }
            _ => None,
        })
        .collect::<AHashSet<String>>();

    rescript_file_locations.extend(
        all_modules
            .values()
            .filter_map(|module| {
                build::get_interface(module)
                    .as_ref()
                    .map(|interface| interface.path.to_string())
            })
            .collect::<AHashSet<String>>(),
    );

    // scan all ast files in all packages
    for package in packages.values() {
        let read_dir = fs::read_dir(std::path::Path::new(&helpers::get_build_path(
            root_path,
            &package.name,
        )))
        .unwrap();

        for entry in read_dir {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let extension = path.extension().and_then(|e| e.to_str());
                    match extension {
                        Some(ext) => match ext {
                            "iast" | "ast" => {
                                let module_name = helpers::file_path_to_module_name(
                                    path.to_str().unwrap(),
                                    &package.namespace,
                                );

                                let ast_file_path = path.to_str().unwrap().to_owned();
                                let res_file_path = build::get_res_path_from_ast(&ast_file_path);
                                match res_file_path {
                                    Some(res_file_path) => {
                                        let _ = ast_modules.insert(
                                            res_file_path.to_owned(),
                                            (
                                                module_name,
                                                package.name.to_owned(),
                                                package.namespace.to_owned(),
                                                entry.metadata().unwrap().modified().unwrap(),
                                                ast_file_path,
                                            ),
                                        );
                                        let _ = ast_rescript_file_locations.insert(res_file_path);
                                    }
                                    None => (),
                                }
                            }
                            _ => (),
                        },
                        None => (),
                    }
                }
                Err(_) => (),
            }
        }
    }

    // delete the .mjs file which appear in our previous compile assets
    // but does not exists anymore
    // delete the compiler assets for which modules we can't find a rescript file
    // location of rescript file is in the AST
    // delete the .mjs file for which we DO have a compiler asset, but don't have a
    // rescript file anymore (path is found in the .ast file)
    let diff = ast_rescript_file_locations
        .difference(&rescript_file_locations)
        .collect::<Vec<&String>>();

    let diff_len = diff.len();

    diff.par_iter().for_each(|res_file_location| {
        let _ = std::fs::remove_file(helpers::change_extension(res_file_location, "mjs"));
        let (_module_name, package_name, package_namespace, _last_modified, _ast_file_path) =
            ast_modules
                .get(&res_file_location.to_string())
                .expect("Could not find module name for ast file");
        remove_asts(
            res_file_location,
            package_name,
            package_namespace,
            root_path,
        );
        remove_compile_assets(
            res_file_location,
            package_name,
            package_namespace,
            root_path,
        );
    });

    ast_rescript_file_locations
        .intersection(&rescript_file_locations)
        .into_iter()
        .for_each(|res_file_location| {
            let (module_name, _package_name, _package_namespace, ast_last_modified, ast_file_path) =
                ast_modules
                    .get(res_file_location)
                    .expect("Could not find module name for ast file");
            let module = all_modules
                .get_mut(module_name)
                .expect("Could not find module for ast file");

            match &mut module.source_type {
                SourceType::MlMap(_) => unreachable!("MlMap is not matched with a ReScript file"),
                SourceType::SourceFile(source_file) => {
                    if helpers::is_interface_ast_file(ast_file_path) {
                        let interface = source_file
                            .interface
                            .as_mut()
                            .expect("Could not find interface for module");

                        let source_last_modified = interface.last_modified;
                        if ast_last_modified > &source_last_modified {
                            interface.dirty = false;
                        }
                    } else {
                        let implementation = &mut source_file.implementation;
                        let source_last_modified = implementation.last_modified;
                        if ast_last_modified > &source_last_modified {
                            implementation.dirty = false;
                        }
                    }
                }
            }
        });

    let ast_module_names = ast_modules
        .values()
        .map(|(module_name, _, _, _, _)| module_name)
        .collect::<AHashSet<&String>>();

    let all_module_names = all_modules
        .keys()
        .map(|module_name| module_name)
        .collect::<AHashSet<&String>>();

    let deleted_module_names = ast_module_names
        .difference(&all_module_names)
        .map(|module_name| {
            // if the module is a namespace, we need to mark the whole namespace as dirty when a module has been deleted
            if let Some(namespace) = helpers::get_namespace_from_module_name(module_name) {
                return namespace;
            }
            return module_name.to_string();
        })
        .collect::<AHashSet<String>>();

    (
        diff_len,
        ast_rescript_file_locations.len(),
        deleted_module_names,
    )
}

pub fn cleanup_after_build(
    modules: &AHashMap<String, Module>,
    compiled_modules: &AHashSet<String>,
    all_modules: &AHashSet<String>,
    project_root: &str,
) {
    let failed_modules = all_modules
        .difference(&compiled_modules)
        .collect::<AHashSet<&String>>();

    modules.par_iter().for_each(|(module_name, module)| {
        if failed_to_compile(module) || failed_modules.contains(module_name) {
            // only retain ast file if it compiled successfully, that's the only thing we check
            // if we see a AST file, we assume it compiled successfully, so we also need to clean
            // up the AST file if compile is not successful
            match &module.source_type {
                SourceType::SourceFile(source_file) => {
                    remove_asts(
                        &source_file.implementation.path,
                        &module.package.name,
                        &module.package.namespace,
                        &project_root,
                    );
                    // remove_compile_assets(
                    //     &source_file.implementation.path,
                    //     &module.package.name,
                    //     &module.package.namespace,
                    //     &project_root,
                    // );
                }
                _ => (),
            }
        }
    });
}
