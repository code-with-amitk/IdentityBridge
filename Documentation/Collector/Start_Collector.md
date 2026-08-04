On this page
- [Collector code](#collector-code)
- [Ubuntu WSL — development](#ubuntu-wsl--development)
- [Windows install (production)](#windows-install-production)
    - [Build collector.exe](#build-collectorexe)
    - [Run (foreground)](#run-foreground)
    - [Install as Windows Service](#install-collectorexe-as-windows-service)

---

## Collector

### Code

`crates/collector-bin/src/main.rs`.

```
1. Parse CLI options (clap): run | install | uninstall  [service on Windows]
    - install
        Read config (--config, default configs/collector.yaml)
        Call ColService::install_service()
        Exit
    - uninstall
        Call ColService::uninstall_service()
        Exit
    - service  [Windows only, SCM]
        Load config → ColService::run() → HTTP until stopped
    - run / (default)
        Load config → init_tracing() → collector::run().await → HTTP server
2. HTTP server listens on http.bind (default 127.0.0.1:8080)
```

### Build on Windows (recommended)

1. Copy the repo to a native path, e.g. C:\IdentityBridge (not \\wsl.localhost\...).
2. Install Rust for Windows.
3. Install Build Tools for Visual Studio with the “Desktop development with C++” workload (provides link.exe).
```bash
In PowerShell:
cd C:\IdentityBridge
cargo build -p collector-bin --release
dir target\release\collector.exe
```
4. Double click `collector.exe` to install