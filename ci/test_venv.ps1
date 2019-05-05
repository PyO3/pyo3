Set-PSDebug -trace 2

python -m venv venv
.\venv\Scripts\Activate.ps1
python -m pip install setuptools-rust

$examplesDirectory = "examples"

foreach ($example in Get-ChildItem $examplesDirectory) {
    Push-Location $(Join-Path $examplesDirectory $example)
    python setup.py install
    Pop-Location
    if ($LastExitCode -ne 0)
    {
        Throw "${example} failed to build"
    }
}

deactivate
Remove-Item -Recurse venv
