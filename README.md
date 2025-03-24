# oaicompat-bench

## run
Prepare `dataset.json` before running, dataset https://github.com/ymcui/Chinese-LLaMA-Alpaca/wiki/C-Eval%E8%AF%84%E6%B5%8B%E7%BB%93%E6%9E%9C%E4%B8%8E%E8%84%9A%E6%9C%AC

```shell
cargo run --release -- -a "http://127.0.0.1:30021/v1" -m Qwen/Qwen2.5-72B -d "/media/nvme/data/test/*.csv" -p 4
```

output 

```text
prefill use: 1.401792474s
prefill: 10 tokens/s, decode: 35 tokens/s
prefill: 0 tokens/s, decode: 42 tokens/s
prefill: 0 tokens/s, decode: 35 tokens/s
prefill: 0 tokens/s, decode: 41 tokens/s
prefill: 0 tokens/s, decode: 45 tokens/s
prefill: 0 tokens/s, decode: 44 tokens/s
prefill: 0 tokens/s, decode: 53 tokens/s
prefill: 0 tokens/s, decode: 55 tokens/s
prefill: 0 tokens/s, decode: 30 tokens/s
prefill: 0 tokens/s, decode: 62 tokens/s
prefill: 0 tokens/s, decode: 32 tokens/s
prefill: 0 tokens/s, decode: 28 tokens/s
prefill: 0 tokens/s, decode: 37 tokens/s
prefill: 0 tokens/s, decode: 37 tokens/s
avg time to decode one token use: 23ms
prefill: 0 tokens/s, decode: 5 tokens/s
prefill use: 16.37630516s
prefill: 6 tokens/s, decode: 24 tokens/s
prefill: 0 tokens/s, decode: 32 tokens/s
prefill: 0 tokens/s, decode: 32 tokens/s
prefill: 0 tokens/s, decode: 32 tokens/s
prefill: 0 tokens/s, decode: 33 tokens/s
prefill: 0 tokens/s, decode: 36 tokens/s
prefill: 0 tokens/s, decode: 39 tokens/s
prefill: 0 tokens/s, decode: 41 tokens/s
prefill: 0 tokens/s, decode: 34 tokens/s
prefill: 0 tokens/s, decode: 38 tokens/s
prefill: 0 tokens/s, decode: 33 tokens/s
prefill: 0 tokens/s, decode: 47 tokens/s
prefill: 0 tokens/s, decode: 37 tokens/s
prefill: 0 tokens/s, decode: 46 tokens/s
prefill: 0 tokens/s, decode: 29 tokens/s
avg time to decode one token use: 27ms
```
