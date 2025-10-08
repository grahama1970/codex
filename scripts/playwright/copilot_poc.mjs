#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';
import readline from 'node:readline/promises';
import { stdin as input, stdout as output } from 'node:process';

const args = Object.fromEntries(process.argv.slice(2).map(a => {
  const m = a.match(/^--([^=]+)=(.*)$/);
  return m ? [m[1], m[2]] : [a.replace(/^--/, ''), true];
}));

const promptPath = args.prompt || 'local/copilot_prompt.txt';
const outPath = args.out || 'local/copilot_review_playwright.txt';
const url = args.url || 'https://github.com/copilot';
const profileDir = args.profile || '.playwright/copilot';
const headless = (args.headless ?? 'false') === 'true';
const autoSend = (args.autoSend ?? 'false') === 'true';
const meanDelay = Number(args.meanDelay || 90);
const waitStable = Number(args.waitStable || 3000);
const pollInterval = Number(args.pollInterval || 1000);

const jitter = (ms) => new Promise(res => setTimeout(res, ms * (0.7 + Math.random() * 0.6)));

async function main() {
  const { chromium } = await import('playwright');
  await fs.mkdir(profileDir, { recursive: true }).catch(() => {});
  await fs.mkdir(path.dirname(outPath), { recursive: true }).catch(() => {});

  const context = await chromium.launchPersistentContext(profileDir, {
    headless,
    viewport: { width: 1280, height: 900 },
  });
  const page = context.pages()[0] || await context.newPage();
  await page.goto(url, { waitUntil: 'domcontentloaded' });

  // If not signed in, allow user to authenticate manually.
  if (await page.locator('text=/Sign in|Sign into|Log in/i').first().isVisible().catch(() => false)) {
    console.log('Please sign in to GitHub in this window. Press Enter here once done…');
    const rl = readline.createInterface({ input, output });
    await rl.question('Continue? ');
    rl.close();
  }

  // Try to focus the chat input via common selectors; else ask user to click it manually.
  const inputSel = 'textarea, [contenteditable="true"], div[role="textbox"]';
  const box = page.locator(inputSel).first();
  if (await box.count() > 0) {
    await box.click().catch(() => {});
  } else {
    console.log('Could not find chat input automatically. Please click the Copilot chat input, then press Enter here…');
    const rl = readline.createInterface({ input, output });
    await rl.question('Ready to type? ');
    rl.close();
  }

  const content = await fs.readFile(promptPath, 'utf8');
  for (const ch of content) {
    await page.keyboard.type(ch, { delay: meanDelay * (0.7 + Math.random() * 0.6) });
  }
  if (autoSend) {
    await jitter(120);
    await page.keyboard.press('Enter');
  } else {
    console.log('Prompt pasted. Press Enter in the browser to send, then return here.');
    const rl = readline.createInterface({ input, output });
    await rl.question('Continue to wait for output? ');
    rl.close();
  }

  // Wait for innerText length to stabilize
  const stableMs = waitStable;
  let stableFor = 0;
  let prevLen = 0;
  while (true) {
    const len = await page.evaluate(() => document.body.innerText.length).catch(() => 0);
    if (len === prevLen) {
      stableFor += pollInterval;
    } else {
      stableFor = 0;
      prevLen = len;
    }
    if (stableFor >= stableMs) break;
    await new Promise(r => setTimeout(r, pollInterval));
  }

  const text = await page.evaluate(() => document.body.innerText);
  await fs.writeFile(outPath, text, 'utf8');
  console.log(`Saved Copilot page text to ${outPath} (chars=${text.length})`);

  await context.close();
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});

