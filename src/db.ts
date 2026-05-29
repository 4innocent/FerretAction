import Database from "@tauri-apps/plugin-sql";
import type { Workflow, Folder, WorkflowNode, WorkflowEdge } from "./types";

function generateId(): string {
  return crypto.randomUUID();
}

let db: Database | null = null;

async function getDb(): Promise<Database> {
  if (!db) {
    db = await Database.load("sqlite:ferretaction.db");
  }
  return db;
}

// ──────────────────────────────────────
// Initialization
// ──────────────────────────────────────

export async function initDb(): Promise<void> {
  const database = await getDb();
  await database.execute("PRAGMA journal_mode=WAL");
  await database.execute("PRAGMA foreign_keys=ON");

  await database.execute(`
    CREATE TABLE IF NOT EXISTS folders (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      parent_id TEXT,
      created_at TEXT NOT NULL DEFAULT (datetime('now')),
      FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE SET NULL
    )
  `);

  await database.execute(`
    CREATE TABLE IF NOT EXISTS workflows (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT NOT NULL DEFAULT '',
      folder_id TEXT,
      status TEXT NOT NULL DEFAULT 'idle',
      created_at TEXT NOT NULL DEFAULT (datetime('now')),
      updated_at TEXT NOT NULL DEFAULT (datetime('now')),
      FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
    )
  `);

  await database.execute(`
    CREATE TABLE IF NOT EXISTS workflow_nodes (
      id TEXT PRIMARY KEY,
      workflow_id TEXT NOT NULL,
      type TEXT NOT NULL,
      label TEXT NOT NULL,
      x REAL NOT NULL DEFAULT 0.0,
      y REAL NOT NULL DEFAULT 0.0,
      config TEXT NOT NULL DEFAULT '{}',
      FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE
    )
  `);

  await database.execute(`
    CREATE TABLE IF NOT EXISTS workflow_edges (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      workflow_id TEXT NOT NULL,
      source_node_id TEXT NOT NULL,
      target_node_id TEXT NOT NULL,
      label TEXT,
      FOREIGN KEY (workflow_id) REFERENCES workflows(id) ON DELETE CASCADE,
      FOREIGN KEY (source_node_id) REFERENCES workflow_nodes(id) ON DELETE CASCADE,
      FOREIGN KEY (target_node_id) REFERENCES workflow_nodes(id) ON DELETE CASCADE
    )
  `);

  await database.execute(`
    CREATE TABLE IF NOT EXISTS settings (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    )
  `);
}

// ──────────────────────────────────────
// Folders
// ──────────────────────────────────────

export async function listFolders(): Promise<Folder[]> {
  const database = await getDb();
  const rows = await database.select<Folder[]>("SELECT id, name, parent_id, created_at FROM folders ORDER BY created_at");
  return rows.map(toFolder);
}

export async function createFolder(name: string, parentId?: string): Promise<Folder> {
  const database = await getDb();
  const id = generateId();
  await database.execute("INSERT INTO folders (id, name, parent_id) VALUES ($1, $2, $3)", [id, name, parentId || null]);
  return { id, name, parentId, createdAt: new Date().toISOString() };
}

export async function deleteFolder(id: string): Promise<void> {
  const database = await getDb();
  await database.execute("DELETE FROM folders WHERE id = $1", [id]);
}

export async function renameFolder(id: string, name: string): Promise<void> {
  const database = await getDb();
  await database.execute("UPDATE folders SET name = $1 WHERE id = $2", [name, id]);
}

// ──────────────────────────────────────
// Workflows
// ──────────────────────────────────────

export async function listWorkflows(): Promise<Workflow[]> {
  const database = await getDb();
  const rows = await database.select<any[]>("SELECT id, name, description, folder_id, status, created_at, updated_at FROM workflows ORDER BY updated_at DESC");
  return rows.map(toWorkflow);
}

