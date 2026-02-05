//! Guest program que corre dentro del zkVM de RISC Zero
//!
//! Este código se ejecuta en el zkVM y genera una prueba ZK de que:
//! "Conozco un secreto S tal que SHA256(S) = H"

use risc0_zkvm::guest::env;
use sha2::{Sha256, Digest};

fn main() {
    // 1. Leer el secreto (input privado - NO se revela en la prueba)
    let secret: Vec<u8> = env::read();

    // 2. Calcular el hash SHA256 del secreto
    let mut hasher = Sha256::new();
    hasher.update(&secret);
    let hash_bytes: [u8; 32] = hasher.finalize().into();

    // 3. Commitear el hash (output público - esto SÍ se revela)
    // Este es el "journal" que el verifier puede ver
    env::commit(&hash_bytes);

    // IMPORTANTE:
    // - `secret` NUNCA sale del zkVM
    // - Solo `hash_bytes` es público
    // - La prueba ZK garantiza que el prover conoce un secreto válido
}
