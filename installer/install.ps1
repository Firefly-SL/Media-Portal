$ProgressPreference = 'SilentlyContinue'

# Directories
$installDir = "$env:LOCALAPPDATA\MPortal"
$configDir = "$env:USERPROFILE\Documents\MPortal"

$repo = "Firefly-SL/Media-Portal"
$configUrl = "https://raw.githubusercontent.com/Firefly-SL/Media-Portal/refs/heads/main/installer/config.toml"
$apiUrl = "https://api.github.com/repos/$repo/releases"
$headers = @{"User-Agent" = "PowerShell-Installer"}

# create directories if needed
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
New-Item -ItemType Directory -Force -Path $configDir | Out-Null

# get latest release info
try {
    $releases = Invoke-RestMethod -Uri $apiUrl -Headers $headers -UseBasicParsing
    $releases = $releases | Where-Object { $_.draft -eq $false }
    $release = $releases | Sort-Object {[datetime]$_.published_at} -Descending | Select-Object -First 1
} catch {
    Write-Host "Error: Could not reach GitHub."
	Read-Host "Press Enter to exit"
    exit 1
}

$latestVersion = $release.tag_name

# function to stop process, download, and track status
function Install-Binary {
    param ([string]$binaryName)

    # unique version file for each binary
    $versionFile = Join-Path $installDir "$binaryName-version.txt"
    $exePath = Join-Path $installDir "$binaryName.exe"
    
    # trim() to ensure no hidden whitespace or newlines
    $oldVersion = if (Test-Path $versionFile) { (Get-Content $versionFile -Raw).Trim() } else { $null }
    $latestVersionClean = $latestVersion.Trim()
    $fileExists = Test-Path $exePath

    if (-not $fileExists -or ($oldVersion -ne $latestVersionClean)) {
        
        $expectedAssetName = "$binaryName-windows-x64.exe"
        $asset = $release.assets | Where-Object { $_.name -eq $expectedAssetName } | Select-Object -First 1

        if ($asset) {
            # check for exactly the binary name (mportal or mportal-daemon)
            $proc = Get-Process -Name $binaryName -ErrorAction SilentlyContinue 
            if ($proc) { 
                Stop-Process -Name $binaryName -Force
                Start-Sleep -Seconds 1 
            }

            # download the file
            Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $exePath
            $latestVersionClean | Out-File -Encoding ASCII $versionFile
            
            if (-not $fileExists) {
                Write-Host "Installed $binaryName version $latestVersionClean"
            } else {
                Write-Host "Updated $binaryName from $oldVersion -> $latestVersionClean"
            }
        } else {
            Write-Host "Warning: could not find a release asset for $expectedAssetName"
        }
    } else {
        Write-Host "$binaryName is up to date."
    }
}

# run installation/updates
Install-Binary -binaryName "mportal"
Install-Binary -binaryName "mportal-daemon"

# config.toml
$configPath = Join-Path $configDir "config.toml"
if (-Not (Test-Path $configPath)) {
    Invoke-WebRequest -Uri $configUrl -OutFile $configPath
}

# ffmpeg
if (-Not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    winget install --id=Gyan.FFmpeg -e --silent --accept-source-agreements --accept-package-agreements | Out-Null
}

# PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if (-Not ($userPath.Split(';') -contains $installDir)) {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    $env:PATH = "$env:PATH;$installDir"
}

# autostart
$startupFolder = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup"
$shortcutPath = Join-Path $startupFolder "mportal-daemon.lnk"
if (-Not (Test-Path $shortcutPath)) {
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($shortcutPath)
    $Shortcut.TargetPath = Join-Path $installDir "mportal-daemon.exe"
    $Shortcut.WorkingDirectory = $installDir
    $Shortcut.Save()
}

# RESTART THE DAEMON
if (-not (Get-Process "mportal-daemon" -ErrorAction SilentlyContinue)) {
    Start-Process -FilePath (Join-Path $installDir "mportal-daemon.exe") -WorkingDirectory $installDir -WindowStyle Hidden
}

Read-Host "Press Enter to exit"