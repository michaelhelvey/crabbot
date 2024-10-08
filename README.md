# CrabBot

A discord bot written in Rust. I'm not sure what it does yet, I just wanted to write a bot and see
how it would go.

## Getting Started

Required dependencies:

- Working Rust toolchain
- ngrok account (if you want to proxy the discord bot to your local machine)
- Just command runner (if you want to use the justfile)
- direnv or something to load .env vars into your shell because the rust code won't auto load the
  file

Running the bot:

_this assumes that you have the bot actually created via the Discord Developer Portal_

1. Start by copying `.env.sample` to `.env` and filling in the values.
2. Execute `just register` to ensure that the list of commands registered to the bot are in sync.
3. Visit the Discord install link at My Applications > CrabBot > Install Link in order to install
   the bot into a server.
4. Run `just proxy` to start an ngrok tunnel to your local machine
5. Run `cargo run` to start the bot

At this point, assuming that it's installed and everything, you should be able to make requests from
discord to the bot running locally on your machine.

## Deployment

Required depenencies:

- Pulumi (authorized with your pulumi account)
- AWS (authorized locally against the account you want to deploy to)

Once you have this installed and setup, run `just deploy`. This will deploy the bot to your AWS
account. Note that this project has no CI; as a personal project, there's just not any benefit for
me to deploy from anything other than my local environment.
