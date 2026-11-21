@echo off
cd /d "%~dp0"
cargo build --release --manifest-path native\Cargo.toml
if errorlevel 1 exit /b 1
copy /y native\target\release\mlbb_lord_turtle.dll mlbb_lord_turtle.dll
