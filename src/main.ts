import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./style.css";

type Page = "calendar" | "today" | "plan" | "library" | "stats" | "settings";

type AppConfig = {
  api: {
    base_url: string;
    api_key: string;
    model: string;
  };
  user: {
    name: string;
    timezone: string;
  };
};

type PublicConfig = {
  api: {
    base_url: string;
    api_key: string;
    model: string;
  };
  user: AppConfig["user"];
  is_configured: boolean;
};

type Task = {
  id: string;
  task: string;
  date: string;
  estimated_time: number;
  difficulty: "simple" | "medium" | "hard" | string;
  priority: string;
  tags: string[];
  status: string;
  completed_at: string | null;
  notes: string;
};

type Plan = {
  id: string;
  name: string;
  created_at: string;
  tasks: Task[];
};

type Resource = {
  id: string;
  content: string;
  description: string;
  type: string;
  captured_at: string;
  linked_tasks: string[];
  status: string;
};

type Stats = {
  total_tasks: number;
  completed: number;
  pending: number;
  skipped: number;
  completion_rate: number;
  by_difficulty: Record<string, { total: number; completed: number }>;
};

type BackupInfo = {
  id: string;
  timestamp: string;
  datetime: string;
  reason: string;
  files: string[];
};

type DreamwalkResult = {
  success: boolean;
  ran: boolean;
  date: string;
  message: string;
};

const appRoot = document.querySelector<HTMLDivElement>("#app");
if (!appRoot) {
  throw new Error("Missing app root");
}
const app = appRoot;
const currentWindow = getCurrentWindow();

const state = {
  page: "today" as Page,
  config: null as PublicConfig | null,
  plans: [] as Plan[],
  calendarDate: new Date(),
  calendarView: (localStorage.getItem("mingdeng.calendarView") as "week" | "month") || "week",
  chatOpen: false,
  widgetExpanded: false,
};

const navItems: Array<{ page: Page; label: string; icon: string }> = [
  { page: "calendar", label: "学习日历", icon: "□" },
  { page: "today", label: "今日任务", icon: "✓" },
  { page: "plan", label: "生成计划", icon: "+" },
  { page: "library", label: "资源库", icon: "◇" },
  { page: "stats", label: "学习统计", icon: "%" },
  { page: "settings", label: "设置", icon: "⚙" },
];

function h(value: unknown): string {
  return String(value ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

function today(): string {
  return formatDate(new Date());
}

function formatDate(date: Date): string {
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function parseDate(date: string): Date {
  const [year, month, day] = date.split("-").map(Number);
  return new Date(year, month - 1, day);
}

function formatDisplayDate(date: string): string {
  const parsed = parseDate(date);
  return parsed.toLocaleDateString("zh-CN", {
    year: "numeric",
    month: "long",
    day: "numeric",
    weekday: "long",
  });
}

function minutesText(minutes: number): string {
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) return `${rest}min`;
  if (rest === 0) return `${hours}h`;
  return `${hours}h ${rest}min`;
}

function difficultyLabel(difficulty: string): string {
  if (difficulty === "simple") return "简单";
  if (difficulty === "hard") return "困难";
  return "中等";
}

function toast(message: string): void {
  const existing = document.querySelector(".toast");
  existing?.remove();
  const el = document.createElement("div");
  el.className = "toast";
  el.textContent = message;
  document.body.appendChild(el);
  window.setTimeout(() => el.remove(), 3200);
}

function startWindowDrag(event: MouseEvent): void {
  if (event.button !== 0) return;
  const target = event.target as HTMLElement | null;
  if (target?.closest("button,input,textarea,select,a,label")) return;
  currentWindow.startDragging().catch((error) => {
    toast(`拖动失败: ${String(error)}`);
  });
}

async function call<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    const message = typeof error === "string" ? error : error instanceof Error ? error.message : JSON.stringify(error);
    toast(message);
    throw error;
  }
}

async function init(): Promise<void> {
  state.config = await call<PublicConfig>("get_config");
  if (!state.config.is_configured) {
    state.widgetExpanded = true;
    await call<boolean>("set_widget_expanded", { expanded: true });
    renderLock("请先配置 OpenAI-compatible API。明灯会在配置后执行梦游并进入主应用。");
    return;
  }
  await enterMainAfterDreamwalk();
}

async function enterMainAfterDreamwalk(): Promise<void> {
  renderLock("正在执行梦游，整理长期记忆...");
  try {
    const result = await call<DreamwalkResult>("run_dreamwalk_if_needed");
    toast(result.message);
    await call<boolean>("set_widget_expanded", { expanded: state.widgetExpanded });
    await renderShell();
  } catch {
    state.widgetExpanded = true;
    await call<boolean>("set_widget_expanded", { expanded: true });
    renderLock("梦游失败。根据当前设定，明灯需要完成梦游后才能进入主应用。", true);
  }
}

