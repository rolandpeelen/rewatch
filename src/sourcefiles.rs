use crate::helpers;
use crate::package_tree::Package;
use ahash::AHashMap;
use rayon::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;

type Dir = String;
type PackageName = String;
type AbsolutePath = String;
type PackagePath = String;

#[derive(Serialize, Debug, Clone, PartialEq, Hash)]
pub struct SourceFiles {
    pub dirs: Vec<Dir>,
    pub subdirs: Vec<(PackageName, AbsolutePath)>,
    pub generated: Vec<String>,
}

pub fn print(project_root: &str, packages: &AHashMap<String, Package>) {
    //packages: &AHashMap<String, Package>
    //
    let (name, package) = packages
        .into_iter()
        .find(|(_name, package)| package.is_root)
        .expect("Could not find root package");

    // First do all child packages
    let child_packages = packages
        .par_iter()
        .filter(|(_name, package)| !package.is_root)
        .map(|(name, package)| {
            let path = helpers::get_bs_build_path(project_root, name, package.is_root);
            let dirs = vec![];
            let subdirs = vec![];

            let _ = File::create(path + "./.sourcefiles.json").map(|mut file| {
                let source_files = SourceFiles {
                    dirs: dirs.clone(),
                    subdirs: subdirs.clone(),
                    generated: vec![],
                };

                file.write(json!(source_files).to_string().as_bytes())
            });

            (
                &package.package_dir,
                dirs,
                subdirs
                    .into_iter()
                    .collect::<AHashMap<PackageName, AbsolutePath>>(),
            )
        })
        .collect::<Vec<(&PackagePath, Vec<Dir>, AHashMap<PackageName, AbsolutePath>)>>();

    let mut all_dirs = vec![];
    let mut all_subdirs: AHashMap<PackageName, AbsolutePath> = AHashMap::new();

    child_packages.iter().for_each(|(package_path, dirs, subdirs)| {
        dirs.iter()
            .for_each(|dir| all_dirs.push(format!("{package_path}/{dir}")));
        all_subdirs.extend(subdirs.to_owned());
    });

    let path = helpers::get_bs_build_path(project_root, name, package.is_root);
    let _ = File::create(path + "./.sourcefiles.json").map(|mut file| {
        let all_source_files = SourceFiles {
            dirs: all_dirs,
            subdirs: all_subdirs
                .into_iter()
                .collect::<Vec<(PackageName, AbsolutePath)>>(),
            generated: vec![],
        };
        file.write(json!(all_source_files).to_string().as_bytes())
    });

    // Then join, and put them all together
}
