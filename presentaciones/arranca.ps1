# Levanta todo en Windows: base, backend y frontend, cada uno en su ventana.
# Si algo falta, te dice qué y se detiene.
#   powershell -ExecutionPolicy Bypass -File .\arranca.ps1
Set-Location $PSScriptRoot

Write-Host "==> Revisando herramientas"
foreach ($h in "cargo", "node", "npm") {
    if (-not (Get-Command $h -ErrorAction SilentlyContinue)) {
        Write-Host "  falta $h (ve la sección 'En tu computadora' del README)" -ForegroundColor Red
        exit 1
    }
}
Write-Host "  $(cargo --version) · node $(node --version)"

Write-Host "==> Base de datos"
$hayDocker = $false
if (Get-Command docker -ErrorAction SilentlyContinue) {
    docker info *> $null
    $hayDocker = ($LASTEXITCODE -eq 0)
}
if ($hayDocker) {
    docker compose up -d
    Write-Host -NoNewline "  esperando a Postgres"
    for ($i = 0; $i -lt 30; $i++) {
        docker compose exec -T bd pg_isready -U avances -d avances *> $null
        if ($LASTEXITCODE -eq 0) { Write-Host " · lista"; break }
        Write-Host -NoNewline "."
        Start-Sleep -Seconds 1
    }
} else {
    Write-Host "  Docker no está corriendo: uso tu PostgreSQL instalado."
    Write-Host "  Si es la primera vez: psql -U postgres -f preparar-bd.sql"
    Write-Host "  y revisa el puerto de DATABASE_URL en backend\.env (normalmente 5432)."
}

if (-not (Test-Path backend\.env)) {
    Copy-Item .env.example backend\.env
    if (-not $hayDocker) {
        (Get-Content backend\.env) -replace "localhost:5433", "localhost:5432" | Set-Content backend\.env
    }
    Write-Host "==> Creé backend\.env"
}

$plantillas = Get-ChildItem -Path plantilla\* -Include *.pptx, *.potx -ErrorAction SilentlyContinue
$enEnv = Select-String -Path backend\.env -Pattern "^PLANTILLA=.+" -Quiet
if (-not $plantillas -and -not $enEnv) {
    Write-Host "  Ojo: no hay plantilla institucional en plantilla\. Las presentaciones saldrán sin fondos ni logos." -ForegroundColor Yellow
}

if (-not (Test-Path frontend\node_modules)) {
    Write-Host "==> Dependencias del frontend"
    Push-Location frontend
    npm install --no-audit --no-fund
    Pop-Location
}

Write-Host "==> Backend en el puerto 3001 (ventana nueva)"
Start-Process powershell -WorkingDirectory "$PSScriptRoot\backend" -ArgumentList "-NoExit", "-Command", "cargo run"
Write-Host "==> Frontend en el puerto 4200 (ventana nueva)"
Start-Process powershell -WorkingDirectory "$PSScriptRoot\frontend" -ArgumentList "-NoExit", "-Command", "npm start"

Write-Host ""
Write-Host "  Abre http://localhost:4200 cuando el frontend diga que compiló."
Write-Host "  La primera vez el backend tarda varios minutos en compilar."
Write-Host "  Ojo: tiene que ser localhost, no 127.0.0.1, o CORS lo bloquea."
Write-Host "  Para detener todo, cierra las dos ventanas."
