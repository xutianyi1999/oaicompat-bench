use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs};
use async_openai::Client;
use clap::{arg, Parser};
use futures_util::StreamExt;
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::signal;
use tokio::time::Instant;

async fn chat_completions_bench(
    client: &Client<OpenAIConfig>,
    prompt: String,
    tokenizer: &Tokenizer,
    prefill_count: &AtomicU64,
    decode_count: &AtomicU64,
) -> Result<()> {
    let mut is_update_prefill = true;
    let mut last = Instant::now();
    let mut session_tokens = 0;

    let req = CreateChatCompletionRequestArgs::default()
        .messages(vec![ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage::from(prompt.as_str()))])
        .max_tokens(65536u32)
        .build()?;

    let mut stream = client.chat()
        .create_stream(req)
        .await?;

    while let Some(res) = stream.next().await {
        let resp = res?;

        if is_update_prefill {
            let now = Instant::now();
            println!("prefill use: {:?}", now - last);
            last = now;

            let enc = tokenizer.encode_fast(prompt.as_str(), false).map_err(|e| anyhow!(e.to_string()))?;
            prefill_count.fetch_add(enc.get_ids().len() as u64, Ordering::Relaxed);

            is_update_prefill = false;
        }

        let mut decode_tokens = 0;

        for choice in resp.choices {
            let content = choice.delta.content.ok_or_else(|| anyhow!("no content"))?;
            let enc = tokenizer.encode_fast(content.as_str(), false).map_err(|e| anyhow!(e.to_string()))?;
            decode_tokens += enc.get_ids().len() as u64;
        }

        session_tokens += decode_tokens;
        decode_count.fetch_add(decode_tokens, Ordering::Relaxed);
    }

    println!("avg time to decode one token use: {:?}ms", last.elapsed().as_millis() as u64 / session_tokens);
    Ok(())
}

#[derive(Parser)]
#[command(version)]
struct Args {
    /// http://127.0.0.1:8080/v1
    #[arg(short, long)]
    api_base: String,

    /// HuggingFace repo, e.g. "Qwen/Qwen2.5-72B"
    #[arg(short, long)]
    model: String,

    #[arg(short, long)]
    dataset_path: String,

    #[arg(short, long)]
    parallel_task: usize
}

fn load_datasets(dataset_path: &str) -> Result<Vec<String>> {
    let paths = glob::glob(dataset_path)?;
    let mut questions = Vec::new();

    for path in paths {
        let path = path?;
        let mut rdr = csv::Reader::from_path(path)?;

        for res in rdr.records() {
            let record = res?;
            let mut record_iter = record.iter();
            record_iter.next();

            let question = format!(
                "回答问题并且原因: {}, A: {}, B: {}, C: {}, D: {}",
                record_iter.next().unwrap(),
                record_iter.next().unwrap(),
                record_iter.next().unwrap(),
                record_iter.next().unwrap(),
                record_iter.next().unwrap(),
            );

            questions.push(question);
        }
    }
    Ok(questions)
}

fn exec(args: Args) -> Result<()> {
    let tokenizer = Tokenizer::from_pretrained(args.model, None).map_err(|e| anyhow!(e.to_string()))?;
    let prompts: Vec<String> = load_datasets(args.dataset_path.as_str())?;
    println!("load {} prompts", prompts.len());

    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let tokenizer = Arc::new(tokenizer);
        
        let mut config = OpenAIConfig::new();
        config = config.with_api_base(args.api_base);
        let client = Client::with_config(config);
        let client = Arc::new(client);

        let prefill_count = Arc::new(AtomicU64::new(0));
        let decode_count = Arc::new(AtomicU64::new(0));

        let sem = tokio::sync::Semaphore::new(args.parallel_task);
        let sem = Arc::new(sem);

        let mut futs = Vec::new();
        for prompt in prompts {
            let tokenizer = tokenizer.clone();
            let client = client.clone();
            let prefill_count = prefill_count.clone();
            let decode_count = decode_count.clone();
            let sem = sem.clone();

            let fut = async {
                tokio::spawn(async move {
                    let _guard = sem.acquire().await?;

                    chat_completions_bench(
                        &client,
                        prompt,
                        &tokenizer,
                        &prefill_count,
                        &decode_count
                    ).await
                }).await??;

                Result::<(), anyhow::Error>::Ok(())
            };
            futs.push(fut);
        }

        let mut total_prefill = 0;
        let mut total_decode = 0;
        let t = Instant::now();

        let fut = async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let prefill = prefill_count.swap(0, Ordering::Relaxed);
                let decode = decode_count.swap(0, Ordering::Relaxed);

                total_prefill += prefill;
                total_decode += decode;
                println!("prefill: {} tokens/s, decode: {} tokens/s", prefill, decode);
            }
        };

        #[cfg(windows)]
        {
            let mut ctrl_c = signal::windows::ctrl_c()?;
            let mut ctrl_close = signal::windows::ctrl_close()?;

            tokio::select! {
                _ = ctrl_c.recv() => (),
                _ = ctrl_close.recv() => (),
                res = futures_util::future::try_join_all(futs) => {
                    res?;
                },
                _ = fut => ()
            };
        }

        #[cfg(unix)]
        {
            let mut terminate = signal::unix::signal(signal::unix::SignalKind::terminate())?;
            let mut interrupt = signal::unix::signal(signal::unix::SignalKind::interrupt())?;

            tokio::select! {
                _ = terminate.recv() => (),
                _ = interrupt.recv() => (),
                res = futures_util::future::try_join_all(futs) => {
                    res?;
                },
                _ = fut => ()
            };
        }

        let secs = t.elapsed().as_secs();
        println!("global prefill: {} tokens/s, global decode: {} tokens/s", total_prefill / secs, total_decode / secs);

        Ok(())
    })
}

fn main() -> ExitCode {
    let args = Args::parse();

    match exec(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{:?}", e);
            ExitCode::FAILURE
        }
    }
}
