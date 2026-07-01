// Request/response types, aligned with the mem1 HTTP API.

export interface MemoryResult {
  id: string;
  content: string;
  user_id: string;
  metadata: Record<string, unknown>;
  created_at: string;
  score?: number | null;
}

export interface AddResponse {
  results: MemoryResult[];
}

export interface SearchResponse {
  results: MemoryResult[];
  formatted_context?: string | null;
}

export interface DeleteAllResponse {
  deleted: number;
}

export interface UsersResponse {
  users: string[];
}

export interface MemoryHistoryResult {
  id: string;
  memory_id: string;
  user_id: string;
  operation: string;
  previous?: MemoryResult | null;
  current?: MemoryResult | null;
  created_at: string;
}

export interface HistoryResponse {
  results: MemoryHistoryResult[];
}

export interface ChatMessage {
  role: string;
  content: string;
}

export type Filters = Record<string, unknown>;

export interface SessionResult {
  id: string;
  user_id: string;
  name?: string | null;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface SessionsResponse {
  results: SessionResult[];
}
