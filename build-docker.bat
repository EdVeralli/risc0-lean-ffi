@echo off
REM Script para construir la imagen Docker paso a paso en Windows
REM Equivalente a build-docker.sh para CMD/PowerShell
REM Usa --dns 8.8.8.8 para resolver problemas de DNS

setlocal

set IMAGE_NAME=risc0-lean-ffi
set BASE_CONTAINER=risc0-build
set DNS_FLAG=--dns 8.8.8.8 --dns 8.8.4.4
set FULL_PATH=/root/.cargo/bin:/root/.elan/bin:/root/.risc0/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

echo === Construyendo imagen %IMAGE_NAME% ===
echo.

REM Limpiar contenedor previo si existe
docker rm -f %BASE_CONTAINER% 2>nul

REM Paso 1: Crear contenedor base con dependencias del sistema
echo ^>^>^> Paso 1/6: Instalando dependencias del sistema...
docker run --name %BASE_CONTAINER% %DNS_FLAG% -e DEBIAN_FRONTEND=noninteractive ubuntu:22.04 bash -c "apt-get update && apt-get install -y curl git build-essential pkg-config libssl-dev cmake clang lld libgmp-dev libuv1-dev libc++-dev libc++abi-dev && rm -rf /var/lib/apt/lists/*"
if %ERRORLEVEL% neq 0 goto :error
docker commit %BASE_CONTAINER% %IMAGE_NAME%:step1
docker rm %BASE_CONTAINER%
echo ^>^>^> Paso 1 completado.

REM Paso 2: Instalar Rust
echo ^>^>^> Paso 2/6: Instalando Rust...
docker run --name %BASE_CONTAINER% %DNS_FLAG% -e HOME=/root -e PATH="%FULL_PATH%" %IMAGE_NAME%:step1 bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
if %ERRORLEVEL% neq 0 goto :error
docker commit %BASE_CONTAINER% %IMAGE_NAME%:step2
docker rm %BASE_CONTAINER%
echo ^>^>^> Paso 2 completado.

REM Paso 3: Instalar Lean 4 (elan)
echo ^>^>^> Paso 3/6: Instalando Lean 4...
docker run --name %BASE_CONTAINER% %DNS_FLAG% -e HOME=/root -e PATH="%FULL_PATH%" %IMAGE_NAME%:step2 bash -c "curl -sSf https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh | sh -s -- -y --default-toolchain leanprover/lean4:v4.26.0"
if %ERRORLEVEL% neq 0 goto :error
docker commit %BASE_CONTAINER% %IMAGE_NAME%:step3
docker rm %BASE_CONTAINER%
echo ^>^>^> Paso 3 completado.

REM Paso 4: Instalar RISC Zero toolchain
echo ^>^>^> Paso 4/6: Instalando RISC Zero toolchain...
docker run --name %BASE_CONTAINER% %DNS_FLAG% -e HOME=/root -e PATH="%FULL_PATH%" %IMAGE_NAME%:step3 bash -c "curl -L https://risczero.com/install | bash && /root/.risc0/bin/rzup install cargo-risczero 1.2.6 && /root/.risc0/bin/rzup install r0vm 1.2.6 && /root/.risc0/bin/rzup install rust && /root/.risc0/bin/rzup install cpp"
if %ERRORLEVEL% neq 0 goto :error
docker commit %BASE_CONTAINER% %IMAGE_NAME%:step4
docker rm %BASE_CONTAINER%
echo ^>^>^> Paso 4 completado.

REM Paso 5: Copiar proyecto y compilar Lean
echo ^>^>^> Paso 5/6: Compilando Lean...
docker run --name %BASE_CONTAINER% %DNS_FLAG% -e HOME=/root -e PATH="%FULL_PATH%" -v "%cd%":/src:ro %IMAGE_NAME%:step4 bash -c "mkdir -p /app && cp -r /src/* /app/ && cd /app/lean_verifier && lake build"
if %ERRORLEVEL% neq 0 goto :error
docker commit %BASE_CONTAINER% %IMAGE_NAME%:step5
docker rm %BASE_CONTAINER%
echo ^>^>^> Paso 5 completado.

REM Paso 6: Compilar Rust + RISC Zero
echo ^>^>^> Paso 6/6: Compilando Rust + RISC Zero...
docker run --name %BASE_CONTAINER% %DNS_FLAG% -e HOME=/root -e PATH="%FULL_PATH%" -e LD_LIBRARY_PATH="/root/.elan/toolchains/leanprover--lean4---v4.26.0/lib/lean" -w /app %IMAGE_NAME%:step5 bash -c "cargo build --release"
if %ERRORLEVEL% neq 0 goto :error
docker commit --change "ENV DEBIAN_FRONTEND=noninteractive" --change "ENV HOME=/root" --change "ENV PATH=%FULL_PATH%" --change "ENV LD_LIBRARY_PATH=/root/.elan/toolchains/leanprover--lean4---v4.26.0/lib/lean" --change "WORKDIR /app" --change "CMD [\"cargo\", \"run\", \"--release\"]" %BASE_CONTAINER% %IMAGE_NAME%
docker rm %BASE_CONTAINER%
echo ^>^>^> Paso 6 completado.

REM Limpiar imagenes intermedias
echo.
echo ^>^>^> Limpiando imagenes intermedias...
for %%s in (step1 step2 step3 step4 step5) do (
    docker rmi %IMAGE_NAME%:%%s 2>nul
)

echo.
echo === Imagen %IMAGE_NAME% construida exitosamente ===
echo.
echo Para ejecutar:
echo   docker run --rm %IMAGE_NAME%
goto :end

:error
echo.
echo === ERROR: Fallo en la construccion. ===
echo Puedes reintentar ejecutando el script de nuevo.
echo Los pasos completados no se repiten.
exit /b 1

:end
endlocal
