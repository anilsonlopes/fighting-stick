[CmdletBinding()]
param(
    [switch]$SkipBuildTools,
    [switch]$SkipProjectCheck
)

$ErrorActionPreference = 'Stop'

function Write-Step([string]$Message) {
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Install-WingetPackage {
    param(
        [Parameter(Mandatory)][string]$Id,
        [string]$Override
    )

    & winget list --id $Id --exact --accept-source-agreements | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Já instalado: $Id" -ForegroundColor DarkGray
        return
    }

    $arguments = @(
        'install', '--id', $Id, '--exact',
        '--accept-package-agreements', '--accept-source-agreements',
        '--disable-interactivity'
    )
    if ($Override) {
        $arguments += @('--override', $Override)
    }

    & winget @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "A instalação de '$Id' falhou (código $LASTEXITCODE)."
    }
}

if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    throw 'winget não foi encontrado. Atualize/instale o App Installer pela Microsoft Store e tente novamente.'
}

# Instaladores atualizam o PATH persistido, mas não o processo PowerShell atual.
$machinePath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$env:Path = "$userPath;$machinePath"

Write-Step 'Instalando Rust stable e Cargo'
if (-not (Test-Path -LiteralPath "$env:USERPROFILE\.cargo\bin\rustup.exe")) {
    Install-WingetPackage -Id 'Rustlang.Rustup'
}

# rustup e cargo são instalados juntos. A chamada explícita garante o toolchain MSVC.
$cargoBin = "$env:USERPROFILE\.cargo\bin"
if ($env:Path -notlike "*$cargoBin*") {
    $env:Path = "$cargoBin;$env:Path"
}
& "$cargoBin\rustup.exe" default stable-msvc
if ($LASTEXITCODE -ne 0) { throw 'Não foi possível configurar o Rust stable-msvc.' }

Write-Step 'Instalando CMake'
if (-not (Get-Command cmake -ErrorAction SilentlyContinue)) {
    Install-WingetPackage -Id 'Kitware.CMake'
}

if (-not $SkipBuildTools) {
    Write-Step 'Instalando Visual Studio Build Tools (compilador C++ e Windows SDK)'
    $vsOverride = '--wait --passive --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'
    Install-WingetPackage -Id 'Microsoft.VisualStudio.2022.BuildTools' -Override $vsOverride
}

# Atualiza novamente para incluir alterações feitas nesta execução.
$machinePath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$env:Path = "$cargoBin;$userPath;$machinePath"

Write-Step 'Verificando ferramentas'
& rustc --version
& cargo --version
& cmake --version | Select-Object -First 1

if (-not $SkipProjectCheck) {
    Write-Step 'Baixando dependências e executando os testes do projeto'
    Push-Location -LiteralPath $PSScriptRoot
    try {
        & cargo test --workspace
        if ($LASTEXITCODE -ne 0) { throw 'Os testes do projeto falharam.' }
    }
    finally {
        Pop-Location
    }
}

Write-Host "`nSetup concluído. Para abrir o app:" -ForegroundColor Green
Write-Host 'cargo run -p fighting-stick-desktop --release'
