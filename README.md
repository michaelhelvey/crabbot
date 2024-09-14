# CrabBot

A discord bot written in Rust. I'm not sure what it does yet, I just wanted to write a bot and see
how it would go.

## Getting Started

Required dependencies:

- Working Rust toolchain
- ngrok account (if you want to proxy the discord bot to your local machine)
- Just command runner (if you want to use the justfile)

Running the bot:

_this assumes that you have the bot actually created via the Discord Developer Portal_

1. Start by copying `.env.sample` to `.env` and filling in the values.
2. Execute `just register` to ensure that the list of commands registered to the bot are in sync.
3. Visit the Discord install link at My Applications > CrabBot > Install Link in order to install
   the bot into a server.

## Deployment

This is a WIP; long-term I'd like to use pulumi or something like that.
