# Redis clone in Rust
## How to run
```bash
# If you have redis-cli installed you might need to stop the redis-server beforehand
cargo run # or ./spawn_redis_server.sh
```
## How to test
```bash
redis-cli # When you write 'PING', you should receive 'PONG' and so on
```
## Resources
https://gitlab.com/dpezely/chat-server
https://docs.rs/mio/latest/mio/guide/
https://app.codecrafters.io/courses/redis/stages/3
