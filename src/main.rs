use anyhow::{anyhow, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs};
use async_openai::Client;
use clap::{arg, Parser};
use futures_util::StreamExt;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokenizers::Tokenizer;

async fn chat_completions_bench(
    client: &Client<OpenAIConfig>,
    prompt: String,
    tokenizer: &Tokenizer,
    prefill_count: &AtomicU64,
    decode_count: &AtomicU64,
) -> Result<()> {
    let req = CreateChatCompletionRequestArgs::default()
        .messages(vec![ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage::from(prompt.as_str()))])
        .max_tokens(65536u32)
        .build()?;

    let mut stream = client.chat()
        .create_stream(req)
        .await?;

    let mut is_update_prefill = false;

    while let Some(res) = stream.next().await {
        if !is_update_prefill {
            let enc = tokenizer.encode_fast(prompt.as_str(), false).map_err(|e| anyhow!(e.to_string()))?;
            prefill_count.fetch_add(enc.get_ids().len() as u64, Ordering::Relaxed);
            is_update_prefill = true;
        }

        let resp = res?;

        for choice in resp.choices {
            let content = choice.delta.content.ok_or_else(|| anyhow!("no content"))?;
            let enc = tokenizer.encode_fast(content.as_str(), false).map_err(|e| anyhow!(e.to_string()))?;
            decode_count.fetch_add(enc.get_ids().len() as u64, Ordering::Relaxed);
        }
    }
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
    dataset_json: PathBuf
}

fn exec(args: Args) -> Result<()> {
    let tokenizer = Tokenizer::from_pretrained(args.model, None).map_err(|e| anyhow!(e.to_string()))?;
    let prompts: Vec<String> = serde_json::from_reader(std::fs::File::open(&args.dataset_json)?)?;
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let tokenizer = Arc::new(tokenizer);
        
        let mut config = OpenAIConfig::new();
        config = config.with_api_base(args.api_base);
        let client = Client::with_config(config);
        let client = Arc::new(client);

        let prefill_count = Arc::new(AtomicU64::new(0));
        let decode_count = Arc::new(AtomicU64::new(0));

        let mut futs = Vec::new();
        for prompt in prompts {
            let tokenizer = tokenizer.clone();
            let client = client.clone();
            let prefill_count = prefill_count.clone();
            let decode_count = decode_count.clone();

            let fut = async {
                tokio::spawn(async move {
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

        tokio::spawn(async move{
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                let prefill = prefill_count.swap(0, Ordering::Relaxed);
                let decode = decode_count.swap(0, Ordering::Relaxed);

                println!("prefill: {} tokens/s, decode: {} tokens/s", prefill, decode);
            }
        });

        futures_util::future::try_join_all(futs).await?;
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
