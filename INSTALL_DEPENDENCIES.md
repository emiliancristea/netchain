# Installing Dependencies for Netchain Build on Windows

The build failed because RocksDB needs libclang for binding generation. Here are several solutions:

## Option 1: Official LLVM Release (Recommended)

1. **Download LLVM**:
   - Go to: https://github.com/llvm/llvm-project/releases/latest
   - Download `LLVM-19.1.5-win64.exe` (latest version)
   - Run the installer and install to `C:\Program Files\LLVM` (default)

2. **Set Environment Variables**:
   ```powershell
   # Run PowerShell as Administrator
   [Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "Machine")
   [Environment]::SetEnvironmentVariable("PATH", $env:PATH + ";C:\Program Files\LLVM\bin", "Machine")
   ```

3. **Restart your terminal** and try building again.

## Option 2: Using Chocolatey (If Available)

```powershell
# Install chocolatey if not available
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))

# Install LLVM
choco install llvm

# Restart terminal after installation
```

## Option 3: Manual Download and Setup

1. **Download pre-built LLVM**:
   - Visit: https://github.com/c3lang/win-llvm/releases
   - Download the latest release (e.g., `llvm_19_1_5.7z`)
   - Extract to `C:\llvm`

2. **Set environment variables**:
   ```powershell
   [Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\llvm\bin", "Machine")
   [Environment]::SetEnvironmentVariable("PATH", $env:PATH + ";C:\llvm\bin", "Machine")
   ```

## After Installation

1. **Restart your PowerShell terminal**
2. **Verify installation**:
   ```powershell
   clang --version
   ```
3. **Try building again**:
   ```powershell
   cargo build --release
   ```

## Alternative: Skip RocksDB for Testing

If you want to test the basic setup without RocksDB, you could temporarily modify the dependencies, but this is not recommended for production use.

## Troubleshooting

- **"clang command not found"**: Environment variables not set correctly, restart terminal
- **"LIBCLANG_PATH not found"**: Make sure the path points to the directory containing `libclang.dll`
- **Permission errors**: Run PowerShell as Administrator when setting environment variables

The recommended option is **Option 1** (Official LLVM Release) as it's the most stable and well-supported approach.