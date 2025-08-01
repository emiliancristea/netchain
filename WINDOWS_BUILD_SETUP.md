# Windows Build Setup for Netchain

To build Netchain on Windows, you need additional tools for native compilation.

## Required Tools

### 1. Visual Studio Build Tools 2022
Download and install Visual Studio Build Tools 2022 with C++ support:
- Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
- During installation, select "C++ build tools" workload

### 2. LLVM/Clang
Download and install LLVM:
- Download from: https://github.com/llvm/llvm-project/releases/latest
- Look for `LLVM-<version>-win64.exe` 
- Install to default location (usually `C:\Program Files\LLVM`)

### 3. Set Environment Variables

After installing LLVM, set the following environment variable:

```powershell
# PowerShell (run as Administrator)
[Environment]::SetEnvironmentVariable("LIBCLANG_PATH", "C:\Program Files\LLVM\bin", "Machine")
```

Or manually add to Windows PATH:
- Open System Properties → Advanced → Environment Variables
- Add `C:\Program Files\LLVM\bin` to PATH
- Add new variable `LIBCLANG_PATH` = `C:\Program Files\LLVM\bin`

### 4. Alternative: Use chocolatey (if available)

```powershell
# Install chocolatey first if not available
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))

# Install LLVM
choco install llvm

# Install Visual Studio Build Tools
choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools"
```

## After Installation

1. **Restart your terminal/PowerShell** to pick up new environment variables
2. **Verify installation**:
   ```powershell
   clang --version
   ```
3. **Try building again**:
   ```powershell
   cargo build --release
   ```

## Alternative Development Options

If you continue to have build issues on Windows, consider:

1. **Windows Subsystem for Linux (WSL2)**:
   ```powershell
   wsl --install
   # Then build inside Ubuntu on WSL2
   ```

2. **Docker Development**:
   ```powershell
   # Use the provided Dockerfile for a containerized build environment
   docker build -t netchain-dev .
   ```

3. **GitHub Codespaces**: Use the cloud development environment with all tools pre-installed.

## Troubleshooting

- **LIBCLANG_PATH not found**: Ensure the environment variable is set correctly and restart your terminal
- **Missing Visual Studio tools**: Make sure C++ build tools are installed, not just Visual Studio IDE
- **Permission errors**: Run PowerShell as Administrator when setting environment variables