function renderLock(message: string, allowRetry = false): void {
  app.innerHTML = `
    <div class="lock-screen">
      <section class="lock-panel">
        <div class="panel-body">
          <h1 class="page-title">MingDeng</h1>
          <p class="page-subtitle">${h(message)}</p>
          <div class="grid two" style="margin-top:18px">
            <div class="field">
              <label>API Base URL</label>
              <input class="input" id="lock-base-url" value="${h(state.config?.api.base_url || "https://api.openai.com/v1")}">
            </div>
            <div class="field">
              <label>模型</label>
              <input class="input" id="lock-model" value="${h(state.config?.api.model || "gpt-4o-mini")}">
            </div>
            <div class="field" style="grid-column:1 / -1">
              <label>API Key</label>
              <input class="input" id="lock-api-key" type="password" placeholder="${h(state.config?.api.api_key || "sk-...")}">
            </div>
            <div class="field">
              <label>用户名</label>
              <input class="input" id="lock-user-name" value="${h(state.config?.user.name || "User")}">
            </div>
            <div class="field">
              <label>时区</label>
              ${timezoneSelect("lock-timezone", state.config?.user.timezone || "Asia/Shanghai")}
            </div>
          </div>
          <div class="toolbar" style="margin-top:18px">
            <button class="button primary" id="lock-save">保存并进入</button>
            ${allowRetry ? `<button class="button" id="lock-retry">重试梦游</button>` : ""}
          </div>
        </div>
      </section>
    </div>
  `;
  document.querySelector("#lock-save")?.addEventListener("click", saveLockConfig);
  document.querySelector("#lock-retry")?.addEventListener("click", enterMainAfterDreamwalk);
}

async function saveLockConfig(): Promise<void> {
  const apiKey = inputValue("lock-api-key");
  const config: AppConfig = {
    api: {
      base_url: inputValue("lock-base-url"),
      api_key: apiKey || "",
      model: inputValue("lock-model"),
    },
    user: {
      name: inputValue("lock-user-name") || "User",
      timezone: inputValue("lock-timezone") || "Asia/Shanghai",
    },
  };
  state.config = await call<PublicConfig>("save_config", { config });
  state.widgetExpanded = false;
  await enterMainAfterDreamwalk();
}

async function renderShell(): Promise<void> {
  const peekTasks = await call<Task[]>("tasks_by_date", { date: today() });
  app.innerHTML = `
    <div class="widget-shell ${state.widgetExpanded ? "expanded" : "collapsed"}">
      <header class="widget-titlebar" data-tauri-drag-region>
        <div class="widget-brand" data-tauri-drag-region>
          <strong data-tauri-drag-region>明灯</strong>
          <span data-tauri-drag-region>${h(formatDisplayDate(today()))}</span>
        </div>
        <div class="drag-grip" title="拖动窗口" data-tauri-drag-region>⋮⋮</div>
        <div class="widget-window-actions">
          <button class="window-button backup" id="widget-backup" title="创建备份"></button>
          <button class="window-button minimize" id="widget-minimize" title="最小化"></button>
          <button class="window-button danger" id="widget-close" title="关闭"></button>
        </div>
      </header>
      <section class="todo-dropdown-head">
        <button class="todo-summary" id="widget-toggle">
          <span class="todo-summary-count">${peekTasks.filter((task) => task.status !== "completed").length}</span>
          <span class="todo-summary-text">${state.widgetExpanded ? "收起今日 TODO" : "展开今日 TODO"}</span>
          <span class="todo-summary-caret">${state.widgetExpanded ? "⌃" : "⌄"}</span>
        </button>
        <button class="button primary compact-add" id="peek-add-task">+</button>
      </section>
      <section class="todo-peek">
        ${renderTodoPeek(peekTasks)}
      </section>
      <section class="widget-expanded-area">
        <nav class="widget-tabs">
          ${navItems.map((item) => navButton(item)).join("")}
        </nav>
        <main class="main widget-main" id="main"></main>
      </section>
      <button class="chat-toggle" id="chat-toggle" title="AI 助手">AI</button>
      <div id="chat-root"></div>
    </div>
  `;
  document.querySelector("#widget-toggle")?.addEventListener("click", toggleWidgetExpanded);
  document.querySelector(".widget-titlebar")?.addEventListener("mousedown", (event) => startWindowDrag(event as MouseEvent));
  document.querySelector(".todo-dropdown-head")?.addEventListener("mousedown", (event) => startWindowDrag(event as MouseEvent));
  document.querySelector("#peek-add-task")?.addEventListener("click", async () => {
    if (!state.widgetExpanded) await setWidgetExpanded(true);
    openTaskEditor();
  });
  document.querySelectorAll<HTMLInputElement>(".peek-task-toggle").forEach((checkbox) => {
    checkbox.addEventListener("change", async () => {
      const taskId = checkbox.dataset.taskId || "";
      await call<boolean>(checkbox.checked ? "complete_task" : "uncomplete_task", { taskId });
      await renderShell();
    });
  });
  document.querySelectorAll<HTMLButtonElement>(".nav-button").forEach((button) => {
    button.addEventListener("click", () => navigate(button.dataset.page as Page));
  });
  document.querySelector("#widget-backup")?.addEventListener("click", async () => {
    await call<BackupInfo>("create_backup", { reason: "Manual backup" });
    toast("备份已创建");
  });
  document.querySelector("#widget-minimize")?.addEventListener("click", () => call<boolean>("minimize_window"));
  document.querySelector("#widget-close")?.addEventListener("click", () => call<boolean>("close_window"));
  document.querySelector("#chat-toggle")?.addEventListener("click", toggleChat);
  if (state.widgetExpanded) {
    await navigate(state.page);
  }
}

