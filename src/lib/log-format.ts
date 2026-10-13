// Tidies `flutter run` output for the logs dialog: drops the box borders that loggers like
// `logger`'s PrettyPrinter draw around each message, folds the timestamp line into the
// message, and groups an HTTP interceptor's `--> GET: url` / `--> headers:` / `<-- status:` /
// `<-- response:` lines into one request block and one response block.

const BOX_ONLY = /^[\s┌└├┼┬┴─┄┈═╔╚╠╟╗╝║│]+$/;
const TIME_LINE = /^(\d{2}:\d{2}:\d{2})\.\d+ \(\+[\d:.]+\)$/;
// `logger`'s level markers (as emoji or `:bug:`-style shortcodes); other emoji like ✅ / ❌ stay.
const EMOJI = /^(?::[a-z_]+:|[🐛💡⛔👾💩]\uFE0F?)\s*/u;
const HTTP = /^(-->|<--) ([A-Za-z]+): ?(.*)$/s;
const METHOD = /^(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS)$/;

function pretty(value: string): string {
  if (/^[[{]/.test(value)) {
    try {
      return JSON.stringify(JSON.parse(value), null, 2);
    } catch {
      // Dart map `toString()` or a truncated body: still decode its \uXXXX escapes.
    }
  }
  return value.replace(/\\u([0-9a-fA-F]{4})/g, (_, hex) => String.fromCharCode(parseInt(hex, 16))).replace(/\\\//g, "/");
}

/** Returns a `push(logs, line)` that appends the tidied line to `logs`, or merges it into an earlier entry. */
export function createLogFormatter() {
  let time = "";
  let lastHeaders = "";
  // Index of the request/response block the next `-->` / `<--` detail line belongs to.
  let block = -1;
  // Unanswered request blocks by "METHOD url", oldest first, so a response can go under its request.
  const pending = new Map<string, number[]>();
  // iOS runs Shortcuts / App Intents indexing inside the app's process, so its chatter (which lists
  // every installed app) lands in the app's console. Skip it, keeping the app's own lines and errors.
  let inShortcutIndexing = false;

  return (logs: string[], raw: string) => {
    if (raw.startsWith("-[WFIsolatedShortcutRunner ")) {
      inShortcutIndexing = !raw.includes("unaliveProcess");
      return;
    }
    if (inShortcutIndexing && !raw.startsWith("flutter: ") && !raw.startsWith("[ERROR")) {
      if (raw.startsWith("Finished in ")) inShortcutIndexing = false;
      return;
    }
    let line = raw.replace(/^flutter: /, "");
    if (BOX_ONLY.test(line)) return;
    line = line.replace(/^[│║]\s?/, "");

    const t = line.trim().match(TIME_LINE);
    if (t) {
      time = t[1];
      return;
    }
    line = line.replace(EMOJI, "");
    const stamp = time ? `${time}  ` : "";
    time = "";

    const m = line.match(HTTP);
    if (!m) {
      block = -1;
      logs.push(stamp + line);
      return;
    }
    const [, dir, key, value] = m;
    if (dir === "-->" && METHOD.test(key)) {
      block = logs.push(`${stamp}→ ${key} ${value}`) - 1;
      const k = `${key} ${value.trim()}`;
      pending.set(k, [...(pending.get(k) ?? []), block]);
      return;
    }
    if (dir === "<--" && key === "status") {
      // `<-- status: 200 GET: https://…` names its request; responses arrive in completion order.
      const s = value.match(/^(\d{3}) ([A-Z]+): (\S+)/);
      const request = s && pending.get(`${s[2]} ${s[3]}`)?.shift();
      if (request !== undefined && request !== null && logs[request] !== undefined) {
        block = request;
        logs[block] += `\n   ← ${s![1]}${stamp ? ` · ${stamp.trim()}` : ""}`;
      } else {
        block = logs.push(`${stamp}← ${value}`) - 1;
      }
      return;
    }
    // Every request repeats the same headers; show them only when they change.
    if (key === "headers") {
      if (value === lastHeaders) return;
      lastHeaders = value;
    }
    const detail = `${key}: ${pretty(value)}`.replace(/\n/g, "\n   ");
    if (block !== -1) {
      logs[block] += `\n   ${detail}`;
    } else {
      block = logs.push(`${stamp}${dir === "-->" ? "→" : "←"} ${detail}`) - 1;
    }
  };
}

const FLUTTER_TEST_PROGRESS = /^\d\d:\d\d \+\d+(?: ~\d+)?(?: -\d+)?: (.*)$/;

/**
 * Same as `createLogFormatter`, plus test-runner tidying: shortens absolute test paths, turns
 * `flutter test`'s `00:02 +1 -1: name [E]` progress lines into `▸ name` / `✗ name` and drops its
 * `loading` lines, and hides xcodebuild's indented compiler command lines.
 */
export function createTestLogFormatter(kind: string | null | undefined) {
  const push = createLogFormatter();
  let lastStarted = "";

  return (logs: string[], raw: string) => {
    if (kind === "nativeIos" && raw.startsWith("    ")) return;
    let line = raw.replace(/\/\S*?\/((?:integration_)?test)\//g, "$1/");

    const progress = line.match(FLUTTER_TEST_PROGRESS);
    if (progress) {
      const rest = progress[1].replace(/^(\S+\.dart): /, (_, file: string) => `${file.split("/").pop()} › `);
      if (rest.startsWith("loading ")) return;
      if (rest === "All tests passed!") line = `✅ ${rest}`;
      else if (rest === "Some tests failed.") line = `❌ ${rest}`;
      else if (rest.endsWith(" [E]")) line = `✗ ${rest.slice(0, -4)}`;
      else if (rest === lastStarted) return;
      else line = `▸ ${(lastStarted = rest)}`;
    }
    push(logs, line);
  };
}
