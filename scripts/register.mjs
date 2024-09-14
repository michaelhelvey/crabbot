#!/usr/bin/env node
/**
 * This script registers the list of guild commands that the bot should have when installed into
 * a discord server.
 */
import { ok as assert } from "node:assert";

const requireEnv = (envKey) => {
  const resolved = process.env[envKey];
  assert(resolved, `Expected process.env. to contain ${envKey}`);
  return resolved;
};

const DISCORD_API_BASE = `https://discord.com/api/v10`;
const DISCORD_USER_AGENT = "DiscordBot (https://github.com/michaelhelvey/crabbot, 1.0.0)";
const DISCORD_APP_ID = requireEnv("DISCORD_APP_ID");
const DISCORD_BOT_TOKEN = requireEnv("DISCORD_BOT_TOKEN");

// Registers what commands the bot has available to it
const commands = [
  {
    name: "test",
    description: "Basic command to test the application functionality",
    type: 1,
    integration_types: [0, 1],
    contexts: [0, 1, 2],
  },
];

async function putCommands(commands) {
  const headers = {
    Authorization: `Bot ${DISCORD_BOT_TOKEN}`,
    "Content-Type": "application/json; charset=UTF-8",
    "User-Agent": DISCORD_USER_AGENT,
  };

  const endpoint = `applications/${DISCORD_APP_ID}/commands`;
  const url = `${DISCORD_API_BASE}/${endpoint}`;

  const response = await fetch(url, {
    method: "PUT",
    headers,
    body: JSON.stringify(commands),
  });

  const data = await response.json();

  if (!response.ok) {
    throw new Error(`Received error response from discord API: ${JSON.stringify(data)}`);
  }

  console.log(`Successfully registered commands (statusCode = ${response.status})`);
  return data;
}

async function main() {
  console.log(`Registering command list with discord:\n${JSON.stringify(commands, null, 2)}`);
  await putCommands(commands);
}

await main();