function renderTodoPeek(tasks: Task[]): string {
  if (!tasks.length) {
    return `<div class="peek-empty">今天没有任务</div>`;
  }
  return `
    <div class="peek-task-list">
      ${tasks
        .slice(0, state.widgetExpanded ? 4 : 2)
        .map(
          (task) => `
            <label class="peek-task ${task.status === "completed" ? "completed" : ""}">
              <input type="checkbox" class="peek-task-toggle" data-task-id="${h(task.id)}" ${task.status === "completed" ? "checked" : ""}>
              <span>${h(task.task)}</span>
              <em>${h(minutesText(task.estimated_time))}</em>
            </label>
          `,
        )
        .join("")}
      ${tasks.length > (state.widgetExpanded ? 4 : 2) ? `<div class="peek-more">+${tasks.length - (state.widgetExpanded ? 4 : 2)} 个更多任务</div>` : ""}
    </div>
  `;
}

async function toggleWidgetExpanded(): Promise<void> {
  await setWidgetExpanded(!state.widgetExpanded);
}

async function setWidgetExpanded(expanded: boolean): Promise<void> {
  state.widgetExpanded = expanded;
  state.chatOpen = expanded ? state.chatOpen : false;
  await call<boolean>("set_widget_expanded", { expanded });
  await renderShell();
}

function navButton(item: { page: Page; label: string; icon: string }): string {
  return `
    <button class="nav-button ${state.page === item.page ? "active" : ""}" data-page="${item.page}">
      <span class="nav-icon">${h(item.icon)}</span>
      <span>${h(item.label)}</span>
    </button>
  `;
}

async function navigate(page: Page): Promise<void> {
  state.page = page;
  document.querySelectorAll(".nav-button").forEach((button) => {
    button.classList.toggle("active", (button as HTMLElement).dataset.page === page);
  });
  if (page === "calendar") await renderCalendar();
  if (page === "today") await renderToday();
  if (page === "plan") await renderPlan();
  if (page === "library") await renderLibrary();
  if (page === "stats") await renderStats();
  if (page === "settings") await renderSettings();
}

function mainEl(): HTMLElement {
  const el = document.querySelector<HTMLElement>("#main");
  if (!el) throw new Error("Missing main element");
  return el;
}

async function loadPlans(): Promise<Plan[]> {
  state.plans = await call<Plan[]>("list_plans");
  return state.plans;
}

function allTasks(): Task[] {
  return state.plans.flatMap((plan) => plan.tasks);
}

async function renderCalendar(): Promise<void> {
  await loadPlans();
  const title = state.calendarView === "week" ? weekTitle(state.calendarDate) : monthTitle(state.calendarDate);
  mainEl().innerHTML = `
    <section class="page">
      <header class="page-header">
        <div>
          <h2 class="page-title">学习日历</h2>
          <p class="page-subtitle">查看和管理学习计划，按周或按月扫描任务安排。</p>
        </div>
        <div class="toolbar">
          <button class="button ${state.calendarView === "week" ? "primary" : ""}" id="view-week">周视图</button>
          <button class="button ${state.calendarView === "month" ? "primary" : ""}" id="view-month">月视图</button>
        </div>
      </header>
      <div class="calendar-nav">
        <button class="button" id="prev-period">上一段</button>
        <h3 class="calendar-title">${h(title)}</h3>
        <button class="button" id="next-period">下一段</button>
      </div>
      <div id="calendar-grid">${state.calendarView === "week" ? weekGrid() : monthGrid()}</div>
      <div class="toolbar" style="margin-top:16px">
        <button class="button primary" id="new-plan">生成学习计划</button>
        <button class="button" id="today-calendar">回到今天</button>
        <button class="button warn" id="reschedule-from-today">从今天重排</button>
        <button class="button warn" id="reschedule-all">包含过期任务重排</button>
      </div>
    </section>
  `;
  document.querySelector("#view-week")?.addEventListener("click", () => switchCalendarView("week"));
  document.querySelector("#view-month")?.addEventListener("click", () => switchCalendarView("month"));
  document.querySelector("#prev-period")?.addEventListener("click", () => movePeriod(-1));
  document.querySelector("#next-period")?.addEventListener("click", () => movePeriod(1));
  document.querySelector("#today-calendar")?.addEventListener("click", () => {
    state.calendarDate = new Date();
    void renderCalendar();
  });
  document.querySelector("#new-plan")?.addEventListener("click", () => navigate("plan"));
  document.querySelector("#reschedule-from-today")?.addEventListener("click", () => reschedule("from_today"));
  document.querySelector("#reschedule-all")?.addEventListener("click", () => reschedule("include_incomplete"));
  document.querySelectorAll<HTMLButtonElement>(".day-cell").forEach((cell) => {
    cell.addEventListener("click", () => openDayModal(cell.dataset.date || today()));
  });
}

function switchCalendarView(view: "week" | "month"): void {
  state.calendarView = view;
  localStorage.setItem("mingdeng.calendarView", view);
  void renderCalendar();
}

