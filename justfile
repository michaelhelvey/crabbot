proxy:
    ngrok http http://localhost:3000

register:
    node ./scripts/register.mjs

deploy:
    pulumi update --stack dev
