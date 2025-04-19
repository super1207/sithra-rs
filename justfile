echo:
  cargo build -p echo
  mv -f ./target/debug/echo.exe ./plugins/echo.exe
  cargo run