function movePeriod(direction: number): void {
  if (state.calendarView === "week") {
    state.calendarDate.setDate(state.calendarDate.getDate() + direction * 7);
  } else {
    state.calendarDate.setMonth(state.calendarDate.getMonth() + direction);
  }
  void renderCalendar();
}

function weekTitle(date: Date): string {
  const start = startOfWeek(date);
  const end = new Date(start);
  end.setDate(start.getDate() + 6);
  return `${formatDate(start)} 至 ${formatDate(end)}`;
}

function monthTitle(date: Date): string {
  return `${date.getFullYear()}年${date.getMonth() + 1}月`;
}

function startOfWeek(date: Date): Date {
  const copy = new Date(date);
  copy.setHours(0, 0, 0, 0);
  copy.setDate(copy.getDate() - copy.getDay());
  return copy;
}

function tasksByDateMap(): Record<string, Task[]> {
  const map: Record<string, Task[]> = {};
  for (const task of allTasks()) {
    map[task.date] ||= [];
    map[task.date].push(task);
  }
  return map;
}

function weekGrid(): string {
  const map = tasksByDateMap();
  const start = startOfWeek(state.calendarDate);
  const names = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  const cells = Array.from({ length: 7 }, (_, index) => {
    const day = new Date(start);
    day.setDate(start.getDate() + index);
    return dayCell(day, map[formatDate(day)] || [], names[index]);
  });
  return `<div class="week-grid">${cells.join("")}</div>`;
}

function monthGrid(): string {
  const map = tasksByDateMap();
  const first = new Date(state.calendarDate.getFullYear(), state.calendarDate.getMonth(), 1);
  const start = startOfWeek(first);
  const names = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  const cells = Array.from({ length: 42 }, (_, index) => {
    const day = new Date(start);
    day.setDate(start.getDate() + index);
    return dayCell(day, map[formatDate(day)] || [], String(day.getDate()), day.getMonth() !== state.calendarDate.getMonth());
  });
  return `
    <div class="month-head">${names.map((name) => `<div>${name}</div>`).join("")}</div>
    <div class="month-grid">${cells.join("")}</div>
  `;
}

function dayCell(day: Date, tasks: Task[], label: string, outside = false): string {
  const date = formatDate(day);
  return `
    <button class="day-cell ${date === today() ? "today" : ""} ${outside ? "outside" : ""}" data-date="${date}">
      <div class="day-label">
        <span>${h(label)}</span>
        <span class="small muted">${tasks.length}</span>
      </div>
      <div class="day-tasks">
        ${tasks
          .slice(0, 4)
          .map((task) => `<div class="day-task" title="${h(task.task)}">${h(difficultyLabel(task.difficulty))} · ${h(task.task)}</div>`)
          .join("")}
        ${tasks.length > 4 ? `<div class="small muted">+${tasks.length - 4} 更多</div>` : ""}
      </div>
    </button>
  `;
}

function openDayModal(date: string): void {
  const tasks = allTasks().filter((task) => task.date === date);
  showModal(`
    <div class="modal-header">
      <h3 class="panel-title">${h(formatDisplayDate(date))}</h3>
      <button class="button icon ghost" data-close>×</button>
    </div>
    <div class="modal-body">
      <div class="task-list">
        ${tasks.length ? tasks.map(renderTaskCard).join("") : `<div class="empty">这一天没有安排任务</div>`}
      </div>
    </div>
  `);
  wireTaskActions();
}

async function reschedule(mode: "from_today" | "include_incomplete"): Promise<void> {
  const result = await call<{ success: boolean; message: string; adjustment_reason?: string }>("reschedule_tasks", {
    request: { mode, plan_id: null },
  });
  toast(result.adjustment_reason ? `${result.message}：${result.adjustment_reason}` : result.message);
  await renderCalendar();
}

async function renderToday(): Promise<void> {
  const date = today();
  const tasks = await call<Task[]>("tasks_by_date", { date });
  mainEl().innerHTML = `
    <section class="page">
      <header class="page-header">
        <div>
          <h2 class="page-title">今日任务</h2>
          <p class="page-subtitle">${h(formatDisplayDate(date))}</p>
        </div>
        <div class="toolbar">
          <button class="button" id="refresh-today">刷新</button>
          <button class="button primary" id="add-task">快速添加任务</button>
        </div>
      </header>
      <div class="task-list">
        ${tasks.length ? tasks.map(renderTaskCard).join("") : `<div class="empty">今天没有任务</div>`}
      </div>
    </section>
  `;
  document.querySelector("#refresh-today")?.addEventListener("click", () => renderToday());
  document.querySelector("#add-task")?.addEventListener("click", () => openTaskEditor());
  wireTaskActions();
}

function renderTaskCard(task: Task): string {
  return `
    <article class="task-card ${task.status === "completed" ? "completed" : ""}" data-task-id="${h(task.id)}">
      <input type="checkbox" class="task-toggle" data-task-id="${h(task.id)}" ${task.status === "completed" ? "checked" : ""}>
      <div>
        <h4 class="task-title">${h(task.task)}</h4>
        <div class="task-meta">
          <span class="badge ${h(task.difficulty)}">${h(difficultyLabel(task.difficulty))}</span>
          <span>${h(task.date)}</span>
          <span>${h(minutesText(task.estimated_time))}</span>
          <span>${h(task.priority)}</span>
        </div>
        ${task.tags.length ? `<div class="tags">${task.tags.map((tag) => `<span class="badge">${h(tag)}</span>`).join("")}</div>` : ""}
        ${task.notes ? `<p class="small muted">${h(task.notes)}</p>` : ""}
      </div>
      <div class="toolbar">
        <button class="button icon task-edit" title="编辑" data-task-id="${h(task.id)}">✎</button>
        <button class="button icon danger task-delete" title="删除" data-task-id="${h(task.id)}">×</button>
      </div>
    </article>
  `;
}

