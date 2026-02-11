# Guia de Ejecucion

Este documento explica como compilar, ejecutar y entender el proyecto paso a paso.

> **Prerequisito:** Tener el entorno instalado. Ver [INSTALL.md](INSTALL.md).

---

## Compilacion

El proyecto tiene dos partes que se compilan por separado: Lean y Rust.

### Paso 1: Compilar Lean

```bash
cd lean_verifier
lake build
cd ..
```

**Que hace `lake build`:**

1. Lee `lakefile.toml` para saber que compilar (la libreria `LeanVerifier`)
2. Verifica los teoremas: si algun teorema tiene un error logico, la compilacion falla
3. Genera codigo C en `.lake/build/ir/LeanVerifier/Verifier.c`
4. Las funciones marcadas con `@[export]` en Lean quedan disponibles como funciones C

```
Verifier.lean                          Verifier.c (generado)
─────────────                          ───────────────────────
@[export lean_create_commitment]   →   uint64_t lean_create_commitment(uint64_t, uint64_t)
@[export lean_verify_commitment]   →   uint8_t  lean_verify_commitment(uint64_t, uint64_t, uint64_t)
@[export lean_verify_hash_eq]      →   uint8_t  lean_verify_hash_eq(uint64_t, uint64_t)
```

**Output esperado:**

```
Building LeanVerifier
[6/6] Linking lean_verifier
```

### Paso 2: Compilar Rust + RISC Zero

```bash
cargo build --release
```

**Que hace `cargo build --release`:**

1. **`methods/build.rs`** ejecuta `risc0_build::embed_methods()`:
   - Compila `methods/guest/src/main.rs` para la arquitectura RISC-V (`riscv32im`)
   - Genera el ELF del guest (el programa que correra dentro del zkVM)
   - Genera `GUEST_ELF` y `GUEST_ID` como constantes Rust

2. **`host/build.rs`** compila la FFI de Lean:
   - Toma el archivo C generado por Lean (`.lake/build/ir/LeanVerifier/Verifier.c`)
   - Lo compila con `cc` (compilador C) usando los headers de Lean
   - Linkea contra `libleanshared` (el runtime de Lean)

3. **`cargo`** compila `host/src/main.rs`:
   - Linkea con el guest ELF embebido (RISC Zero)
   - Linkea con las funciones C de Lean (FFI)
   - Produce el binario final en `target/release/host`

**Output esperado:**

```
   Compiling methods v0.1.0
   Compiling host v0.1.0
    Finished `release` profile [optimized] target(s) in Xm XXs
```

> **Nota:** La primera compilacion tarda varios minutos porque compila todas las dependencias de RISC Zero (~300 crates). Las siguientes son mucho mas rapidas.

---

## Ejecucion

### Local

```bash
cargo run --release
```

O directamente:

```bash
./target/release/host
```

> **Nota:** Si usas `./target/release/host` directamente, asegurate de que `LD_LIBRARY_PATH` (Linux) o `DYLD_LIBRARY_PATH` (macOS) incluya el directorio del runtime de Lean:
> ```bash
> export LD_LIBRARY_PATH="$HOME/.elan/toolchains/leanprover--lean4---v4.26.0/lib/lean"
> ```

### Docker

```bash
docker run --rm risc0-lean-ffi
```

- `--rm`: elimina el contenedor al terminar (no deja basura)
- La imagen ya tiene todo compilado; solo ejecuta el binario

---

## Roles: Host, Prover, Verifier y Guest

Antes de explicar que hace el programa, es importante entender los roles involucrados.

En un sistema de pruebas de conocimiento cero (ZKP) hay tres roles:

| Rol | Quien es | Que hace |
|-----|----------|----------|
| **Prover** | Quien quiere demostrar algo | "Yo conozco un secreto cuyo hash es X" — sin revelar el secreto |
| **Verifier** | Quien quiere verificar esa afirmacion | Recibe la prueba y la valida criptograficamente. **No conoce el secreto** |
| **Guest** | El programa que corre dentro del zkVM | Ejecuta la computacion (SHA256) de forma aislada. Su ejecucion correcta es lo que la prueba STARK certifica |

### Que es el Host

El **Host** (`host/src/main.rs`) es el programa que corre en tu maquina, **fuera** del zkVM. Es el que orquesta todo.

En esta demo, el host cumple **ambos** roles (prover y verifier) en un mismo programa:

- **Como Prover:** conoce el secreto, lo envia al guest dentro del zkVM, y obtiene la prueba generada (`receipt`)
- **Como Verifier:** recibe el `receipt` y lo verifica con `receipt.verify(GUEST_ID)`

> **En un sistema real**, prover y verifier serian **programas distintos en maquinas distintas**. El prover generaria la prueba y se la enviaria al verifier. El verifier no necesita el secreto — solo necesita el `receipt` y el `GUEST_ID` para validar. Aqui ambos roles estan en el mismo programa para simplificar la demo.

### Que es el Guest

