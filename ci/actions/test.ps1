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

Invoke-Call { cargo test --verbose --features="num-bigint num-complex" }

foreach ($example in Get-ChildItem -dir "examples")
{
    Set-Location $example
    Invoke-Call { tox -c "tox.ini" -e py }
}