export async function getWorkflow(id: string): Promise<{
  workflow: Workflow;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
}> {
  const database = await getDb();

  const wfRows = await database.select<any[]>("SELECT * FROM workflows WHERE id = $1", [id]);
  if (wfRows.length === 0) throw new Error(`Workflow ${id} not found`);
  const workflow = toWorkflow(wfRows[0]);

  const nodeRows = await database.select<any[]>("SELECT id, type, label, x, y, config FROM workflow_nodes WHERE workflow_id = $1", [id]);
  const nodes: WorkflowNode[] = nodeRows.map((r) => ({
    id: r.id,
    type: r.type,
    label: r.label,
    x: r.x,
    y: r.y,
    config: JSON.parse(r.config || "{}"),
  }));

  const edgeRows = await database.select<any[]>("SELECT source_node_id, target_node_id, label FROM workflow_edges WHERE workflow_id = $1", [id]);
  const edges: WorkflowEdge[] = edgeRows.map((r) => ({
    source: r.source_node_id,
    target: r.target_node_id,
    label: r.label || undefined,
  }));

  return { workflow, nodes, edges };
}

export async function createWorkflow(name: string, description?: string, folderId?: string): Promise<Workflow> {
  const database = await getDb();
  const id = generateId();
  const now = new Date().toISOString();
  await database.execute(
    "INSERT INTO workflows (id, name, description, folder_id, status, created_at, updated_at) VALUES ($1, $2, $3, $4, 'idle', $5, $5)",
    [id, name, description || "", folderId || null, now],
  );
  return { id, name, description, folderId, status: "idle", createdAt: now, updatedAt: now };
}

export async function updateWorkflow(id: string, name: string, description?: string): Promise<void> {
  const database = await getDb();
  await database.execute(
    "UPDATE workflows SET name = $1, description = $2, updated_at = $3 WHERE id = $4",
    [name, description || "", new Date().toISOString(), id],
  );
}

export async function deleteWorkflow(id: string): Promise<void> {
  const database = await getDb();
  await database.execute("DELETE FROM workflows WHERE id = $1", [id]);
}

// ──────────────────────────────────────
// Workflow content (nodes + edges)
// ──────────────────────────────────────

export async function saveWorkflowContent(
  workflowId: string,
  nodes: WorkflowNode[],
  edges: WorkflowEdge[],
): Promise<void> {
  const database = await getDb();

  // Delete old nodes (cascades to edges), then insert new ones
  await database.execute("DELETE FROM workflow_nodes WHERE workflow_id = $1", [workflowId]);

  for (const node of nodes) {
    await database.execute(
      "INSERT INTO workflow_nodes (id, workflow_id, type, label, x, y, config) VALUES ($1, $2, $3, $4, $5, $6, $7)",
      [node.id, workflowId, node.type, node.label, node.x, node.y, JSON.stringify(node.config)],
    );
  }

  for (const edge of edges) {
    await database.execute(
      "INSERT INTO workflow_edges (workflow_id, source_node_id, target_node_id, label) VALUES ($1, $2, $3, $4)",
      [workflowId, edge.source, edge.target, edge.label || null],
    );
  }

  await database.execute("UPDATE workflows SET updated_at = $1 WHERE id = $2", [new Date().toISOString(), workflowId]);
}

// ──────────────────────────────────────
// Helpers
// ──────────────────────────────────────

function toFolder(row: any): Folder {
  return {
    id: row.id,
    name: row.name,
    parentId: row.parent_id || undefined,
    createdAt: row.created_at,
  };
}

function toWorkflow(row: any): Workflow {
  return {
    id: row.id,
    name: row.name,
    description: row.description || undefined,
    folderId: row.folder_id || undefined,
    status: row.status || "idle",
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

// ──────────────────────────────────────
// Settings (key-value store)
// ──────────────────────────────────────

export async function loadSettings(): Promise<Record<string, string>> {
  const database = await getDb();
  const rows = await database.select<{ key: string; value: string }[]>(
    "SELECT key, value FROM settings",
  );
  const map: Record<string, string> = {};
  for (const row of rows) {
    map[row.key] = row.value;
  }
  return map;
}

export async function saveSetting(key: string, value: string): Promise<void> {
  const database = await getDb();
  await database.execute(
    "INSERT INTO settings (key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2",
    [key, value],
  );
}
