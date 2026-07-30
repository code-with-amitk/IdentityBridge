## On this page

- [How Collector Starts](#how-collector-starts)
- [Where CLI options are defined](#where-cli-options-are-defined)
- [Running on WSL (or Linux) — development](#running-on-wsl-or-linux--development)
- [Running on Windows — production](#running-on-windows--production)

---

# Collector

How the `collector` binary starts, which CLI options exist, and how to run it on **WSL/Linux** (development) vs **Windows** (production).

## How Collector Starts

Entry point: `crates/collector-bin/src/main.rs`.

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

---

## Where CLI options are defined

CLI options are defined in `crates/collector-bin/src/main.rs` with **clap**. Help is printed only when you pass `--help`:

```bash
cargo run -p collector-bin -- --help
cargo run -p collector-bin -- run --help
cargo run -p collector-bin -- install --help
```

On Windows, after building:

```powershell
.\target\release\collector.exe --help
```

---

## Running on WSL (or Linux) — development

WSL is suitable for **building and testing** the HTTP server and config loading. It is **not** a production Collector host (no Windows Service, no AD LDAP/event log against real DCs from this path yet).

### Run in foreground (default)

```bash
# implicit run (no subcommand)
cargo run -p collector-bin

# explicit run
cargo run -p collector-bin -- run

# custom config path
cargo run -p collector-bin -- -c configs/collector.yaml

curl http://127.0.0.1:8080/healthz
curl http://127.0.0.1:8080/
```

`install` and `uninstall` fail on WSL/Linux — Windows Service is Windows-only.

---

## Running on Windows — production

Production Collector runs on **Windows Server** as a Windows Service (or foreground for debugging).

### Build

From **Developer PowerShell** or **cmd** in the repo root:

```powershell
cd C:\path\to\IdentityBridge
cargo build -p collector-bin --release
```

Binary: `target\release\collector.exe`

### Run in foreground (debugging)

```powershell
copy configs\collector.example.yaml configs\collector.yaml
.\target\release\collector.exe
# or
.\target\release\collector.exe run --config configs\collector.yaml
```

Open **on the server** (RDP/console): `http://127.0.0.1:8080/`

Port **8080 is localhost only** — not exposed on the LAN (Phase 1).

### Install as Windows Service

Run **elevated** (Administrator):

```powershell
.\target\release\collector.exe install --config C:\IdentityBridge\configs\collector.yaml
sc.exe start IdentityBridgeCollector
```

Service name: **`IdentityBridgeCollector`**

The installer registers the service to run:

```text
collector.exe service --config <your-config-path>
```

That path is invoked by the Service Control Manager — do not run `service` manually unless debugging SCM integration.

### Stop and remove the service

```powershell
sc.exe stop IdentityBridgeCollector
.\target\release\collector.exe uninstall
```

### Windows CLI subcommands

| Subcommand | Purpose |
|---|---|
| *(none)* or `run` | Foreground process — HTTP server + logs in console |
| `install` | Register Windows Service (Admin) |
| `uninstall` | Remove Windows Service (Admin) |
| `service` | Internal — used by SCM after `install`; loads config and runs until stopped |
