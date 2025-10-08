#!/usr/bin/env node
import fs from 'node:fs/promises';
import path from 'node:path';

const args = Object.fromEntries(process.argv.slice(2).map(a => {
  const m = a.match(/^--([^=]+)=(.*)$/);
  return m ? [m[1], m[2]] : [a.replace(/^--/, ''), true];
}));

const endpoint = args.endpoint || 'http://localhost:9222';
const url = args.url || 'https://github.com/copilot';
const promptPath = args.prompt || 'local/copilot_prompt.txt';
const outPath = args.out || 'local/copilot_review_cdp.txt';
const autoSend = (args.autoSend ?? 'false') === 'true';
const meanDelay = Number(args.meanDelay || 90);
const waitStable = Number(args.waitStable || 3000);
const pollInterval = Number(args.pollInterval || 1000);

const jitterDelay = () => meanDelay * (0.7 + Math.random() * 0.6);

async function main() {
  const { chromium } = await import('playwright');
  await fs.mkdir(path.dirname(outPath), { recursive: true }).catch(() => {});
  const browser = await chromium.connectOverCDP(endpoint);
  const contexts = browser.contexts();
  const context = contexts[0] || await browser.newContext();
  const page = context.pages()[0] || await context.newPage();
  await page.bringToFront().catch(() => {});
  await page.goto(url, { waitUntil: 'domcontentloaded' });

  // Focus input if possible; otherwise user can click it.
  const inputSel = 'textarea, [contenteditable="true"], div[role="textbox"]';
  const box = page.locator(inputSel).first();
  if (await box.count() > 0) {
    await box.click().catch(() => {});
  }

  const content = await fs.readFile(promptPath, 'utf8');
  for (const ch of content) {
    await page.keyboard.type(ch, { delay: jitterDelay() });
  }
  if (autoSend) {
    await page.keyboard.press('Enter');
  }

  // Wait for innerText to stabilize
  let prevLen = 0, stableFor = 0;
  while (true) {
    const len = await page.evaluate(() => document.body.innerText.length).catch(() => 0);
    if (len === prevLen) stableFor += pollInterval; else { stableFor = 0; prevLen = len; }
    if (stableFor >= waitStable) break;
    await new Promise(r => setTimeout(r, pollInterval));
  }
  const text = await page.evaluate(() => document.body.innerText);
  await fs.writeFile(outPath, text, 'utf8');
  console.log(`[copilot-cdp] saved ${outPath} (chars=${text.length})`);
  await browser.close();
}

main().catch(e => { console.error(e); process.exit(1); });

