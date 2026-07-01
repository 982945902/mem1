// Async HTTP client for the mem1 server (native fetch, all methods return Promises).

import type {
  AddResponse,
  ChatMessage,
  DeleteAllResponse,
  Filters,
  HistoryResponse,
  MemoryResult,
  SearchResponse,
  UsersResponse,
} from "./types.js";

export class ClientError extends Error {
  code: string;
  traceId?: string;
  constructor(code: string, message: string, traceId?: string) {
    super(`[${code}] ${message}`);
    this.name = "ClientError";
    this.code = code;
    this.traceId = traceId;
  }
}

export interface Mem1ClientOptions {
  baseUrl?: string;
  apiKey?: string;
  timeoutMs?: number;
}

export class Mem1Client {
  private baseUrl: string;
  private apiKey?: string;
  private timeoutMs: number;

  constructor(opts: Mem1ClientOptions = {}) {
    this.baseUrl = (opts.baseUrl ?? "http://127.0.0.1:8080").replace(/\/+$/, "");
    this.apiKey = opts.apiKey;
    this.timeoutMs = opts.timeoutMs ?? 120_000;
  }

  private headers(): Record<string, string> {
    const h: Record<string, string> = { "Content-Type": "application/json" };
    if (this.apiKey) h["Authorization"] = `Bearer ${this.apiKey}`;
    return h;
  }

  private async request(
    method: string,
    path: string,
    opts: { query?: Record<string, unknown>; body?: unknown } = {},
  ): Promise<Response> {
    const url = new URL(this.baseUrl + path);
    for (const [k, v] of Object.entries(opts.query ?? {})) {
      if (v !== undefined && v !== null) url.searchParams.set(k, String(v));
    }
    const ctrl = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), this.timeoutMs);
    try {
      return await fetch(url, {
        method,
        headers: this.headers(),
        body: opts.body !== undefined ? JSON.stringify(opts.body) : undefined,
        signal: ctrl.signal,
      });
    } finally {
      clearTimeout(timer);
    }
  }

  private async raise(r: Response): Promise<never> {
    let body: Record<string, unknown> = {};
    if ((r.headers.get("content-type") ?? "").startsWith("application/json")) {
      body = (await r.json().catch(() => ({}))) as Record<string, unknown>;
    }
    throw new ClientError(
      (body.code as string) ?? "UNKNOWN",
      (body.message as string) ?? (await r.text().catch(() => r.statusText)),
      body.trace_id as string | undefined,
    );
  }

  async add(userId: string, content: string, metadata: Filters = {}): Promise<AddResponse> {
    const r = await this.request("POST", "/memories", {
      body: { user_id: userId, content, metadata },
    });
    if (r.status !== 201) return this.raise(r);
    return (await r.json()) as AddResponse;
  }

  async addMessages(
    userId: string,
    messages: ChatMessage[],
    metadata: Filters = {},
  ): Promise<AddResponse> {
    const r = await this.request("POST", "/memories", {
      body: { user_id: userId, messages, metadata },
    });
    if (r.status !== 201) return this.raise(r);
    return (await r.json()) as AddResponse;
  }

  async search(
    userId: string,
    query: string,
    limit = 10,
    filters: Filters = {},
  ): Promise<SearchResponse> {
    const r = await this.request("POST", "/memories/search", {
      body: { user_id: userId, query, limit, filters },
    });
    if (r.status !== 200) return this.raise(r);
    return (await r.json()) as SearchResponse;
  }

  async list(
    userId: string,
    limit = 10,
    offset = 0,
    filters: Filters = {},
  ): Promise<AddResponse> {
    const r = await this.request("GET", "/memories", {
      query: { user_id: userId, limit, offset, ...filters },
    });
    if (r.status !== 200) return this.raise(r);
    return (await r.json()) as AddResponse;
  }

  async get(memoryId: string, userId: string): Promise<MemoryResult | null> {
    const r = await this.request("GET", `/memories/${memoryId}`, {
      query: { user_id: userId },
    });
    if (r.status === 404) return null;
    if (r.status !== 200) return this.raise(r);
    return (await r.json()) as MemoryResult;
  }

  async update(
    memoryId: string,
    userId: string,
    content?: string | null,
    metadata: Filters = {},
  ): Promise<MemoryResult> {
    const r = await this.request("PATCH", `/memories/${memoryId}`, {
      body: { user_id: userId, content: content ?? null, metadata },
    });
    if (r.status !== 200) return this.raise(r);
    return (await r.json()) as MemoryResult;
  }

  async delete(memoryId: string, userId: string): Promise<boolean> {
    const r = await this.request("DELETE", `/memories/${memoryId}`, {
      query: { user_id: userId },
    });
    if (r.status === 404) return false;
    if (r.status !== 204) return this.raise(r);
    return true;
  }

  async deleteAll(userId: string, filters: Filters = {}): Promise<DeleteAllResponse> {
    const r = await this.request("DELETE", "/memories", {
      query: { user_id: userId, ...filters },
    });
    if (r.status !== 200) return this.raise(r);
    return (await r.json()) as DeleteAllResponse;
  }

  async history(memoryId: string, userId: string): Promise<HistoryResponse> {
    const r = await this.request("GET", `/memories/${memoryId}/history`, {
      query: { user_id: userId },
    });
    if (r.status !== 200) return this.raise(r);
    return (await r.json()) as HistoryResponse;
  }

  async users(): Promise<UsersResponse> {
    const r = await this.request("GET", "/users");
    if (r.status !== 200) return this.raise(r);
    return (await r.json()) as UsersResponse;
  }

  async reset(): Promise<DeleteAllResponse> {
    const r = await this.request("POST", "/reset");
    if (r.status !== 200) return this.raise(r);
    return (await r.json()) as DeleteAllResponse;
  }
}
