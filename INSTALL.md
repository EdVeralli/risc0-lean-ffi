# Guia de Instalacion

Este documento explica paso a paso como preparar el entorno para compilar y ejecutar el proyecto.

## Que se necesita instalar

El proyecto usa tres herramientas principales:

| Herramienta | Version | Para que se usa |
|-------------|---------|-----------------|
| **Lean 4** | v4.26.0 | Escribir funciones con pruebas formales (teoremas) |
| **Rust** | stable (1.80+) | Lenguaje del programa principal (host y guest) |
| **RISC Zero** | v1.2.6 | Toolchain del zkVM que genera pruebas ZK (STARK) |

Ademas se necesitan dependencias del sistema para compilar el runtime de Lean y las librerias criptograficas.

---

## Opcion A: Docker (recomendado)

Docker empaqueta todo el entorno. No necesitas instalar Lean, Rust ni RISC Zero en tu maquina.

### Requisito unico

- [Docker Desktop](https://docs.docker.com/get-docker/) instalado y corriendo.

### Instalacion en Windows

1. Instalar [Docker Desktop para Windows](https://docs.docker.com/desktop/install/windows-install/)
   - Requisito: Windows 10/11 con **WSL2** habilitado
   - Durante la instalacion, Docker Desktop pedira activar WSL2 si no esta activo
   - Reiniciar la maquina despues de instalar
2. Abrir Docker Desktop y esperar a que el icono de la barra de tareas diga "Docker Desktop is running"
3. Abrir una consola (**CMD**) y verificar:

```cmd
docker --version
docker run hello-world
```

Si `hello-world` funciona, Docker esta listo.

### Instalacion en macOS / Linux

- macOS: Instalar [Docker Desktop para Mac](https://docs.docker.com/desktop/install/mac-install/)
- Linux: Instalar [Docker Engine](https://docs.docker.com/engine/install/) o Docker Desktop

### Construccion de la imagen

**Metodo 1: `docker build` (ideal)**

Funciona en cualquier sistema operativo (Windows, macOS, Linux):

```bash
docker build -t risc0-lean-ffi .
```

Esto ejecuta el `Dockerfile`, que internamente:

1. Parte de Ubuntu 22.04
2. Instala dependencias del sistema (`apt-get`)
3. Instala Rust via `rustup`
4. Instala Lean 4 via `elan` (el gestor de versiones de Lean)
5. Instala el toolchain de RISC Zero via `rzup`
6. Compila el proyecto Lean (`lake build`)
7. Compila el proyecto Rust + RISC Zero (`cargo build --release`)

> **Nota sobre Windows:** Docker Desktop en Windows usa WSL2 (Linux real), por lo que la red dentro del contenedor funciona bien. `docker build` deberia funcionar sin problemas.

**Metodo 2: Si `docker build` falla por problemas de red**

Si `docker build` falla con errores como "Could not connect to archive.ubuntu.com" o "DNS resolution failed", hay scripts alternativos que construyen la imagen paso a paso:

En **macOS / Linux** (bash):
```bash
chmod +x build-docker.sh
./build-docker.sh
```

En **Windows** (CMD):
```cmd
build-docker.bat
```

Estos scripts hacen lo mismo que el `Dockerfile`, pero paso a paso:
- Usan `docker run` (en vez de `docker build`) para cada paso
- Agregan `--dns 8.8.8.8 --dns 8.8.4.4` para forzar DNS de Google
- Guardan el progreso con `docker commit` despues de cada paso
- Si un paso falla, se puede reintentar sin perder los pasos anteriores

### Ejecucion

Una vez construida la imagen, en cualquier sistema operativo:

```bash
docker run --rm risc0-lean-ffi
```

### Tiempo estimado

La primera construccion tarda **15-20 minutos** (depende de la conexion a internet y la maquina). Las siguientes son mucho mas rapidas gracias al cache.

---

## Opcion B: Instalacion local

> **Importante:** RISC Zero solo publica binarios para **Linux x86_64** y **macOS Apple Silicon (arm64)**. Si tienes una Mac Intel, debes usar Docker (Opcion A).

### Paso 1: Dependencias del sistema

#### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
    curl git build-essential pkg-config libssl-dev \
    cmake clang lld \
    libgmp-dev libuv1-dev libc++-dev libc++abi-dev
```

Que es cada paquete:

| Paquete | Para que |
|---------|----------|
| `curl`, `git` | Descargar herramientas y clonar repos |
| `build-essential` | Compilador C/C++ (gcc, g++, make) |
| `pkg-config`, `libssl-dev` | Necesarios para compilar crates Rust con OpenSSL |
| `cmake`, `clang`, `lld` | Compilador y linker usados por RISC Zero |
| `libgmp-dev` | Aritmetica de precision arbitraria (runtime de Lean) |
| `libuv1-dev` | I/O asincrono (runtime de Lean) |
| `libc++-dev`, `libc++abi-dev` | Runtime de C++ (runtime de Lean) |

#### macOS (Apple Silicon)

```bash
# Xcode Command Line Tools (incluye clang, make, etc.)
xcode-select --install

# Homebrew (si no lo tienes)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Dependencias
brew install cmake gmp libuv
```

### Paso 2: Instalar Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

Esto instala:
- **`rustup`**: gestor de versiones de Rust
- **`cargo`**: gestor de paquetes y build system de Rust
- **`rustc`**: el compilador de Rust

Despues de instalar, activar el PATH:

```bash
source "$HOME/.cargo/env"
```

Verificar:

```bash
rustc --version    # debe mostrar 1.80+ o superior
cargo --version
```

### Paso 3: Instalar Lean 4

```bash
curl -sSf https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh | sh -s -- -y
```

Esto instala:
- **`elan`**: gestor de versiones de Lean (similar a `rustup` para Rust)
- **`lean`**: el compilador/verificador de Lean 4
- **`lake`**: el build system de Lean (similar a `cargo` para Rust)

El archivo `lean_verifier/lean-toolchain` fija la version a `leanprover/lean4:v4.26.0`. Cuando ejecutes `lake build`, elan descargara automaticamente esa version exacta.

Verificar:

```bash
elan --version
lean --version     # debe mostrar leanprover/lean4:v4.26.0
lake --version
```

### Paso 4: Instalar RISC Zero

```bash
# Instalar rzup (gestor de RISC Zero)
curl -L https://risczero.com/install | bash

# Activar PATH
source "$HOME/.bashrc"   # o ~/.zshrc segun tu shell

# Instalar los componentes del toolchain
rzup install cargo-risczero 1.2.6    # plugin de cargo para RISC Zero
rzup install r0vm 1.2.6              # la VM que ejecuta el guest
rzup install rust                     # toolchain de Rust para compilar guests (target risc0)
rzup install cpp                      # toolchain de C++ para RISC Zero
```

Que es cada componente:

| Componente | Para que |
|------------|----------|
| `cargo-risczero` | Plugin de cargo que sabe compilar guests para el zkVM |
| `r0vm` | La maquina virtual RISC-V que ejecuta el guest y genera la prueba |
| `rust` (risc0) | Toolchain de Rust con target `riscv32im` para compilar el guest |
| `cpp` (risc0) | Toolchain de C++ necesario para librerias internas de RISC Zero |

Verificar:

```bash
cargo risczero --version   # debe mostrar 1.2.6
```

### Resumen de versiones

| Herramienta | Version | Comando para verificar |
|-------------|---------|----------------------|
| Rust | 1.80+ | `rustc --version` |
| Lean 4 | v4.26.0 | `lean --version` |
| RISC Zero | 1.2.6 | `cargo risczero --version` |
| elan | cualquiera | `elan --version` |
| lake | (viene con lean) | `lake --version` |

---

## Estructura del proyecto

Para entender que se instalo y donde queda cada cosa:

```
$HOME/
├── .cargo/          # Rust: rustc, cargo, crates compilados
├── .elan/           # Lean 4: lean, lake, toolchains
│   └── toolchains/
│       └── leanprover--lean4---v4.26.0/
│           ├── bin/         # lean, lake, leanc
│           ├── include/     # headers C para FFI
│           └── lib/lean/    # libleanshared.so (runtime)
└── .risc0/          # RISC Zero: rzup, cargo-risczero, r0vm
    └── bin/
```

El archivo `host/build.rs` busca el toolchain de Lean en `$HOME/.elan/toolchains/leanprover--lean4---v4.26.0` para:
1. Encontrar los headers C (`include/`) al compilar el codigo generado por Lean
2. Linkear contra `libleanshared` (`lib/lean/`) que contiene todo el runtime de Lean

---

## Problemas comunes

### "Lean C file not found"

```
Lean C file not found: ../lean_verifier/.lake/build/ir/LeanVerifier/Verifier.c
Run 'cd lean_verifier && lake build' first.
```

**Causa:** No se compilo Lean antes de Rust.
**Solucion:** Ejecutar `cd lean_verifier && lake build && cd ..` antes de `cargo build`.

### "unable to find library -lleanshared"

**Causa:** El toolchain de Lean no esta instalado o la version no coincide.
**Solucion:** Verificar que existe `$HOME/.elan/toolchains/leanprover--lean4---v4.26.0/lib/lean/libleanshared.so`.

### RISC Zero: "rzup: command not found"

**Causa:** El PATH no incluye `~/.risc0/bin`.
**Solucion:** Agregar a tu shell profile (`~/.bashrc` o `~/.zshrc`):
```bash
export PATH="$HOME/.risc0/bin:$PATH"
```

### Docker: DNS/red falla durante build

**Causa:** Docker Desktop (especialmente en Mac Intel) puede tener problemas de red en BuildKit.
**Solucion:** Usar `./build-docker.sh` (macOS/Linux) o `build-docker.bat` (Windows) en lugar de `docker build`. Ver seccion "Metodo 2" arriba.

### Windows: "docker build" no encuentra los archivos

**Causa:** Asegurate de estar en el directorio del proyecto.
**Solucion:**
```cmd
cd C:\ruta\al\proyecto\risc0-lean-ffi
docker build -t risc0-lean-ffi .
```

### Windows: WSL2 no esta habilitado

**Causa:** Docker Desktop requiere WSL2 en Windows.
**Solucion:** Abrir CMD como Administrador (click derecho > "Ejecutar como administrador") y ejecutar:
```cmd
wsl --install
```
Reiniciar la maquina y abrir Docker Desktop de nuevo.
