# End-to-end form interaction regression for the real multi-process renderer.
# Builds the child binary first so the browser test cannot launch stale code.

$ErrorActionPreference = "Stop"
$ProjectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$GuardBin = Join-Path $ProjectRoot "target\test-guard.exe"
$GuardSource = Join-Path $ProjectRoot "scripts\test-guard.rs"

Push-Location $ProjectRoot
try {
    if (-not (Test-Path -LiteralPath $GuardBin)) {
        rustc -O $GuardSource -o $GuardBin
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }

    & $GuardBin --per-proc-mem 4294967296 --total-mem 8589934592 --time-limit 900 -- cargo build -p zero-renderer
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    & $GuardBin --per-proc-mem 4294967296 --total-mem 8589934592 --time-limit 1200 -- `
        cargo test -p zero-browser form_fixture_physical_clicks_reach_controls_at_windows_scale_factors -- `
        --nocapture --test-threads=1
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
