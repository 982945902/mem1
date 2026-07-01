# Async HTTP client for the mem1 server.

from typing import Any, Optional

import httpx

from mem1.models import (
    AddResponse,
    DeleteAllResponse,
    HistoryResponse,
    MemoryResult,
    SearchResponse,
    UsersResponse,
)


class ClientError(Exception):
    def __init__(self, code: str, message: str, trace_id: Optional[str] = None):
        self.code = code
        self.message = message
        self.trace_id = trace_id
        super().__init__(f"[{code}] {message}")


class Mem1Client:
    """Async HTTP client for mem1. Reuses a single httpx.AsyncClient across calls.

    Use as an async context manager, or call ``aclose()`` when done:

        async with Mem1Client() as c:
            await c.add(user_id="u", content="...")
    """

    def __init__(
        self,
        base_url: str = "http://127.0.0.1:8080",
        api_key: Optional[str] = None,
        timeout: float = 120.0,
    ):
        self.base_url = base_url.rstrip("/")
        self._api_key = api_key
        headers = {"Content-Type": "application/json"}
        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"
        self._http = httpx.AsyncClient(base_url=self.base_url, headers=headers, timeout=timeout)

    async def __aenter__(self) -> "Mem1Client":
        return self

    async def __aexit__(self, *exc: Any) -> None:
        await self.aclose()

    async def aclose(self) -> None:
        await self._http.aclose()

    @staticmethod
    def _raise(r: httpx.Response) -> None:
        body = r.json() if r.headers.get("content-type", "").startswith("application/json") else {}
        raise ClientError(
            body.get("code", "UNKNOWN"),
            body.get("message", r.text),
            body.get("trace_id"),
        )

    async def add(self, user_id: str, content: str, metadata: Optional[dict] = None) -> AddResponse:
        r = await self._http.post(
            "/memories",
            json={"user_id": user_id, "content": content, "metadata": metadata or {}},
        )
        if r.status_code != 201:
            self._raise(r)
        return AddResponse.model_validate(r.json())

    async def add_messages(
        self,
        user_id: str,
        messages: list[dict[str, str]],
        metadata: Optional[dict] = None,
    ) -> AddResponse:
        r = await self._http.post(
            "/memories",
            json={"user_id": user_id, "messages": messages, "metadata": metadata or {}},
        )
        if r.status_code != 201:
            self._raise(r)
        return AddResponse.model_validate(r.json())

    async def search(
        self,
        user_id: str,
        query: str,
        limit: int = 10,
        filters: Optional[dict[str, Any]] = None,
    ) -> SearchResponse:
        r = await self._http.post(
            "/memories/search",
            json={"user_id": user_id, "query": query, "limit": limit, "filters": filters or {}},
        )
        if r.status_code != 200:
            self._raise(r)
        return SearchResponse.model_validate(r.json())

    async def list(
        self,
        user_id: str,
        limit: int = 10,
        offset: int = 0,
        filters: Optional[dict[str, Any]] = None,
    ) -> AddResponse:
        params: dict[str, Any] = {"user_id": user_id, "limit": limit, "offset": offset}
        params.update(filters or {})
        r = await self._http.get("/memories", params=params)
        if r.status_code != 200:
            self._raise(r)
        return AddResponse.model_validate(r.json())

    async def get(self, memory_id: str, user_id: str) -> Optional[MemoryResult]:
        r = await self._http.get(f"/memories/{memory_id}", params={"user_id": user_id})
        if r.status_code == 404:
            return None
        if r.status_code != 200:
            self._raise(r)
        return MemoryResult.model_validate(r.json())

    async def update(
        self,
        memory_id: str,
        user_id: str,
        content: Optional[str] = None,
        metadata: Optional[dict[str, Any]] = None,
    ) -> MemoryResult:
        r = await self._http.patch(
            f"/memories/{memory_id}",
            json={"user_id": user_id, "content": content, "metadata": metadata or {}},
        )
        if r.status_code != 200:
            self._raise(r)
        return MemoryResult.model_validate(r.json())

    async def delete(self, memory_id: str, user_id: str) -> bool:
        r = await self._http.delete(f"/memories/{memory_id}", params={"user_id": user_id})
        if r.status_code == 404:
            return False
        if r.status_code != 204:
            self._raise(r)
        return True

    async def delete_all(
        self,
        user_id: str,
        filters: Optional[dict[str, Any]] = None,
    ) -> DeleteAllResponse:
        params: dict[str, Any] = {"user_id": user_id}
        params.update(filters or {})
        r = await self._http.request("DELETE", "/memories", params=params)
        if r.status_code != 200:
            self._raise(r)
        return DeleteAllResponse.model_validate(r.json())

    async def history(self, memory_id: str, user_id: str) -> HistoryResponse:
        r = await self._http.get(f"/memories/{memory_id}/history", params={"user_id": user_id})
        if r.status_code != 200:
            self._raise(r)
        return HistoryResponse.model_validate(r.json())

    async def users(self) -> UsersResponse:
        r = await self._http.get("/users")
        if r.status_code != 200:
            self._raise(r)
        return UsersResponse.model_validate(r.json())

    async def reset(self) -> DeleteAllResponse:
        r = await self._http.post("/reset")
        if r.status_code != 200:
            self._raise(r)
        return DeleteAllResponse.model_validate(r.json())
