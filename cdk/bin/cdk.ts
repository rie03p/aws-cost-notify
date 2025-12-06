#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib/core';
import { CostStack } from '../lib/cost-stack';
import 'dotenv/config';

const app = new cdk.App();

const discordWebhookUrl = process.env.DISCORD_WEBHOOK_URL
if (!discordWebhookUrl) {
  console.error("Environment variable DISCORD_WEBHOOK_URL is not set.");
  process.exit(1);
}

new CostStack(app, 'CostStack', {discordWebhookUrl});
