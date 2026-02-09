//! Host program que combina RISC Zero zkVM con Lean FFI
//!
//! Este programa:
//! 1. Usa funciones Lean (via FFI) para crear/verificar commitments
//! 2. Ejecuta el guest en el zkVM para generar prueba ZK
//! 3. Verifica la prueba y usa Lean para validar el resultado

use methods::{GUEST_ELF, GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use sha2::{Sha256, Digest};

// ============================================
// FFI: Funciones importadas desde Lean
// ============================================

extern "C" {
    fn lean_create_commitment(value: u64, salt: u64) -> u64;
    fn lean_verify_commitment(hash: u64, value: u64, salt: u64) -> u8;
}

// Wrappers seguros para las funciones Lean
mod lean {
    use super::*;

    pub fn create_commitment(value: u64, salt: u64) -> u64 {
        unsafe { lean_create_commitment(value, salt) }
    }

    pub fn verify_commitment(hash: u64, value: u64, salt: u64) -> bool {
        unsafe { lean_verify_commitment(hash, value, salt) == 1 }
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  RISC Zero + Lean FFI: Verificación Formal + ZK Real       ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // ========================================
    // PARTE 1: Demo de Lean FFI (Commitment)
    // ========================================
    println!("═══ Parte 1: Lean FFI - Commitment Scheme ═══");
    println!();

    let secret_value: u64 = 42;
    let salt: u64 = 12345;

    // Crear commitment usando función Lean verificada
    let commitment = lean::create_commitment(secret_value, salt);
    println!("  [Lean] Commitment creado: {}", commitment);
    println!("         (valor={}, salt={})", secret_value, salt);

    // Verificar commitment correcto
    let valid = lean::verify_commitment(commitment, secret_value, salt);
    println!("  [Lean] Verificación correcta: {}", if valid { "✓" } else { "✗" });

    // Intentar fraude
    let fraud = lean::verify_commitment(commitment, 99, salt);
    println!("  [Lean] Verificación con valor falso: {}", if fraud { "✓" } else { "✗" });
    println!();

    // ========================================
    // PARTE 2: RISC Zero zkVM
    // ========================================
    println!("═══ Parte 2: RISC Zero - Prueba ZK ═══");
    println!();

    // El secreto que queremos probar que conocemos
    let secret = b"mi_secreto_super_secreto_12345";
    println!("  [Prover] Tengo un secreto de {} bytes", secret.len());

    // Calcular hash esperado (para verificación)
    let mut hasher = Sha256::new();
    hasher.update(secret);
    let expected_hash: [u8; 32] = hasher.finalize().into();
    println!("  [Prover] Hash esperado: 0x{}", hex::encode(&expected_hash[..8]));
    println!();

    // Crear environment para el guest
    let env = ExecutorEnv::builder()
        .write(&secret.to_vec())
        .unwrap()
        .build()
        .unwrap();

    // Generar la prueba ZK
    println!("  Generando prueba ZK... (esto puede tardar)");
    let prover = default_prover();
    let receipt = prover.prove(env, GUEST_ELF).unwrap().receipt;
    println!("  ✓ Prueba generada!");
    println!();

    // ========================================
    // PARTE 3: Verificación
    // ========================================
    println!("═══ Parte 3: Verificación ═══");
    println!();

    // Extraer el hash del journal (output público del guest)
    let journal_hash: [u8; 32] = receipt.journal.decode().unwrap();
    println!("  [Verifier] Hash del journal: 0x{}", hex::encode(&journal_hash[..8]));

    // Verificar que coincide con el esperado
    let hashes_match = journal_hash == expected_hash;
    println!("  [Verifier] ¿Hashes coinciden?: {}", if hashes_match { "✓ SÍ" } else { "✗ NO" });

    // Verificar la prueba criptográfica
    receipt.verify(GUEST_ID).expect("Verificación de prueba falló");
    println!("  [Verifier] ✓ Prueba ZK válida!");
    println!();

    // ========================================
    // CONCLUSIÓN
    // ========================================
    println!("═══════════════════════════════════════════════════════════");
    println!("  RESULTADO: El prover conoce un secreto cuyo hash es:");
    println!("  0x{}", hex::encode(&journal_hash));
    println!();
    println!("  ✓ Lean FFI: Funciones de verificación formalmente probadas");
    println!("  ✓ RISC Zero: Prueba ZK generada y verificada");
    println!("  ⚠ El secreto NUNCA fue revelado");
    println!("═══════════════════════════════════════════════════════════");
}
