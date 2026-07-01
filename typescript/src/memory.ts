// mem0-style async Memory class over the mem1 HTTP API.

import { Mem1Client, type Mem1ClientOptions } from "./client.js";
import type {
  AddResponse,
  ChatMessage,
  DeleteAllResponse,
  Filters,
  HistoryResponse,
  MemoryResult,
  SearchResponse,
  SessionResult,
  SessionsResponse,
  UsersResponse,
} from "./types.js";

const DEFAULT_USER = "default_user";

export class Memory {
  private client: Mem1Client;

  constructor(opts: Mem1ClientOptions = {}) {
    this.client = new Mem1Client(opts);
  }

  async add(
    messages: string | ChatMessage[],
    userId: string = DEFAULT_USER,
    metadata: Filters = {},
  ): Promise<AddResponse> {
    if (typeof messages === "string") {
      return this.client.add(userId, messages, metadata);
    }
    const normalized = messages
      .filter((m) => m && m.content)
      .map((m) => ({ role: String(m.role ?? "message"), content: String(m.content ?? "") }));
    const payload = normalized.length
      ? normalized
      : [{ role: "message", content: "(no content)" }];
    return this.client.addMessages(userId, payload, metadata);
  }

  async search(
    query: string,
    userId: string = DEFAULT_USER,
    limit = 10,
    filters: Filters = {},
  ): Promise<SearchResponse> {
    return this.client.search(userId, query, limit, filters);
  }

  async getAll(
    userId: string = DEFAULT_USER,
    limit = 10,
    offset = 0,
    filters: Filters = {},
  ): Promise<AddResponse> {
    return this.client.list(userId, limit, offset, filters);
  }

  async get(memoryId: string, userId: string = DEFAULT_USER): Promise<MemoryResult | null> {
    return this.client.get(memoryId, userId);
  }

  async update(
    memoryId: string,
    data?: string | null,
    userId: string = DEFAULT_USER,
    metadata: Filters = {},
  ): Promise<MemoryResult> {
    return this.client.update(memoryId, userId, data ?? null, metadata);
  }

  async delete(memoryId: string, userId: string = DEFAULT_USER): Promise<boolean> {
    return this.client.delete(memoryId, userId);
  }

  async deleteAll(userId: string = DEFAULT_USER, filters: Filters = {}): Promise<DeleteAllResponse> {
    return this.client.deleteAll(userId, filters);
  }

  async history(memoryId: string, userId: string = DEFAULT_USER): Promise<HistoryResponse> {
    return this.client.history(memoryId, userId);
  }

  async users(): Promise<UsersResponse> {
    return this.client.users();
  }

  async reset(): Promise<DeleteAllResponse> {
    return this.client.reset();
  }

  async createSession(
    userId: string = DEFAULT_USER,
    opts: { id?: string; name?: string; metadata?: Filters } = {},
  ): Promise<SessionResult> {
    return this.client.createSession(userId, opts);
  }

  async listSessions(userId: string = DEFAULT_USER): Promise<SessionsResponse> {
    return this.client.listSessions(userId);
  }

  async getSession(
    sessionId: string,
    userId: string = DEFAULT_USER,
  ): Promise<SessionResult | null> {
    return this.client.getSession(sessionId, userId);
  }

  async deleteSession(
    sessionId: string,
    userId: string = DEFAULT_USER,
    cascade = false,
  ): Promise<DeleteAllResponse> {
    return this.client.deleteSession(sessionId, userId, cascade);
  }
}