function wireTaskActions(): void {
  document.querySelectorAll<HTMLInputElement>(".task-toggle").forEach((checkbox) => {
    checkbox.addEventListener("change", async () => {
      const taskId = checkbox.dataset.taskId || "";
      await call<boolean>(checkbox.checked ? "complete_task" : "uncomplete_task", { taskId });
      await refreshCurrentPage();
    });
  });
  document.querySelectorAll<HTMLButtonElement>(".task-delete").forEach((button) => {
    button.addEventListener("click", async (event) => {
      event.stopPropagation();
      if (!confirm("确定要删除这个任务吗？")) return;
      await call<boolean>("delete_task", { taskId: button.dataset.taskId || "" });
      closeModal();
      await refreshCurrentPage();
    });
  });
  document.querySelectorAll<HTMLButtonElement>(".task-edit").forEach((button) => {
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      const task = allTasks().find((item) => item.id === button.dataset.taskId);
      if (task) openTaskEditor(task);
    });
  });
}

function openTaskEditor(task?: Task): void {
  showModal(`
    <div class="modal-header">
      <h3 class="panel-title">${task ? "编辑任务" : "添加任务"}</h3>
      <button class="button icon ghost" data-close>×</button>
    </div>
    <div class="modal-body">
      <div class="grid two">
        <div class="field" style="grid-column:1 / -1">
          <label>任务描述</label>
          <input class="input" id="task-title" value="${h(task?.task || "")}">
        </div>
        <div class="field">
          <label>日期</label>
          <input class="input" id="task-date" type="date" value="${h(task?.date || today())}">
        </div>
        <div class="field">
          <label>预计时长</label>
          <input class="input" id="task-time" type="number" min="15" step="15" value="${h(task?.estimated_time || 60)}">
        </div>
        <div class="field">
          <label>难度</label>
          <select class="select" id="task-difficulty">
            ${["simple", "medium", "hard"].map((value) => `<option value="${value}" ${task?.difficulty === value ? "selected" : ""}>${difficultyLabel(value)}</option>`).join("")}
          </select>
        </div>
        <div class="field">
          <label>优先级</label>
          <select class="select" id="task-priority">
            ${["low", "medium", "high"].map((value) => `<option value="${value}" ${task?.priority === value ? "selected" : ""}>${value}</option>`).join("")}
          </select>
        </div>
        <div class="field" style="grid-column:1 / -1">
          <label>标签</label>
          <input class="input" id="task-tags" value="${h(task?.tags.join(", ") || "")}">
        </div>
        <div class="field" style="grid-column:1 / -1">
          <label>备注</label>
          <textarea class="textarea" id="task-notes">${h(task?.notes || "")}</textarea>
        </div>
      </div>
      <div class="toolbar" style="margin-top:16px">
        <button class="button primary" id="task-save">保存</button>
      </div>
    </div>
  `);
  document.querySelector("#task-save")?.addEventListener("click", async () => {
    const payload = {
      task: inputValue("task-title"),
      date: inputValue("task-date"),
      estimated_time: Number(inputValue("task-time") || 60),
      difficulty: inputValue("task-difficulty"),
      priority: inputValue("task-priority"),
      tags: inputValue("task-tags").split(",").map((tag) => tag.trim()).filter(Boolean),
      notes: inputValue("task-notes"),
    };
    if (!payload.task) {
      toast("请输入任务描述");
      return;
    }
    if (task) {
      await call<boolean>("update_task", { taskId: task.id, updates: payload });
    } else {
      await call<Task>("create_task", { task: { ...payload, plan_id: null } });
    }
    closeModal();
    await refreshCurrentPage();
  });
}

async function renderPlan(): Promise<void> {
  const plans = await loadPlans();
  mainEl().innerHTML = `
    <section class="page">
      <header class="page-header">
        <div>
          <h2 class="page-title">生成学习计划</h2>
          <p class="page-subtitle">输入学习目标，AI 会拆解任务并写入日历。</p>
        </div>
      </header>
      <section class="panel">
        <div class="panel-body">
          <div class="field">
            <label>学习目标</label>
            <textarea class="textarea" id="plan-input" placeholder="例如：我想在三周内学习 vLLM、SGLang 和推理优化，每天 2 小时。"></textarea>
          </div>
          <div class="toolbar" style="margin-top:12px">
            <div class="field" style="width:180px">
              <label>开始日期</label>
              <input class="input" id="plan-start-date" type="date" value="${today()}">
            </div>
            <button class="button primary" id="generate-plan" style="align-self:end">生成计划</button>
          </div>
        </div>
      </section>
      <section>
        <h3 class="panel-title">已有计划</h3>
        <div class="grid two">
          ${plans.length ? plans.map(renderPlanCard).join("") : `<div class="empty">暂无计划</div>`}
        </div>
      </section>
    </section>
  `;
  document.querySelector("#generate-plan")?.addEventListener("click", generatePlan);
  document.querySelectorAll<HTMLButtonElement>(".plan-delete").forEach((button) => {
    button.addEventListener("click", async () => {
      if (!confirm("确定要删除这个计划和其中所有任务吗？")) return;
      await call<boolean>("delete_plan", { planId: button.dataset.planId || "" });
      await renderPlan();
    });
  });
  document.querySelectorAll<HTMLButtonElement>(".plan-calendar").forEach((button) => {
    button.addEventListener("click", () => {
      state.calendarDate = new Date();
      void navigate("calendar");
    });
  });
}

