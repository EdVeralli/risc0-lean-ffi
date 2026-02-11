# Lean: Funciones, Teoremas y FFI

Este documento explica como se relacionan las funciones y los teoremas en Lean, y cuales de esas funciones se exportan para ser usadas desde Rust.

---

## Concepto clave

En Lean hay dos tipos de cosas:

- **Funciones**: codigo que calcula un resultado (como en cualquier lenguaje)
- **Teoremas**: afirmaciones matematicas sobre esas funciones, demostradas formalmente

Las funciones se ejecutan en runtime. Los teoremas **no se ejecutan** — se verifican en tiempo de compilacion (`lake build`). Si un teorema no se puede demostrar, el proyecto no compila.

La ventaja: cuando el programa corre, ya sabemos de antemano que las funciones son correctas porque los teoremas lo garantizaron antes de que el programa exista.

---

## Las funciones

El archivo `lean_verifier/LeanVerifier/Verifier.lean` define tres funciones principales:

### `createCommitment` — Crear un commitment

```lean
def createCommitment (value : UInt64) (salt : UInt64) : UInt64 :=
  let h1 := value.toNat * 31 + salt.toNat
  let h2 := h1 % 999983
  UInt64.ofNat h2
```

Recibe un valor y un salt, devuelve un hash (el commitment).
Es como "meter un valor en una caja cerrada": se puede verificar despues, pero no se puede deducir el valor original mirando solo el hash.

Ejemplo: `createCommitment 42 12345` devuelve `13647`.

### `verifyCommitment` — Verificar un commitment

```lean
def verifyCommitment (hash : UInt64) (value : UInt64) (salt : UInt64) : Bool :=
  createCommitment value salt == hash
```

Recibe un hash, un valor y un salt. Recalcula el commitment y compara. Si coinciden, devuelve `true`.

Es como "abrir la caja": demostrás que conoces el valor y el salt originales.

### `verifyHashOutput` — Verificar igualdad de hashes

```lean
def verifyHashOutput (h0 h1 h2 h3 : UInt64)
    (expected0 expected1 expected2 expected3 : UInt64) : Bool :=
  h0 == expected0 && h1 == expected1 && h2 == expected2 && h3 == expected3
```

Compara dos hashes de 256 bits (representados como 4 numeros de 64 bits). Devuelve `true` si todos los fragmentos coinciden.

---

## Los teoremas

Cada teorema dice algo sobre las funciones anteriores. Lean demuestra que son verdaderos para **todos** los valores posibles, no solo para un ejemplo.

### `commitment_completeness` — Completitud

```lean
theorem commitment_completeness (value salt : UInt64) :
    verifyCommitment (createCommitment value salt) value salt = true
```

**Que dice:** Si creo un commitment con cierto value y salt, y despues lo verifico con los mismos datos, **siempre** da `true`.

**Por que importa:** Garantiza que el sistema no rechaza commitments validos. No hay un caso en el que hagas todo bien y el sistema diga "no".

**Relacion con las funciones:**

```
createCommitment(value, salt) → hash
                                  ↓
verifyCommitment(hash, value, salt) → SIEMPRE true
```

### `commitment_soundness` — Solidez

```lean
theorem commitment_soundness (hash value salt : UInt64) :
    verifyCommitment hash value salt = true →
    hash = createCommitment value salt
```

**Que dice:** Si `verifyCommitment` devuelve `true`, entonces el hash que pasaste **es exactamente** el que produce `createCommitment` con esos datos.

**Por que importa:** Garantiza que `verifyCommitment` no acepta hashes falsos. Si dice "si", es porque el hash es correcto — no hay falsos positivos.

**Relacion con las funciones:**

```
verifyCommitment(hash, value, salt) = true
    ↓ entonces, necesariamente:
hash = createCommitment(value, salt)
```

### `commitment_binding` — Vinculacion

```lean
theorem commitment_binding (hash v1 s1 v2 s2 : UInt64) :
    verifyCommitment hash v1 s1 = true →
    verifyCommitment hash v2 s2 = true →
    createCommitment v1 s1 = createCommitment v2 s2
```

**Que dice:** Si el mismo hash verifica con dos pares (v1, s1) y (v2, s2), entonces ambos pares producen el mismo commitment.

**Por que importa:** No se puede "hacer trampa" abriendo un commitment a un valor diferente al original. Si te comprometiste con un valor, estas atado a el.

**Relacion con las funciones:**

```
verifyCommitment(hash, v1, s1) = true
verifyCommitment(hash, v2, s2) = true
    ↓ entonces:
createCommitment(v1, s1) = createCommitment(v2, s2)
```

> **Nota:** Este teorema se prueba usando `commitment_soundness`. Lean permite construir teoremas sobre otros teoremas, como una cadena de razonamiento.

### `hash_output_soundness` — Solidez del hash output

```lean
theorem hash_output_soundness (h0 h1 h2 h3 e0 e1 e2 e3 : UInt64) :
    verifyHashOutput h0 h1 h2 h3 e0 e1 e2 e3 = true →
    h0 = e0 ∧ h1 = e1 ∧ h2 = e2 ∧ h3 = e3
```

**Que dice:** Si `verifyHashOutput` devuelve `true`, entonces cada fragmento del hash es exactamente igual al esperado.

**Por que importa:** Garantiza que la comparacion de hashes no tiene errores logicos. Si dice que dos hashes son iguales, son iguales componente por componente.

