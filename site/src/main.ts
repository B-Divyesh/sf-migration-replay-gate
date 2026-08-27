import "./style.css";
import { scenarioFor } from "./replay.mjs";

type ScenarioKey = "clean" | "repeat" | "partial";

const tabs = Array.from(document.querySelectorAll<HTMLButtonElement>("[role='tab']"));
const runButton = document.querySelector<HTMLButtonElement>("#run-replay");
const terminal = document.querySelector<HTMLElement>("#terminal-code code");
const result = document.querySelector<HTMLElement>("#terminal-result");
let selected: ScenarioKey = "clean";
let replayTimer: number | undefined;

function selectScenario(tab: HTMLButtonElement, focus = false) {
  selected = tab.dataset.scenario as ScenarioKey;
  for (const candidate of tabs) {
    const active = candidate === tab;
    candidate.setAttribute("aria-selected", String(active));
    candidate.tabIndex = active ? 0 : -1;
  }
  const panel = document.querySelector<HTMLElement>("#terminal-output");
  panel?.setAttribute("aria-labelledby", tab.id);
  if (focus) tab.focus();
  renderEmpty();
}

function renderEmpty() {
  window.clearTimeout(replayTimer);
  if (terminal) terminal.innerHTML = `<span class="muted">${scenarioFor(selected).label} selected. Run the recorded replay.</span>`;
  if (result) {
    result.className = "terminal-result";
    result.innerHTML = `<span class="result-label">Ready to replay</span><span class="result-meta">No database connection is made in this demo.</span>`;
  }
}

function escapeHtml(value: string) {
  const span = document.createElement("span");
  span.textContent = value;
  return span.innerHTML;
}

function runReplay() {
  if (!terminal || !result || !runButton) return;
  const scenario = scenarioFor(selected);
  window.clearTimeout(replayTimer);
  runButton.disabled = true;
  runButton.innerHTML = `<span class="spinner" aria-hidden="true"></span> Replaying ${scenario.label.toLowerCase()}…`;
  terminal.innerHTML = `<span class="muted">Starting isolated replay…</span>`;
  result.className = "terminal-result loading";
  result.innerHTML = `<span class="result-label">Replay in progress</span><span class="result-meta">Starting disposable Postgres</span>`;
  replayTimer = window.setTimeout(() => {
    terminal.innerHTML = scenario.lines
      .map(([tone, line]: [string, string]) => `<span class="${tone}">${escapeHtml(line)}</span>`)
      .join("\n");
    result.className = `terminal-result ${scenario.status}`;
    result.innerHTML = `<span class="result-label">${scenario.verdict}</span><span class="result-meta">${scenario.meta}</span>`;
    runButton.disabled = false;
    runButton.innerHTML = `<span aria-hidden="true">↻</span> Replay again`;
  }, window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 0 : 650);
}

for (const [index, tab] of tabs.entries()) {
  tab.addEventListener("click", () => selectScenario(tab));
  tab.addEventListener("keydown", (event) => {
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    let next = index;
    if (event.key === "ArrowLeft") next = (index - 1 + tabs.length) % tabs.length;
    if (event.key === "ArrowRight") next = (index + 1) % tabs.length;
    if (event.key === "Home") next = 0;
    if (event.key === "End") next = tabs.length - 1;
    selectScenario(tabs[next], true);
  });
}

runButton?.addEventListener("click", runReplay);

for (const button of document.querySelectorAll<HTMLButtonElement>("[data-copy-target]")) {
  button.addEventListener("click", async () => {
    const target = document.getElementById(button.dataset.copyTarget ?? "");
    if (!target) return;
    const original = button.textContent ?? "Copy";
    try {
      await navigator.clipboard.writeText(target.innerText);
      button.textContent = "Copied";
    } catch {
      button.textContent = "Select text to copy";
      target.focus();
    }
    window.setTimeout(() => (button.textContent = original), 1600);
  });
}

const offlineBanner = document.querySelector<HTMLElement>("#offline-banner");
function updateNetworkState() {
  if (offlineBanner) offlineBanner.hidden = navigator.onLine;
}
window.addEventListener("online", updateNetworkState);
window.addEventListener("offline", updateNetworkState);
updateNetworkState();

if ("serviceWorker" in navigator && location.protocol !== "http:") {
  window.addEventListener("load", () => navigator.serviceWorker.register("/sw.js"));
}
