dev:
    npx wrangler dev

deploy:
    npx wrangler deploy

setup:
    npx wrangler d1 execute --local bus-db --file=./schema.sql