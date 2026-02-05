//! Build script que compila el código Lean y lo linkea con Rust
//!
//! Este script:
//! 1. Compila el código C generado por Lean
//! 2. Linkea contra el runtime de Lean (libleanrt)

use std::env;
use std::path::PathBuf;

fn main() {
    // Path al proyecto Lean
    let lean_project = PathBuf::from("../lean_verifier");

    // Path al toolchain de Lean (ajustar versión si es necesario)
    let home = env::var("HOME").unwrap();
    let lean_toolchain = PathBuf::from(&home)
        .join(".elan/toolchains/leanprover--lean4---v4.26.0");

    // Archivo C generado por Lean
    let lean_c_file = lean_project.join(".lake/build/ir/LeanVerifier/Verifier.c");

    // Solo compilar si existe el archivo C (después de `lake build`)
    if lean_c_file.exists() {
        // Compilar el código C de Lean
        cc::Build::new()
            .file(&lean_c_file)
            .include(lean_toolchain.join("include"))
            .opt_level(2)
            .compile("lean_verifier");

        // Linkear contra el runtime de Lean
        println!(
            "cargo:rustc-link-search={}",
            lean_toolchain.join("lib/lean").display()
        );
        println!("cargo:rustc-link-lib=static=leanrt");

        // En macOS necesitamos linkear contra libc++
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=c++");

        // En Linux necesitamos linkear contra libstdc++
        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-lib=stdc++");
    } else {
        println!("cargo:warning=Lean C file not found. Run 'cd lean_verifier && lake build' first.");
    }

    // Rebuild si cambian los archivos Lean
    println!("cargo:rerun-if-changed=../lean_verifier/LeanVerifier/Verifier.lean");
    println!("cargo:rerun-if-changed=../lean_verifier/.lake/build/ir/LeanVerifier/Verifier.c");
}
