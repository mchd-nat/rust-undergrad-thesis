# app.py
import asyncio
import os
from pathlib import Path
from typing import List, Optional
from uuid import uuid4

import httpx
from bs4 import BeautifulSoup
from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import FileResponse, JSONResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, HttpUrl

app = FastAPI()

BASE_DIR = Path(__file__).parent

app.mount("/static", StaticFiles(directory=BASE_DIR / "static"), name="static")
app.mount("/assets", StaticFiles(directory=BASE_DIR / "assets"), name="assets")
app.mount("/scripts", StaticFiles(directory=BASE_DIR / "scripts"), name="scripts")

@app.get("/", response_class=FileResponse)
async def root():
    return BASE_DIR / "index.html"


class CrawlerRequest(BaseModel):
    url: HttpUrl


class CheckResult(BaseModel):
    check: str
    passed: bool
    error: Optional[str] = None


class TaskStatus(BaseModel):
    ready: bool
    results: Optional[List[CheckResult]] = None


tasks: dict[str, TaskStatus] = {}


PRIVACY_PATTERNS = [
    "/legal/privacy-policy",
    "/privacy",
    "/privacy-policy",
    "/politica-de-privacidade",
]

async def fetch(url: str) -> httpx.Response:
    async with httpx.AsyncClient(
        follow_redirects=True,
        timeout=10.0,
        headers={"User-Agent": "DataSniffingCaramelo (python)"},
    ) as client:
        return await client.get(url)


def has_privacy_link(soup: BeautifulSoup) -> bool:
    for a in soup.find_all("a", href=True):
        href = a["href"].lower()
        if any(p in href for p in PRIVACY_PATTERNS):
            return True

    footer = soup.find("footer")
    if footer:
        for a in footer.find_all("a", href=True):
            if "/legal/" in a["href"].lower():
                return True
    return False


async def check_cookie_consent(resp: httpx.Response) -> bool:
    
    return "set-cookie" not in resp.headers


async def run_crawler(target_url: str) -> List[CheckResult]:
    results: List[CheckResult] = []

    try:
        resp = await fetch(target_url)
    except Exception as exc:
        results.append(
            CheckResult(
                check="Erro ao acessar URL",
                passed=False,
                error=str(exc),
            )
        )
        return results

    soup = BeautifulSoup(resp.text, "lxml")

    results.append(
        CheckResult(
            check="Política de Privacidade",
            passed=has_privacy_link(soup),
        )
    )

    results.append(
        CheckResult(
            check="Coleta cookies somente após consentimento",
            passed=await check_cookie_consent(resp),
        )
    )

    password_ok = False
    if any(tok in target_url.lower() for tok in ("cadastro", "signup", "register")):
        for inp in soup.find_all("input", {"type": "password"}):
            attrs = getattr(inp, "attrs", {}) or {}
            if "pattern" in attrs or "minlength" in attrs:
                password_ok = True
                break
    results.append(
        CheckResult(
            check="Política de força de senha",
            passed=password_ok,
        )
    )

    return results


@app.post("/run-crawler")
async def start_crawler(payload: CrawlerRequest):
    task_id = str(uuid4())
    tasks[task_id] = TaskStatus(ready=False, results=None)

    async def background():
        res = await run_crawler(str(payload.url))
        tasks[task_id] = TaskStatus(ready=True, results=res)

    asyncio.create_task(background())
    return {"success": True, "task_id": task_id}


@app.get("/crawler-result/{task_id}")
async def get_result(task_id: str):
    if task_id not in tasks:
        raise HTTPException(status_code=404, detail="Task not found")
    return tasks[task_id]