El **Guest** (`methods/guest/src/main.rs`) es el programa que corre **dentro** del zkVM (una maquina virtual RISC-V). Es el codigo cuya ejecucion correcta queda certificada por la prueba STARK.

- Recibe inputs privados con `env::read()` — estos **nunca** salen del zkVM
- Publica outputs con `env::commit()` — estos van al **journal** (output publico)
- No tiene acceso a internet, disco ni nada externo — esta completamente aislado

---

## Que hace el programa

El programa ejecuta tres partes en secuencia. Ahora que sabemos los roles, veamos que hace cada parte:

### Parte 1: Lean FFI - Commitment Scheme

```
═══ Parte 1: Lean FFI - Commitment Scheme ═══

  [Lean] Commitment creado: 13647
         (valor=42, salt=12345)
  [Lean] Verificación correcta: ✓
  [Lean] Verificación con valor falso: ✗
```

**Que esta pasando:**

El **host** llama a funciones escritas en Lean 4 via FFI (Foreign Function Interface):

1. Llama a `lean_create_commitment(42, 12345)` — una funcion escrita en Lean, compilada a C, linkeada en Rust
2. Lean calcula: `(42 * 31 + 12345) % 999983 = 13647`
3. Verifica que `lean_verify_commitment(13647, 42, 12345)` retorna `true` (el commitment es correcto)
4. Verifica que `lean_verify_commitment(13647, 99, 12345)` retorna `false` (valor falso, no pasa)

**Por que importa:** Estas funciones tienen teoremas probados en Lean:
- `commitment_completeness`: un commitment correcto **siempre** verifica
- `commitment_soundness`: si verifica, el hash **es** correcto
- `commitment_binding`: no se puede abrir el commitment a un valor diferente

Lean verifica estos teoremas en tiempo de compilacion. Si el codigo contradice los teoremas, `lake build` falla.

### Parte 2: Prover - Genera la prueba ZK

```
═══ Parte 2: RISC Zero - Prueba ZK ═══

  [Prover] Tengo un secreto de 30 bytes
  [Prover] Hash esperado: 0x98ade8a85025b727

  Generando prueba ZK... (esto puede tardar)
  ✓ Prueba generada!
```

**Que esta pasando:**

El **host** (actuando como **prover**) quiere demostrar que conoce un secreto sin revelarlo:

1. El host tiene un secreto: `"mi_secreto_super_secreto_12345"` (30 bytes)
2. Calcula `SHA256(secreto)` localmente — esto es solo un "me lo anoto" para la demo (ver nota abajo)
3. Envia el secreto al **guest** como input privado
4. El **guest** corre dentro del **zkVM**:
   - Lee el secreto (input privado — jamas sale del zkVM)
   - Calcula `SHA256(secreto)` dentro del zkVM
   - Publica el hash en el **journal** (output publico)
5. RISC Zero genera una **prueba STARK** que certifica que el guest ejecuto correctamente
6. El resultado es un **receipt** = journal (hash publico) + prueba STARK

```
              ┌────────────────────────────────┐
  PROVER      │         zkVM (Guest)           │
  (host)      │                                │
              │                                │
  secreto ──→ │  hash = SHA256(secreto)        │ ──→ receipt:
  (privado)   │  env::commit(&hash)            │     - hash (journal, publico)
              │                                │     - prueba STARK
              └────────────────────────────────┘
                    El secreto MUERE aqui.
                    Solo el hash sale.
```

### Parte 3: Verifier - Valida la prueba ZK

```
═══ Parte 3: Verificación ═══

  [Verifier] Hash del journal: 0x98ade8a85025b727
  [Verifier] ¿Hashes coinciden?: ✓ SÍ
  [Verifier] ✓ Prueba ZK válida!
```

**Que esta pasando:**

El **host** (ahora actuando como **verifier**) valida la prueba:

1. Lee el hash del **journal** (output publico que el guest produjo dentro del zkVM)
2. Llama a `receipt.verify(GUEST_ID)` — **esta es la verificacion ZK real**:
   - Valida la prueba STARK criptograficamente
   - Confirma que se ejecuto el guest correcto (identificado por `GUEST_ID`)
   - Si alguien modifico el guest o el journal, la verificacion falla

> **Nota didactica sobre "¿Hashes coinciden?":** Esta comparacion solo existe porque en esta demo el host es prover **y** verifier a la vez. Como el host conoce el secreto (es el prover), puede calcular SHA256 localmente y compararlo con el hash del journal. **En un sistema real, un verifier NO podria hacer esta comparacion** porque no conoce el secreto — solo recibe el receipt y hace `receipt.verify(GUEST_ID)`. Esta linea esta en la demo solo para mostrar que el guest calculo el hash correctamente, pero no es parte del protocolo ZK.

### Resultado final

