$env:PATH="$env:PYTHON;$env:PYTHON\\Scripts;$env:PATH"

Start-FileDownload "https://static.rust-lang.org/dist/rust-nightly-${env:TARGET}.msi"
Start-Process -FilePath "msiexec.exe" -ArgumentList "/i rust-nightly-$env:TARGET.msi INSTALLDIR=`"$((Get-Location).Path)\rust-nightly-$env:TARGET`" /quiet /qn /norestart" -Wait
$env:PATH="$env:PATH;$((Get-Location).Path)/rust-nightly-$env:TARGET/bin"

$pythonLocation = Invoke-Expression "python -c `"import sys; print(sys.base_prefix)`""
$env:LIBPATH = "$env:LIBPATH; $( Join-Path $pythonLocation "libs" )"
