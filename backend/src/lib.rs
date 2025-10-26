use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::*;

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

async fn post_nl(mut req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let request_data: Nl2SqlRequest = match req.json().await {
        Ok(data) => data,
        Err(e) => {
            return Response::error(format!("Bad Request: {}", e), 400);
        }
    };

    let model = ctx.env.var("AI_MODEL").ok().map(|v| v.to_string())
        .unwrap_or_else(|| "@cf/meta/llama-3.3-70b-instruct-fp8".to_string());
    let ai = ctx.env.ai("AI")?;
    
    let system_prompt = format!(
        "You are a SQL generator for given user prompt. Output ONLY a SQL query for D1 (SQLite dialect).\n
        Do not include any explanations, markdown, or other text.\nReturn SQL only.",
    );
    let inputs = json!({"messages":[
        {"role":"system","content":system_prompt},
        {"role":"user","content":request_data.prompt}
    ]});

    let ai_response: LlamaResponse = match ai.run(&model, inputs).await {
        Ok(resp) => resp,
        Err(e) => {
            return Response::error(format!("AI Error: {}", e), 500);
        }
    };
    let sql = ai_response.response;

    let db = ctx.env.d1("cf_ai_assistant_db")?;
    let stmt = db.prepare(
        "INSERT INTO history(natural, sql, executed) VALUES (?1, ?2, 0) RETURNING id"
    );

    let new_id: i64 = match stmt
        .bind(&[request_data.prompt.into(), sql.clone().into()])?
        .first(Some("id"))
        .await? 
    {
        Some(id) => id,
        None => return Response::error("Failed to save history and get ID", 500),
    };
    let final_response = Nl2SqlResponse {
        generated_sql: sql,
        history_id: new_id,
    };

    Response::from_json(&final_response)
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
            return Response::error(format!("Bad Request: {}", e), 400);
        }
    }; 
    
    let sql_to_run = request_data.sql_to_run;
    
    let user_db = ctx.env.d1("my_app_db")?;
    let query_result: D1Result = match user_db.prepare(&sql_to_run).all().await {
        Ok(result) => result,
        Err(e) => {
            return Response::error(format!("SQL Execution Error: {}", e), 400);
        }
    };
    
    let history_db = ctx.env.d1("cf_ai_assistant_db")?;
    if let Some(id) = request_data.history_id {
        let _ = history_db
            .prepare("UPDATE history SET executed = 1 WHERE id = ?")
            .bind(&[id.into()])?
            .run()
            .await;
    } else {
        // TODO: No history_id provided; Insert new history record
    }

    let results: Vec<Value> = query_result.results()?;
    Response::from_json(&results)
}

#[derive(Serialize, Deserialize)]
struct HistoryEntry {
    id: i64,
    natural: String,
    sql: String,
    executed: i64,
    created_at: String,
}

pub async fn get_history(_req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let db = ctx.env.d1("cf_ai_assistant_db")?;
    let d1_result = db.prepare(
        "SELECT id, natural, sql, executed, created_at
         FROM history ORDER BY id DESC LIMIT 5"
    ).all().await?;

    let entries: Vec<HistoryEntry> = match d1_result.results() {
        Ok(data) => data,
        Err(e) => {
            return Response::error(format!("Failed to parse D1 results: {}", e), 500);
        }
    };

    Response::from_json(&entries)
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
        .post_async("/execsql", post_exec_sql)
        .get_async("/history", get_history)
        .get_async("/ping", get_ping)
        .run(req, env).await
}