# cf_ai_query_assistant
AI-powered natural-language-to-SQL assistant built with Cloudflare Workers, Workers AI, and D1 using [workers-rs](https://github.com/cloudflare/workers-rs). 
Type a question, get generated SQL, run it against D1, and browse recent query history. Make sure your wrangler.toml has the required D1 and AI bindings before running.

## Demo
A live demo is deployed and accessible at: https://cf-ai-query-assistant.pages.dev


![demo](https://github.com/user-attachments/assets/993ce2d3-8408-4771-905d-5a08b1b8b14e)
  

## Or run locally 
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

## Notes

This application assumes a target database (Cloudflare D1) is already provisioned and connected to the backend Worker.

For security, the backend actively blocks destructive SQL operations (e.g., DROP TABLE, ALTER TABLE, TRUNCATE). Only read-only queries (like SELECT) and safe write operations (like INSERT, UPDATE) are permitted.

## Diagram
```ASCII
                                                       ┌─────────────┐
                                                       │Cloudflare D1│
                                                       │SQL database │
                                                       └─▲───────────┘
┌──────────────────┐       ┌──────────────────┐          │            
│ Cloudflare Pages ├──────►│Cloudflare Workers├──────────┘            
│(Next.js frontend)│       │   (workers-rs)   ├──────────┐            
└──────────────────┘       └──────────────────┘        ┌─▼───────────┐
                                                       │ Cloudflare  │
                                                       │ Workers AI  │
                                                       └─────────────┘
```

## Further Improvements
- [] Rearchitecture with RAG, support schema upload
- [x] Invalidate certain schemas via sqlparser-rs 
- [x] Refactor code (CORS, cargo clippy)

