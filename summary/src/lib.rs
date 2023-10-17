use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use spin_sdk::{
    http::{Params, Request, Response, Router},
    http_component,
    key_value::Store,
    llm::{self, InferencingParams},
};

#[derive(Serialize, Deserialize)]
struct Prompt {
    id: String,
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct History {
    id: String,
    prompts: Vec<Prompt>,
}

#[derive(Serialize, Deserialize)]
struct Summary {
    id: String,
    summary: String,
}

/// A simple Spin HTTP component.
#[http_component]
fn handle_summary(req: Request) -> Result<Response> {
    let mut router = Router::new();
    router.post("/api/summary/:id", generate_summary);

    router.handle(req)
}

fn generate_summary(_req: Request, params: Params) -> Result<Response> {
    let id = params
        .get("id")
        .context("cannot get conversation ID from param")?;

    let kv = Store::open_default()?;
    let history: History = kv.get_json(id)?;

    let mut prompt = String::new();
    for p in history.prompts {
        prompt.push_str(&format!("{}:{}", p.role, p.content));
    }

    println!("Sending prompt to LLM: {}", prompt);
    let prompt = PROMPT.replace("{HISTORY}", &prompt);
    let res = llm::infer_with_options(
        llm::InferencingModel::Llama2Chat,
        &prompt,
        InferencingParams {
            max_tokens: 50,
            ..Default::default()
        },
    )?;

    println!("{:?}", res);
    let summary = serde_json::to_vec(&Summary {
        id: id.to_string(),
        summary: res.text,
    })?;

    Ok(http::Response::builder()
        .status(200)
        .body(Some(summary.into()))?)
}

// get the conversation ID from the request
// load the conversation history from the KV storage
// deserialize that into a History object
// take the history and make an LLM request to generate the summary
// return the summary

const PROMPT: &str = r#"
<<SYS>>
You are a bot that generates short summaries for conversations given a conversation history between a user and an AI assistant. The summaries should be as concise as possible
<</SYS>>
Follow the pattern of the following examples:
[INST]
PROMPTS
User: When was da Vinci born?
Assistant: Leonardo da Vinci was born on April 15, 1452.
User: how old was he when he painted the Mona Lisa?
SUMMARY: The life and career of Leonardo da Vinci
[/INST]
[INST]
PROMPTS
User: What is the climate in California?
Assistant: The climate in California is generally mild, with cool, wet winters and dry summers. The state's diverse geography and coastal location result in a wide range of climates.
SUMMARY: The climate of California
[/INST]
PROMPTS
{HISTORY}
SUMMARY: 
"#;
