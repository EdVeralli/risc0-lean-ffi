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

    // Verificar que el archivo C de Lean existe (requiere `lake build` previo)
    if !lean_c_file.exists() {
        panic!(
            "Lean C file not found: {}\nRun 'cd lean_verifier && lake build' first.",
            lean_c_file.display()
        );
    }

    // Compilar el código C de Lean
    cc::Build::new()
        .file(&lean_c_file)
        .include(lean_toolchain.join("include"))
        .opt_level(2)
        .compile("lean_verifier");

    // Linkear contra el runtime de Lean (shared library incluye todo)
    let lean_lib = lean_toolchain.join("lib/lean");
    println!("cargo:rustc-link-search={}", lean_lib.display());
    println!("cargo:rustc-link-lib=dylib=leanshared");

    // Rebuild si cambian los archivos Lean
    println!("cargo:rerun-if-changed=../lean_verifier/LeanVerifier/Verifier.lean");
    println!("cargo:rerun-if-changed=../lean_verifier/.lake/build/ir/LeanVerifier/Verifier.c");
}
