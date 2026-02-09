# RISC Zero + Lean 4 FFI

Integracion de **RISC Zero zkVM** con **Lean 4 via FFI** (Foreign Function Interface).

El proyecto demuestra como usar funciones formalmente verificadas en Lean 4 desde un programa Rust que genera pruebas de conocimiento cero (Zero-Knowledge Proofs) con RISC Zero.

## Que hace este proyecto

| Componente | Funcion |
|------------|---------|
| **Lean 4** | Funciones con teoremas probados (verificacion formal) |
| **FFI** | Las funciones Lean se compilan a C y se llaman desde Rust |
| **RISC Zero** | Genera pruebas ZK reales (STARK) dentro de un zkVM |

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

## Documentacion

- **[INSTALL.md](INSTALL.md)** — Como instalar todas las dependencias (Docker o local)
- **[USAGE.md](USAGE.md)** — Como compilar, ejecutar y entender cada parte del proyecto

## Inicio rapido (Docker)

```bash
# Construir la imagen (primera vez ~15-20 min)
docker build -t risc0-lean-ffi .

# Ejecutar
docker run --rm risc0-lean-ffi
```

> Si `docker build` falla por problemas de red (comun en Docker Desktop Mac Intel), usar `./build-docker.sh`. Ver [INSTALL.md](INSTALL.md) para mas detalles.

## Estructura del proyecto

```
risc0-lean-ffi/
├── lean_verifier/                # Proyecto Lean 4
│   ├── LeanVerifier/
│   │   └── Verifier.lean         #   Funciones + teoremas + @[export] FFI
│   ├── Main.lean                 #   Punto de entrada para testing standalone
│   ├── lakefile.toml             #   Configuracion del build system (Lake)
│   └── lean-toolchain            #   Version fija: leanprover/lean4:v4.26.0
├── host/                         # Programa principal (Rust)
│   ├── build.rs                  #   Compila C de Lean y linkea con libleanshared
│   ├── src/main.rs               #   Usa Lean FFI + genera/verifica prueba ZK
│   └── Cargo.toml                #   Dependencias: risc0-zkvm, sha2, hex, cc
├── methods/                      # Guest para el zkVM
│   ├── guest/
│   │   ├── src/main.rs           #   Codigo que corre dentro del zkVM (SHA256)
│   │   └── Cargo.toml            #   Dependencias del guest (risc0-zkvm, sha2)
│   ├── build.rs                  #   Compila guest para RISC-V (risc0-build)
│   └── Cargo.toml                #   Metadata del crate methods
├── Cargo.toml                    # Workspace: host + methods
├── Cargo.lock                    # Versiones exactas de dependencias
├── Dockerfile                    # Build automatizado del entorno completo
├── build-docker.sh               # Build alternativo para macOS/Linux (workaround DNS)
├── build-docker.bat              # Build alternativo para Windows (workaround DNS)
├── .dockerignore                 # Excluye target/, .lake/, .git/ del contexto Docker
├── .gitignore                    # Excluye target/, .lake/, .DS_Store
├── INSTALL.md                    # Guia de instalacion detallada
├── USAGE.md                      # Guia de ejecucion y explicacion del proyecto
└── README.md                     # Este archivo
```

## Teoremas probados en Lean

El archivo `Verifier.lean` contiene funciones con teoremas que Lean verifica en tiempo de compilacion:

```lean
-- Completeness: un commitment correcto SIEMPRE verifica
theorem commitment_completeness (value salt : UInt64) :
    verifyCommitment (createCommitment value salt) value salt = true

-- Soundness: si verifica, el hash ES correcto
theorem commitment_soundness (hash value salt : UInt64) :
    verifyCommitment hash value salt = true →
    hash = createCommitment value salt

-- Binding: no se puede abrir el commitment a un valor diferente
theorem commitment_binding (hash v1 s1 v2 s2 : UInt64) :
    verifyCommitment hash v1 s1 = true →
    verifyCommitment hash v2 s2 = true →
    createCommitment v1 s1 = createCommitment v2 s2

-- Soundness del hash output (para RISC Zero)
theorem hash_output_soundness (h0 h1 h2 h3 e0 e1 e2 e3 : UInt64) :
    verifyHashOutput h0 h1 h2 h3 e0 e1 e2 e3 = true →
    h0 = e0 ∧ h1 = e1 ∧ h2 = e2 ∧ h3 = e3
```

> **Nota academica:** Las funciones hash estan axiomatizadas (`axiom cryptoHash`, `axiom hash_collision_resistant`). En un sistema de produccion se reemplazarian por implementaciones reales con pruebas completas. Los axiomas son intencionales para fines didacticos.

## Output esperado

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
  [Prover] Hash esperado: 0x98ade8a85025b727

  Generando prueba ZK... (esto puede tardar)
  ✓ Prueba generada!

═══ Parte 3: Verificación ═══

  [Verifier] Hash del journal: 0x98ade8a85025b727
  [Verifier] ¿Hashes coinciden?: ✓ SÍ
  [Verifier] ✓ Prueba ZK válida!

═══════════════════════════════════════════════════════════
  RESULTADO: El prover conoce un secreto cuyo hash es:
  0x98ade8a85025b7274eb274a93ae88acd166f73fe60a0d5a7989a79be692e1891

  ✓ Lean FFI: Funciones de verificación formalmente probadas
  ✓ RISC Zero: Prueba ZK generada y verificada
  ⚠ El secreto NUNCA fue revelado
═══════════════════════════════════════════════════════════
```

## Por que esto importa

1. **Lean 4**: Garantiza matematicamente que las funciones de verificacion son correctas
2. **FFI**: Esas funciones se ejecutan en produccion (no son solo especificacion)
3. **RISC Zero**: Genera pruebas ZK reales (STARK) verificables por cualquiera

> "A single bug in a zkVM can compromise billions of dollars"
>
> Con Lean FFI, las funciones criticas tienen **garantias formales**.

## Versiones

| Herramienta | Version |
|-------------|---------|
| Lean 4 | v4.26.0 |
| Rust | 1.80+ (stable) |
| RISC Zero | 1.2.6 |

## Referencias

- [RISC Zero Documentation](https://dev.risczero.com/)
- [Lean 4 FFI](https://lean-lang.org/lean4/doc/dev/ffi.html)
- [LambdaClass: Lean for ZK](https://blog.lambdaclass.com/lean-4-for-zk-systems/)
