IF NOT "%1"=="r" (
    cargo build
    target\debug\engine_wgpu.exe
) ELSE (
    cargo build --release
    target\release\engine_wgpu.exe
)