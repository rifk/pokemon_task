# Pokemon Server

REST server to provide pokemon info written in rust.

## Running

Can start server by running
```
cargo run
```
this will start the server at `localhost:5000`.

## TODO

Few things that could be better if this was going to a prod deployment:

- logging - currently only using `println!`, should switch to proper logging library like tracing and log messages at appropriate levels (error/warn/info/debug)
- graceful shutdown of server
- config file - read in values from a file or some other source to set timeout value and port the server runs on

