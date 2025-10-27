npx wrangler d1 create cf-ai-assistant-db
npx wrangler d1 migrations apply cf_ai_assistant_db --local
npx wrangler d1 create my-app-db

npx wrangler d1 migrations apply my_app_db --local

npx wrangler dev