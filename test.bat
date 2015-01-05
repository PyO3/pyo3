@c:\cygwin\bin\touch src/lib.rs
cargo build
@if errorlevel 1 exit /b 1
rustc testmodule.rs --extern rust-cpython=target\librust-cpython-211b85e007ec6e28.rlib -L "c:\Program Files\Python27\libs" -o testmodule.pyd
@if errorlevel 1 exit /b 1
python -c "import testmodule"
