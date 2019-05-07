Set-PSDebug -trace 2

function Invoke-Call
{
    param ([scriptblock]$ScriptBlock)
    & @ScriptBlock
    if ($LastExitCode -ne 0)
    {
        exit $LastExitCode
    }
}

Invoke-Call { cargo test --verbose }

$examplesDirectory = "examples"

foreach ($example in Get-ChildItem $examplesDirectory)
{
    Push-Location $( Join-Path $examplesDirectory $example )
    Invoke-Call { tox -c "tox.ini" -e py }
    Pop-Location
}
