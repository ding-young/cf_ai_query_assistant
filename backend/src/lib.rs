use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlparser::{ast::Statement, dialect::SQLiteDialect, parser::Parser};
use worker::*;

const ALLOWED_ORIGINS: [&str; 2] = [
    "https://cf-ai-query-assistant.pages.dev",
    "http://localhost:3000",
];
static BASE_CORS: Lazy<Cors> = Lazy::new(|| {
    Cors::default()
        .with_methods(vec![Method::Get, Method::Post, Method::Options])
        .with_allowed_headers(["Content-Type"])
});

#[derive(Deserialize)]
struct Nl2SqlRequest {
    prompt: String,
}

#[derive(Deserialize)]
struct LlamaResponse {
    response: String,
}

#[derive(Serialize)]
struct Nl2SqlResponse {
    generated_sql: String,
    history_id: i64,
}

#[derive(Serialize)]
struct AiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct AiChatInput<'a> {
    messages: Vec<AiMessage<'a>>,
}

fn create_cors(req: &Request) -> worker::Result<Cors> {
    let mut cors = BASE_CORS.clone();
    if let Some(origin_header) = req.headers().get("Origin")?
        && ALLOWED_ORIGINS.contains(&origin_header.as_str())
    {
        cors = cors.with_origins(vec![origin_header]);
    }

    Ok(cors)
}

#[inline]
async fn cors_preflight(req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    Response::empty()?
        .with_status(204)
        .with_cors(&create_cors(&req)?)
}

async fn post_nl(mut req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let request_data: Nl2SqlRequest = match req.json().await {
        Ok(data) => data,
        Err(e) => {
            return Response::error(format!("Bad Request: {}", e), 400)?
                .with_cors(&create_cors(&req)?);
        }
    };

    let model = ctx
        .env
        .var("model")
        .ok()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "@cf/meta/llama-3.1-8b-instruct-fast".to_string());
    let ai = ctx.env.ai("ai")?;

    let system_prompt = "You are a SQL generator for given user prompt. Output ONLY a SQL query for D1 (SQLite dialect).\n
        Do not include any explanations, markdown, or other text.\nReturn SQL only.".to_string();

    let inputs = AiChatInput {
        messages: vec![
            AiMessage {
                role: "system",
                content: &system_prompt,
            },
            AiMessage {
                role: "user",
                content: &request_data.prompt,
            },
        ],
    };

    let ai_response: LlamaResponse = match ai.run(&model, inputs).await {
        Ok(resp) => resp,
        Err(e) => {
            // TODO: Proper logging
            console_log!("Failed to run model: {e}");
            return Response::error(format!("AI Error: {}", e), 500)?
                .with_cors(&create_cors(&req)?);
        }
    };
    let sql = ai_response.response;

    let db = ctx.env.d1("cf_ai_assistant_db")?;
    let stmt =
        db.prepare("INSERT INTO history(natural, sql, executed) VALUES (?1, ?2, 0) RETURNING id");

    let new_id: i64 = match stmt
        .bind(&[request_data.prompt.into(), sql.clone().into()])?
        .first(Some("id"))
        .await?
    {
        Some(id) => id,
        None => {
            // TODO: Proper logging
            console_log!("Failed to save history and get ID");

            return Response::error("Failed to save history and get ID", 500)?
                .with_cors(&create_cors(&req)?);
        }
    };
    let final_response = Nl2SqlResponse {
        generated_sql: sql,
        history_id: new_id,
    };

    Response::from_json(&final_response)?.with_cors(&create_cors(&req)?)
}

#[derive(Deserialize)]
struct ExecuteRequest {
    history_id: Option<i64>,
    sql_to_run: String,
}
async fn post_exec_sql(mut req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let request_data: ExecuteRequest = match req.json().await {
        Ok(data) => data,
        Err(e) => {
            return Response::error(format!("Bad Request: {}", e), 400)?
                .with_cors(&create_cors(&req)?);
        }
    };

    let sql_to_run = request_data.sql_to_run;

    // Parse SQL and return error if syntax is invalid
    let dialect = SQLiteDialect {};
    let parse_result = Parser::parse_sql(&dialect, &sql_to_run);
    let statements = match parse_result {
        Ok(stmts) => stmts,
        Err(e) => {
            return Response::error(format!("SQL Syntax Error: {}", e), 400)?
                .with_cors(&create_cors(&req)?);
        }
    };

    // Block dangerous statements
    for stmt in &statements {
        match stmt {
            Statement::Drop { .. }
            | Statement::AlterTable { .. }
            | Statement::AlterSchema { .. }
            | Statement::AlterRole { .. }
            | Statement::Truncate { .. }
            | Statement::CreateRole { .. }
            | Statement::CreateDatabase { .. } => {
                return Response::error(
                 "Your SQL contains operations that are not allowed for safety reasons (e.g., DROP, ALTER, TRUNCATE, etc.).",
                    403
            )?.with_cors(&create_cors(&req)?);
            }
            _ => {}
        }
    }

    let user_db = ctx.env.d1("my_app_db")?;
    let query_result: D1Result = match user_db.prepare(&sql_to_run).all().await {
        Ok(result) => result,
        Err(e) => {
            return Response::error(format!("SQL Execution Error: {}", e), 400)?
                .with_cors(&create_cors(&req)?);
        }
    };

    let history_db = ctx.env.d1("cf_ai_assistant_db")?;
    if let Some(id) = request_data.history_id {
        let _ = history_db
            .prepare("UPDATE history SET executed = 1 WHERE id = ?")
            .bind(&[wasm_bindgen::JsValue::from_f64(id as f64)])?
            .run()
            .await;
    } else {
        // TODO: No history_id provided; Insert new history record
    }

    let results: Vec<Value> = query_result.results()?;
    Response::from_json(&results)?.with_cors(&create_cors(&req)?)
}

#[derive(Serialize, Deserialize)]
struct HistoryEntry {
    id: i64,
    natural: String,
    sql: String,
    executed: i64,
    created_at: String,
}

pub async fn get_history(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let db = ctx.env.d1("cf_ai_assistant_db")?;
    let d1_result = db
        .prepare(
            "SELECT id, natural, sql, executed, created_at
         FROM history ORDER BY id DESC LIMIT 5",
        )
        .all()
        .await?;

    let entries: Vec<HistoryEntry> = match d1_result.results() {
        Ok(data) => data,
        Err(e) => {
            return Response::error(format!("Failed to parse D1 results: {}", e), 500)?
                .with_cors(&create_cors(&req)?);
        }
    };

    Response::from_json(&entries)?.with_cors(&create_cors(&req)?)
}

pub async fn get_ping(_req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    Response::ok("pong")
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    // Create an instance of the Router, which can use parameters (/user/:name) or wildcard values
    // (/file/*pathname). Alternatively, use `Router::with_data(D)` and pass in arbitrary data for
    // routes to access and share using the `ctx.data()` method.
    let router = Router::new();

    router
        .post_async("/nl2sql", post_nl)
        .options_async("/nl2sql", cors_preflight)
        .post_async("/execsql", post_exec_sql)
        .options_async("/execsql", cors_preflight)
        .get_async("/history", get_history)
        .options_async("/history", cors_preflight)
        .get_async("/ping", get_ping)
        .run(req, env)
        .await
}