function renderPlanCard(plan: Plan): string {
  const completed = plan.tasks.filter((task) => task.status === "completed").length;
  return `
    <article class="card">
      <div class="card-body">
        <h4 class="task-title">${h(plan.name)}</h4>
        <p class="muted small">共 ${plan.tasks.length} 个任务，已完成 ${completed} 个</p>
        <div class="toolbar">
          <button class="button plan-calendar" data-plan-id="${h(plan.id)}">查看日历</button>
          <button class="button danger plan-delete" data-plan-id="${h(plan.id)}">删除</button>
        </div>
      </div>
    </article>
  `;
}

async function generatePlan(): Promise<void> {
  const input = inputValue("plan-input");
  if (!input) {
    toast("请输入学习目标");
    return;
  }
  const button = document.querySelector<HTMLButtonElement>("#generate-plan");
  if (button) button.disabled = true;
  try {
    const plan = await call<Plan>("generate_plan", {
      request: { user_input: input, start_date: inputValue("plan-start-date") },
    });
    toast(`已生成计划：${plan.name}`);
    await renderPlan();
    showModal(`
      <div class="modal-header">
        <h3 class="panel-title">${h(plan.name)}</h3>
        <button class="button icon ghost" data-close>×</button>
      </div>
      <div class="modal-body">
        <div class="task-list">
          ${plan.tasks.map(renderTaskCard).join("")}
        </div>
        <div class="toolbar" style="margin-top:16px">
          <button class="button primary" id="generated-open-calendar">查看日历</button>
        </div>
      </div>
    `);
    document.querySelector("#generated-open-calendar")?.addEventListener("click", () => {
      closeModal();
      void navigate("calendar");
    });
    wireTaskActions();
  } finally {
    if (button) button.disabled = false;
  }
}

async function renderLibrary(): Promise<void> {
  const resources = await call<Resource[]>("list_resources");
  mainEl().innerHTML = `
    <section class="page">
      <header class="page-header">
        <div>
          <h2 class="page-title">资源库</h2>
          <p class="page-subtitle">保存视频、文章、论文和碎片资料，AI 会尝试关联到任务。</p>
        </div>
      </header>
      <section class="panel">
        <div class="panel-body">
          <div class="grid two">
            <div class="field" style="grid-column:1 / -1">
              <label>资源链接或描述</label>
              <input class="input" id="resource-content">
            </div>
            <div class="field">
              <label>简短描述</label>
              <input class="input" id="resource-description">
            </div>
            <div class="field">
              <label>类型</label>
              <select class="select" id="resource-type">
                <option value="video">视频</option>
                <option value="article">文章</option>
                <option value="paper">论文</option>
                <option value="other">其他</option>
              </select>
            </div>
          </div>
          <div class="toolbar" style="margin-top:12px">
            <button class="button primary" id="add-resource">添加资源</button>
          </div>
        </div>
      </section>
      <div class="grid">
        ${resources.length ? resources.map(renderResourceCard).join("") : `<div class="empty">暂无资源</div>`}
      </div>
    </section>
  `;
  document.querySelector("#add-resource")?.addEventListener("click", addResource);
  document.querySelectorAll<HTMLButtonElement>(".resource-delete").forEach((button) => {
    button.addEventListener("click", async () => {
      if (!confirm("确定删除这个资源吗？")) return;
      await call<boolean>("delete_resource", { resourceId: button.dataset.resourceId || "" });
      await renderLibrary();
    });
  });
  document.querySelectorAll<HTMLButtonElement>(".resource-edit").forEach((button) => {
    button.addEventListener("click", async () => {
      const description = prompt("新的资源描述");
      if (description === null) return;
      await call<boolean>("update_resource", {
        resourceId: button.dataset.resourceId || "",
        updates: { description },
      });
      await renderLibrary();
    });
  });
}

function renderResourceCard(resource: Resource): string {
  return `
    <article class="resource-card">
      <div>
        <h4 class="task-title">${h(resource.description || "资源")}</h4>
        <p class="small muted">${h(resource.type)} · ${new Date(resource.captured_at).toLocaleString("zh-CN")} · 关联任务 ${resource.linked_tasks.length}</p>
        <a href="${h(resource.content)}" target="_blank" rel="noreferrer">${h(resource.content)}</a>
      </div>
      <div class="toolbar">
        <button class="button icon resource-edit" data-resource-id="${h(resource.id)}">✎</button>
        <button class="button icon danger resource-delete" data-resource-id="${h(resource.id)}">×</button>
      </div>
    </article>
  `;
}