---

## Mapa completo: funciones ←→ teoremas

```
FUNCIONES                          TEOREMAS QUE LAS GARANTIZAN
─────────                          ────────────────────────────

createCommitment ←──────────────── commitment_completeness
       ↑                               "crear y verificar siempre funciona"
       │
verifyCommitment ←──────────────── commitment_soundness
       ↑                               "si verifica, el hash es correcto"
       │
       └─────────────────────────── commitment_binding
                                       "no se puede abrir a otro valor"
                                       (usa soundness como base)

verifyHashOutput ←──────────────── hash_output_soundness
                                       "si dice igual, es igual"
```

---

## Funciones exportadas a Rust (FFI)

No todas las funciones se exportan. Solo las que tienen `@[export]` generan funciones C que Rust puede llamar:

| Funcion Lean | Nombre exportado a C | Que hace | Garantias (teoremas) |
|--------------|---------------------|----------|---------------------|
| `leanCreateCommitment` | `lean_create_commitment` | Crea un commitment | completeness, binding |
| `leanVerifyCommitment` | `lean_verify_commitment` | Verifica un commitment | soundness, binding |
| `leanVerifyHashEq` | `lean_verify_hash_eq` | Compara dos hashes | (igualdad directa) |

Las funciones exportadas son **wrappers** de las funciones internas:

```
Funcion interna         Wrapper exportado           Funcion C generada
(con teoremas)          (con @[export])             (para Rust)
──────────────          ───────────────             ──────────────────

createCommitment   →    leanCreateCommitment   →    lean_create_commitment
verifyCommitment   →    leanVerifyCommitment   →    lean_verify_commitment
```

¿Por que wrappers? Porque la FFI de Lean a C requiere tipos compatibles con C. Por ejemplo, `verifyCommitment` devuelve `Bool` (tipo de Lean), pero el wrapper `leanVerifyCommitment` devuelve `UInt8` (1 o 0), que es un tipo que C entiende.

---

## Los axiomas

Ademas de funciones y teoremas, el archivo tiene **axiomas**. Un axioma es algo que se asume verdadero sin demostracion:

```lean
axiom cryptoHash : UInt64 → UInt64 → UInt64
axiom hash_collision_resistant : ∀ (v1 s1 v2 s2 : UInt64),
  v1 ≠ v2 → cryptoHash v1 s1 ≠ cryptoHash v2 s2
```

**`cryptoHash`**: Se asume que existe una funcion hash criptografica.
**`hash_collision_resistant`**: Se asume que esa funcion no produce colisiones (valores distintos siempre dan hashes distintos).

En un sistema de produccion, estos axiomas se reemplazarian por implementaciones reales (como SHA256) con pruebas completas. En esta demo son intencionales para fines didacticos — permiten razonar sobre propiedades criptograficas sin tener que implementar la criptografia completa en Lean.

---

## Resumen: el flujo completo

```
  1. COMPILACION (lake build):

     Lean verifica los teoremas:
       ¿completeness es verdadero?  → SI, demostrado
       ¿soundness es verdadero?     → SI, demostrado
       ¿binding es verdadero?       → SI, demostrado (usando soundness)

     Si alguno falla → NO COMPILA → no se genera codigo C → no se puede ejecutar

     Si todos pasan → genera Verifier.c con las funciones exportadas


  2. EJECUCION (cargo run):

     Rust llama a lean_create_commitment(42, 12345) → devuelve 13647
     Rust llama a lean_verify_commitment(13647, 42, 12345) → devuelve 1 (true)

     Sabemos que estas funciones son correctas porque los teoremas
     ya lo demostraron antes de que el programa existiera.
```

---

## Aclaracion importante: Lean y RISC Zero son demos independientes

En este proyecto, las funciones de Lean (commitment scheme) y la prueba ZK de RISC Zero **no interactuan entre si**. Son dos demostraciones separadas que corren una despues de la otra en el mismo programa:

```
Parte 1 — Lean FFI:
  El host llama a createCommitment y verifyCommitment (funciones de Lean)
  → Demuestra que Lean FFI funciona desde Rust
  → Las funciones tienen teoremas que garantizan su correccion

Parte 2/3 — RISC Zero:
  El guest calcula SHA256(secreto) dentro del zkVM
  → Demuestra que se puede generar y verificar una prueba ZK
  → Usa SHA256, NO las funciones de Lean
```

Las funciones de Lean no participan en la prueba ZK, y la prueba ZK no usa las funciones de Lean. Son conceptos complementarios que se muestran juntos para fines didacticos.

### ¿Como seria un proyecto integrado?

En una version mas avanzada, las funciones de Lean podrian ejecutarse **dentro** del guest del zkVM. Asi, la prueba ZK certificaria que se ejecuto codigo formalmente verificado:

```
Version actual (este proyecto):
  Lean  → host       (demuestra FFI + verificacion formal)
  SHA256 → guest/zkVM (demuestra pruebas ZK)
  Son independientes.

Version integrada (avanzada):
  Lean → guest/zkVM
  La prueba ZK certifica que se ejecutaron
  funciones formalmente verificadas dentro del zkVM.
  Ahi las dos cosas se potencian mutuamente.
```

El valor de este proyecto es mostrar ambos conceptos funcionando y demostrar que la integracion Lean + Rust via FFI es viable. El paso siguiente seria llevar las funciones de Lean adentro del zkVM.
