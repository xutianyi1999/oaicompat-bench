# oaicompat-bench

## run
Prepare `dataset.json` before running, dataset example: `{"how are you today", "What is LLM?"}`

```shell
cargo run --release -- -a "http://127.0.0.1:30021/v1" -m Qwen/Qwen2.5-72B -d ./dataset.json
```

output 

```text
prefill use: 700.064913ms
prefill: 6 tokens/s, decode: 14 tokens/s
prefill: 0 tokens/s, decode: 64 tokens/s
prefill: 0 tokens/s, decode: 70 tokens/s
prefill: 0 tokens/s, decode: 77 tokens/s
prefill: 0 tokens/s, decode: 58 tokens/s
prefill: 0 tokens/s, decode: 58 tokens/s
prefill: 0 tokens/s, decode: 90 tokens/s
prefill: 0 tokens/s, decode: 25 tokens/s
avg time to decode one token use: 16
prefill: 0 tokens/s, decode: 5 tokens/s
prefill use: 9.485791484s
prefill: 6 tokens/s, decode: 12 tokens/s
prefill: 0 tokens/s, decode: 43 tokens/s
prefill: 0 tokens/s, decode: 20 tokens/s
prefill: 0 tokens/s, decode: 51 tokens/s
prefill: 0 tokens/s, decode: 19 tokens/s
prefill: 0 tokens/s, decode: 33 tokens/s
prefill: 0 tokens/s, decode: 39 tokens/s
prefill: 0 tokens/s, decode: 16 tokens/s
prefill: 0 tokens/s, decode: 33 tokens/s
prefill: 0 tokens/s, decode: 34 tokens/s
prefill: 0 tokens/s, decode: 34 tokens/s
prefill: 0 tokens/s, decode: 27 tokens/s
prefill: 0 tokens/s, decode: 32 tokens/s
prefill: 0 tokens/s, decode: 19 tokens/s
prefill: 0 tokens/s, decode: 25 tokens/s
prefill: 0 tokens/s, decode: 26 tokens/s
prefill: 0 tokens/s, decode: 30 tokens/s
avg time to decode one token use: 33
```