async function addResource(): Promise<void> {
  const content = inputValue("resource-content");
  if (!content) {
    toast("请输入资源内容");
    return;
  }
  await call<Resource>("create_resource", {
    resource: {
      content,
      description: inputValue("resource-description"),
      type: inputValue("resource-type"),
      auto_link: true,
    },
  });
  await renderLibrary();
}

async function renderStats(): Promise<void> {
  const stats = await call<Stats>("get_stats");
  mainEl().innerHTML = `
    <section class="page">
      <header class="page-header">
        <div>
          <h2 class="page-title">学习统计</h2>
          <p class="page-subtitle">按任务数量和难度查看完成情况。</p>
        </div>
      </header>
      <div class="grid three">
        ${statCard("总任务", stats.total_tasks)}
        ${statCard("已完成", stats.completed)}
        ${statCard("完成率", `${stats.completion_rate}%`)}
      </div>
      <section class="panel" style="margin-top:18px">
        <div class="panel-body">
          <h3 class="panel-title">难度分布</h3>
          ${["simple", "medium", "hard"].map((key) => difficultyStat(key, stats.by_difficulty[key] || { total: 0, completed: 0 })).join("")}
        </div>
      </section>
    </section>
  `;
}

function statCard(label: string, value: string | number): string {
  return `
    <div class="stats-card">
      <p class="muted small">${h(label)}</p>
      <p class="stats-number">${h(value)}</p>
    </div>
  `;
}

function difficultyStat(key: string, value: { total: number; completed: number }): string {
  const pct = value.total ? Math.round((value.completed / value.total) * 100) : 0;
  return `
    <div style="margin-bottom:14px">
      <div class="toolbar" style="justify-content:space-between">
        <strong>${difficultyLabel(key)}</strong>
        <span class="muted small">${value.completed}/${value.total}</span>
      </div>
      <div class="progress"><span style="width:${pct}%"></span></div>
    </div>
  `;
}

async function renderSettings(): Promise<void> {
  state.config = await call<PublicConfig>("get_config");
  const backups = await call<BackupInfo[]>("list_backups");
  mainEl().innerHTML = `
    <section class="page">
      <header class="page-header">
        <div>
          <h2 class="page-title">设置</h2>
          <p class="page-subtitle">配置 AI、用户信息、备份和记忆。</p>
        </div>
      </header>
      <section class="panel">
        <div class="panel-body">
          <h3 class="panel-title">API 配置</h3>
          <div class="grid two">
            <div class="field">
              <label>API Base URL</label>
              <input class="input" id="settings-base-url" value="${h(state.config.api.base_url)}">
            </div>
            <div class="field">
              <label>模型</label>
              <input class="input" id="settings-model" value="${h(state.config.api.model)}">
            </div>
            <div class="field" style="grid-column:1 / -1">
              <label>API Key</label>
              <input class="input" id="settings-api-key" type="password" placeholder="${h(state.config.api.api_key || "sk-...")}">
            </div>
          </div>
          <div class="toolbar" style="margin-top:14px">
            <button class="button primary" id="save-api-settings">保存 API 配置</button>
          </div>
        </div>
      </section>
      <section class="panel">
        <div class="panel-body">
          <h3 class="panel-title">用户设置</h3>
          <div class="grid two">
            <div class="field">
              <label>用户名</label>
              <input class="input" id="settings-user-name" value="${h(state.config.user.name)}">
            </div>
            <div class="field">
              <label>时区</label>
              ${timezoneSelect("settings-timezone", state.config.user.timezone)}
            </div>
          </div>
          <div class="toolbar" style="margin-top:14px">
            <button class="button primary" id="save-user-settings">保存用户设置</button>
            <button class="button warn" id="run-dreamwalk">立即梦游</button>
            <button class="button danger" id="clear-memory">清除记忆</button>
          </div>
        </div>
      </section>
      <section class="panel">
        <div class="panel-body">
          <div class="toolbar" style="justify-content:space-between">
            <h3 class="panel-title">备份</h3>
            <button class="button success" id="create-settings-backup">创建备份</button>
          </div>
          <div class="grid">
            ${backups.length ? backups.map(renderBackup).join("") : `<div class="empty">暂无备份</div>`}
          </div>
        </div>
      </section>
    </section>
  `;
  document.querySelector("#save-api-settings")?.addEventListener("click", saveApiSettings);
  document.querySelector("#save-user-settings")?.addEventListener("click", saveUserSettings);
  document.querySelector("#run-dreamwalk")?.addEventListener("click", async () => {
    const result = await call<DreamwalkResult>("run_dreamwalk_if_needed");
    toast(result.message);
  });
  document.querySelector("#clear-memory")?.addEventListener("click", async () => {
    if (!confirm("确定要清除 Markdown 记忆吗？")) return;
    await call<boolean>("clear_memory");
    toast("记忆已清除");
  });
  document.querySelector("#create-settings-backup")?.addEventListener("click", async () => {
    await call<BackupInfo>("create_backup", { reason: "Manual backup" });
    await renderSettings();
  });
  document.querySelectorAll<HTMLButtonElement>(".backup-restore").forEach((button) => {
    button.addEventListener("click", async () => {
      if (!confirm("确定恢复这个备份吗？当前数据会先自动备份。")) return;
      await call<BackupInfo>("restore_backup", { backupId: button.dataset.backupId || "" });
      await refreshCurrentPage();
    });
  });
  document.querySelectorAll<HTMLButtonElement>(".backup-delete").forEach((button) => {
    button.addEventListener("click", async () => {
      if (!confirm("确定删除这个备份吗？")) return;
      await call<boolean>("delete_backup", { backupId: button.dataset.backupId || "" });
      await renderSettings();
    });
  });
}