```
═══════════════════════════════════════════════════════════
  RESULTADO: El prover conoce un secreto cuyo hash es:
  0x98ade8a85025b7274eb274a93ae88acd166f73fe60a0d5a7989a79be692e1891

  ✓ Lean FFI: Funciones de verificación formalmente probadas
  ✓ RISC Zero: Prueba ZK generada y verificada
  ⚠ El secreto NUNCA fue revelado
═══════════════════════════════════════════════════════════
```

## Flujo completo del proyecto

```
┌─────────────────────────────────────────────────────────────────────┐
│                          COMPILACION                                │
│                                                                     │
│  Verifier.lean ──lake build──→ Verifier.c ──cc──→ lean_verifier.a   │
│                                                        │            │
│  guest/main.rs ──risc0-build──→ GUEST_ELF              │            │
│                                      │                 │            │
│  host/main.rs ──cargo───────────────→ host (binario) ←─┘            │
│                                        + libleanshared (dinamica)   │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                        EJECUCION (host)                             │
│                                                                     │
│  Parte 1 — Lean FFI:                                                │
│    Host llama funciones Lean via FFI (commitment scheme)            │
│    Estas funciones tienen teoremas verificados por Lean             │
│                                                                     │
│  Parte 2 — Prover (genera la prueba):                               │
│    1. Host (como prover) envia el secreto al Guest                  │
│    2. Guest corre dentro del zkVM:                                  │
│       - Lee el secreto (input privado)                              │
│       - Calcula SHA256(secreto)                                     │
│       - Publica el hash en el journal (output publico)              │
│    3. RISC Zero genera la prueba STARK                              │
│    4. El resultado es un receipt = journal + prueba                 │
│                                                                     │
│  Parte 3 — Verifier (valida la prueba):                             │
│    1. Host (como verifier) lee el hash del journal                  │
│    2. "¿Hashes coinciden?" → solo didactico (*)                     │
│    3. receipt.verify(GUEST_ID) → ESTA es la verificacion real       │
│       - Valida la prueba STARK criptograficamente                   │
│       - Confirma que se ejecuto el guest correcto (GUEST_ID)        │
│       - Si alguien modifico el guest o el journal, FALLA            │
│    4. Resultado: el prover conoce un secreto valido                 │
│       sin haberlo revelado                                          │
└─────────────────────────────────────────────────────────────────────┘

(*) "¿Hashes coinciden?" compara el hash local del host con el del
    journal. Esto solo es posible porque el host es prover Y verifier.
    En un sistema real, el verifier NO conoce el secreto, por lo tanto
    no puede calcular el hash por su cuenta. Solo hace
    receipt.verify(GUEST_ID).

En un sistema real:
  Prover ──genera receipt──→ lo envia ──→ Verifier
  El verifier solo necesita el receipt y el GUEST_ID para validar.
  No necesita el secreto ni acceso al zkVM.
```

---

## Archivos clave y que hace cada uno

| Archivo | Lenguaje | Que hace |
|---------|----------|----------|
| `lean_verifier/LeanVerifier/Verifier.lean` | Lean 4 | Define funciones (`createCommitment`, `verifyCommitment`) con teoremas y las exporta a C con `@[export]` |
| `lean_verifier/lean-toolchain` | config | Fija la version de Lean a `v4.26.0` |
| `lean_verifier/lakefile.toml` | config | Configuracion del build system de Lean |
| `host/build.rs` | Rust | Script de compilacion: compila el C de Lean y linkea con `libleanshared` |
| `host/src/main.rs` | Rust | Programa principal: usa Lean FFI + genera y verifica prueba ZK |
| `methods/guest/src/main.rs` | Rust | Codigo que corre dentro del zkVM: calcula SHA256 y publica el hash |
| `methods/build.rs` | Rust | Compila el guest para RISC-V y genera `GUEST_ELF` |
| `Cargo.toml` | config | Workspace de Rust: define `host` y `methods` como miembros |
| `Dockerfile` | Docker | Build automatizado del entorno completo |
| `build-docker.sh` | Bash | Build alternativo paso a paso para macOS/Linux (workaround DNS) |
| `build-docker.bat` | Batch | Build alternativo paso a paso para Windows (workaround DNS) |

---

## Modificar el proyecto

### Cambiar el secreto

En `host/src/main.rs`, linea 70:
```rust
let secret = b"mi_secreto_super_secreto_12345";
```
Cambiar por cualquier string. El hash del output cambiara.

### Cambiar las funciones Lean

1. Editar `lean_verifier/LeanVerifier/Verifier.lean`
2. Recompilar Lean: `cd lean_verifier && lake build && cd ..`
3. Recompilar Rust: `cargo build --release`

Si agregas una funcion nueva con `@[export]`, debes declararla en `host/src/main.rs` en el bloque `extern "C"`.

### Cambiar el guest (lo que se prueba con ZK)

Editar `methods/guest/src/main.rs`. El guest puede hacer cualquier computacion; lo importante es:
- `env::read()` para leer inputs privados
- `env::commit()` para publicar outputs (van al journal)
- Todo lo que no se haga `commit` queda privado
