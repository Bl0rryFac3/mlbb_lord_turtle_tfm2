@echo off
cd /d "%~dp0"
cargo test --locked --offline --manifest-path mod\mlbb_lord_turtle\native\Cargo.toml
if errorlevel 1 exit /b 1
cargo build --release --locked --offline --manifest-path mod\mlbb_lord_turtle\native\Cargo.toml
if errorlevel 1 exit /b 1
copy /y mod\mlbb_lord_turtle\native\target\release\mlbb_lord_turtle.dll mod\mlbb_lord_turtle\mlbb_lord_turtle.dll
