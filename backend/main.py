from pathlib import Path

from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles

from backend.routers import auth, config, courses, devoirs, plan
from backend.settings import ALLOWED_ORIGINS

app = FastAPI(
    title="Tableau de bord étudiant",
    version="3.0.0",
    description="API du tableau de bord étudiant : emploi du temps ADE, devoirs et planification IA.",
)


@app.middleware("http")
async def ajouter_entetes_securite(request: Request, call_next):
    response = await call_next(request)
    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["X-Frame-Options"] = "DENY"
    response.headers["Referrer-Policy"] = "strict-origin-when-cross-origin"
    response.headers["Permissions-Policy"] = "camera=(), microphone=(), geolocation=()"
    return response


app.add_middleware(
    CORSMiddleware,
    allow_origins=ALLOWED_ORIGINS,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/api/health", tags=["system"])
def healthcheck():
    return {"status": "ok", "version": app.version}


app.include_router(auth.router)
app.include_router(courses.router)
app.include_router(devoirs.router)
app.include_router(config.router)
app.include_router(plan.router)


_FRONTEND_DIR = Path(__file__).resolve().parent.parent / "frontend"
if _FRONTEND_DIR.is_dir():
    app.mount("/", StaticFiles(directory=_FRONTEND_DIR, html=True), name="frontend")
