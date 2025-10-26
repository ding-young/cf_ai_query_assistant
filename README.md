# cf_ai_query_assistant
AI-powered natural-language-to-SQL assistant built with Cloudflare Workers, Workers AI, and D1 using [workers-rs](https://github.com/cloudflare/workers-rs). 
Type a question, get generated SQL, run it against D1, and browse recent query history. Make sure your wrangler.toml has the required D1 and AI bindings before running.

## Run locally 
```bash
# Create D1 databases. Note that migrations/ include dummy data as well. 
## db for saving query history 
npx wrangler d1 create cf-ai-assistant-db 
npx wrangler d1 migrations apply cf_ai_assistant_db --local
## this may be your local db for another app
npx wrangler d1 create my-app-db 
npx wrangler d1 migrations apply my_app_db --local

# Backend
cd backend
npx wrangler dev

# Frontend
cd frontend
npm install
npm run dev
```

## Demo
![demo](https://github.com/user-attachments/assets/993ce2d3-8408-4771-905d-5a08b1b8b14e)

## Further Improvements
- [] Rearchitecture with RAG, support schema upload
- [] Invalidate certain schemas via sqlparser-rs 
- [] Refactor code (CORS, cargo clippy)
