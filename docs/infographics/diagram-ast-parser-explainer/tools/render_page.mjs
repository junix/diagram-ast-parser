#!/usr/bin/env node
/**
 * CDP 渲染：固定版 chrome-headless-shell 打开 index.html（file://，零外部请求断言），
 * 逐片截图：每片先 window.scrollTo(0,y) 再回读 window.scrollY 断言一致才截图，
 * 切片写入 render/slices/，几何写入 render/layout.json。
 *
 * 固定版 chrome-headless-shell（playwright 缓存 chromium_headless_shell-1234，舰队同机共用），
 * 不用随系统升级漂移的本机 Chrome；去 --headless=new，加 --disable-gpu + srgb 色彩剖面。
 *
 * 环境变量：TREE_DIR、WORK_DIR 必填；CHROME_BIN 可探测；RENDER_DPR 默认 2。
 */
'use strict';
import { spawn } from 'node:child_process';
import { mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const TREE = process.env.TREE_DIR || fail('必须设置 TREE_DIR');
const WORK = process.env.WORK_DIR || fail('必须设置 WORK_DIR');
const DPR = Number(process.env.RENDER_DPR || '2');
const CHROME = process.env.CHROME_BIN ||
  (process.env.HOME +
   '/Library/Caches/ms-playwright/chromium_headless_shell-1234/chrome-headless-shell-mac-arm64/chrome-headless-shell');
const VIEW_W = 1200;
const VIEW_H = 1000;

function fail(msg) {
  console.error(`render_page: ${msg}`);
  process.exit(1);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function freePort() {
  const net = await import('node:net');
  return new Promise((res) => {
    const srv = net.createServer();
    srv.listen(0, '127.0.0.1', () => {
      const p = srv.address().port;
      srv.close(() => res(p));
    });
  });
}

class CDP {
  constructor(ws) {
    this.ws = ws;
    this.id = 0;
    this.pending = new Map();
    this.events = [];
    ws.addEventListener('message', (ev) => {
      const msg = JSON.parse(ev.data);
      if (msg.id && this.pending.has(msg.id)) {
        const { resolve, reject } = this.pending.get(msg.id);
        this.pending.delete(msg.id);
        msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
      } else if (msg.method) {
        this.events.push(msg);
      }
    });
  }
  send(method, params = {}) {
    const id = ++this.id;
    this.ws.send(JSON.stringify({ id, method, params }));
    return new Promise((resolve, reject) => this.pending.set(id, { resolve, reject }));
  }
}

async function main() {
  const port = await freePort();
  const profile = join(WORK, 'chrome-profile');
  rmSync(profile, { recursive: true, force: true });
  const chrome = spawn(CHROME, [
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${profile}`,
    '--hide-scrollbars', '--no-first-run',
    '--no-default-browser-check', '--disable-features=Translate',
    '--disable-gpu', '--force-color-profile=srgb',
    'about:blank',
  ], { stdio: ['ignore', 'ignore', 'pipe'] });
  let stderr = '';
  chrome.stderr.on('data', (d) => { stderr += d; });

  let wsUrl = null;
  for (let i = 0; i < 100 && !wsUrl; i++) {
    await sleep(100);
    try {
      const res = await fetch(`http://127.0.0.1:${port}/json/list`);
      const tabs = await res.json();
      const page = tabs.find((t) => t.type === 'page');
      if (page) wsUrl = page.webSocketDebuggerUrl;
    } catch { /* retry */ }
  }
  if (!wsUrl) fail(`无法连接 CDP: ${stderr.slice(0, 400)}`);

  const ws = new WebSocket(wsUrl);
  await new Promise((res, rej) => {
    ws.addEventListener('open', res);
    ws.addEventListener('error', rej);
  });
  const cdp = new CDP(ws);
  const requests = [];
  await cdp.send('Page.enable');
  await cdp.send('Network.enable');
  await cdp.send('Page.navigate', { url: `file://${join(TREE, 'index.html')}` });
  await sleep(2500); // load + 系统字体稳定

  const evalJS = async (expr) => {
    const r = await cdp.send('Runtime.evaluate', {
      expression: expr, returnByValue: true,
    });
    if (r.exceptionDetails) fail(`evaluate 失败: ${JSON.stringify(r.exceptionDetails)}`);
    return r.result.value;
  };

  for (const ev of cdp.events) {
    if (ev.method === 'Network.requestWillBeSent') {
      requests.push(ev.params.request.url);
    }
  }
  const external = requests.filter((u) => !u.startsWith('file://'));
  if (external.length) fail(`存在非 file:// 请求: ${external.join(', ')}`);

  await cdp.send('Emulation.setDeviceMetricsOverride', {
    width: VIEW_W, height: VIEW_H, deviceScaleFactor: DPR, mobile: false,
  });
  await sleep(400);

  const geo = await evalJS(`(() => {
    const de = document.documentElement;
    const sections = [...document.querySelectorAll('section')].map((s) => ({
      id: s.id, top: s.offsetTop, height: s.offsetHeight,
    }));
    return { width: de.clientWidth, height: de.scrollHeight,
             bodyHeight: document.body.scrollHeight, sections };
  })()`);
  if (geo.width !== VIEW_W) fail(`页面宽 ${geo.width} != ${VIEW_W}`);

  const slicesDir = join(TREE, 'render', 'slices');
  rmSync(slicesDir, { recursive: true, force: true });
  mkdirSync(slicesDir, { recursive: true });

  const scrollLog = [];
  let idx = 0;
  for (let y = 0; y < geo.height; y += VIEW_H, idx++) {
    const target = Math.min(y, Math.max(0, geo.height - VIEW_H));
    await evalJS(`window.scrollTo(0, ${target})`);
    await sleep(120);
    const actual = await evalJS(`window.scrollY`);
    if (actual !== target) {
      fail(`切片 ${idx} 滚动断言失败: scrollTo(0,${target}) 后 scrollY=${actual}`);
    }
    scrollLog.push({ slice: idx, scrollTo: target, scrollY: actual, ok: true });
    const shot = await cdp.send('Page.captureScreenshot', { format: 'png' });
    writeFileSync(join(slicesDir, `${String(idx).padStart(3, '0')}.png`),
      Buffer.from(shot.data, 'base64'));
  }

  const layout = {
    page_width: VIEW_W,
    page_height: geo.height,
    dpr: DPR,
    full2x_expected: [VIEW_W * DPR, geo.height * DPR],
    viewport_css: VIEW_H,
    slice_count: idx,
    scroll_assertions: scrollLog.length,
    scroll_assertions_ok: scrollLog.every((s) => s.ok),
    zero_external_requests: external.length === 0,
    request_count: requests.length,
    sections: geo.sections,
    scrolls: scrollLog,
  };
  writeFileSync(join(TREE, 'render', 'layout.json'),
    JSON.stringify(layout, null, 2) + '\n');
  console.log(`render_page: ${idx} 片全部滚动断言通过，页面 ${VIEW_W}x${geo.height} CSS px`);

  ws.close();
  chrome.kill('SIGTERM');
  process.exit(0);
}

main().catch((e) => fail(e.stack || String(e)));
