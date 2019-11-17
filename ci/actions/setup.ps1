$env:PATH="$env:PYTHON;$env:PYTHON\\Scripts;$env:PATH"
$pythonLocation = Invoke-Expression "python -c `"import sys; print(sys.base_prefix)`""
$env:LIBPATH = "$env:LIBPATH; $( Join-Path $pythonLocation "libs" )"
