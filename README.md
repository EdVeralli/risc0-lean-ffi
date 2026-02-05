# RISC Zero + Lean 4 FFI

Integración completa de **RISC Zero zkVM** con **Lean 4 via FFI**.

## ¿Qué hace este proyecto?

Combina lo mejor de ambos mundos:

| Componente | Función |
|------------|---------|
| **Lean 4** | Funciones con teoremas probados (verificación formal) |
| **FFI** | Las funciones Lean se ejecutan desde Rust |
| **RISC Zero** | Genera pruebas ZK reales (STARK) |

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Lean 4    │ →   │      C      │ →   │    Rust     │
│             │     │             │     │   (Host)    │
│ • Teoremas  │     │  Generado   │     │ • Lean FFI  │
│ • @[export] │     │  por Lean   │     │ • RISC Zero │
└─────────────┘     └─────────────┘     └─────────────┘
                                              │
                                              ▼
                                        ┌─────────────┐
                                        │   zkVM      │
                                        │  (Guest)    │
                                        │             │
                                        │ Genera      │
                                        │ prueba ZK   │
                                        └─────────────┘
```

## Estructura

```
risc0-lean-ffi/
├── lean_verifier/           # Código Lean con @[export]
│   ├── LeanVerifier/
│   │   └── Verifier.lean    # Funciones + teoremas
│   └── Main.lean
├── host/                    # Rust host
│   ├── build.rs             # Compila Lean C → linkea
│   └── src/main.rs          # Usa Lean FFI + RISC Zero
├── methods/
│   └── guest/               # Código que corre en zkVM
│       └── src/main.rs
└── README.md
```

## Teoremas Probados (Lean)

```lean
-- Completeness: commitment correcto siempre verifica
theorem commitment_completeness (value salt : UInt64) :
    verifyCommitment (createCommitment value salt) value salt = true

-- Soundness: si verifica, el hash es correcto
theorem commitment_soundness (hash value salt : UInt64) :
    verifyCommitment hash value salt = true →
    hash = createCommitment value salt

-- Binding: no se puede abrir a otro valor
theorem commitment_binding (hash v1 s1 v2 s2 : UInt64) :
    verifyCommitment hash v1 s1 = true →
    verifyCommitment hash v2 s2 = true →
    createCommitment v1 s1 = createCommitment v2 s2
```

## Requisitos

- [Lean 4](https://lean-lang.org/) con elan
- [Rust](https://rustup.rs/)
- [RISC Zero](https://risczero.com/) toolchain

### Instalar Lean 4

```bash
curl -sSf https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh | sh
```

### Instalar RISC Zero

```bash
curl -L https://risczero.com/install | bash
rzup install
```

## Compilar y Ejecutar

### 1. Compilar Lean

```bash
cd lean_verifier
lake build
cd ..
```

### 2. Ejecutar (sin Lean FFI)

```bash
cargo run --release
```

### 3. Ejecutar (con Lean FFI)

```bash
cargo run --release --features lean-ffi
```

## Output Esperado

```
╔════════════════════════════════════════════════════════════╗
║  RISC Zero + Lean FFI: Verificación Formal + ZK Real       ║
╚════════════════════════════════════════════════════════════╝

═══ Parte 1: Lean FFI - Commitment Scheme ═══

  [Lean] Commitment creado: 13647
         (valor=42, salt=12345)
  [Lean] Verificación correcta: ✓
  [Lean] Verificación con valor falso: ✗

═══ Parte 2: RISC Zero - Prueba ZK ═══

  [Prover] Tengo un secreto de 30 bytes
  [Prover] Hash esperado: 0x5a8e4016...

  Generando prueba ZK... (esto puede tardar)
  ✓ Prueba generada!

═══ Parte 3: Verificación ═══

  [Verifier] Hash del journal: 0x5a8e4016...
  [Verifier] ¿Hashes coinciden?: ✓ SÍ
  [Verifier] ✓ Prueba ZK válida!

═══════════════════════════════════════════════════════════
  RESULTADO: El prover conoce un secreto cuyo hash es:
  0x5a8e40160945dfb96fce9e0149808a80035899d02c1eca6a155b5eaa2da74a48

  ✓ Lean FFI: Funciones de verificación formalmente probadas
  ✓ RISC Zero: Prueba ZK generada y verificada
  ⚠ El secreto NUNCA fue revelado
═══════════════════════════════════════════════════════════
```

## ¿Por qué esto importa?

1. **Lean**: Garantiza matemáticamente que las funciones de verificación son correctas
2. **FFI**: Esas funciones se ejecutan en producción (no solo especificación)
3. **RISC Zero**: Genera pruebas ZK reales verificables por cualquiera

> "A single bug in a zkVM can compromise billions of dollars"
>
> Con Lean FFI, las funciones críticas tienen **garantías formales**.

## Referencias

- [RISC Zero Documentation](https://dev.risczero.com/)
- [Lean 4 FFI](https://lean-lang.org/lean4/doc/dev/ffi.html)
- [LambdaClass: Lean for ZK](https://blog.lambdaclass.com/lean-4-for-zk-systems/)
