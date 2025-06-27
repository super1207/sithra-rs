default:
    just --list

example_server:
    cargo build -p sithra-server --example server_a
    cargo build -p sithra-server --example server_b
    cd target/debug/examples && ./server_a
