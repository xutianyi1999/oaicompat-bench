# oaicompat-bench

### run
Prepare `dataset.json` before running, dataset example: `{"how are you today", "What is LLM?"}`


```shell
cargo run --release -- -a "http://127.0.0.1:30021/v1" -m Qwen/Qwen2.5-72B -d ./dataset.json
```