function renderBackup(backup: BackupInfo): string {
  return `
    <article class="resource-card">
      <div>
        <h4 class="task-title">${h(new Date(backup.datetime).toLocaleString("zh-CN"))}</h4>
        <p class="small muted">${h(backup.reason)} · ${backup.files.join(", ")}</p>
      </div>
      <div class="toolbar">
        <button class="button backup-restore" data-backup-id="${h(backup.id)}">恢复</button>
        <button class="button danger backup-delete" data-backup-id="${h(backup.id)}">删除</button>
      </div>
    </article>
  `;
}

async function saveApiSettings(): Promise<void> {
  const existing = state.config;
  if (!existing) return;
  const apiKey = inputValue("settings-api-key");
  const config: AppConfig = {
    api: {
      base_url: inputValue("settings-base-url"),
      api_key: apiKey || "",
      model: inputValue("settings-model"),
    },
    user: existing.user,
  };
  state.config = await call<PublicConfig>("save_config", { config });
  toast("API 配置已保存");
}

async function saveUserSettings(): Promise<void> {
  const existing = state.config;
  if (!existing) return;
  const apiKey = inputValue("settings-api-key");
  const config: AppConfig = {
    api: {
      base_url: existing.api.base_url,
      api_key: apiKey || "",
      model: existing.api.model,
    },
    user: {
      name: inputValue("settings-user-name") || "User",
      timezone: inputValue("settings-timezone") || "Asia/Shanghai",
    },
  };
  state.config = await call<PublicConfig>("save_config", { config });
  toast("用户设置已保存");
}

function timezoneSelect(id: string, selected: string): string {
  const zones = ["Asia/Shanghai", "America/New_York", "Europe/London", "Asia/Tokyo", "UTC"];
  return `
    <select class="select" id="${id}">
      ${zones.map((zone) => `<option value="${zone}" ${selected === zone ? "selected" : ""}>${zone}</option>`).join("")}
    </select>
  `;
}

function toggleChat(): void {
  state.chatOpen = !state.chatOpen;
  renderChat();
}

function renderChat(): void {
  const root = document.querySelector("#chat-root");
  if (!root) return;
  if (!state.chatOpen) {
    root.innerHTML = "";
    return;
  }
  root.innerHTML = `
    <section class="chat-panel">
      <div class="chat-header">
        <strong style="flex:1">AI 助手</strong>
        <button class="button icon ghost" id="chat-close">×</button>
      </div>
      <div class="chat-messages" id="chat-messages">
        <div class="message assistant">你好，我会结合 Markdown 长期记忆回答学习问题。</div>
      </div>
      <div class="chat-input">
        <input class="input" id="chat-text" placeholder="输入消息">
        <button class="button primary" id="chat-send">发送</button>
      </div>
    </section>
  `;
  document.querySelector("#chat-close")?.addEventListener("click", toggleChat);
  document.querySelector("#chat-send")?.addEventListener("click", sendChat);
  document.querySelector("#chat-text")?.addEventListener("keydown", (event) => {
    if ((event as KeyboardEvent).key === "Enter") void sendChat();
  });
}

async function sendChat(): Promise<void> {
  const input = document.querySelector<HTMLInputElement>("#chat-text");
  const messages = document.querySelector<HTMLElement>("#chat-messages");
  if (!input || !messages) return;
  const message = input.value.trim();
  if (!message) return;
  input.value = "";
  messages.insertAdjacentHTML("beforeend", `<div class="message user">${h(message)}</div>`);
  messages.scrollTop = messages.scrollHeight;
  try {
    const response = await call<string>("send_chat", { request: { message } });
    messages.insertAdjacentHTML("beforeend", `<div class="message assistant">${h(response)}</div>`);
  } catch {
    messages.insertAdjacentHTML("beforeend", `<div class="message assistant">发送失败</div>`);
  }
  messages.scrollTop = messages.scrollHeight;
}

function showModal(content: string): void {
  closeModal();
  const wrapper = document.createElement("div");
  wrapper.className = "modal-backdrop";
  wrapper.innerHTML = `<section class="modal">${content}</section>`;
  document.body.appendChild(wrapper);
  wrapper.addEventListener("click", (event) => {
    if (event.target === wrapper || (event.target as HTMLElement).hasAttribute("data-close")) closeModal();
  });
}

function closeModal(): void {
  document.querySelector(".modal-backdrop")?.remove();
}

function inputValue(id: string): string {
  const field = document.querySelector<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>(`#${id}`);
  return field?.value.trim() || "";
}

async function refreshCurrentPage(): Promise<void> {
  closeModal();
  await navigate(state.page);
}

void init();
