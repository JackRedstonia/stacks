use std::env;
use std::path::PathBuf;

fn main() {
    // This build script is purely from the SDL example build script, but
    // modified for 64 bit platforms. This build script will not do anything
    // unless it is run on a Windows platform.
    //
    // I honestly do not know why such a build script was ever necessary in the
    // first place for something so simple as getting SDL2.lib files, but this
    // is Windows, what do you expect? I'm glad I do not use this rubbish yard
    // of an operating system for anything other than testing obvious bugs.
    //
    // May the gods of crustaceans (and Rustaceans) bless Linus Torvalds and
    // Graydon Hoare that I only have to deal with 0.1% of Windows'
    // never-ending bullcrap.

    let target = env::var("TARGET").unwrap();
    if target.contains("pc-windows") {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let mut lib_dir = manifest_dir.clone();
        let mut dll_dir = manifest_dir.clone();
        lib_dir.push("msvc");
        dll_dir.push("msvc");
        lib_dir.push("lib");
        dll_dir.push("dll");
        lib_dir.push("64");
        dll_dir.push("64");
        println!("cargo:rustc-link-search=all={}", lib_dir.display());
        for entry in std::fs::read_dir(dll_dir).expect("Can't read DLL dir") {
            let entry_path = entry.expect("Invalid fs entry").path();
            let file_name_result = entry_path.file_name();
            let mut new_file_path = manifest_dir.clone();
            if let Some(file_name) = file_name_result {
                let file_name = file_name.to_str().unwrap();
                if file_name.ends_with(".dll") {
                    new_file_path.push(file_name);
                    std::fs::copy(&entry_path, new_file_path.as_path())
                        .expect("Can't copy from DLL dir");
                }
            }
        }
    }
}
